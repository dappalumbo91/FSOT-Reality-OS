#!/usr/bin/env python3
"""Generate full Reality OS domain table from monorepo coverage.

Includes EVERY domain we actually cover — not a lazy 35-core subset:

  1. All `domain_interfaces` rows in fsot_atlas.sqlite (core + extension)
  2. All green residual domains from benchmark_margin_audit.json
  3. All neurolab core domains from scientific_domain_expansion_map.json

D_eff / factor resolution order:
  interface row → matching data/*_benchmark.json → DOMAIN_FACTORS → defaults
"""

from __future__ import annotations

import json
import os
import sqlite3
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def resolve_mono() -> Path:
    candidates = [
        Path(os.environ.get("FSOT_MONOREPO_ROOT", "")),
        ROOT.parent / "FSOT-2.1-Lean",
        Path(r"C:\Users\damia\Desktop\FSOT-2.1-Lean"),
    ]
    for c in candidates:
        if c and (c / "data" / "fsot_atlas.sqlite").is_file():
            return c.resolve()
    raise SystemExit("FSOT monorepo not found (set FSOT_MONOREPO_ROOT)")


def f64(x, default: float = 0.0) -> float:
    if x is None:
        return default
    try:
        return float(x)
    except (TypeError, ValueError):
        return default


def normalize_domain_name(name: str) -> str:
    """Strip benchmark file noise so registry names are domain IDs."""
    s = str(name).strip()
    if s.lower().endswith(".json"):
        s = s[: -len(".json")]
    for suffix in ("_benchmark", "-benchmark", "_panel_benchmark"):
        if s.lower().endswith(suffix):
            s = s[: -len(suffix)]
            break
    # Title-ish keep underscores (FSOT domain style)
    return s


def load_benchmark_index(mono: Path) -> dict[str, dict]:
    """Map domain name / file stem → {D_eff, factor, domain}."""
    idx: dict[str, dict] = {}
    for p in (mono / "data").glob("*_benchmark.json"):
        try:
            doc = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        if not isinstance(doc, dict):
            continue
        dname = doc.get("domain") or doc.get("panel") or p.stem.replace("_benchmark", "")
        entry = {
            "domain": str(dname),
            "D_eff": doc.get("D_eff", doc.get("d_eff")),
            "factor": doc.get("domain_factor", doc.get("factor")),
            "file": p.name,
        }
        keys = {
            str(dname),
            str(dname).replace(" ", "_"),
            p.stem.replace("_benchmark", ""),
            p.stem.replace("_benchmark", "").replace("-", "_"),
        }
        for k in keys:
            if k:
                idx[k] = entry
                idx[k.lower()] = entry
    return idx


def load_domain_factors(mono: Path) -> dict[str, float]:
    sys.path.insert(0, str(mono / "scripts"))
    try:
        from fsot_api_predict_lib import DOMAIN_FACTORS  # type: ignore

        return {str(k): float(v) for k, v in DOMAIN_FACTORS.items()}
    except Exception:
        return {}


