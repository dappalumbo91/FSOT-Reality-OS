# Reality OS — status checkpoint (pause for main Lean)

**Date:** 2026-08-06  
**Repo tip:** `main` (v0.6 FSOTB suite + v0.5 IDT IRQ0 + policy docs)  
**Upstream science / verification:** https://github.com/dappalumbo91/FSOT-2.1-Lean  

We **pause OS feature work** here to return focus to the **main Lean multiprover + empirical residual system**. This file freezes *where Reality OS is* so we do not lose the thread.

---

## Practical loop (clarified — no confusion)

This is **what we already do on this machine** (Windows or any host with Rust/QEMU):

1. **Host = development workstation** (not “install Ubuntu as the product OS”).  
2. **Peek** at Linux/Ubuntu only as a **schematic textbook** when stuck on structure.  
3. **Implement** only FSOT-native Rust under pin D1D38A.  
4. **Prove** with `cargo bootimage` + QEMU serial markers.

Ubuntu/Linux are **pathways to learn from**, not code we ship as Reality OS.  
See [`REFERENCE_OS_PATHWAYS.md`](REFERENCE_OS_PATHWAYS.md).

---

## Where we are (kernel)

| Version | What shipped | QEMU |
|--------:|--------------|------|
| 0.1 | Bare-metal boot, scalar, HW pack | green |
| 0.2 | Full domain table **530** | green |
| 0.3 | Trinary ISA + mem + sched crates | green |
| 0.4 | `map_physical_memory` heap + `hello.fsotb` + 530 ready-queue | green |
| 0.5 | **IDT + PIC + IRQ0** hardware timer | green |
| 0.6 | **FSOTB suite**: hello + call_ret + spawn_join | green |

**Last proved markers (v0.6):**  
`FSOT_ROS_OVERALL=ok` · `FSOTB_SUITE_OK` programs 3/3 · `IRQ0_OK` · `SCHED_TASKS=530` · heap/hello green · license MIT OR Apache-2.0.

**Rebuild:**

```powershell
cd kernel
cargo +nightly bootimage -p reality_os_kernel --release
qemu-system-x86_64 -drive format=raw,file=target/x86_64-fsot-kernel/release/bootimage-reality_os_kernel.bin `
  -display none -serial stdio -device isa-debug-exit,iobase=0xf4,iosize=0x04 -no-reboot
```

---

## Next steps (when we resume OS — ordered)

1. **Full IMM14 wire execute** for CALL/SPAWN (reduce oracle shortcuts on call_ret/spawn_join).  
2. **Host plant** — integrate `fsot-pc-monitor` under `host/` as *dev telemetry fold*, not kernel.  
3. **FSOT-native drivers / ABI** (console, storage later) — schematic from reference OSes, **our code only**.  
4. Installable disk image that is **Reality OS**, not “Ubuntu + tarball.”

**Not next:** forking Linux, GPL-wrapping the product kernel, or treating Python residual CLI as the OS.

---

## Why we circle back to main Lean now

The OS is a **runtime embodiment**. Credibility of FSOT as science rests on the **verification monorepo**:

- residual green gate vs measured data  
- zero free-parameter discipline  
- multiprover triangulation  
- Mathlib depth campaign  
- clean-clone reproducibility  

Those live in **FSOT-2.1-Lean**, not in this OS lab alone.  
See monorepo note: `docs/EMPIRICAL_CLAIM_EVIDENCE.md` (kill commands + artifacts).
