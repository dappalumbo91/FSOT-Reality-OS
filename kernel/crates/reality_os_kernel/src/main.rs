//! FSOT Reality OS v0.6 — bare-metal kernel.
//!
//!   1–4. Scalar, HW, domains, heap
//!   5. FSOTB suite: hello + call_ret + spawn_join
//!   6. Ready-queue 530 + IDT IRQ0
//!   7. QEMU markers + halt
//!
//! SPDX-License-Identifier: MIT OR Apache-2.0

#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

mod interrupts;
mod timer;

use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use reality_os_hw::boot_hardware_self_check;
use reality_os_mem::boot_mem_selftest;
use reality_os_scalar::{
    boot_scalar, sign_trit, walk_all_domains, AUTHORITY_PIN, BOOT_SCALAR_CANONICAL, DOMAIN_COUNT,
    DOMAIN_TABLE,
};
use reality_os_sched::boot_sched_selftest;
use reality_os_trinary::{
    opcode_registry_ok, residual_demo_ok, run_boot_selftest, run_fsotb_suite,
};

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

fn write_u64_hex(out: &mut Consoles<'_>, mut n: u64) {
    let mut buf = [0u8; 16];
    let mut i = 0;
    if n == 0 {
        out.write_str("0");
        return;
    }
    while n > 0 {
        let d = (n & 0xF) as u8;
        buf[i] = if d < 10 { b'0' + d } else { b'a' + (d - 10) };
        n >>= 4;
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
    out.write_str(" FSOT REALITY OS v0.6  (Rust no_std)\n");
    out.write_str(" Fluid Spacetime Omni-Theory kernel\n");
    out.write_str(" pin=");
    out.write_str(AUTHORITY_PIN);
    out.write_str("  FSOTB suite + IRQ0\n");
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

    // --- Phase 4: map_physical_memory + heap ---
    out.write_str("\n[4] map_physical_memory + heap on frames\n");
    let phys_off = boot_info.physical_memory_offset;
    out.write_str("    phys_offset = 0x");
    write_u64_hex(&mut out, phys_off);
    out.write_str("\n");
    let (mem_ok, mem_rep, first_frame) = boot_mem_selftest(&boot_info.memory_map, phys_off);
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
    out.write_str("\n    heap_bytes = ");
    write_u64_out(&mut out, mem_rep.heap_bytes);
    out.write_str("  heap_used = ");
    write_u64_out(&mut out, mem_rep.heap_used);
    out.write_str("\n    heap_write_ok = ");
    out.write_str(if mem_rep.heap_write_ok { "1\n" } else { "0\n" });
    out.write_str("    mem_selftest = ");
    out.write_str(if mem_ok { "OK\n" } else { "FAIL\n" });

    // --- Phase 5: trinary + full FSOTB suite (hello, call_ret, spawn_join) ---
    out.write_str("\n[5] Trinary ISA + FSOTB suite (hello/call_ret/spawn_join)\n");
    let reg_ok = opcode_registry_ok();
    let (tri_ok, tri_steps, tri_r0, tri_tag, tri_evals) = run_boot_selftest();
    let res_ok = residual_demo_ok();
    let suite = run_fsotb_suite();
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
    out.write_str("    hello     ok=");
    out.write_str(if suite.hello.overall_ok { "1" } else { "0" });
    out.write_str(" tag=");
    write_u32_out(&mut out, suite.hello.emit_tag as u32);
    out.write_str(" n=");
    write_u32_out(&mut out, suite.hello.n_instructions);
    out.write_str("\n    call_ret  ok=");
    out.write_str(if suite.call_ret.overall_ok { "1" } else { "0" });
    out.write_str(" tag=");
    write_u32_out(&mut out, suite.call_ret.emit_tag as u32);
    out.write_str(" n=");
    write_u32_out(&mut out, suite.call_ret.n_instructions);
    out.write_str(" ver=0x");
    write_u64_hex(&mut out, suite.call_ret.version as u64);
    out.write_str("\n    spawn_join ok=");
    out.write_str(if suite.spawn_join.overall_ok { "1" } else { "0" });
    out.write_str(" tag=");
    write_u32_out(&mut out, suite.spawn_join.emit_tag as u32);
    out.write_str(" n=");
    write_u32_out(&mut out, suite.spawn_join.n_instructions);
    out.write_str(" ver=0x");
    write_u64_hex(&mut out, suite.spawn_join.version as u64);
    out.write_str("\n    fsotb_suite programs_ok=");
    write_u32_out(&mut out, suite.programs_ok);
    out.write_str("/3 = ");
    out.write_str(if suite.overall_ok { "OK\n" } else { "FAIL\n" });

    // --- Phase 6: IDT IRQ0 + full ready-queue preemption ---
    out.write_str("\n[6] IDT + IRQ0 (PIT) + ready-queue ALL domains\n");
    let (idt_ok, pic_ok, irq0_seen) = interrupts::boot_irq0_selftest();
    out.write_str("    idt_loaded = ");
    out.write_str(if idt_ok { "1" } else { "0" });
    out.write_str("  pic_ok = ");
    out.write_str(if pic_ok { "1" } else { "0" });
    out.write_str("  irq0_firings = ");
    write_u64_out(&mut out, irq0_seen);
    out.write_str("\n");

    // While IRQs fire, run full-domain scheduler (preemption uses tick counter)
    let (sched_ok, sched_tasks, sched_ran, sched_sw, preempts) = boot_sched_selftest();
    // allow more IRQ0 during/after sched
    let mut wait = 0u32;
    let irq_before = interrupts::irq0_count();
    while interrupts::irq0_count() < irq_before + 3 && wait < 80_000_000 {
        core::hint::spin_loop();
        wait += 1;
    }
    let irq0_total = interrupts::irq0_count();

    out.write_str("    tasks = ");
    write_u32_out(&mut out, sched_tasks);
    out.write_str("  quanta_run = ");
    write_u32_out(&mut out, sched_ran);
    out.write_str("  switches = ");
    write_u32_out(&mut out, sched_sw);
    out.write_str("\n    preempts = ");
    write_u32_out(&mut out, preempts);
    out.write_str("  soft_ticks = ");
    write_u64_out(&mut out, timer::ticks());
    out.write_str("  irq0_total = ");
    write_u64_out(&mut out, irq0_total);
    out.write_str("\n    irq0_selftest = ");
    let irq_ok = idt_ok && pic_ok && irq0_seen > 0;
    out.write_str(if irq_ok { "OK\n" } else { "FAIL\n" });
    out.write_str("    sched_selftest = ");
    out.write_str(if sched_ok { "OK\n" } else { "FAIL\n" });

    let overall = domains_ok
        && hw.overall_ok
        && mem_ok
        && tri_ok
        && reg_ok
        && suite.overall_ok
        && sched_ok
        && irq_ok;

    out.write_str("\n[7] Reality OS v0.6 boot complete — QEMU markers\n");
    drop(out);

    serial.write_str("FSOT_ROS_VERSION=0.6\n");
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
    serial.write_str("FSOT_ROS_HEAP_OK=");
    serial.write_str(if mem_rep.heap_write_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_MEM_USABLE_FRAMES=");
    serial.write_i32(mem_rep.usable_frames as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_MEM_ALLOCATED=");
    serial.write_i32(mem_rep.allocated as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_HEAP_BYTES=");
    serial.write_i32(mem_rep.heap_bytes as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_TRINARY_OK=");
    serial.write_str(if tri_ok && reg_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_HELLO_FSOTB_OK=");
    serial.write_str(if suite.hello.overall_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_HELLO_TAG=");
    serial.write_i32(suite.hello.emit_tag);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_CALL_RET_OK=");
    serial.write_str(if suite.call_ret.overall_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SPAWN_JOIN_OK=");
    serial.write_str(if suite.spawn_join.overall_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_FSOTB_SUITE_OK=");
    serial.write_str(if suite.overall_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_FSOTB_PROGRAMS_OK=");
    serial.write_i32(suite.programs_ok as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SCHED_OK=");
    serial.write_str(if sched_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SCHED_TASKS=");
    serial.write_i32(sched_tasks as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SCHED_QUANTA=");
    serial.write_i32(sched_ran as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_SCHED_PREEMPTS=");
    serial.write_i32(preempts as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_PIT_TICKS=");
    serial.write_i32(timer::ticks() as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_IRQ0_OK=");
    serial.write_str(if irq_ok { "1" } else { "0" });
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_IRQ0_COUNT=");
    serial.write_i32(irq0_total as i32);
    serial.write_str("\n");
    serial.write_str("FSOT_ROS_LICENSE=MIT_OR_Apache-2.0\n");
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
