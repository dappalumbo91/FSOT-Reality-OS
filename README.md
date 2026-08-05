# FSOT Reality OS

**A real operating system kernel in Rust (`no_std`), booted under QEMU.**  
**Python is not the OS.** Python residual CLI is a formula shell only.

| Pin | Master formula |
|-----|----------------|
| **D1D38A** | \(S = K(T_1+T_2+T_3)\) · \(c = m\,(1+\|S\|\,f)\) |

## What actually boots (v0.4 — heap + hello.fsotb + full preemptive queue)

```text
kernel/
  crates/reality_os_scalar    # S engine + FULL domain table (530)
  crates/reality_os_hw        # processor / RAM / trit-pack laws
  crates/reality_os_trinary   # FSOTB ISA + embedded monorepo hello.fsotb
  crates/reality_os_mem       # map_physical_memory heap on frames
  crates/reality_os_sched     # ready-queue all 530 domains + tick preemption
  crates/reality_os_kernel    # bare-metal + PIT timer
  assets/hello.fsotb          # wire blob from FSOT-2.1-Lean vendor/trinary_os
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

**Last verified boot (v0.4):**  
`FSOT_ROS_OVERALL=ok` · `HEAP_OK` · `HELLO_FSOTB_OK` tag=42 ·  
`SCHED_TASKS=530` · preempts=1060 · quanta=1060.  
Artifacts: `data/reality_os_kernel.img`, `data/reality_os_qemu_serial.log`.

### Boot phases

1. Console + boot scalar `KernelInit`  
2. Hardware self-check  
3. Full domain table walk (530) + registry dump  
4. **`map_physical_memory` + heap** on allocated frames (write/readback)  
5. **Trinary ISA + real `hello.fsotb`** wire load (magic/seeds/decode/run)  
6. **Ready-queue all 530 domains + PIT timer preemption**  
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

## Roadmap (real OS, next)

- [x] v0.1 bare-metal kernel + QEMU boot  
- [x] v0.2 full covered domain table (530)  
- [x] v0.3 trinary ISA + frame allocator + cooperative scheduler  
- [x] v0.4 `map_physical_memory` heap + `hello.fsotb` + 530 ready-queue + PIT preemption  
- [ ] IDT IRQ0 path (hardware interrupt preemption vs PIT poll)  
- [ ] Load call_ret / spawn_join FSOTB oracles  
- [ ] Host plant: integrate `fsot-pc-monitor` under `host/`  
- [ ] Optional: Linux userspace policy plane  

See [`kernel/README.md`](kernel/README.md) · [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md).
