#!/usr/bin/env python3
"""Fail when a production `crate::<module>` reference crosses a forbidden
layer boundary (multi-crate roadmap M1-10, #507).

The scan follows the method recorded in `doc/DEPENDENCY-MATRIX.md`: every
`src/**/*.rs` file is read, block and line comments are dropped, `#[cfg(test)]`
items are skipped by brace counting (the same rule `make scan-banned` uses),
and every explicit `crate::<top_level_module>` reference is an edge from the
file's top-level module to that module. Root re-exports such as
`crate::Options` name no module and are ignored; `use crate::{...}` groups,
including multi-line and nested ones, are expanded entry by entry.

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

ARGS = [a for a in sys.argv[1:] if not a.startswith("--")]
SRC = Path(ARGS[0]) / "src" if ARGS else Path.cwd() / "src"

# Module -> target crate layer (ADR-0001 D2). Files under `src/error/` are
# mapped one by one through ERROR_FILE_LAYER (ADR-0001 D6, M1-14); the bare
# `error` entry only covers `src/error/mod.rs`.
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
    "error": "facade",
    "prelude": "facade",
}

# `src/error/<file>.rs` -> target crate layer (ADR-0001 D6).
ERROR_FILE_LAYER = {
    "common": "core",
    "decimal": "core",
    "options": "core",
    "position": "core",
    "trade": "core",
    "interpolation": "math",
    "curves": "math",
    "surfaces": "math",
    "greeks": "pricing",
    "volatility": "pricing",
    "pricing": "pricing",
    "simulation": "simulation",
    "chains": "market",
    "csv": "market",
    "transaction": "analytics",
    "metrics": "analytics",
    "probability": "analytics",
    "strategies": "strategies",
    "graph": "visualization",
    "unified": "facade",
    "mod": "facade",
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
    "facade": set(LAYER_OF.values()) | set(ERROR_FILE_LAYER.values()),
}

# A bare `crate::error::Name` import cannot be attributed to an error file
# without a symbol table, so `error` as a TARGET is accepted from every layer;
# qualified `crate::error::<file>::Name` references are checked against the
# file's layer. Enum variants that still hold a higher layer's error type are
# the documented residue for the 0.22.0 batch (see `src/error/mod.rs`).
ALWAYS_ALLOWED_TARGETS = {"error"}

# Files whose simulation edge is the `synthetic`-gated market capability.
SYNTHETIC_FILES = {
    "chains/generators.rs",
    "series/generators.rs",
    "chains/mod.rs",
    # `ChainError::Simulation` and `From<SimulationError> for ChainError`
    # (ADR-0003 section 4: the payload becomes market-owned after the bump).
    "error/chains.rs",
}

# (source module, target module) -> issue that removes the edge.
# Populated from the state of `main` after the M1 PRs; keep it sorted.
DEFERRED: dict[tuple[str, str], str] = {
    # `SimulationError::GraphError(#[from] GraphError)`: variant removal,
    # ADR-0001 D6, batch behind the 0.22.0 bump.
    ("error/simulation", "error/graph"): "0.22.0 batch (ADR-0001 D6)",
    # `StrategyError::Simulation(Box<SimulationError>)` and the
    # `From<StrategyError> for SimulationError` conversion that lives next to
    # its source: strategies must not depend on simulation once extracted
    # (M1-08, #505); variant removal in the 0.22.0 batch.
    ("error/strategies", "error/simulation"): "0.22.0 batch (ADR-0001 D6, #505)",
    # `ProfitLossRange::new` returns the analytics-owned `ProbabilityError`;
    # the core constructor gets a core-owned error in the 0.22.0 batch
    # (ADR-0001 D6, M1-01).
    ("model", "error/probability"): "0.22.0 batch (ADR-0001 D6, #498)",
    # `Strategable: ... + Graph` (`src/strategies/base.rs`): the
    # visualization supertrait bound leaves the strategy contract in the
    # 0.22.0 batch (M1-08, #505); dropping a supertrait is a public break.
    ("strategies", "visualization"): "0.22.0 batch (#505, Strategable: Graph bound)",
}

MARKER = "// facade-compat:"
EDGE_RE = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)(?:::([a-z_][a-z0-9_]*))?")


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


def scan(src: Path = SRC) -> tuple[dict[tuple[str, str], list[str]], set[tuple[str, str]]]:
    edges: dict[tuple[str, str], list[str]] = {}
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        parts = rel.split("/")
        top = parts[0].removesuffix(".rs")
        if top in ("lib", "prelude"):
            continue
        if top == "error":
            stem = parts[1].removesuffix(".rs") if len(parts) > 1 else "mod"
            source = f"error/{stem}"
            source_layer = ERROR_FILE_LAYER.get(stem)
        else:
            source = top
            source_layer = LAYER_OF.get(top)
        if source_layer is None:
            print(f"unknown module {source!r} in {rel}; add it to LAYER_OF or ERROR_FILE_LAYER", file=sys.stderr)
            sys.exit(2)
        text = "\n".join(production_lines(strip_comments(path.read_text())))
        targets: list[str] = []
        for match in EDGE_RE.finditer(text):
            targets.append(normalise(match.group(1), match.group(2)))
        # `use crate::{a::b, c, d::{e, f}}`, possibly spanning several lines:
        # every entry's first segment names a module (or a root re-export).
        for match in re.finditer(r"crate::\{", text):
            depth, i = 0, match.end() - 1
            while i < len(text):
                if text[i] == "{":
                    depth += 1
                elif text[i] == "}":
                    depth -= 1
                    if depth == 0:
                        break
                i += 1
            body = text[match.end():i]
            for path_segments in expand_group(body):
                if not path_segments:
                    continue
                sub = path_segments[1] if len(path_segments) > 1 else None
                targets.append(normalise(path_segments[0], sub))
        for target in targets:
            if target not in LAYER_OF and not target.startswith("error/"):
                continue
            if target == source:
                continue
            edges.setdefault((source, target), []).append(rel)
    return edges, set(edges)


def normalise(module: str, sub: str | None) -> str:
    """`error` plus a known file stem becomes `error/<stem>`."""
    if module == "error" and sub in ERROR_FILE_LAYER:
        return f"error/{sub}"
    return module


def split_top_level(body: str) -> list[str]:
    """Split a brace-group body on the commas that sit at depth zero."""
    parts, depth, current = [], 0, []
    for ch in body:
        if ch == "{":
            depth += 1
        elif ch == "}":
            depth -= 1
        if ch == "," and depth == 0:
            parts.append("".join(current))
            current = []
        else:
            current.append(ch)
    parts.append("".join(current))
    return parts


def expand_group(body: str) -> list[list[str]]:
    """Flatten `a::{b::c, d::{e, f}}` into `[[a, b, c], [a, d, e], [a, d, f]]`."""
    paths: list[list[str]] = []
    for entry in split_top_level(body):
        entry = entry.strip()
        if not entry:
            continue
        brace = entry.find("{")
        if brace == -1:
            paths.append([seg.strip() for seg in entry.split("::") if seg.strip()])
            continue
        prefix = [seg.strip() for seg in entry[:brace].split("::") if seg.strip()]
        inner = entry[brace + 1 : entry.rfind("}")]
        for tail in expand_group(inner):
            paths.append(prefix + tail)
    return paths


def layer_of(name: str) -> str:
    if name.startswith("error/"):
        return ERROR_FILE_LAYER[name.split("/", 1)[1]]
    return LAYER_OF[name]


def violations_of(edges: dict[tuple[str, str], list[str]]) -> list[str]:
    violations: list[str] = []
    for (src, dst), files in sorted(edges.items()):
        if dst in ALWAYS_ALLOWED_TARGETS:
            continue
        src_layer, dst_layer = layer_of(src), layer_of(dst)
        if dst_layer in ALLOWED[src_layer]:
            continue
        if dst_layer == "simulation" and src_layer == "market" and all(f in SYNTHETIC_FILES for f in files):
            continue
        if (src, dst) in DEFERRED:
            continue
        where = ", ".join(sorted(set(files)))
        violations.append(f"{src} -> {dst} ({src_layer} -> {dst_layer}) in {where}")
    return violations


def self_test() -> int:
    """Prove the scanner sees what it must and ignores what it may."""
    import tempfile

    cases = {
        # (file, content) -> expected violation count
        "forbidden edge": (("model/x.rs", "use crate::pricing::black_scholes;\n"), 1),
        "allowed edge": (("pricing/x.rs", "use crate::model::Options;\n"), 0),
        "comment only": (("model/x.rs", "// use crate::pricing::black_scholes;\n/* crate::chains::X */\n"), 0),
        "test module only": (("model/x.rs", "#[cfg(test)]\nmod t {\n    use crate::pricing::black_scholes;\n}\n"), 0),
        "marked compat re-export": (("model/x.rs", "pub use crate::pricing::black_scholes; // facade-compat: pricing\n"), 0),
        "multiline import": (("model/x.rs", "use crate::{\n    Options,\n    pricing::black_scholes,\n};\n"), 1),
        "nested brace import": (("model/x.rs", "use crate::{error::{strategies::StrategyError, DecimalError}, model::Options};\n"), 1),
        "qualified error path": (("model/x.rs", "use crate::error::strategies::StrategyError;\n"), 1),
        "synthetic file": (("chains/generators.rs", "use crate::simulation::WalkParams;\n"), 0),
    }
    failures = 0
    for name, ((rel, content), expected) in cases.items():
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "src"
            target = root / rel
            target.parent.mkdir(parents=True)
            target.write_text(content)
            edges, _ = scan(root)
            got = len([v for v in violations_of(edges) if (rel.split("/")[0], ) ])
            status = "ok" if got == expected else "FAIL"
            if got != expected:
                failures += 1
            print(f"self-test {status}: {name} (expected {expected}, got {got})")
    return 1 if failures else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    edges, present = scan()
    violations = violations_of(edges)
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
