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

## v0.3 proved (QEMU)

| Check | Result |
|-------|--------|
| Domains walked | **530** residual_finite=530 |
| Memory | usable_frames≈32k, allocated=20, mem_ok |
| Trinary | registry 0–26, selftest r0=7 emit=42 |
| Scheduler | 32 tasks, 64 quanta, sched_ok |
| Overall | `FSOT_ROS_OVERALL=ok` |
