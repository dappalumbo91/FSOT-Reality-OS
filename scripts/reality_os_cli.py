#!/usr/bin/env python3
"""FSOT Reality OS CLI — standalone entry (independent of FSOT-2.1-Lean tree)."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from reality_os.core import (  # noqa: E402
    boot_message,
    compute_domain_S,
    compute_S_raw,
    list_core_interfaces,
    linux_os_path,
    matter_status,
    quantum_status,
    residual_predict,
    snapshot,
    trinary_status,
)


def main() -> int:
    p = argparse.ArgumentParser(
        description="FSOT Reality OS — fluid-spacetime host kernel (standalone)"
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    sub.add_parser("boot", help="Boot banner").set_defaults(
        func=lambda a: (print(boot_message()), 0)[1]
    )
    sub.add_parser("snapshot", help="Full JSON snapshot").set_defaults(
        func=lambda a: (print(json.dumps(snapshot(), indent=2)), 0)[1]
    )
    sub.add_parser("interfaces", help="35 core domain interfaces").set_defaults(
        func=lambda a: (print(json.dumps(list_core_interfaces(), indent=2)), 0)[1]
    )
    sub.add_parser("quantum", help="Quantum core coverage").set_defaults(
        func=lambda a: (print(json.dumps(quantum_status(), indent=2)), 0)[1]
    )
    sub.add_parser("trinary", help="Trinary string syntax / Metatron ABI").set_defaults(
        func=lambda a: (print(json.dumps(trinary_status(), indent=2)), 0)[1]
    )
    sub.add_parser("syntax", help="Alias of trinary").set_defaults(
        func=lambda a: (print(json.dumps(trinary_status(), indent=2)), 0)[1]
    )
    sub.add_parser("matter", help="Matter/antimatter duals + eta").set_defaults(
        func=lambda a: (print(json.dumps(matter_status(), indent=2)), 0)[1]
    )
    sub.add_parser("linux-path", help="Linux OS build roadmap").set_defaults(
        func=lambda a: (print(json.dumps(linux_os_path(), indent=2)), 0)[1]
    )

    sp = sub.add_parser("S", help="Domain or raw scalar")
    sp.add_argument("domain", nargs="?")
    sp.add_argument("--d-eff", type=float, default=12.0)
    sp.add_argument("--delta-psi", type=float, default=1.0)
    sp.add_argument("--hits", type=float, default=0.0)
    sp.add_argument("--unobserved", action="store_true")

    def _S(a: argparse.Namespace) -> int:
        if a.domain:
            print(json.dumps({"domain": a.domain, "S": compute_domain_S(a.domain)}, indent=2))
        else:
            print(
                json.dumps(
                    {
                        "D_eff": a.d_eff,
                        "S": compute_S_raw(a.d_eff, a.delta_psi, not a.unobserved, a.hits),
                    },
                    indent=2,
                )
            )
        return 0

    sp.set_defaults(func=_S)

    pp = sub.add_parser("predict", help="c = m (1+|S|f)")
    pp.add_argument("domain", nargs="?", default="Planetary_Science")
    pp.add_argument("measured", nargs="?", type=float, default=1.0)

    def _pred(a: argparse.Namespace) -> int:
        c, err = residual_predict(float(a.measured), a.domain)
        print(
            json.dumps(
                {
                    "domain": a.domain,
                    "S": compute_domain_S(a.domain),
                    "measured": float(a.measured),
                    "computed": c,
                    "error_pct": err,
                    "law": "c = m * (1 + |S| * f)",
                },
                indent=2,
            )
        )
        return 0

    pp.set_defaults(func=_pred)

    args = p.parse_args()
    return int(args.func(args))


if __name__ == "__main__":
    raise SystemExit(main())
