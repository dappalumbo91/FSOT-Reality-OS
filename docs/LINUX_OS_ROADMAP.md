# Superseded → Reference OS pathways

This file previously described “attach Reality OS to Linux schematics” as if
Linux code were the product path.

**Current policy:** Ubuntu/Linux are **reference pathways only**. We **do not**
use Linux source as Reality OS. We build our own OS through FSOT.

**Read instead:** [`REFERENCE_OS_PATHWAYS.md`](REFERENCE_OS_PATHWAYS.md)

## Kernel status (implementation lives under `kernel/`)

| Milestone | Status |
|-----------|--------|
| Bare-metal boot + QEMU | done |
| Full domain table (530) | done |
| Heap on mapped physical frames | done |
| hello.fsotb wire load | done |
| Ready-queue + IDT IRQ0 | done |
| Full desktop/server surface | **in progress — FSOT-native only** |

Do not add Linux tree vendors or GPL kernel forks to this repository.
