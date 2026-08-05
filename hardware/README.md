# Reality OS hardware spine — Rust + QEMU (not Python)

**Python is not the OS.** This directory documents and drives the **already-built**
execution spine in the FSOT verification monorepo.

## Authority monorepo paths (do not reimplement in Python)

| Component | Path in FSOT-2.1-Lean |
|-----------|------------------------|
| Scalar kernel (`no_std` + host) | `verification/rust/fsot_scalar_kernel` |
| Hardware processor/RAM kernel | `verification/rust/fsot_hardware_kernel` |
| Observer serial | `verification/rust/fsot_observer_serial` |
| Obligation replay | `verification/rust/fsot_obligation_replay` |
| Bootable bridge | `vendor/rust_lean_bridge` |
| QEMU golden / BIOS | `verification/qemu/` |
| Trinary ISA | `vendor/trinary_os/` |
| Bare-metal runner | `scripts/run_fsot_hardware_bare_metal.py` |
| QEMU harness | `scripts/run_rust_lean_bridge_qemu_harness.py` |

## Run (from this Reality OS repo)

```powershell
# Default: monorepo next to this tree (../FSOT-2.1-Lean) or $env:FSOT_MONOREPO_ROOT
python scripts/run_hardware_spine.py
python scripts/run_hardware_spine.py --skip-qemu   # host Rust only
```

Or from the monorepo (same spine):

```powershell
python scripts/run_fsot_reality_os.py hardware --run
```

## Honesty

| Layer | Language | Role |
|-------|----------|------|
| Formula / residual shell | Python | pin D1D38A, `S`, `c=m(1+|S|f)` |
| **OS execution** | **Rust + QEMU** | kernels, serial, disk boot |
| Trinary bytecode | ISA + `.fsotb` | `vendor/trinary_os` |

Empty crates in *this* sibling without monorepo linkage are a bug, not a product.
