# Architecture

## Hard rule

**Python cannot be the operating system.**  
Python residual CLI = formula authority shell (pin D1D38A, \(S\), \(c=m(1+|S|f)\)).

**OS execution spine** = Rust kernels + QEMU already in the FSOT-2.1-Lean monorepo.

```text
┌──────────────────────────────────────────────────────────────┐
│  Formula shell (Python) — residual only                      │
│  reality_os_cli.py · engine/fsot_compute.py · pin D1D38A     │
├──────────────────────────────────────────────────────────────┤
│  OS execution spine (Rust + QEMU) — REQUIRED, not "future"   │
│                                                              │
│  monorepo verification/rust/                                 │
│    fsot_scalar_kernel      host + no_std scalar parity       │
│    fsot_hardware_kernel    processor / RAM gates + serial    │
│    fsot_observer_serial    UART / boot markers               │
│    fsot_obligation_replay  multiprover residual replay       │
│                                                              │
│  monorepo vendor/rust_lean_bridge   bootable observer kernel │
│  monorepo verification/qemu         golden disk + BIOS       │
│  monorepo vendor/trinary_os         Metatron ISA / .fsotb    │
│                                                              │
│  runners:                                                    │
│    scripts/run_fsot_hardware_bare_metal.py                   │
│    scripts/run_rust_lean_bridge_qemu_harness.py              │
│    (this repo) scripts/run_hardware_spine.py                 │
└──────────────────────────────────────────────────────────────┘
```

## Why sibling Reality OS was wrong before

It was scaffolded as Python-first with “port Rust later.” That ignored the
monorepo architecture you already had. Correct model:

| Layer | What it is |
|-------|------------|
| Shell | Python residual + pin |
| Spine | Rust + QEMU (monorepo) |
| ISA | Trinary OS bytecode |

## Commands

```powershell
# From monorepo
python scripts/run_fsot_reality_os.py hardware          # inventory
python scripts/run_fsot_reality_os.py hardware --run    # execute spine

# From this sibling
python scripts/run_hardware_spine.py
```

See [`hardware/README.md`](../hardware/README.md).
