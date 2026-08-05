# Linux → Reality OS roadmap

You meant a real operating system. This document is the build plan.

## Principle

Use **open-source Linux (or a minimal open kernel)** as the *proven schematic* — process model, memory, VFS, networking, drivers, userspace init.  

**Do not** rebuild their physics. **Do** replace or wrap control paths with **FSOT Reality OS** architecture:

- Same pin **D1D38A**  
- Same \(S = K(T_1+T_2+T_3)\)  
- Same residual law \(c = m(1+|S|f)\)  
- Dimensional interfaces as first-class “process / namespace” identity  
- Trinary opcodes as optional native ISA / syscall encoding layer  
- Emerge/damp \(\mathrm{sign}(S)\) as scheduling / admission syntax  

## Candidate open bases (examples)

Pick one primary base (all open code you can read end-to-end):

| Base | Why |
|------|-----|
| Linux kernel + musl/busybox userspace | Full schematic, huge proven surface |
| Linux + Buildroot / Yocto | Reproducible distro builds |
| Linux From Scratch (LFS) | Educational full control |
| seL4 / unikernel later | Optional high-assurance path after host Reality OS is solid |

Start with **host Reality OS + userspace on stock Linux**, then deepen into kernel hooks.

## Phases

### Phase 0 — This repository (done as v0.1)

- [x] Independent engine + CLI  
- [x] Quantum / trinary / matter surfaces  
- [x] Smoke tests  

### Phase 1 — Userspace Reality OS services on Linux

- [ ] `realityosd` daemon: domain routing, residual predict API (Unix socket / REST local)  
- [ ] CLI parity with monorepo Reality OS (`boot`, `S`, `predict`, `trinary`, …)  
- [ ] Config: pin file, domain factor table, interface table  
- [ ] Optional: sync job that pulls green residual tables from FSOT-2.1-Lean releases  

### Phase 2 — Integrate into proven Linux schematics

Pick subsystems and **add** Reality OS capabilities rather than rewriting everything:

| Linux schematic | Reality OS attachment |
|-----------------|------------------------|
| `init` / systemd unit | Boot fabric snapshot; pin check |
| `cgroups` / sched | Optional S-weighted nice / admission (policy layer) |
| `namespaces` | Map to dimensional interface IDs (\(D_{\mathrm{eff}}\), domain) |
| `seccomp` / LSM | Syscall filter profiles labeled by domain family |
| `/proc` or `/sys/fs/reality` | Export live \(S\), pin, interface table |
| networking netfilter | Optional residual-class tagging of flows (research) |

### Phase 3 — Trinary ISA / machine language

- [ ] Userspace trit encoder (`sign(S)` strings, opcode stream)  
- [ ] Emulator for 27 Metatron opcodes on Linux (user mode)  
- [ ] Bridge programs from monorepo `vendor/trinary_os`  

### Phase 4 — Kernel module / eBPF

- [ ] Read-only `realityfs` or eBPF maps exporting fabric state  
- [ ] No free parameters in kernel: only pin-locked constants  

### Phase 5 — Bare metal / QEMU

- [ ] Port `fsot_scalar_kernel` (Rust no_std) from monorepo verification tree  
- [ ] QEMU boot that prints boot scalar (existing monorepo path)  
- [ ] Optional: Reality OS init as PID 1 in a minimal initramfs  

## What we will not do

- Invent free residual coefficients per driver  
- Treat Linux as “wrong physics to throw away” — it is the proven OS schematic  
- Merge multiprover atlas into the kernel — keep verification upstream  

## Sync rule

When FSOT-2.1-Lean advances the pin or residual factors:

1. Copy `engine/fsot_compute.py` + pin JSON  
2. Update `reality_os/residual.py` DOMAIN_FACTORS if needed  
3. Bump version; run `tests/test_smoke.py`  
4. Tag release of Reality OS  

## Success criteria (OS, not metaphor)

- Boots a Linux userspace that runs `reality_os_cli.py boot` as a system service  
- `/sys` or `/run/reality` exposes pin + live core \(S\) table  
- A process can query residual predict without the monorepo tree  
- Trinary string for core ladder is stable across boots for fixed pin  
