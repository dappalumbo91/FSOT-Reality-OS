# FSOT Reality OS

**Standalone host kernel for the FSOT fluid-spacetime fabric — built to become a real operating system.**

This is **not** a dump of the full [FSOT-2.1-Lean](https://github.com/dappalumbo91/FSOT-2.1-Lean) verification monorepo.  
That tree remains the **formula authority + residual atlas + multiprover lab**.  
**This repository** is the **independently reproducible Reality OS**: one engine, one residual law, one CLI, and a clear path to Linux-based OS construction.

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
python tests/test_smoke.py
```

---

## Layout

```text
FSOT-Reality-OS/
  engine/                 # fsot_compute authority + pin + dynamics
  reality_os/             # host kernel (core, residual, quantum/trinary, matter)
  vendor/trinary_os/isa/  # Metatron 27-opcode ABI
  scripts/reality_os_cli.py
  docs/                   # Linux OS roadmap, upstream sync
  tests/
```

---

## What “operating system” means here

You meant it. The plan is **not** a metaphor-only simulator forever:

1. **This repo (host Reality OS)** — complete scalar fabric as a process/services layer  
2. **Open-source Linux (or minimal kernel tree)** — proven schematics: process model, VFS, sched, net, drivers  
3. **Re-route subsystems through FSOT architecture** — same pin, same \(S\), same residual law  
4. **Trinary opcodes + dimensional interfaces** as native syntax, not bolted-on apps  
5. **Hardware path** — Rust `no_std` scalar kernel → QEMU / bare metal (ported from verification lab)

See [`docs/LINUX_OS_ROADMAP.md`](docs/LINUX_OS_ROADMAP.md).

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
