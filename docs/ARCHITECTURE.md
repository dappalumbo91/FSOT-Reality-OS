# Architecture

## Hard rule

**Python is not the operating system.**

```text
┌────────────────────────────────────────────────────────────┐
│  FSOT REALITY OS KERNEL  (Rust no_std)                     │
│  kernel/crates/reality_os_kernel                           │
│    bootloader → VGA/UART → S → HW gates → domains → QEMU   │
├────────────────────────────────────────────────────────────┤
│  reality_os_scalar   S = K(T1+T2+T3)  pin D1D38A seeds     │
│  reality_os_hw       collapse θ, trit pack, VRAM law       │
├────────────────────────────────────────────────────────────┤
│  Host formula shell (optional Python)                      │
│  residual predict / CLI — never claims ring-0              │
└────────────────────────────────────────────────────────────┘
```

## v0.1 proved

| Check | Result |
|-------|--------|
| `cargo build -p reality_os_kernel --release` | pass |
| `cargo bootimage` | `bootimage-reality_os_kernel.bin` |
| QEMU serial | `FSOT_ROS_OVERALL=ok` |
| Boot scalar | matches canonical `0.0992889562686172` |

## Why this is an OS start (honest)

v0.1 is a **bare-metal kernel binary** that owns boot, console, seed arithmetic,  
hardware laws, and a domain process table. It is not Linux yet. It is not a  
Python script. Next increments: allocator, full domain table, trinary ISA,  
scheduler — still in Rust.
