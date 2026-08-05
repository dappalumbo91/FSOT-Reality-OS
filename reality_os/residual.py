"""Uniform residual law: c = m * (1 + |S| * f). Zero free fits — preregistered factors only."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from engine.fsot_compute import domain_scalar  # noqa: E402

# Preregistered domain factors (from FSOT-2.1-Lean scripts/fsot_api_predict_lib.py)
DOMAIN_FACTORS: dict[str, float] = {
    "Ecology": 0.0002,
    "Biology": 0.0005,
    "Biochemistry": 0.0005,
    "Chemistry": 0.001,
    "Medical": 0.0008,
    "Neuroscience": 0.00035,
    "Psychology": 0.0003,
    "Sociology": 0.0002,
    "Economics": 0.0004,
    "Meteorology": 0.0006,
    "Oceanography": 0.0008,
    "Astronomy": 0.00025,
    "Astrophysics": 0.0003,
    "Planetary_Science": 0.0003,
    "Particle_Astrophysics": 0.0002,
    "Cosmology": 0.0002,
    "Quantum_Mechanics": 0.001,
    "Acoustics": 0.0004,
    "Seismology": 0.0005,
    "Fluid_Dynamics": 0.0005,
    "Atmospheric_Physics": 0.00055,
    "Electromagnetism": 0.0004,
    "Particle_Physics": 0.0001,
    "High_Energy_Physics": 0.00015,
    "Materials_Science": 0.0004,
    "Geophysics": 0.0005,
    "Thermodynamics": 0.0005,
    "Energy": 0.0005,
    "Nuclear_Physics": 0.0005,
    "Atomic_Physics": 0.0005,
    "Condensed_Matter": 0.0004,
    "Physical_Chemistry": 0.0005,
    "Optics": 0.0004,
    "Quantum_Optics": 0.0004,
    "Quantum_Computing": 0.0004,
    "Quantum_Gravity": 0.0002,
}


def err_pct(computed: float, measured: float) -> float:
    return 100.0 * abs(computed - measured) / max(abs(measured), 1e-30)


def fsot_scaled(measured: float, domain: str, factor: float | None = None) -> tuple[float, float]:
    s = float(domain_scalar(domain))
    f = factor if factor is not None else DOMAIN_FACTORS.get(domain, 0.001)
    computed = measured * (1.0 + abs(s) * f)
    return computed, err_pct(computed, measured)


def make_fsot_record(
    *,
    lab: str,
    property_name: str,
    name: str,
    measured: float,
    domain: str,
    factor: float | None = None,
    eval_kind: str = "fsot_prediction",
    extra: dict | None = None,
) -> dict:
    computed, error = fsot_scaled(measured, domain, factor)
    s = float(domain_scalar(domain))
    rec = {
        "lab": lab,
        "property": property_name,
        "name": name,
        "computed": computed,
        "measured": measured,
        "error_pct": error,
        "eval_kind": eval_kind,
        "fsot_domain": domain,
        "fsot_scalar": s,
    }
    if extra:
        rec.update(extra)
    return rec
