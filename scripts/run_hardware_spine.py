#!/usr/bin/env python3
"""Execute Reality OS hardware spine = monorepo Rust kernels + QEMU.

Python residual CLI is not an OS. This script refuses to pretend otherwise:
it locates FSOT-2.1-Lean and runs the crates/harnesses already there.
"""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def resolve_monorepo() -> Path | None:
    env = os.environ.get("FSOT_MONOREPO_ROOT", "").strip()
    candidates = []
    if env:
        candidates.append(Path(env))
    candidates.extend(
        [
            ROOT.parent / "FSOT-2.1-Lean",
            Path(r"C:\Users\damia\Desktop\FSOT-2.1-Lean"),
            ROOT / "upstream" / "FSOT-2.1-Lean",
        ]
    )
    for c in candidates:
        if (c / "verification" / "rust" / "fsot_hardware_kernel").is_dir() and (
            c / "scripts" / "run_fsot_hardware_bare_metal.py"
        ).is_file():
            return c.resolve()
    return None


def main() -> int:
    ap = argparse.ArgumentParser(description="Run monorepo Rust+QEMU Reality OS spine")
    ap.add_argument("--skip-qemu", action="store_true")
    ap.add_argument(
        "--monorepo",
        type=str,
        default="",
        help="Path to FSOT-2.1-Lean (or set FSOT_MONOREPO_ROOT)",
    )
    args = ap.parse_args()

    mono = Path(args.monorepo).resolve() if args.monorepo else resolve_monorepo()
    if mono is None:
        print(
            json.dumps(
                {
                    "overall_ok": False,
                    "error": (
                        "FSOT-2.1-Lean monorepo not found. Set FSOT_MONOREPO_ROOT or "
                        "pass --monorepo. Reality OS does not reimplement kernels in Python."
                    ),
                },
                indent=2,
            )
        )
        return 2

    steps = [("bare_metal", mono / "scripts" / "run_fsot_hardware_bare_metal.py")]
    if not args.skip_qemu:
        steps.append(
            ("qemu_harness", mono / "scripts" / "run_rust_lean_bridge_qemu_harness.py")
        )

    out: dict = {
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "os_spine": "rust_qemu",
        "formula_shell": "python_pin_D1D38A",
        "monorepo": str(mono),
        "steps": {},
    }
    all_ok = True
    for name, script in steps:
        if not script.is_file():
            out["steps"][name] = {"status": "missing", "path": str(script)}
            all_ok = False
            continue
        proc = subprocess.run(
            [sys.executable, str(script)],
            cwd=str(mono),
            capture_output=True,
            text=True,
            timeout=900,
        )
        out["steps"][name] = {
            "status": "passed" if proc.returncode == 0 else "failed",
            "returncode": proc.returncode,
            "stdout_tail": (proc.stdout or "")[-1500:],
            "stderr_tail": (proc.stderr or "")[-800:],
        }
        if proc.returncode != 0:
            all_ok = False

    out["overall_ok"] = all_ok
    report = ROOT / "data" / "hardware_spine_report.json"
    report.parent.mkdir(parents=True, exist_ok=True)
    report.write_text(json.dumps(out, indent=2), encoding="utf-8")
    print(json.dumps(out, indent=2))
    print(f"Wrote {report}")
    return 0 if all_ok else 1


if __name__ == "__main__":
    raise SystemExit(main())
