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
* edges listed in `DEFERRED`, scoped to the files that carry them: known
  violations whose removal is a breaking change batched behind the 0.22.0
  bump (ADR-0001, "Sequencing under the semver gate"). Each entry names the
  issue that removes it and every such line is annotated `// deferred edge`
  in the source. The same module pair in any other file is a violation. A
  deferred edge that no longer exists is reported so the list can be pruned.

The run also prints the number of `// facade-compat` lines per layer, so
marker creep is visible in the CI log.

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
    # ADR-0001 D9: strategies and backtest have no math edge.
    "strategies": {"core", "pricing", "market", "analytics", "strategies"},
    "backtest": {
        "core", "pricing", "simulation", "market", "analytics",
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
# File-level on purpose: the only simulation references in `chains/mod.rs`
# are the `#[cfg(feature = "synthetic")]` re-exports, and the generator
# modules are compiled only under that feature.
SYNTHETIC_FILES = {
    "chains/generators.rs",
    "series/generators.rs",
    "chains/mod.rs",
    # `ChainError::Simulation` and `From<SimulationError> for ChainError`
    # (ADR-0003 section 4: the payload becomes market-owned after the bump).
    "error/chains.rs",
}

# (source module, target module) -> (files that may carry the edge, the issue
# that removes it). Scoped to files on purpose: a new file introducing the
# same module pair is a fresh violation, not tolerated debt. Every listed
# line is annotated `// deferred edge` in the source.
DEFERRED: dict[tuple[str, str], tuple[frozenset[str], str]] = {
    # Inherent pricing wrappers on `Options` forward to `pricing::OptionPricing`.
    ("model", "pricing"): (frozenset({"model/option.rs"}), "0.22.0 batch (#499, API-BASELINE 3.3)"),
    # `impl LegAble for Leg` computes Greeks in its `Option` arms.
    ("model", "greeks"): (frozenset({"model/leg/leg_enum.rs"}), "0.22.0 batch (#498, ADR-0001 D6)"),
    # `Trade::pnl() -> PnL` is public inherent API returning an analytics type.
    ("model", "pnl"): (frozenset({"model/trade.rs"}), "0.22.0 batch (#498)"),
    # `ProfitLossRange::calculate_probability` wrapper and its parameter types.
    ("model", "analytics"): (frozenset({"model/profit_range.rs"}), "0.22.0 batch (#498)"),
    # `ProfitLossRange::new` returns the analytics-owned `ProbabilityError`.
    ("model", "error/probability"): (frozenset({"model/profit_range.rs"}), "0.22.0 batch (#498, ADR-0001 D6)"),
    # `PricingEngine::MonteCarlo` still stores the concrete `Simulator`.
    ("pricing", "simulation"): (frozenset({"pricing/unified.rs"}), "0.22.0 batch (#508, ADR-0001 D3)"),
    # `Simulate::simulate` returns `SimulationStatsResult`; `SimulationStats`
    # stores `SimulationResult` (both embed analytics types).
    ("simulation", "backtesting"): (frozenset({"simulation/stats.rs", "simulation/traits.rs"}), "0.22.0 batch (#504)"),
    # `impl BasicAble for Simulator/RandomWalk` lives in strategies.
    ("strategies", "simulation"): (frozenset({"strategies/simulation_impls.rs"}), "0.22.0 batch (#505)"),
    # `Strategable: ... + Graph` supertrait bound.
    ("strategies", "visualization"): (frozenset({"strategies/base.rs"}), "0.22.0 batch (#505)"),
    # `SimulationError::GraphError(#[from] GraphError)`.
    ("error/simulation", "error/graph"): (frozenset({"error/simulation.rs"}), "0.22.0 batch (ADR-0001 D6)"),
    # `StrategyError::Simulation(Box<SimulationError>)` and the conversion
    # that lives next to its source.
    ("error/strategies", "error/simulation"): (frozenset({"error/strategies.rs"}), "0.22.0 batch (ADR-0001 D6, #505)"),
}

MARKER = "// facade-compat:"
EDGE_RE = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)(?:::([a-z_][a-z0-9_]*))?")


