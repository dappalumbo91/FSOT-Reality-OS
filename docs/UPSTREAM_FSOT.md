# Upstream: FSOT-2.1-Lean

**Authority monorepo:** https://github.com/dappalumbo91/FSOT-2.1-Lean  

That tree holds:

- Full residual atlas (hundreds of green panels)  
- Multiprover spines (Lean, Coq, Isabelle, SMT, …)  
- Open-science ingest, GR/SM depth, hardware verification kits  
- Atlas SQLite with connective edges  

**This repo** holds the **Reality OS surface**: engine pin + host kernel CLI + OS roadmap.

## What to copy when authority moves

| Source (monorepo) | Destination (this repo) |
|-------------------|-------------------------|
| `vendor/fsot_compute.py` | `engine/fsot_compute.py` |
| `vendor/fsot_compute_AUTHORITY_PIN.json` | `engine/fsot_compute_AUTHORITY_PIN.json` |
| `vendor/fsot_dynamics.py` (optional) | `engine/fsot_dynamics.py` |
| `scripts/fsot_api_predict_lib.py` DOMAIN_FACTORS | `reality_os/residual.py` |
| `vendor/trinary_os/isa/fsotb_opcode_registry.json` | `vendor/trinary_os/isa/` |
| Reality OS improvements | port CLI features here |

## What stays only in monorepo

- Green benchmark JSON corpus  
- Multiprover obligation generation  
- Full scientific catalog multiprover runs  

Reality OS may **consume** published residual tables later as data packs — not required for boot of the host kernel.
