//! FSOT Reality OS scalar engine — `no_std`.
//!
//! Master formula: `S = K * (T1 + T2 + T3)`
//! Residual law (host): `c = m * (1 + |S| * f)` with preregistered `f`.
//! Seeds match pin **D1D38A** authority (`fsot_compute` / monorepo scalar kernel).

#![no_std]

use libm::{cos, exp, log, sin, sqrt};

/// Authority pin prefix (human); full SHA lives in engine pin JSON on host.
pub const AUTHORITY_PIN: &str = "D1D38A";

pub const K: f64 = 0.4202216641606967;
pub const ALPHA: f64 = 0.0008082937414140405;
pub const PSI_CON: f64 = 0.6321205588285577;
pub const ETA_EFF: f64 = 0.46694220692425986;
pub const BETA: f64 = 2.620866911333223e-17;
pub const C_EFF: f64 = 0.9577022026205613;
pub const A_BLEED: f64 = 1.046973630587551;
pub const B_IN: f64 = 0.7879407922764435;
pub const A_IN: f64 = 1.6668538450045731;
pub const CHAOS: f64 = -0.33102418261048183;
pub const P_NEW: f64 = 0.30030227667037146;
pub const C_FACTOR: f64 = 0.28760015181918397;
pub const POOF: f64 = 0.1534822148944508;
pub const THETA_S: f64 = 0.29089654054517305;
pub const SUCTION: f64 = 0.14703398542810284;
pub const P_VAR: f64 = 0.9579871226722757;

pub const BOOT_D_EFF: f64 = 8.0;
pub const BOOT_DELTA_PSI: f64 = 0.7;
pub const BOOT_RECENT_HITS: f64 = 0.0;
pub const BOOT_OBSERVED: bool = true;
/// Canonical boot scalar (monorepo golden).
pub const BOOT_SCALAR_CANONICAL: f64 = 0.09928895626861721;

const GAMMA_EULER: f64 = 0.5772156649;
const PHI: f64 = 1.6180339887;

/// Core domain interface entry: name + D_eff (first 8 of 35 for early boot table).
#[derive(Clone, Copy)]
pub struct DomainIface {
    pub name: &'static str,
    pub d_eff: f64,
    /// Preregistered residual factor f (not free-fit).
    pub factor: f64,
}

/// Seed domain table carried in the kernel (subset; full 35 expand later).
pub const DOMAIN_TABLE: &[DomainIface] = &[
    DomainIface {
        name: "Particle_Physics",
        d_eff: 12.0,
        factor: 0.05,
    },
    DomainIface {
        name: "Quantum_Mechanics",
        d_eff: 6.0,
        factor: 0.05,
    },
    DomainIface {
        name: "Cosmology",
        d_eff: 20.0,
        factor: 0.05,
    },
    DomainIface {
        name: "Planetary_Science",
        d_eff: 14.0,
        factor: 0.05,
    },
    DomainIface {
        name: "Biology",
        d_eff: 16.0,
        factor: 0.05,
    },
    DomainIface {
        name: "Neuroscience",
        d_eff: 15.0,
        factor: 0.05,
    },
    DomainIface {
        name: "Energy",
        d_eff: 12.0,
        factor: 0.05,
    },
    DomainIface {
        name: "KernelInit",
        d_eff: BOOT_D_EFF,
        factor: 0.0,
    },
];

/// Simplified FSOT scalar (T2 = 0 POC; same as monorepo bare-metal kernel).
pub fn compute_s(d_eff: f64, delta_psi: f64, observed: bool, recent_hits: f64) -> f64 {
    let n = 1.0_f64;
    let p = 1.0_f64;
    let d = if d_eff > 1.0 { d_eff } else { 1.0 };
    let dp = delta_psi;
    let hits = recent_hits;

    let growth = exp(ALPHA * (1.0 - hits / n) * GAMMA_EULER / PHI);
    let base = (n * p / sqrt(d))
        * cos((PSI_CON + dp) / ETA_EFF)
        * exp(-ALPHA * hits / n + 1.0 + B_IN * dp)
        * (1.0 + growth * C_EFF);
    let mut t1 = base * (1.0 + P_NEW * log(d / 25.0));
    if observed {
        t1 = t1 * exp(C_FACTOR * P_VAR) * cos(dp + P_VAR);
    }

    let t2 = 0.0_f64;

    let valve = BETA
        * cos(dp)
        * (n * p / sqrt(d))
        * (1.0 + CHAOS * (d - 25.0) / 25.0)
        * (1.0 + POOF * cos(THETA_S + core::f64::consts::PI) + SUCTION * sin(THETA_S));
    let acoustic = 1.0
        + (A_BLEED * sin(1.0_f64) * sin(1.0_f64)) / PHI
        + (A_IN * cos(1.0_f64) * cos(1.0_f64)) / PHI;
    let phase = 1.0 + B_IN * P_VAR;
    let t3 = valve * acoustic * phase;

    K * (t1 + t2 + t3)
}

pub fn boot_scalar() -> f64 {
    compute_s(BOOT_D_EFF, BOOT_DELTA_PSI, BOOT_OBSERVED, BOOT_RECENT_HITS)
}

/// Residual prediction: `c = m * (1 + |S| * f)`.
#[inline]
pub fn residual_predict(measured: f64, s: f64, factor: f64) -> f64 {
    let abs_s = if s >= 0.0 { s } else { -s };
    measured * (1.0 + abs_s * factor)
}

/// Sign of S → trit: +1 / 0 / -1 (trinary syntax seed).
#[inline]
pub fn sign_trit(s: f64) -> i8 {
    if s > 1e-12 {
        1
    } else if s < -1e-12 {
        -1
    } else {
        0
    }
}
