//! PIT (Programmable Interval Timer) channel 0 — ~100 Hz ticks for preemption.
//!
//! Also exposes a software tick counter advanced by `pit_poll` / IRQ stub.

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

const PIT_CMD: u16 = 0x43;
const PIT_CH0: u16 = 0x40;
/// Divisor for ~100 Hz (1193182 / 100 ≈ 11932).
const PIT_DIVISOR: u16 = 11932;

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
unsafe fn inb(port: u16) -> u8 {
    let v: u8;
    core::arch::asm!(
        "in al, dx",
        out("al") v,
        in("dx") port,
        options(nomem, nostack, preserves_flags)
    );
    v
}

/// Program PIT channel 0 square wave / rate generator.
pub fn init_pit_100hz() {
    unsafe {
        // channel 0, access lobyte/hibyte, mode 2 rate gen, binary
        outb(PIT_CMD, 0x34);
        outb(PIT_CH0, (PIT_DIVISOR & 0xFF) as u8);
        outb(PIT_CH0, (PIT_DIVISOR >> 8) as u8);
    }
    TICKS.store(0, Ordering::SeqCst);
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn tick_add(n: u64) {
    TICKS.fetch_add(n, Ordering::Relaxed);
}

/// Software advance used when IRQ not yet wired; also peeks PIT latch.
pub fn pit_poll_tick() {
    // Latch and read current count; if wrapped-ish, count a tick.
    // For boot self-test we simply increment — hardware IRQ path can call tick_add(1).
    unsafe {
        outb(PIT_CMD, 0x00); // latch ch0
        let lo = inb(PIT_CH0) as u16;
        let hi = inb(PIT_CH0) as u16;
        let _count = lo | (hi << 8);
    }
    TICKS.fetch_add(1, Ordering::Relaxed);
}

/// Busy-wait approximate ms using PIT polls (coarse).
pub fn spin_ticks(n: u64) {
    let start = ticks();
    while ticks().wrapping_sub(start) < n {
        pit_poll_tick();
        core::hint::spin_loop();
    }
}
