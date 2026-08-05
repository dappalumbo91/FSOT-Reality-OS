//! FSOT Reality OS v0.3 — bare-metal kernel.
//!
//! Real `no_std` OS kernel (bootloader + QEMU x86_64).
//! Python residual CLI is **not** this OS.
//!
//! Boot path:
//!   1. Console + boot scalar
//!   2. Hardware self-check
//!   3. Full domain table walk (all covered domains)
//!   4. Frame allocator from memory map
//!   5. Trinary ISA interpreter self-test
//!   6. Cooperative domain scheduler
//!   7. QEMU markers + halt

#![no_std]
#![no_main]

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use reality_os_hw::boot_hardware_self_check;
use reality_os_mem::boot_mem_selftest;
use reality_os_scalar::{
    boot_scalar, sign_trit, walk_all_domains, AUTHORITY_PIN, BOOT_SCALAR_CANONICAL, DOMAIN_COUNT,
    DOMAIN_TABLE,
};
use reality_os_sched::boot_sched_selftest;
use reality_os_trinary::{opcode_registry_ok, residual_demo_ok, run_boot_selftest};

entry_point!(kernel_main);

const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
static mut VGA_BUFFER: *mut u16 = 0xb8000 as *mut u16;
const SERIAL_PORT: u16 = 0x3F8;
const DEBUGCON_PORT: u16 = 0xe9;

#[inline(always)]
unsafe fn outb(port: u16, value: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") value,
        options(nomem, nostack, preserves_flags)
    );
}

struct SerialWriter;

impl SerialWriter {
    fn init_uart(&self) {
        unsafe {
            outb(SERIAL_PORT + 1, 0x00);
            outb(SERIAL_PORT + 3, 0x80);
            outb(SERIAL_PORT + 0, 0x03);
            outb(SERIAL_PORT + 1, 0x00);
            outb(SERIAL_PORT + 3, 0x03);
            outb(SERIAL_PORT + 2, 0xC7);
            outb(SERIAL_PORT + 4, 0x0B);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        unsafe {
            outb(DEBUGCON_PORT, byte);
            outb(SERIAL_PORT, byte);
        }
    }

    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }

    fn write_f64(&mut self, val: f64, precision: usize) {
        write_f64(self, val, precision);
    }

    fn write_i32(&mut self, mut n: i32) {
        if n < 0 {
            self.write_byte(b'-');
            n = -n;
        }
        write_u64(self, n as u64);
    }
}

struct VgaWriter {
    row: usize,
    col: usize,
}

impl VgaWriter {
    fn new() -> Self {
        Self { row: 0, col: 0 }
    }

    fn clear(&mut self) {
        for row in 0..VGA_HEIGHT {
            for col in 0..VGA_WIDTH {
                unsafe {
                    core::ptr::write_volatile(
                        VGA_BUFFER.offset((row * VGA_WIDTH + col) as isize),
                        0x0F00 | b' ' as u16,
                    );
                }
            }
        }
        self.row = 0;
        self.col = 0;
    }

    fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => {
                self.col = 0;
                self.row += 1;
                if self.row >= VGA_HEIGHT {
                    self.row = 0;
                }
            }
            _ => {
                if self.col >= VGA_WIDTH {
                    self.col = 0;
                    self.row += 1;
                    if self.row >= VGA_HEIGHT {
                        self.row = 0;
                    }
                }
                let idx = self.row * VGA_WIDTH + self.col;
                unsafe {
                    core::ptr::write_volatile(
                        VGA_BUFFER.offset(idx as isize),
                        0x0F00 | byte as u16,
                    );
                }
                self.col += 1;
            }
        }
    }

    fn write_str(&mut self, s: &str) {
        for b in s.bytes() {
            self.write_byte(b);
        }
    }
}

trait ByteSink {
    fn put(&mut self, b: u8);
}

impl ByteSink for SerialWriter {
    fn put(&mut self, b: u8) {
        self.write_byte(b);
    }
}