def strip_comments(text: str) -> str:
    """Drop comments; a `facade-compat` marker exempts only a `pub use` re-export.

    Any other marked line (a plain import, a trait bound) keeps its code and is
    scanned like an unmarked one, so the marker cannot hide a live edge.
    """
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    out = []
    for line in text.splitlines():
        code = line.split("//")[0]
        if MARKER in line and code.strip().startswith("pub use "):
            code = ""
        out.append(code)
    return "\n".join(out)


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
                opens = lines[i].count("{")
                # An item whose braces open and close on the same line
                # (`mod tests {}`) still starts, and ends, the test item.
                if opens:
                    started = True
                depth += opens - lines[i].count("}")
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
        # `use crate::{a::b, c, d::{e, f}}` and the qualified form
        # `use crate::a::{b::c, d}`, possibly spanning several lines: the
        # segments before the brace prefix every entry, and the first segment
        # of the result names a module (or a root re-export).
        for match in re.finditer(r"crate::((?:[a-z_][a-z0-9_]*::)*)\{", text):
            prefix = [seg for seg in match.group(1).split("::") if seg]
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
            for tail in expand_group(body):
                path_segments = prefix + tail
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
        deferred = DEFERRED.get((src, dst))
        if deferred is not None and set(files) <= deferred[0]:
            continue
        where = ", ".join(sorted(set(files)))
        if deferred is not None:
            extra = ", ".join(sorted(set(files) - deferred[0]))
            where = f"{extra} (deferred only for {', '.join(sorted(deferred[0]))})"
        violations.append(f"{src} -> {dst} ({src_layer} -> {dst_layer}) in {where}")
    return violations


def marked_lines(src: Path = SRC) -> dict[str, int]:
    """Count `// facade-compat` lines per source layer so marker creep shows in the log."""
    counts: dict[str, int] = {}
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        parts = rel.split("/")
        top = parts[0].removesuffix(".rs")
        if top == "error":
            layer = ERROR_FILE_LAYER.get(parts[1].removesuffix(".rs") if len(parts) > 1 else "mod", "facade")
        else:
            layer = LAYER_OF.get(top, "facade")
        n = sum(1 for line in path.read_text().splitlines() if MARKER in line)
        if n:
            counts[layer] = counts.get(layer, 0) + n
    return counts


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
        "marked plain import is not exempt": (("model/x.rs", "use crate::pricing::black_scholes; // facade-compat: pricing\n"), 1),
        "multiline import": (("model/x.rs", "use crate::{\n    Options,\n    pricing::black_scholes,\n};\n"), 1),
        "nested brace import": (("model/x.rs", "use crate::{error::{strategies::StrategyError, DecimalError}, model::Options};\n"), 1),
        "qualified error path": (("model/x.rs", "use crate::error::strategies::StrategyError;\n"), 1),
        "qualified group": (("model/x.rs", "use crate::error::{strategies::StrategyError};\n"), 1),
        "multiline qualified group": (("model/x.rs", "use crate::error::{\n    strategies::StrategyError,\n};\n"), 1),
        "empty test module then import": (("model/x.rs", "#[cfg(test)]\nmod tests {}\nuse crate::strategies::Strategy;\n"), 1),
        "test module file then import": (("model/x.rs", "#[cfg(test)]\nmod tests;\nuse crate::strategies::Strategy;\n"), 1),
        "synthetic file": (("chains/generators.rs", "use crate::simulation::WalkParams;\n"), 0),
        "deferred pair in its file": (("pricing/unified.rs", "use crate::simulation::simulator::Simulator;\n"), 0),
        "deferred pair in another file": (("pricing/other.rs", "use crate::simulation::simulator::Simulator;\n"), 1),
    }
    failures = 0
    for name, ((rel, content), expected) in cases.items():
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "src"
            target = root / rel
            target.parent.mkdir(parents=True)
            target.write_text(content)
            edges, _ = scan(root)
            got = len(violations_of(edges))
            status = "ok" if got == expected else "FAIL"
            if got != expected:
                failures += 1
            print(f"self-test {status}: {name} (expected {expected}, got {got})")
    # A deferred entry whose edge is gone must be reported as stale.
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp) / "src"
        (root / "model").mkdir(parents=True)
        (root / "model" / "x.rs").write_text("use crate::error::DecimalError;\n")
        _, present = scan(root)
        stale = [key for key in DEFERRED if key not in present]
        ok = len(stale) == len(DEFERRED)
        print(f"self-test {'ok' if ok else 'FAIL'}: stale deferred entries are detected ({len(stale)} of {len(DEFERRED)})")
        if not ok:
            failures += 1
    return 1 if failures else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    edges, present = scan()
    violations = violations_of(edges)
    stale = [f"{s} -> {d} ({meta[1]})" for (s, d), meta in DEFERRED.items() if (s, d) not in present]
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
    marks = ", ".join(f"{layer}={n}" for layer, n in sorted(marked_lines().items())) or "none"
    print(f"OK: no forbidden module edge ({deferred_count} deferred edges tolerated; facade-compat lines per layer: {marks})")
    return 0


if __name__ == "__main__":
    sys.exit(main())
