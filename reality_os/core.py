#!/usr/bin/env python3
"""Reality OS core — standalone host kernel (no dependency on FSOT-2.1-Lean tree layout)."""

from __future__ import annotations

import json
import sys
from dataclasses import asdict, dataclass
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from engine.fsot_compute import DOMAINS, ScalarInput, compute_scalar, domain_scalar  # noqa: E402
from mpmath import mpf  # noqa: E402
from reality_os.residual import DOMAIN_FACTORS, fsot_scaled  # noqa: E402

PIN_PATH = ROOT / "engine" / "fsot_compute_AUTHORITY_PIN.json"
OPCODE_REG = ROOT / "vendor" / "trinary_os" / "isa" / "fsotb_opcode_registry.json"


@dataclass
class RealityState:
    pin: str
    master_formula: str
    core_domain_count: int
    ontology: str
    hardware_notes: str


def pin_prefix() -> str:
    if not PIN_PATH.exists():
        return "D1D38A"
    try:
        doc = json.loads(PIN_PATH.read_text(encoding="utf-8"))
        for key in ("authority_sha256", "sha256", "pin", "sha256_prefix"):
            if doc.get(key):
                val = str(doc[key])
                return val[:6].upper() if len(val) >= 6 else val
    except Exception:
        pass
    return "D1D38A"


def compute_domain_S(domain: str) -> float:
    return float(domain_scalar(domain))


def compute_S_raw(
    d_eff: float,
    delta_psi: float = 1.0,
    observed: bool = True,
    recent_hits: float = 0.0,
    delta_theta: float = 1.0,
) -> float:
    si = ScalarInput(
        N=mpf(1),
        P=mpf(1),
        D_eff=mpf(d_eff),
        delta_psi=mpf(delta_psi),
        delta_theta=mpf(delta_theta),
        recent_hits=mpf(recent_hits),
        observed=observed,
        rho=mpf(1),
        scale=mpf(1),
        amplitude=mpf(1),
    )
    return float(compute_scalar(si))


def residual_predict(measured: float, domain: str) -> tuple[float, float]:
    return fsot_scaled(measured, domain)


def list_core_interfaces() -> list[dict[str, Any]]:
    rows = []
    for name, cfg in sorted(DOMAINS.items(), key=lambda x: (x[1].D_eff, x[0])):
        s = float(domain_scalar(name))
        fac = DOMAIN_FACTORS.get(name)
        rows.append(
            {
                "domain": name,
                "D_eff": int(cfg.D_eff),
                "observed": bool(cfg.observed),
                "hits": int(cfg.hits),
                "S": s,
                "sign": "emergence" if s > 0 else "damping" if s < 0 else "zero",
                "f": fac,
                "floor_pct": abs(s) * float(fac) * 100.0 if fac else None,
            }
        )
    return rows


def quantum_status() -> dict[str, Any]:
    cores = ["Quantum_Mechanics", "Quantum_Computing", "Quantum_Optics", "Quantum_Gravity"]
    live = {d: float(domain_scalar(d)) for d in cores}
    return {
        "covered": True,
        "note": (
            "Quantum is first-class: same residual law at QM/QC/QO/QG interfaces. "
            "Not a bolted-on theory."
        ),
        "live_core_S": live,
        "core_count": len(cores),
    }


def trinary_status() -> dict[str, Any]:
    out: dict[str, Any] = {"available": OPCODE_REG.exists()}
    if OPCODE_REG.exists():
        reg = json.loads(OPCODE_REG.read_text(encoding="utf-8"))
        out["abi"] = {
            "opcodes": len(reg.get("opcodes") or []),
            "word_width_trits": reg.get("word_width_trits"),
            "register_count": reg.get("register_count"),
            "note": "27=3^3 Metatron opcodes; 25 registers = D_eff ceiling",
        }
    # Reality string from core sample
    sample = [
        "Particle_Physics",
        "Quantum_Mechanics",
        "Atomic_Physics",
        "Chemistry",
        "Biology",
        "Neuroscience",
        "Nuclear_Physics",
        "Astronomy",
        "Planetary_Science",
        "Cosmology",
    ]
    enc = []
    for d in sample:
        s = float(domain_scalar(d))
        trit = 1 if s > 0 else (-1 if s < 0 else 0)
        enc.append({"domain": d, "S": s, "trit": trit, "symbol": {1: "+", 0: "0", -1: "-"}[trit]})
    out["reality_string"] = "".join(e["symbol"] for e in enc)
    out["encoding"] = enc
    out["note"] = "trit = sign(S): + emerge, 0 null, - damp — machine alphabet of the continuum"
    return out


def matter_status() -> dict[str, Any]:
    try:
        from reality_os.matter_antimatter import (  # type: ignore
            antimatter_conjugate_S,
            matter_S,
            seed_eta_baryon_photon,
            seed_Omega_b_h2,
        )

        return {
            "S_matter": matter_S("Particle_Physics"),
            "S_conjugate": antimatter_conjugate_S("Particle_Physics"),
            "eta": seed_eta_baryon_photon(),
            "Omega_b_h2": seed_Omega_b_h2(),
            "note": "Matter emergence + conjugate dual + seed eta; same fluid S",
        }
    except Exception as exc:  # noqa: BLE001
        return {"error": str(exc), "hint": "matter_antimatter module import"}


def linux_os_path() -> dict[str, Any]:
    return {
        "vision": (
            "Build a real operating system by taking open-source Linux (or a minimal "
            "POSIX/kernel tree) and replacing/augmenting subsystems with Reality OS "
            "capabilities: scheduling via residual S, process identity as dimensional "
            "interfaces, trinary opcodes as native ISA layer, quantum/matter duals as "
            "kernel services — same pin D1D38A formula, not their physics rewritten as free fits."
        ),
        "phases": [
            "1. Host Reality OS (this repo) — complete engine CLI",
            "2. Userspace services on Linux: residual predict, domain routing daemons",
            "3. Kernel modules / eBPF / sched hooks that call fsot scalar kernel",
            "4. Optional: trinary ISA emulator + hardware path (Rust no_std → bare metal)",
            "5. Full distribution: Linux base + Reality OS fabric as first-class init/system",
        ],
        "upstream_formula": "https://github.com/dappalumbo91/FSOT-2.1-Lean",
        "this_repo_role": "Independently reproducible Reality OS kernel + OS build lab",
    }


def snapshot() -> dict[str, Any]:
    cores = list_core_interfaces()
    return {
        "pin": pin_prefix(),
        "master_formula": "S = K*(T1+T2+T3); c = m*(1+|S|*f)",
        "ontology": "fluid_spacetime_omni_D_eff_ceiling_25",
        "core_domain_count": len(cores),
        "quantum": quantum_status(),
        "trinary": trinary_status(),
        "matter": matter_status(),
        "linux_os_path": linux_os_path(),
        "version": "0.1.0",
    }


def boot_message() -> str:
    st = snapshot()
    q = st["quantum"]["live_core_S"]
    lines = [
        "FSOT Reality OS (standalone)",
        f"  pin={st['pin']}",
        f"  formula={st['master_formula']}",
        f"  ontology={st['ontology']}",
        f"  core_domains={st['core_domain_count']}",
        f"  QM S={q.get('Quantum_Mechanics'):.6f}  cosmo band via Cosmology interface",
        f"  reality_string={st['trinary'].get('reality_string')}",
        f"  role=independent OS kernel lab; upstream formula FSOT-2.1-Lean",
    ]
    return "\n".join(lines)


def as_public_dict(st: RealityState) -> dict[str, Any]:
    return asdict(st)