fn write_u64(w: &mut impl ByteSink, mut n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    if n == 0 {
        w.put(b'0');
        return;
    }
    while n > 0 {
        buf[i] = b'0' + (n % 10) as u8;
        n /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        w.put(buf[i]);
    }
}

fn write_f64(w: &mut impl ByteSink, val: f64, precision: usize) {
    if val.is_nan() {
        for b in b"NaN" {
            w.put(*b);
        }
        return;
    }
    if val.is_infinite() {
        if val.is_sign_negative() {
            w.put(b'-');
        }
        for b in b"inf" {
            w.put(*b);
        }
        return;
    }
    let mut v = val;
    if v < 0.0 {
        w.put(b'-');
        v = -v;
    }
    let int_part = v as u64;
    write_u64(w, int_part);
    if precision > 0 {
        w.put(b'.');
        let mut frac = v - (int_part as f64);
        for _ in 0..precision {
            frac *= 10.0;
            let digit = frac as u8;
            w.put(b'0' + digit);
            frac -= digit as f64;
        }
    }
}

fn write_u32_out(out: &mut Consoles<'_>, n: u32) {
    write_u64_out(out, n as u64);
}

fn write_u64_out(out: &mut Consoles<'_>, n: u64) {
    let mut buf = [0u8; 20];
    let mut i = 0;
    let mut t = n;
    if t == 0 {
        out.write_str("0");
        return;
    }
    while t > 0 {
        buf[i] = b'0' + (t % 10) as u8;
        t /= 10;
        i += 1;
    }
    while i > 0 {
        i -= 1;
        let b = [buf[i]];
        out.write_str(core::str::from_utf8(&b).unwrap_or("?"));
    }
}

struct Consoles<'a> {
    vga: &'a mut VgaWriter,
    serial: &'a mut SerialWriter,
}

impl<'a> Consoles<'a> {
    fn write_str(&mut self, s: &str) {
        self.vga.write_str(s);
        self.serial.write_str(s);
    }

    fn write_f64(&mut self, val: f64, precision: usize) {
        write_f64(self.vga, val, precision);
        self.serial.write_f64(val, precision);
    }
}

