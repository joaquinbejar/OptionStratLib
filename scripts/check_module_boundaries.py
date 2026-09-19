#!/usr/bin/env python3
"""Fail when a production `crate::<module>` reference crosses a forbidden
layer boundary (multi-crate roadmap M1-10, #507).

The scan follows the method recorded in `doc/DEPENDENCY-MATRIX.md`: every
`src/**/*.rs` file is read, block and line comments are dropped, `#[cfg(test)]`
items are skipped by brace counting (the same rule `make scan-banned` uses),
and every explicit `crate::<top_level_module>` reference is an edge from the
file's top-level module to that module. Root re-exports such as
`crate::Options` name no module and are ignored.

Three kinds of lines are exempt from the layer rule:

* lines carrying `// facade-compat: <layer>`: compatibility re-exports that
  keep a 0.21 path alive from a lower module and become the facade crate's
  own module files at extraction time;
* files listed in `SYNTHETIC_FILES`: the market-to-simulation edge that the
  `synthetic` feature gates (ADR-0003);
* edges listed in `DEFERRED`: known violations whose removal is a breaking
  change batched behind the 0.22.0 bump (ADR-0001, "Sequencing under the
  semver gate"). Each entry names the issue that removes it. A deferred edge
  that no longer exists is reported so the list can be pruned.

Exit status is 1 on any other cross-layer edge, 0 otherwise. Run
`make check-graph`; an optional first argument names the crate root to scan
(default: the current directory).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

SRC = Path(sys.argv[1]) / "src" if len(sys.argv) > 1 else Path.cwd() / "src"

# Module -> target crate layer (ADR-0001 D2). `error` is scanned as a source
# through its own row and accepted as a target from everywhere until M1-14
# partitions it (its removal from the "always allowed" set is tracked there).
LAYER_OF = {
    "model": "core",
    "constants": "core",
    "utils": "core",
    "geometrics": "math",
    "curves": "math",
    "surfaces": "math",
    "pricing": "pricing",
    "greeks": "pricing",
    "volatility": "pricing",
    "simulation": "simulation",
    "chains": "market",
    "series": "market",
    "pnl": "analytics",
    "risk": "analytics",
    "metrics": "analytics",
    "analytics": "analytics",
    "strategies": "strategies",
    "backtesting": "backtest",
    "visualization": "visualization",
    "error": "error",
    "prelude": "facade",
}

# Layer -> layers it may reference (the approved DAG, ADR-0001 D9).
ALLOWED = {
    "core": {"core"},
    "math": {"core", "math"},
    "pricing": {"core", "math", "pricing"},
    "simulation": {"core", "math", "pricing", "simulation"},
    "market": {"core", "math", "pricing", "market"},
    "analytics": {"core", "math", "pricing", "market", "analytics"},
    "strategies": {"core", "math", "pricing", "market", "analytics", "strategies"},
    "backtest": {
        "core", "math", "pricing", "simulation", "market", "analytics",
        "strategies", "backtest",
    },
    "visualization": {
        "core", "math", "pricing", "simulation", "market", "analytics",
        "strategies", "backtest", "visualization",
    },
    # `error` is a leaf in the target graph; its current reverse references
    # are listed in DEFERRED.
    "error": set(),
    "facade": set(LAYER_OF.values()),
}

# Every layer may name `error` until M1-14 partitions it.
ALWAYS_ALLOWED_TARGETS = {"error"}

# Files whose simulation edge is the `synthetic`-gated market capability.
SYNTHETIC_FILES = {
    "chains/generators.rs",
    "series/generators.rs",
    "chains/mod.rs",
}

# (source module, target module) -> issue that removes the edge.
# Populated from the state of `main` after the M1 PRs; keep it sorted.
DEFERRED: dict[tuple[str, str], str] = {
}

MARKER = "// facade-compat:"
EDGE_RE = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)")


def strip_comments(text: str) -> str:
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    return "\n".join(line.split("//")[0] if MARKER not in line else "" for line in text.splitlines())


def production_lines(text: str) -> list[str]:
    """Drop `#[cfg(test)]` items by brace counting."""
    out: list[str] = []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        line = lines[i]
        if line.strip() == "#[cfg(test)]":
            depth = 0
            started = False
            i += 1
            while i < len(lines):
                depth += lines[i].count("{") - lines[i].count("}")
                if depth > 0:
                    started = True
                if started and depth <= 0:
                    break
                if not started and lines[i].rstrip().endswith(";"):
                    break
                i += 1
            i += 1
            continue
        out.append(line)
        i += 1
    return out


def scan() -> tuple[dict[tuple[str, str], list[str]], set[tuple[str, str]]]:
    edges: dict[tuple[str, str], list[str]] = {}
    for path in sorted(SRC.rglob("*.rs")):
        rel = path.relative_to(SRC).as_posix()
        top = rel.split("/")[0].removesuffix(".rs")
        if top in ("lib", "prelude"):
            continue
        source_layer = LAYER_OF.get(top)
        if source_layer is None:
            print(f"unknown top-level module {top!r} in {rel}; add it to LAYER_OF", file=sys.stderr)
            sys.exit(2)
        text = "\n".join(production_lines(strip_comments(path.read_text())))
        for match in EDGE_RE.finditer(text):
            target = match.group(1)
            if target not in LAYER_OF or target == top:
                continue
            edges.setdefault((top, target), []).append(rel)
    return edges, set(edges)


def main() -> int:
    edges, present = scan()
    violations: list[str] = []
    for (src, dst), files in sorted(edges.items()):
        if dst in ALWAYS_ALLOWED_TARGETS:
            continue
        src_layer, dst_layer = LAYER_OF[src], LAYER_OF[dst]
        if dst_layer in ALLOWED[src_layer]:
            continue
        if dst == "simulation" and src_layer == "market" and all(f in SYNTHETIC_FILES for f in files):
            continue
        if (src, dst) in DEFERRED:
            continue
        where = ", ".join(sorted(set(files)))
        violations.append(f"{src} -> {dst} ({src_layer} -> {dst_layer}) in {where}")
    stale = [f"{s} -> {d} ({issue})" for (s, d), issue in DEFERRED.items() if (s, d) not in present]
    if stale:
        print("deferred edges no longer present, prune them from DEFERRED:")
        for item in stale:
            print(f"  {item}")
    if violations:
        print("forbidden module edges (see doc/DEPENDENCY-MATRIX.md, ADR-0001 D9):")
        for item in violations:
            print(f"  {item}")
        return 1
    deferred_count = sum(1 for key in DEFERRED if key in present)
    print(f"OK: no forbidden module edge ({deferred_count} deferred edges tolerated)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
