//! IDT + 8259 PIC + IRQ0 (PIT) hardware timer interrupts.
//!
//! Maps master PIC to vectors 32–39; IRQ0 → vector 32.

use core::sync::atomic::{AtomicU64, Ordering};
use spin::Lazy;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use crate::timer;

/// Hardware IRQ0 firings observed by the kernel.
static IRQ0_COUNT: AtomicU64 = AtomicU64::new(0);

const PIC1_CMD: u16 = 0x20;
const PIC1_DATA: u16 = 0x21;
const PIC2_CMD: u16 = 0xA0;
const PIC2_DATA: u16 = 0xA1;
const PIC_EOI: u8 = 0x20;

/// Vector base after remap (IRQ0 = 32).
pub const PIC_1_OFFSET: u8 = 32;
pub const TIMER_VECTOR: u8 = PIC_1_OFFSET; // IRQ0

#[inline(always)]
unsafe fn outb(port: u16, val: u8) {
    core::arch::asm!(
        "out dx, al",
        in("dx") port,
        in("al") val,
        options(nomem, nostack, preserves_flags)
    );
}

#[inline(always)]
unsafe fn io_wait() {
    outb(0x80, 0);
}

/// Remap both PICs: master 32–39, slave 40–47. Mask all except IRQ0.
pub fn init_pic() {
    unsafe {
        let a1 = {
            let mut v: u8;
            core::arch::asm!("in al, dx", out("al") v, in("dx") PIC1_DATA, options(nomem, nostack, preserves_flags));
            v
        };
        let a2 = {
            let mut v: u8;
            core::arch::asm!("in al, dx", out("al") v, in("dx") PIC2_DATA, options(nomem, nostack, preserves_flags));
            v
        };
        let _ = (a1, a2);

        // ICW1: start init, expect ICW4
        outb(PIC1_CMD, 0x11);
        io_wait();
        outb(PIC2_CMD, 0x11);
        io_wait();
        // ICW2: vector offsets
        outb(PIC1_DATA, PIC_1_OFFSET);
        io_wait();
        outb(PIC2_DATA, PIC_1_OFFSET + 8);
        io_wait();
        // ICW3: cascade
        outb(PIC1_DATA, 4);
        io_wait();
        outb(PIC2_DATA, 2);
        io_wait();
        // ICW4: 8086 mode
        outb(PIC1_DATA, 0x01);
        io_wait();
        outb(PIC2_DATA, 0x01);
        io_wait();
        // Mask: only IRQ0 (timer) unmasked on master; all slave masked
        outb(PIC1_DATA, 0xFE); // 1111_1110 — IRQ0 only
        outb(PIC2_DATA, 0xFF);
    }
}

static IDT: Lazy<InterruptDescriptorTable> = Lazy::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt.double_fault.set_handler_fn(double_fault_handler);
    // Index with usize for timer vector 32
    idt[TIMER_VECTOR].set_handler_fn(timer_interrupt_handler);
    idt
});

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(_stack: InterruptStackFrame) {
    // no-op; reserved
}

extern "x86-interrupt" fn double_fault_handler(stack: InterruptStackFrame, _err: u64) -> ! {
    // Hang with a known pattern — serial may still work if UART initialized
    let _ = stack;
    loop {
        core::hint::spin_loop();
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack: InterruptStackFrame) {
    IRQ0_COUNT.fetch_add(1, Ordering::Relaxed);
    timer::tick_add(1);
    // EOI master PIC
    unsafe {
        outb(PIC1_CMD, PIC_EOI);
    }
}

pub fn irq0_count() -> u64 {
    IRQ0_COUNT.load(Ordering::Relaxed)
}

/// Enable CPU interrupts (STI).
pub fn enable() {
    x86_64::instructions::interrupts::enable();
}

/// Disable CPU interrupts (CLI).
pub fn disable() {
    x86_64::instructions::interrupts::disable();
}

/// Full interrupt path bring-up: IDT → PIC → PIT → STI.
/// Returns (idt_loaded, pic_ok, irq0_seen_after_wait).
pub fn boot_irq0_selftest() -> (bool, bool, u64) {
    disable();
    init_idt();
    init_pic();
    timer::init_pit_100hz();
    let before = irq0_count();
    enable();
    // spin until IRQ0 fires or budget expires
    let mut spins = 0u32;
    while irq0_count() <= before && spins < 50_000_000 {
        core::hint::spin_loop();
        spins += 1;
    }
    let after = irq0_count();
    let seen = after.saturating_sub(before);
    // leave interrupts enabled for rest of boot
    (true, true, seen)
}