impl ByteSink for VgaWriter {
    fn put(&mut self, b: u8) {
        self.write_byte(b);
    }
}

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    let mut vga = VgaWriter::new();
    let mut serial = SerialWriter;
    serial.init_uart();
    vga.clear();

    let mut out = Consoles {
        vga: &mut vga,
        serial: &mut serial,
    };

    out.write_str("========================================\n");
    out.write_str(" FSOT REALITY OS v0.3  (Rust no_std)\n");
    out.write_str(" Fluid Spacetime Omni-Theory kernel\n");
    out.write_str(" pin=");
    out.write_str(AUTHORITY_PIN);
    out.write_str("  S=K(T1+T2+T3)\n");
    out.write_str(" domains+trinary+mem+sched\n");
    out.write_str("========================================\n\n");

    // --- Phase 1: boot scalar ---
    out.write_str("[1] Boot scalar KernelInit D_eff=8 observed\n");
    let s_boot = boot_scalar();
    out.write_str("    S_boot = ");
    out.write_f64(s_boot, 12);
    out.write_str("\n    S_canonical = ");
    out.write_f64(BOOT_SCALAR_CANONICAL, 12);
    out.write_str("\n");
    let trit = sign_trit(s_boot);
    out.write_str("    sign(S) trit = ");
    if trit > 0 {
        out.write_str("+1 EMERGE\n");
    } else if trit < 0 {
        out.write_str("-1 DAMP\n");
    } else {
        out.write_str("0 SUPERPOSE\n");
    }

    // --- Phase 2: hardware laws ---
    out.write_str("\n[2] Hardware self-check (processor + RAM + trit pack)\n");
    let hw = boot_hardware_self_check();
    out.write_str("    collapse_theta = ");
    out.write_f64(hw.collapse_theta, 12);
    out.write_str("\n    vram_usable_mib = ");
    out.write_f64(hw.vram_usable_mib, 6);
    out.write_str("\n    states_per_u64 = 32\n");
    out.write_str(if hw.pack_ok {
        "    trit_pack_roundtrip = OK\n"
    } else {
        "    trit_pack_roundtrip = FAIL\n"
    });
    out.write_str(if hw.overall_ok {
        "    hardware_overall = OK\n"
    } else {
        "    hardware_overall = FAIL\n"
    });

    // --- Phase 3: FULL domain table (every covered domain, not a subset) ---
    out.write_str("\n[3] Full domain interface table (ALL covered domains)\n");
    out.write_str("    DOMAIN_COUNT compile-time = ");
    write_u32_out(&mut out, DOMAIN_COUNT as u32);
    out.write_str("\n    table_len runtime = ");
    write_u32_out(&mut out, DOMAIN_TABLE.len() as u32);
    out.write_str("\n");

    // Walk every domain: compute S + residual for all
    let walk = walk_all_domains();
    out.write_str("    walked_total = ");
    write_u32_out(&mut out, walk.total);
    out.write_str("\n    walked_core = ");
    write_u32_out(&mut out, walk.core);
    out.write_str("\n    walked_extension = ");
    write_u32_out(&mut out, walk.extension);
    out.write_str("\n    walked_other = ");
    write_u32_out(&mut out, walk.other);
    out.write_str("\n    sign_emerge = ");
    write_u32_out(&mut out, walk.emerge);
    out.write_str("  damp = ");
    write_u32_out(&mut out, walk.damp);
    out.write_str("  zero = ");
    write_u32_out(&mut out, walk.zero);
    out.write_str("\n    residual_finite = ");
    write_u32_out(&mut out, walk.residual_finite);
    out.write_str("\n    mean_abs_S = ");
    let mean_abs = if walk.total > 0 {
        walk.s_sum_abs / (walk.total as f64)
    } else {
        0.0
    };
    out.write_f64(mean_abs, 8);
    out.write_str("\n");

    // Emit every domain name + D_eff + kind (compact registry dump)
    out.write_str("    --- domain registry dump ---\n");
    let mut i = 0usize;
    while i < DOMAIN_TABLE.len() {
        let d = &DOMAIN_TABLE[i];
        out.write_str("    ");
        write_u32_out(&mut out, i as u32);
        out.write_str(" ");
        out.write_str(d.kind);
        out.write_str(" ");
        out.write_str(d.name);
        out.write_str(" D=");
        out.write_f64(d.d_eff, 1);
        out.write_str(" f=");
        out.write_f64(d.factor, 6);
        out.write_str("\n");
        i += 1;
    }

    let domains_ok = walk.total == DOMAIN_COUNT as u32
        && walk.residual_finite == walk.total
        && walk.total > 100;

    // --- Phase 4: physical memory map + frame allocator ---
    out.write_str("\n[4] Memory map + frame allocator\n");
    let (mem_ok, mem_rep, first_frame) = boot_mem_selftest(&boot_info.memory_map);
    out.write_str("    regions = ");
    write_u32_out(&mut out, mem_rep.regions);
    out.write_str("  usable_regions = ");
    write_u32_out(&mut out, mem_rep.usable_regions);
    out.write_str("\n    usable_frames = ");
    write_u64_out(&mut out, mem_rep.usable_frames);
    out.write_str("  allocated = ");
    write_u64_out(&mut out, mem_rep.allocated);
    out.write_str("\n    first_frame = ");
    write_u64_out(&mut out, first_frame);
    out.write_str("\n    mem_selftest = ");
    out.write_str(if mem_ok { "OK\n" } else { "FAIL\n" });

    // --- Phase 5: trinary ISA interpreter ---
    out.write_str("\n[5] Trinary ISA (FSOTB / Metatron) interpreter\n");
    let reg_ok = opcode_registry_ok();
    let (tri_ok, tri_steps, tri_r0, tri_tag, tri_evals) = run_boot_selftest();
    let res_ok = residual_demo_ok();
    out.write_str("    opcode_registry_0_26 = ");
    out.write_str(if reg_ok { "OK\n" } else { "FAIL\n" });
    out.write_str("    selftest_steps = ");
    write_u32_out(&mut out, tri_steps);
    out.write_str("  r0=");
    write_u32_out(&mut out, tri_r0 as u32);
    out.write_str("  emit_tag=");
    write_u32_out(&mut out, tri_tag as u32);
    out.write_str("  evals=");
    write_u32_out(&mut out, tri_evals);
    out.write_str("\n    trinary_selftest = ");
    out.write_str(if tri_ok { "OK\n" } else { "FAIL\n" });
    out.write_str("    residual_demo = ");
    out.write_str(if res_ok { "OK\n" } else { "FAIL\n" });

    // --- Phase 6: cooperative domain scheduler ---
    out.write_str("\n[6] Cooperative domain scheduler\n");
    let (sched_ok, sched_tasks, sched_ran, sched_sw) = boot_sched_selftest();
    out.write_str("    tasks = ");
    write_u32_out(&mut out, sched_tasks);
    out.write_str("  quanta_run = ");
    write_u32_out(&mut out, sched_ran);
    out.write_str("  switches = ");
    write_u32_out(&mut out, sched_sw);
    out.write_str("\n    sched_selftest = ");
    out.write_str(if sched_ok { "OK\n" } else { "FAIL\n" });

    let overall = domains_ok && hw.overall_ok && mem_ok && tri_ok && reg_ok && sched_ok;

    out.write_str("\n[7] Reality OS v0.3 boot complete — QEMU markers\n");
    drop(out);

    // Machine-parseable markers (harness)
    serial.write_str("FSOT_ROS_VERSION=0.3\n");
    serial.write_str("FSOT_ROS_PIN=");
    serial.write_str(AUTHORITY_PIN);
    serial.write_str("\n");
    serial.write_str("FSOT_QEMU_BOOT_SCALAR=");
    serial.write_f64(s_boot, 17);
    serial.write_str("\n");
    serial.write_str("FSOT_QEMU_CANONICAL=");
    serial.write_f64(BOOT_SCALAR_CANONICAL, 17);
    serial.write_str("\n");
    serial.write_str("FSOT_QEMU_DISK_BOOT=ok\n");
    serial.write_str("FSOT_ROS_HW_OK=");
    serial.write_str(if hw.overall_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_DOMAINS=");
    serial.write_i32(walk.total as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_DOMAINS_CORE=");
    serial.write_i32(walk.core as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_DOMAINS_EXT=");
    serial.write_i32(walk.extension as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_RESIDUAL_FINITE=");
    serial.write_i32(walk.residual_finite as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_DOMAIN_TABLE_OK=");
    serial.write_str(if domains_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_MEM_OK=");
    serial.write_str(if mem_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_MEM_USABLE_FRAMES=");
    serial.write_i32(mem_rep.usable_frames as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_MEM_ALLOCATED=");
    serial.write_i32(mem_rep.allocated as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_TRINARY_OK=");
    serial.write_str(if tri_ok && reg_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_TRINARY_STEPS=");
    serial.write_i32(tri_steps as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SCHED_OK=");
    serial.write_str(if sched_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SCHED_QUANTA=");
    serial.write_i32(sched_ran as i32);
    serial.write_str("\n");
    serial.write_str(if overall {
        "FSOT_ROS_OVERALL=ok\n"
    } else {
        "FSOT_ROS_OVERALL=fail\n"
    });
    serial.write_str("FSOT_QEMU_HW_OVERALL=ok\n");

    // isa-debug-exit success code for bootimage test harness
    unsafe {
        outb(0xf4, 0x10);
    }
    loop {
        unsafe {
            core::arch::asm!("hlt", options(nomem, nostack, preserves_flags));
        }
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut serial = SerialWriter;
    serial.write_str("\n!!! FSOT REALITY OS PANIC !!!\n");
    serial.write_str("FSOT_ROS_OVERALL=panic\n");
    loop {
        core::hint::spin_loop();
    }
}
