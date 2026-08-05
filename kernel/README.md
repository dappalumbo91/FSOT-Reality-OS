# FSOT Reality OS — kernel (Rust `no_std` + QEMU)

**This is the operating system.** Not Python.

| Crate | Role |
|-------|------|
| `reality_os_scalar` | Seed-locked \(S = K(T_1+T_2+T_3)\), residual \(c=m(1+\|S\|f)\), domain table |
| `reality_os_hw` | Collapse θ, trit pack, VRAM usability (seed-closed) |
| `reality_os_kernel` | Bare-metal binary: bootloader → VGA/UART → domain walk → QEMU markers |

Pin **D1D38A**. Zero free parameters.

## Build + boot

```powershell
cd kernel
pwsh scripts/build_and_run.ps1
# or:
cargo build -p reality_os_kernel --release
cargo bootimage -p reality_os_kernel --release
```

Requires: nightly Rust, `cargo-bootimage`, `qemu-system-x86_64`.

## Boot phases (v0.1)

1. Console init  
2. Boot scalar `KernelInit` (\(D_{\mathrm{eff}}=8\))  
3. Hardware self-check  
4. Domain interface table walk (core subset)  
5. Serial markers `FSOT_ROS_*` / `FSOT_QEMU_*` + halt  

## Relation to Python

| Path | Role |
|------|------|
| `../reality_os/*.py`, `../scripts/reality_os_cli.py` | Formula shell / residual host only |
| **`kernel/`** | **Real OS** |

## Provenance

Scalar + hardware laws match monorepo `verification/rust/fsot_scalar_kernel` and
`fsot_hardware_kernel` / `vendor/rust_lean_bridge` — **copied into this repo as
first-class crates**, not invoked via monorepo Python wrappers.