def main() -> int:
    mono = resolve_mono()
    bench_idx = load_benchmark_index(mono)
    factors = load_domain_factors(mono)

    # --- 1) atlas interfaces ---
    con = sqlite3.connect(str(mono / "data" / "fsot_atlas.sqlite"))
    cur = con.cursor()
    cur.execute(
        """
        SELECT domain, kind, d_eff, hits, delta_psi, observed, domain_factor_f
        FROM domain_interfaces
        """
    )
    iface_rows = cur.fetchall()
    con.close()

    table: dict[str, dict] = {}

    def upsert(name: str, **fields) -> None:
        name = normalize_domain_name(name)
        if not name:
            return
        cur_row = table.get(name, {"domain": name})
        for k, v in fields.items():
            if k.startswith("_"):
                continue
            if v is not None and (k not in cur_row or cur_row[k] is None):
                cur_row[k] = v
            elif v is not None and k in ("d_eff", "factor", "kind") and fields.get("_force"):
                cur_row[k] = v
        # prefer non-null overwrites for authoritative iface fields
        if fields.get("_from") == "iface":
            for k, v in fields.items():
                if k.startswith("_"):
                    continue
                if v is not None:
                    cur_row[k] = v
        cur_row["domain"] = name
        table[name] = cur_row

    for domain, kind, d_eff, hits, delta_psi, observed, factor in iface_rows:
        upsert(
            domain,
            kind=kind or "extension",
            d_eff=d_eff,
            hits=hits,
            delta_psi=delta_psi,
            observed=observed,
            factor=factor,
            _from="iface",
        )

    # --- 2) neurolab core 35 ---
    em_path = mono / "data" / "scientific_domain_expansion_map.json"
    if em_path.is_file():
        em = json.loads(em_path.read_text(encoding="utf-8"))
        for row in em.get("neurolab_domains") or []:
            if isinstance(row, dict):
                name = row.get("domain")
            else:
                name = row
            if not name:
                continue
            upsert(name, kind="core", _from="neurolab")

    # --- 3) all green margin domains (full residual coverage) ---
    margin_path = mono / "data" / "benchmark_margin_audit.json"
    margin_names: list[str] = []
    if margin_path.is_file():
        mdoc = json.loads(margin_path.read_text(encoding="utf-8"))
        for row in mdoc.get("all_domains") or []:
            if not isinstance(row, dict) or row.get("excluded"):
                continue
            if row.get("green_gate_pass") is False:
                continue
            name = row.get("domain")
            if not name:
                # derive from file
                f = row.get("file") or ""
                name = Path(str(f)).stem.replace("_benchmark", "")
            if name:
                margin_names.append(str(name))
                upsert(str(name), kind=table.get(str(name), {}).get("kind") or "extension")

    # --- resolve D_eff / factor for everyone ---
    for name, row in table.items():
        be = bench_idx.get(name) or bench_idx.get(name.lower())
        if row.get("d_eff") is None and be and be.get("D_eff") is not None:
            row["d_eff"] = be["D_eff"]
        if row.get("factor") is None:
            if be and be.get("factor") is not None:
                row["factor"] = be["factor"]
            elif name in factors:
                row["factor"] = factors[name]
            else:
                # common route aliases
                for k, v in factors.items():
                    if k.lower() in name.lower() or name.lower() in k.lower():
                        row["factor"] = v
                        break
        if row.get("d_eff") is None:
            row["d_eff"] = 12.0
        if row.get("factor") is None:
            row["factor"] = 0.001
        if row.get("hits") is None:
            row["hits"] = 0.0
        if row.get("delta_psi") is None:
            row["delta_psi"] = 1.0
        if row.get("observed") is None:
            row["observed"] = 1
        if not row.get("kind"):
            row["kind"] = "extension"

    # KernelInit always present for boot
    if "KernelInit" not in table:
        upsert(
            "KernelInit",
            kind="core",
            d_eff=8.0,
            hits=0.0,
            delta_psi=0.7,
            observed=1,
            factor=0.0,
            _from="iface",
        )
        table["KernelInit"].update(
            {
                "kind": "core",
                "d_eff": 8.0,
                "hits": 0.0,
                "delta_psi": 0.7,
                "observed": 1,
                "factor": 0.0,
            }
        )

    rows = sorted(table.values(), key=lambda r: (str(r.get("kind") or ""), str(r["domain"])))

    # --- emit Rust ---
    lines: list[str] = [
        "//! AUTO-GENERATED full FSOT domain interface table for Reality OS kernel.",
        f"//! Monorepo: {mono.as_posix()}",
        "//! Sources: domain_interfaces + green margin domains + neurolab core",
        "//! Regenerate: python scripts/gen_domain_table_from_monorepo.py",
        "//!",
        "//! Coverage: ALL covered domains (not a 35-subset).",
        "",
        "/// One dimensional interface registered in the OS domain table.",
        "#[derive(Clone, Copy)]",
        "pub struct DomainIface {",
        "    pub name: &'static str,",
        "    pub kind: &'static str,",
        "    pub d_eff: f64,",
        "    pub hits: f64,",
        "    pub delta_psi: f64,",
        "    pub observed: bool,",
        "    /// Preregistered residual factor f (seed route — not free-fit).",
        "    pub factor: f64,",
        "}",
        "",
        f"pub const DOMAIN_COUNT: usize = {len(rows)};",
        "",
        "/// Full domain table — every covered scientific interface.",
        "pub const DOMAIN_TABLE: &[DomainIface] = &[",
    ]

    for r in rows:
        name = str(r["domain"]).replace("\\", "\\\\").replace('"', '\\"')
        kind_s = str(r.get("kind") or "extension").replace('"', "")
        d = f64(r.get("d_eff"), 12.0)
        h = f64(r.get("hits"), 0.0)
        dp = f64(r.get("delta_psi"), 1.0)
        obs = bool(r.get("observed")) if r.get("observed") is not None else True
        f = f64(r.get("factor"), 0.001)
        lines.append(
            '    DomainIface { name: "'
            + name
            + '", kind: "'
            + kind_s
            + '", d_eff: '
            + repr(d)
            + ", hits: "
            + repr(h)
            + ", delta_psi: "
            + repr(dp)
            + ", observed: "
            + ("true" if obs else "false")
            + ", factor: "
            + repr(f)
            + " },"
        )

    lines.extend(
        [
            "];",
            "",
            "pub fn count_by_kind(kind: &str) -> usize {",
            "    let mut n = 0usize;",
            "    let mut i = 0usize;",
            "    while i < DOMAIN_TABLE.len() {",
            "        if DOMAIN_TABLE[i].kind.as_bytes() == kind.as_bytes() {",
            "            n += 1;",
            "        }",
            "        i += 1;",
            "    }",
            "    n",
            "}",
            "",
        ]
    )

    rust_path = ROOT / "kernel" / "crates" / "reality_os_scalar" / "src" / "domains.rs"
    rust_path.parent.mkdir(parents=True, exist_ok=True)
    rust_path.write_text("\n".join(lines) + "\n", encoding="utf-8")

    json_path = ROOT / "data" / "domain_table_full.json"
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(
        json.dumps(
            {
                "count": len(rows),
                "monorepo": str(mono),
                "sources": [
                    "domain_interfaces",
                    "benchmark_margin_audit green domains",
                    "neurolab_domains",
                    "benchmark D_eff fill",
                ],
                "margin_green_names": len(set(margin_names)),
                "iface_rows": len(iface_rows),
                "domains": [
                    {
                        "domain": r["domain"],
                        "kind": r.get("kind"),
                        "d_eff": r.get("d_eff"),
                        "factor": r.get("factor"),
                        "hits": r.get("hits"),
                        "delta_psi": r.get("delta_psi"),
                        "observed": r.get("observed"),
                    }
                    for r in rows
                ],
            },
            indent=2,
        ),
        encoding="utf-8",
    )

    core_n = sum(1 for r in rows if r.get("kind") == "core")
    ext_n = sum(1 for r in rows if r.get("kind") == "extension")
    print(f"Wrote {rust_path} ({rust_path.stat().st_size} bytes)")
    print(f"Wrote {json_path}")
    print(
        f"DOMAIN_COUNT={len(rows)} core={core_n} extension={ext_n} "
        f"other={len(rows) - core_n - ext_n}"
    )
    print(f"margin_green_unique≈{len(set(margin_names))} iface={len(iface_rows)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
