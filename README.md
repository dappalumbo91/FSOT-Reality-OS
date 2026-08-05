# FSOT Reality OS

**FSOT Reality OS lab — formula shell (Python) + OS spine (Rust/QEMU from monorepo).**

**Hard rule:** Python is **not** an operating system. The execution spine is the
**Rust + QEMU architecture already in** [FSOT-2.1-Lean](https://github.com/dappalumbo91/FSOT-2.1-Lean)
(`verification/rust/*`, `vendor/rust_lean_bridge`, `verification/qemu`). This repo
is the residual/host CLI surface that **must call that spine**, not replace it.

This is **not** a dump of the full multiprover atlas. Upstream monorepo remains
formula authority + residual atlas + multiprover + **hardware kernels**.

| Pin | Master formula |
|-----|----------------|
| **D1D38A** | \(S = K(T_1+T_2+T_3)\) · \(c = m\,(1+\|S\|\,f)\) |

Ontology: **fluid spacetime omni-theory**, \(D_{\mathrm{eff}}\) ceiling **25**.

---

## Quick start

```powershell
cd FSOT-Reality-OS
python -m pip install -r requirements.txt
python scripts/reality_os_cli.py boot
python scripts/reality_os_cli.py S Quantum_Mechanics
python scripts/reality_os_cli.py predict Planetary_Science 2.77
python scripts/reality_os_cli.py quantum
python scripts/reality_os_cli.py trinary
python scripts/reality_os_cli.py matter
python scripts/reality_os_cli.py linux-path
# OS spine (Rust kernels + QEMU in monorepo — not optional)
python scripts/run_hardware_spine.py
python tests/test_smoke.py
```

---

## Layout

```text
FSOT-Reality-OS/
  engine/                 # pin D1D38A residual authority (Python shell only)
  reality_os/             # formula CLI (S, predict, quantum/trinary) — NOT the OS
  hardware/               # documents + points at monorepo Rust/QEMU spine
  scripts/reality_os_cli.py
  scripts/run_hardware_spine.py   # executes monorepo bare-metal + QEMU
  docs/                   # architecture (Rust spine first), Linux roadmap
  tests/
```

---

## What “operating system” means here

You meant it. The plan is **not** a Python metaphor:

1. **OS spine (already in monorepo)** — `fsot_scalar_kernel`, `fsot_hardware_kernel`, `rust_lean_bridge`, QEMU disk/serial — **run it** via `scripts/run_hardware_spine.py`  
2. **This repo (formula shell)** — pin-locked residual CLI only; never claimed as ring‑0  
3. **Open-source Linux (or minimal kernel tree)** — proven schematics for full desktop/server OS  
4. **Re-route subsystems through FSOT architecture** — same pin, same \(S\), same residual law  
5. **Trinary opcodes + dimensional interfaces** as native syntax (`vendor/trinary_os`)

See [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · [`hardware/README.md`](hardware/README.md) · [`docs/LINUX_OS_ROADMAP.md`](docs/LINUX_OS_ROADMAP.md).

---

## Relationship to FSOT-2.1-Lean

| FSOT-2.1-Lean | FSOT-Reality-OS |
|---------------|-----------------|
| Full residual atlas (400+ panels) | Host OS kernel + independent reproduce |
| Multiprover Lean/Coq/… | Consumes verified formula (pin D1D38A) |
| Research leaves, open science ingest | Clean surface for OS engineering |
| Updates formula/atlas | Sync engine pin + residual factors when authority moves |

Upstream: https://github.com/dappalumbo91/FSOT-2.1-Lean  
Sync notes: [`docs/UPSTREAM_FSOT.md`](docs/UPSTREAM_FSOT.md)

---

## First-class fabric (included)

- **35 core domain interfaces** (live \(S\), emerge/damp syntax)  
- **Quantum** — QM / optics / computing / gravity  
- **Trinary string language** — trit = \(\mathrm{sign}(S)\), 27 opcodes, 25 regs = \(D_{\mathrm{eff}}\) ceiling  
- **Matter / antimatter** — conjugate dual + seed \(\eta\), \(\Omega_b h^2\)  
- **Residual predict** — preregistered \(f\) only  

---

## License / science note

Formula authority and residual gates remain scientific artifacts of the upstream monorepo.  
This OS lab is for engineering a runnable system on top of that closed formula — not free-parameter retuning.
