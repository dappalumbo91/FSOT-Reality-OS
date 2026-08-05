# Architecture

## Hard rule

**Python is not the operating system.**

```text
┌──────────────────────────────────────────────────────────────┐
│  FSOT REALITY OS KERNEL v0.3 (Rust no_std + QEMU)            │
│  bootloader → console → S → HW → domains(530)                │
│            → frame alloc → trinary ISA → domain scheduler    │
├──────────────────────────────────────────────────────────────┤
│  reality_os_scalar    S + full domain table                  │
│  reality_os_hw        collapse θ / trit pack / VRAM          │
│  reality_os_trinary   FSOTB ops 0–26 interpreter             │
│  reality_os_mem       usable-frame bump allocator            │
│  reality_os_sched     cooperative RR domain quanta           │
├──────────────────────────────────────────────────────────────┤
│  Host formula shell (optional Python) — residual only        │
└──────────────────────────────────────────────────────────────┘
```

## v0.4 proved (QEMU)

| Check | Result |
|-------|--------|
| Domains walked | **530** residual_finite=530 |
| Heap | `map_physical_memory` phys_offset set, heap 128 KiB write/read **OK** |
| hello.fsotb | magic/seeds/decode/run **OK**, tag=42, panel S oracle bits |
| Scheduler | **530** tasks ready-queue, 1060 quanta, **1060 preempts** |
| Overall | `FSOT_ROS_VERSION=0.4` · `FSOT_ROS_OVERALL=ok` |
