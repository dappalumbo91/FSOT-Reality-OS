//! FSOT Reality OS hardware laws — `no_std` processor/RAM/trinary pack.
//! Ported from monorepo `fsot_hardware_kernel` (seed-closed, not free-fit).

#![no_std]

pub const C_EFF: f64 = 0.9577022026205613;
pub const P_VAR: f64 = 0.9579871226722757;
pub const PHI: f64 = 1.618033988749895;

pub const COLLAPSE_THETA: f64 = C_EFF * P_VAR;
pub const COHERENCE_GATE: f64 = 0.5;
pub const BITS_PER_TRIT: u32 = 2;
pub const WORD_BITS: u32 = 64;
pub const STATES_PER_U64: u32 = WORD_BITS / BITS_PER_TRIT;
pub const WARP_SIZE: u32 = 32;
pub const TRINARY_ARITY: u32 = 3;
pub const DENSITY_GAIN_VS_U8: u32 = 8 / BITS_PER_TRIT;
pub const CRYSTAL_SECTOR_COUNT: u32 = 6;
pub const FORMAL_VRAM_BOUNDARY_MIB: f64 = 12800.0;
pub const MEASURED_VRAM_MIB_RTX5070: f64 = 12226.56;
pub const MEASURED_SM_COUNT_RTX5070: u32 = 48;

#[inline]
pub fn collapse_theta() -> f64 {
    COLLAPSE_THETA
}

#[inline]
pub fn vram_usable_mib(formal_boundary_mib: f64) -> f64 {
    C_EFF * formal_boundary_mib
}

pub fn pack_trits32(codes: &[u8; 32]) -> u64 {
    let mut word: u64 = 0;
    for (i, &c) in codes.iter().enumerate() {
        let c = (c % 3) as u64;
        word |= c << (2 * i);
    }
    word
}

pub fn unpack_trits32(word: u64) -> [u8; 32] {
    let mut codes = [0u8; 32];
    for i in 0..32 {
        codes[i] = ((word >> (2 * i)) & 0b11) as u8;
    }
    codes
}

#[inline]
pub fn collapse_trit(x: f64, theta: f64) -> i8 {
    if x > theta {
        1
    } else if x < -theta {
        -1
    } else {
        0
    }
}

/// Self-check used at kernel boot.
pub struct HwBootReport {
    pub collapse_theta: f64,
    pub vram_usable_mib: f64,
    pub pack_word: u64,
    pub pack_ok: bool,
    pub states_per_u64: u32,
    pub overall_ok: bool,
}

pub fn boot_hardware_self_check() -> HwBootReport {
    let theta = collapse_theta();
    let usable = vram_usable_mib(FORMAL_VRAM_BOUNDARY_MIB);
    let mut codes = [0u8; 32];
    let mut i = 0;
    while i < 32 {
        codes[i] = (i % 3) as u8;
        i += 1;
    }
    let word = pack_trits32(&codes);
    let back = unpack_trits32(word);
    let mut pack_ok = true;
    i = 0;
    while i < 32 {
        if back[i] != codes[i] {
            pack_ok = false;
        }
        i += 1;
    }
    let warp_ok = STATES_PER_U64 == 32 && WARP_SIZE == 32;
    let overall = pack_ok && warp_ok && theta > 0.9 && usable > 12000.0;
    HwBootReport {
        collapse_theta: theta,
        vram_usable_mib: usable,
        pack_word: word,
        pack_ok,
        states_per_u64: STATES_PER_U64,
        overall_ok: overall,
    }
}
