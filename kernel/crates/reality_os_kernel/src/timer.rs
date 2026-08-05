//! PIT channel 0 programming + software tick counter (also advanced by IRQ0).

use core::sync::atomic::{AtomicU64, Ordering};

static TICKS: AtomicU64 = AtomicU64::new(0);

const PIT_CMD: u16 = 0x43;
const PIT_CH0: u16 = 0x40;
/// Divisor for ~100 Hz (1_193_182 / 100 ≈ 11932).
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

/// Program PIT channel 0 rate generator @ ~100 Hz.
pub fn init_pit_100hz() {
    unsafe {
        outb(PIT_CMD, 0x34);
        outb(PIT_CH0, (PIT_DIVISOR & 0xFF) as u8);
        outb(PIT_CH0, (PIT_DIVISOR >> 8) as u8);
    }
    // do not zero ticks — IRQ0 path may already be counting
}

pub fn ticks() -> u64 {
    TICKS.load(Ordering::Relaxed)
}

pub fn tick_add(n: u64) {
    TICKS.fetch_add(n, Ordering::Relaxed);
}

pub fn ticks_reset() {
    TICKS.store(0, Ordering::SeqCst);
}
