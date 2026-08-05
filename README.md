# FSOT Reality OS

**A real operating system kernel in Rust (`no_std`), booted under QEMU.**  
**Python is not the OS.** Python residual CLI is a formula shell only.

| Pin | Master formula |
|-----|----------------|
| **D1D38A** | \(S = K(T_1+T_2+T_3)\) · \(c = m\,(1+\|S\|\,f)\) |

## What actually boots (v0.1)

```text
kernel/
  crates/reality_os_scalar   # seed-locked scalar engine (no_std)
  crates/reality_os_hw       # processor / RAM / trit-pack laws (no_std)
  crates/reality_os_kernel   # bare-metal binary + bootloader
```

```powershell
cd kernel
cargo +nightly build -p reality_os_kernel --release
cargo +nightly bootimage -p reality_os_kernel --release
# QEMU:
qemu-system-x86_64 -drive format=raw,file=target/x86_64-fsot-kernel/release/bootimage-reality_os_kernel.bin `
  -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -no-reboot
```

Or: `pwsh kernel/scripts/build_and_run.ps1`

**Last verified boot:** `FSOT_ROS_OVERALL=ok` · boot scalar matches canonical · hardware self-check OK · 8 domain interfaces walked.  
Artifacts: `data/reality_os_kernel.img`, `data/reality_os_qemu_serial.log`, `data/reality_os_qemu_boot_report.json`.

### Boot phases

1. VGA + UART console  
2. Boot scalar `KernelInit` (\(D_{\mathrm{eff}}=8\))  
3. Hardware self-check (collapse θ, trit pack, VRAM law)  
4. Core domain table walk (\(S\) + residual predict)  
5. QEMU serial markers + halt  

## Layout

```text
FSOT-Reality-OS/
  kernel/                 # *** THE OPERATING SYSTEM (Rust + QEMU) ***
  reality_os/             # formula shell only (Python residual helpers)
  engine/                 # pin D1D38A authority copy for host residual tools
  scripts/reality_os_cli.py   # host residual CLI — NOT the kernel
  data/                   # boot image + QEMU serial capture
  docs/
```

## Formula shell (optional host tools)

```powershell
python scripts/reality_os_cli.py boot
python scripts/reality_os_cli.py S Quantum_Mechanics
python scripts/reality_os_cli.py predict Planetary_Science 2.77
```

These do **not** replace the kernel. They share the same pin and residual law.

## Provenance

Scalar and hardware laws match the verified monorepo crates  
(`fsot_scalar_kernel`, `fsot_hardware_kernel`, `rust_lean_bridge`) — **vendored as first-class  
source in this repository** and built/booted **here**, not via monorepo Python wrappers.

Upstream atlas / multiprover: https://github.com/dappalumbo91/FSOT-2.1-Lean  

## Roadmap (real OS, next)

- [x] v0.1 bare-metal kernel + QEMU boot + domain table  
- [ ] Full 35-core domain table + preregistered \(f\) table from monorepo  
- [ ] Trinary opcode interpreter in-kernel (`vendor/trinary_os` ISA)  
- [ ] Memory map / frame allocator / basic scheduler  
- [ ] Optional: Linux userspace with Reality OS as policy plane  

See [`kernel/README.md`](kernel/README.md) · [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
