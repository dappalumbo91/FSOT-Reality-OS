# Reference OS pathways — learn from Ubuntu/Linux, build through FSOT

## Policy (frozen)

| We do | We do **not** |
|-------|----------------|
| Study **Ubuntu**, mainline **Linux**, and other open OS designs as **pathways** | Fork Linux or ship Linux kernel source as Reality OS |
| Map schematic layers (boot, mem, sched, VFS, net, drivers, userspace) to **FSOT-native** components | Wrap or re-route Linux control paths as the product |
| Build **our own** kernel and services under pin **D1D38A** | Treat GPL Linux code as something we paste into this tree |
| Use QEMU / host tools for development | Claim “we run Ubuntu” as the OS |

**One law, own OS:**

\[
S = K(T_1+T_2+T_3),\qquad c = m\,(1+|S|\,f)
\]

Dimensional interfaces (domain table), trinary FSOTB ISA, residual law, and seed pin are the **constitution**.  
Ubuntu/Linux are the **textbook**, not the **codebase**.

---

## Why look at Ubuntu / Linux at all

A full OS is a large surface. Open trees show *what has to exist* and *how pieces connect*:

| Schematic layer (reference) | What we learn | FSOT Reality OS implementation (ours) |
|----------------------------|---------------|----------------------------------------|
| Bootloader / early boot | how control is handed to a kernel | `bootloader` crate + our `reality_os_kernel` entry |
| Physical memory map | usable RAM, frame ownership | `reality_os_mem` + `map_physical_memory` heap |
| Interrupt / timer | preemption, time base | IDT + PIC + **IRQ0** + PIT |
| Scheduler | who runs next | domain ready-queue + quanta (FSOT \(S\) / trit later as policy) |
| Process / task identity | namespaces, isolation ideas | **domain interfaces** (\(D_{\mathrm{eff}}\), factor \(f\)) as first-class tasks |
| Syscall / ABI | stable user↔kernel language | **FSOTB / Metatron** opcodes (not Linux syscall table) |
| Drivers / devices | device model existence | FSOT-native drivers later; study Linux *interfaces*, write our own |
| Userspace / init | services after boot | host plant / services we author; not systemd-as-product |
| VFS / networking | catalog of capabilities | design when needed; still **our** code |

Reading Ubuntu source or docs is **research**. Shipping Reality OS is **authorship under FSOT**.

---

## How we use a reference distro day-to-day

1. **Run** Ubuntu (or similar) as a **workstation** to edit, build, QEMU-boot Reality OS.  
2. **Read** kernel/docs when stuck on a schematic question (e.g. “what does a timer IRQ need?”).  
3. **Translate** the answer into Rust `no_std` + FSOT types — never copy Linux `.c` into this repo.  
4. **Verify** with our gates: pin D1D38A, QEMU serial markers, monorepo residual atlas when applicable.

Host plant tools (e.g. PC monitor under `plant_host_v1`) may run **on** Ubuntu for telemetry. That is a **sensor fold**, not “Ubuntu is Reality OS.”

---

## Explicit non-goals

- Not a Linux distribution  
- Not a Linux kernel module as the main product (would force GPLv2 entanglement)  
- Not “FSOT policy plane on stock Linux” as the definition of done  
- Not free residual coefficients invented per driver  

---

## Definition of done (full OS, FSOT-native)

- [x] Boots under QEMU with FSOT scalar + domain table  
- [x] Own memory allocator path + timer IRQ path  
- [x] Own trinary ISA + wire program load  
- [ ] Richer userspace ABI (FSOTB / native calls) without Linux syscalls  
- [ ] Storage / console / net **drivers we write** (schematic informed by reference OSes)  
- [ ] Installable disk image that is **Reality OS**, not Ubuntu with a tarball  

---

## License alignment

| Artifact | License |
|----------|---------|
| Linux kernel (reference only) | GPLv2-only — we do not redistribute it as our kernel |
| FSOT Reality OS (this repo) | **MIT OR Apache-2.0** — see root `LICENSE` |

Studying Linux does not put Reality OS under GPLv2. Only **shipping** GPL-derived code would.
