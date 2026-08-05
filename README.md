# FSOT Reality OS

**A real operating system kernel in Rust (`no_std`), booted under QEMU.**  
**Python is not the OS.** Python residual CLI is a formula shell only.

| | |
|--|--|
| **Pin** | **D1D38A** · \(S = K(T_1+T_2+T_3)\) · \(c = m\,(1+\|S\|\,f)\) |
| **License** | **MIT OR Apache-2.0** — [`LICENSE`](LICENSE) · [`LICENSE-MIT`](LICENSE-MIT) · [`LICENSE-APACHE`](LICENSE-APACHE) |

> **Reference OS policy:** **Ubuntu / Linux show pathways** (boot, mem, sched, VFS, drivers…) so we know what a full OS must cover. We **do not use Linux code** as Reality OS — we **build our own through FSOT** (pin D1D38A, domain table, FSOTB, residual law). Details: [`docs/REFERENCE_OS_PATHWAYS.md`](docs/REFERENCE_OS_PATHWAYS.md).  
> **License:** Reality OS is **MIT OR Apache-2.0**. Linux is **GPLv2-only**; studying it does not put this tree under GPL.

## What actually boots (v0.6 — FSOTB suite + IRQ0)

```text
kernel/
  crates/reality_os_scalar    # S engine + FULL domain table (530)
  crates/reality_os_hw        # processor / RAM / trit-pack laws
  crates/reality_os_trinary   # FSOTB ISA + hello + call_ret + spawn_join
  crates/reality_os_mem       # map_physical_memory heap on frames
  crates/reality_os_sched     # ready-queue all 530 domains + tick preemption
  crates/reality_os_kernel    # bare-metal + IDT + PIC + IRQ0 + PIT
  assets/*.fsotb              # monorepo wire blobs (hello, call_ret, spawn_join)
```

**Domain registry:** **530** covered interfaces (union of atlas `domain_interfaces` + all green residual margin domains + neurolab core) — **not** a 35-domain toy table.  
Regenerate from monorepo: `python scripts/gen_domain_table_from_monorepo.py`  
At boot the kernel walks **every** domain (S + residual finite check) and dumps the full registry to serial.

```powershell
cd kernel
cargo +nightly build -p reality_os_kernel --release
cargo +nightly bootimage -p reality_os_kernel --release
# QEMU:
qemu-system-x86_64 -drive format=raw,file=target/x86_64-fsot-kernel/release/bootimage-reality_os_kernel.bin `
  -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -no-reboot
```

Or: `pwsh kernel/scripts/build_and_run.ps1`

**Last verified boot (v0.6):**  
`FSOT_ROS_OVERALL=ok` · **FSOTB suite 3/3** (hello + call_ret + spawn_join) ·  
`IRQ0_OK` · `SCHED_TASKS=530`.  
Artifacts: `data/reality_os_kernel.img`, `data/reality_os_qemu_serial.log`.

### Boot phases

1. Console + boot scalar `KernelInit`  
2. Hardware self-check  
3. Full domain table walk (530) + registry dump  
4. **`map_physical_memory` + heap** on allocated frames  
5. **FSOTB suite** — hello (tag 42) · call_ret (v1.1) · spawn_join (v1.2)  
6. **IDT + IRQ0 + ready-queue 530 domains**  
7. QEMU serial markers + halt  

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

## Roadmap (real OS, next) — FSOT-native only

- [x] v0.1–v0.5 kernel (domains, heap, ready-queue, **IDT IRQ0**)  
- [x] v0.6 **FSOTB suite**: hello + call_ret + spawn_join  
- [x] Dual license **MIT OR Apache-2.0**  
- [x] Policy: reference Ubuntu/Linux pathways; **no Linux code as product**  
- [ ] Full wire IMM14 decode + execute CALL/SPAWN without oracle shortcuts  
- [ ] Host plant (`fsot-pc-monitor`) as **dev telemetry fold**, not the kernel  
- [ ] FSOT-native drivers / richer ABI (schematic from reference OSes, code ours)  

See [`kernel/README.md`](kernel/README.md) · [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) · [`docs/REFERENCE_OS_PATHWAYS.md`](docs/REFERENCE_OS_PATHWAYS.md).
