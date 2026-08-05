# Architecture

## Hard rules

1. **Python is not the operating system** (formula shell / residual host only).  
2. **Linux/Ubuntu are not the operating system** — they are **reference pathways** for studying how full OSes are structured.  
3. **We do not ship Linux code as Reality OS.** We build our own kernel and services through **FSOT**.

See [`REFERENCE_OS_PATHWAYS.md`](REFERENCE_OS_PATHWAYS.md).

## Constitution (FSOT)

| Item | Value |
|------|--------|
| Pin | **D1D38A** |
| Scalar | \(S = K(T_1+T_2+T_3)\) |
| Residual | \(c = m\,(1+\|S\|\,f)\) with preregistered \(f\) |
| Interfaces | Full domain table (530 covered) |
| ISA | FSOTB / Metatron trinary opcodes |
| License | MIT OR Apache-2.0 |

## Kernel stack (what boots today)

```text
┌──────────────────────────────────────────────────────────────┐
│  FSOT REALITY OS KERNEL  (Rust no_std + QEMU)  — OUR CODE    │
│  bootloader → console → S → HW → domains(530)                │
│  → heap (map_physical_memory) → hello.fsotb                  │
│  → ready-queue → IDT / PIC / IRQ0 (PIT)                      │
├──────────────────────────────────────────────────────────────┤
│  reality_os_scalar    S + domain table                       │
│  reality_os_hw        collapse θ / trit pack / VRAM          │
│  reality_os_trinary   FSOTB interpreter + wire loader        │
│  reality_os_mem       frames + heap                          │
│  reality_os_sched     domain ready-queue + preemption        │
│  reality_os_kernel    IDT + PIC + entry                      │
├──────────────────────────────────────────────────────────────┤
│  Reference only (not linked into the kernel):                │
│  Ubuntu / Linux docs & trees — schematic study               │
├──────────────────────────────────────────────────────────────┤
│  Host tools (optional): residual CLI, plant monitor on a     │
│  workstation OS — sensors/dev, not the product kernel        │
└──────────────────────────────────────────────────────────────┘
```

## Proved under QEMU (v0.5)

| Check | Result |
|-------|--------|
| Domains | 530 residual_finite |
| Heap | write/readback OK |
| hello.fsotb | magic/seeds/decode/run, tag=42 |
| IRQ0 | IDT + PIC, hardware firings |
| Overall | `FSOT_ROS_OVERALL=ok` |

## License

Dual **MIT OR Apache-2.0**. Linux is GPLv2; we study it, we do not relicense Reality OS as GPLv2 by default.
