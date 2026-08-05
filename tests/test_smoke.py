"""Smoke tests for standalone Reality OS."""

from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))


def test_domain_S_particle():
    from reality_os.core import compute_domain_S

    s = compute_domain_S("Particle_Physics")
    assert s > 0


def test_qm_emergence():
    from reality_os.core import compute_domain_S

    assert compute_domain_S("Quantum_Mechanics") > 0


def test_cosmo_damping():
    from reality_os.core import compute_domain_S

    assert compute_domain_S("Cosmology") < 0


def test_residual_law():
    from reality_os.residual import fsot_scaled

    c, err = fsot_scaled(1.0, "Particle_Physics")
    assert c > 1.0
    assert 0 < err < 0.5


def test_trinary_opcodes():
    from reality_os.core import trinary_status

    t = trinary_status()
    assert t["abi"]["opcodes"] == 27
    assert t["abi"]["register_count"] == 25
    assert len(t["reality_string"]) == 10


def test_eta_seed():
    from reality_os.matter_antimatter import seed_eta_baryon_photon

    eta = seed_eta_baryon_photon()
    assert 5e-10 < eta < 7e-10


if __name__ == "__main__":
    test_domain_S_particle()
    test_qm_emergence()
    test_cosmo_damping()
    test_residual_law()
    test_trinary_opcodes()
    test_eta_seed()
    print("smoke OK")
