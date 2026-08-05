"""FSOT scalar engine (authority pin D1D38A). Upstream: FSOT-2.1-Lean vendor/fsot_compute.py."""

from .fsot_compute import (  # noqa: F401
    DOMAINS,
    compute_scalar,
    domain_scalar,
    ScalarInput,
    PI,
    E,
    PHI,
    GAMMA,
    G_CAT,
    K,
    POOF,
    C_FACTOR,
)

__all__ = [
    "DOMAINS",
    "compute_scalar",
    "domain_scalar",
    "ScalarInput",
    "PI",
    "E",
    "PHI",
    "GAMMA",
    "G_CAT",
    "K",
    "POOF",
    "C_FACTOR",
]
