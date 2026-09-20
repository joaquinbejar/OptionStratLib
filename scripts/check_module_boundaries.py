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
# file's layer. `error` alone (the module itself, `use crate::error::{self}`)
# names no type and stays allowed; every type reference is resolved to its
# file by `error_types` / `resolve_error_refs` below (#590).
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
    # --- Surfaced by the error-type resolver (#590). Each entry names the
    # issue that owns its resolution; none is new debt, all were invisible
    # because the reference is spelled `crate::error::Name`.
    # `LegAble::pnl_at_price` and `Position::pnl_at_expiration` report
    # `PricingError`; the retype to a core-owned error is the core boundary.
    ("model", "error/pricing"): (
        frozenset({
            "model/leg/traits.rs", "model/leg/leg_enum.rs", "model/leg/spot.rs",
            "model/leg/future.rs", "model/leg/perpetual.rs", "model/position.rs",
        }),
        "retype to a core error (M1 exit proposal D3 row 5, #507)",
    ),
    # The Greek methods of `LegAble` report `GreeksError`; they move to the
    # pricing-owned extension trait with the methods themselves.
    ("model", "error/greeks"): (
        frozenset({
            "model/leg/traits.rs", "model/leg/leg_enum.rs", "model/leg/spot.rs",
            "model/leg/future.rs", "model/leg/perpetual.rs",
        }),
        "0.22.0 batch (#498, ADR-0001 D6)",
    ),
    # `Options::calculate_implied_volatility` wrapper signature.
    ("model", "error/volatility"): (frozenset({"model/option.rs"}), "0.22.0 batch (#499)"),
    # `calculate_optimal_price_range` is a chain helper living in core.
    ("model", "error/chains"): (frozenset({"model/utils.rs"}), "re-home to chains::utils (M1 exit proposal D2, #507)"),
    # `utils::csv` is market-owned (ADR-0001 D2), behind the I/O feature.
    ("utils", "error/csv"): (frozenset({"utils/csv.rs"}), "re-home to market (ADR-0001 D2, #525)"),
    # `process_n_times_iter` returns the facade-level unified `Error`.
    ("utils", "error/unified"): (frozenset({"utils/others.rs"}), "facade-owned or retyped (ADR-0001 D2, #506)"),
    # `MetricsError` is defined under analytics today but depends only on
    # `CurveError`/`SurfaceError`; re-homing it to math dissolves these three.
    ("curves", "error/metrics"): (frozenset({"curves/curve.rs"}), "re-home MetricsError to math (M1 exit proposal D2, #507)"),
    ("surfaces", "error/metrics"): (frozenset({"surfaces/surface.rs"}), "re-home MetricsError to math (M1 exit proposal D2, #507)"),
    ("geometrics", "error/metrics"): (frozenset({"geometrics/analysis/traits.rs"}), "re-home MetricsError to math (M1 exit proposal D2, #507)"),
    # `pub type ResultPoint<P> = Result<P, ChainError>` used by the curve and
    # surface constructors; the alias carries the edge to everyone who
    # re-exports or names it (the `surfaces` uses are `#[cfg(test)]`).
    ("geometrics", "error/chains"): (
        frozenset({
            "geometrics/construction/types.rs",
            "geometrics/construction/mod.rs",
            "geometrics/mod.rs",
        }),
        "retype ResultPoint (M1 exit proposal D3 row 6, #507)",
    ),
    # `generate_ou_process` is a simulation kernel living in volatility.
    ("volatility", "error/simulation"): (frozenset({"volatility/utils.rs"}), "re-home to simulation (M1 exit proposal D2, #507)"),
    # The `synthetic` chain walk driver belongs with the market generators.
    ("simulation", "error/chains"): (frozenset({"simulation/walk_driver.rs"}), "re-home to market synthetic (#512)"),
    # Variant payloads that hold a higher layer's error (ADR-0001 D6, #511):
    # `OptionsError::Greeks`, `CurveError::{Greeks, Graph}`,
    # `SurfaceError::{Greeks, Graph}`, `VolatilityError::Chain`,
    # `SimulationError::{Chain, Strategy}`.
    ("error/options", "error/greeks"): (frozenset({"error/options.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/curves", "error/greeks"): (frozenset({"error/curves.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/curves", "error/graph"): (frozenset({"error/curves.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/surfaces", "error/greeks"): (frozenset({"error/surfaces.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/surfaces", "error/graph"): (frozenset({"error/surfaces.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/volatility", "error/chains"): (frozenset({"error/volatility.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/simulation", "error/chains"): (frozenset({"error/simulation.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
    ("error/simulation", "error/strategies"): (frozenset({"error/simulation.rs"}), "0.22.0 batch (ADR-0001 D6, #511)"),
}

MARKER = "// facade-compat:"
EDGE_RE = re.compile(r"\bcrate::([a-z_][a-z0-9_]*)(?:::([a-z_][a-z0-9_]*))?")


def strip_comments(text: str, *, exempt_marked: bool = True) -> str:
    """Drop comments; a `facade-compat` marker exempts only a `pub use` re-export.

    Any other marked line (a plain import, a trait bound) keeps its code and is
    scanned like an unmarked one, so the marker cannot hide a live edge.
    `exempt_marked=False` keeps the marked re-exports too: the re-export map
    must follow them, because a marked line keeps a type reachable under
    another path and its consumers are not exempt (#590).
    """
    text = re.sub(r"/\*.*?\*/", "", text, flags=re.S)
    out = []
    for line in text.splitlines():
        code = line.split("//")[0]
        if exempt_marked and MARKER in line and code.strip().startswith("pub use "):
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


def module_paths(src: Path) -> set[tuple[str, ...]]:
    """Every module path in the tree, ancestors included (`error` as well as `error::chains`)."""
    modules: set[tuple[str, ...]] = set()
    for path in src.rglob("*.rs"):
        parts = module_path_of(path.relative_to(src).as_posix())
        for i in range(len(parts) + 1):
            modules.add(parts[:i])
    return modules


def module_path_of(rel: str) -> tuple[str, ...]:
    """`model/leg/foo.rs` -> (model, leg, foo); `model/mod.rs` -> (model,); `lib.rs` -> ()."""
    parts = rel.removesuffix(".rs").split("/")
    if parts[-1] == "mod":
        parts = parts[:-1]
    if parts == ["lib"]:
        return ()
    return tuple(parts)


TYPE_DEF_RE = re.compile(r"^\s*pub(?:\([^)]*\))?\s+(?:enum|struct|type)\s+([A-Z][A-Za-z0-9_]*)", re.M)
LOCAL_DEF_RE = re.compile(r"\b(?:enum|struct|type|trait|union)\s+([A-Z][A-Za-z0-9_]*)")
ALIAS_DEF_RE = re.compile(r"^\s*pub(?:\([^)]*\))?\s+type\s+([A-Z][A-Za-z0-9_]*)[^=;]*=\s*([^;]+);", re.M)
USE_RE = re.compile(r"\buse\s+([^;]+);", re.S)
IDENT_RE = re.compile(r"\b([A-Z][A-Za-z0-9_]*)\b")
# `crate::error::Name`, `crate::error::<file>::Name`, or `crate::<mods>::Name`
# in any position (a `use` or an expression path).
QUALIFIED_RE = re.compile(r"\bcrate::((?:[a-z_][a-z0-9_]*::)+)([A-Z][A-Za-z0-9_]*)\b")


def error_types(src: Path = SRC) -> tuple[dict[str, str], set[str]]:
    """Public types defined under `src/error/` -> owning file stem, plus the ambiguous names.

    A name defined in two error files cannot be resolved from a bare
    `crate::error::Name`, so it is reported as ambiguous and every candidate
    is taken (the conservative direction: a real inversion is never silent).
    """
    candidates: dict[str, set[str]] = {}
    for path in sorted((src / "error").glob("*.rs")) if (src / "error").is_dir() else []:
        stem = path.stem
        if stem == "mod":
            continue
        text = "\n".join(production_lines(strip_comments(path.read_text(), exempt_marked=False)))
        for name in TYPE_DEF_RE.findall(text):
            candidates.setdefault(name, set()).add(stem)
    names = {name: sorted(stems)[0] for name, stems in candidates.items()}
    ambiguous = {name for name, stems in candidates.items() if len(stems) > 1}
    AMBIGUOUS_STEMS.clear()
    AMBIGUOUS_STEMS.update({name: sorted(stems) for name, stems in candidates.items() if len(stems) > 1})
    return names, ambiguous


# Error type name -> every file that defines it, when more than one does.
AMBIGUOUS_STEMS: dict[str, list[str]] = {}


def use_entries(text: str) -> list[tuple[list[str], str | None]]:
    """Every `use` declaration in `text`, one entry per imported path.

    Returns `(segments, alias)`; a glob ends in `*`, `self` is kept as a
    segment so `use crate::error::{self as err}` can bind a module alias.
    """
    entries: list[tuple[list[str], str | None]] = []
    for match in USE_RE.finditer(text):
        body = match.group(1).strip()
        brace = body.find("{")
        if brace == -1:
            groups = [[seg.strip() for seg in body.split("::") if seg.strip()]]
        else:
            prefix = [seg.strip() for seg in body[:brace].split("::") if seg.strip()]
            groups = [prefix + tail for tail in expand_group(body[brace + 1 : body.rfind("}")])]
        for segments in groups:
            if not segments:
                continue
            alias = None
            last = segments[-1]
            if " as " in last:
                last, alias = (part.strip() for part in last.split(" as ", 1))
                segments = segments[:-1] + [last]
            entries.append((segments, alias))
    return entries


def absolute(segments: list[str], module: tuple[str, ...], modules: set[tuple[str, ...]] | None = None) -> list[str] | None:
    """Crate-relative segments for a `use` path, or `None` when it names another crate.

    Handles `crate::`, `self::`, `super::` and the uniform (bare-relative)
    form `use chain::OptionChain;` that names a submodule of the current
    module, which is the house style in this crate's `mod.rs` files.
    """
    if not segments:
        return None
    head, rest = segments[0], segments[1:]
    if head == "crate":
        return rest
    if head == "self":
        return list(module) + rest
    if head == "super":
        base = list(module)
        while rest and rest[0] == "super":
            base = base[:-1]
            rest = rest[1:]
        return base[:-1] + rest
    if modules is not None and (tuple(module) + (head,)) in modules:
        return list(module) + segments
    return None


def resolution_tables(
    src: Path, types: dict[str, str]
) -> tuple[dict[tuple[tuple[str, ...], str], str], dict[tuple[tuple[str, ...], str], str]]:
    """`(module, name)` -> error file stem, for public re-exports and for every binding.

    The public table answers "can any module reach this type through that
    path"; the binding table also holds private `use` lines, which a child
    module reaches through `super::`. Both are iterated to a fixed point so a
    re-export of a re-export resolves.
    """
    public: dict[tuple[tuple[str, ...], str], str] = {}
    private: dict[tuple[tuple[str, ...], str], str] = {}
    for name, stem in types.items():
        public[(("error",), name)] = stem
        public[(("error", stem), name)] = stem
    files: list[tuple[tuple[str, ...], list[str], list[str]]] = []
    aliases: list[tuple[tuple[str, ...], str]] = []
    modules = module_paths(src)
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        text = "\n".join(production_lines(strip_comments(path.read_text(), exempt_marked=False)))
        pubs = [m.group(0) for m in re.finditer(r"\bpub(?:\([^)]*\))?\s+use\s+[^;]+;", text, re.S)]
        alls = [m.group(0) for m in re.finditer(r"\buse\s+[^;]+;", text, re.S)]
        files.append((module_path_of(rel), pubs, alls))
        aliases.append((module_path_of(rel), text))
    for table, selector in ((public, 1), (private, 2)):
        changed = True
        while changed:
            changed = False
            for module, pubs, alls in files:
                for decl in (pubs if selector == 1 else alls):
                    for segments, alias in use_entries(decl):
                        abs_path = absolute(segments, module, modules)
                        if not abs_path:
                            continue
                        if abs_path[-1] == "*":
                            prefix = tuple(abs_path[:-1])
                            source = {**public, **table}
                            for (mod, name), stem in list(source.items()):
                                if mod == prefix and (module, name) not in table:
                                    table[(module, name)] = stem
                                    changed = True
                            continue
                        source = public if selector == 1 else {**public, **private}
                        stem = source.get((tuple(abs_path[:-1]), abs_path[-1]))
                        if stem is not None:
                            key = (module, alias or abs_path[-1])
                            if key not in table:
                                table[key] = stem
                                changed = True
    # `pub type Alias<..> = .. SomeError ..` makes the alias a path to the
    # error type for every consumer of the alias. The right-hand side is
    # resolved through the defining file's own bindings, never by spelling:
    # `use std::io::Error; pub type IoResult<T> = Result<T, Error>;` names a
    # foreign type, not this crate's unified `Error`.
    added = True
    while added:
        added = False
        for module, text in aliases:
            local = file_error_bindings(text, module, public, private, modules)
            for alias_name, rhs in ALIAS_DEF_RE.findall(text):
                stem = None
                for match in QUALIFIED_RE.finditer(rhs):
                    mods = tuple(seg for seg in match.group(1).split("::") if seg)
                    stem = public.get((mods, match.group(2)))
                    if stem is not None:
                        break
                if stem is None:
                    for ident in IDENT_RE.findall(rhs):
                        if ident in local:
                            stem = local[ident]
                            break
                if stem is not None and (module, alias_name) not in public:
                    public[(module, alias_name)] = stem
                    added = True
    return public, private


def file_error_bindings(
    text: str,
    module: tuple[str, ...],
    public: dict[tuple[tuple[str, ...], str], str],
    private: dict[tuple[tuple[str, ...], str], str],
    modules: set[tuple[str, ...]],
) -> dict[str, str]:
    """Names this file explicitly binds to a crate error type, by local spelling.

    A name bound from another crate (`use std::io::Error`) is absent, so an
    alias over it is never attributed to a crate error.
    """
    bound: dict[str, str] = {}
    for segments, alias in use_entries(text):
        abs_path = absolute(segments, module, modules)
        if not abs_path or abs_path[-1] in ("*", "self"):
            continue
        mods, name = tuple(abs_path[:-1]), abs_path[-1]
        stem = public.get((mods, name))
        if stem is None and mods == module[: len(mods)]:
            stem = private.get((mods, name))
        if stem is not None:
            bound[alias or name] = stem
    return bound


def resolve_error_refs(
    text: str,
    module: tuple[str, ...],
    types: dict[str, str],
    ambiguous: set[str],
    public: dict[tuple[tuple[str, ...], str], str],
    private: dict[tuple[tuple[str, ...], str], str],
    modules: set[tuple[str, ...]],
) -> set[str]:
    """Error file stems a production file refers to, through any resolvable path."""
    stems: set[str] = set()
    shadowed: set[str] = set()
    glob_names: dict[str, str] = {}
    module_aliases: dict[str, tuple[str, ...]] = {}
    # A type defined in this file shadows a glob-imported name of the same name.
    shadowed.update(LOCAL_DEF_RE.findall(text))

    def lookup(mods: tuple[str, ...], name: str) -> str | None:
        stem = public.get((mods, name))
        if stem is not None:
            return stem
        # A private binding in an ancestor module is visible through `super::`.
        if mods == module[: len(mods)]:
            return private.get((mods, name))
        return None

    def record(name: str, mods: tuple[str, ...]) -> None:
        if name in ambiguous and mods == ("error",):
            stems.update(AMBIGUOUS_STEMS.get(name, []))
            return
        stem = lookup(mods, name)
        if stem is not None:
            stems.add(stem)

    for segments, alias in use_entries(text):
        abs_path = absolute(segments, module, modules)
        local = alias or segments[-1]
        if abs_path is None:
            shadowed.add(local)
            continue
        if not abs_path:
            continue
        if abs_path[-1] == "self":
            module_aliases[local] = tuple(abs_path[:-1])
            continue
        if abs_path[-1] == "*":
            prefix = tuple(abs_path[:-1])
            for (mods, name), stem in {**public, **private}.items():
                if mods == prefix:
                    glob_names[name] = stem
            continue
        if alias and tuple(abs_path) in modules:
            module_aliases[alias] = tuple(abs_path)
            continue
        shadowed.add(local)
        record(abs_path[-1], tuple(abs_path[:-1]))
    for alias_name, mods in module_aliases.items():
        for match in re.finditer(rf"\b{re.escape(alias_name)}::((?:[a-z_][a-z0-9_]*::)*)([A-Z][A-Za-z0-9_]*)\b", text):
            tail = tuple(seg for seg in match.group(1).split("::") if seg)
            record(match.group(2), mods + tail)
    for match in QUALIFIED_RE.finditer(text):
        mods = tuple(seg for seg in match.group(1).split("::") if seg)
        record(match.group(2), mods)
    if glob_names:
        for ident in set(IDENT_RE.findall(text)):
            if ident in glob_names and ident not in shadowed:
                stems.add(glob_names[ident])
    return stems


def scan(src: Path = SRC) -> tuple[dict[tuple[str, str], list[str]], set[tuple[str, str]]]:
    edges: dict[tuple[str, str], list[str]] = {}
    types, ambiguous = error_types(src)
    public, private = resolution_tables(src, types)
    modules = module_paths(src)
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
        # Error types, by the file that defines them (#590).
        for stem in resolve_error_refs(text, module_path_of(rel), types, ambiguous, public, private, modules):
            if stem in ERROR_FILE_LAYER:
                targets.append(f"error/{stem}")
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

    # A minimal `src/error/strategies.rs` so the resolver has a type to own.
    err = ("error/strategies.rs", "pub enum StrategyError { A }\n")

    # Each case is a list of (file, content) pairs, so a fixture can span the
    # defining module, a re-exporting module and the consumer (#590).
    # `(files, expected violations[, substring the violation text must contain])`:
    # the substring pins WHICH file carries the edge, so a case cannot pass on
    # a violation raised somewhere else in the fixture.
    cases: dict[str, tuple, ] = {
        "forbidden edge": ([("model/x.rs", "use crate::pricing::black_scholes;\n")], 1),
        "allowed edge": ([("pricing/x.rs", "use crate::model::Options;\n")], 0),
        "comment only": ([("model/x.rs", "// use crate::pricing::black_scholes;\n/* crate::chains::X */\n")], 0),
        "test module only": ([("model/x.rs", "#[cfg(test)]\nmod t {\n    use crate::pricing::black_scholes;\n}\n")], 0),
        "marked compat re-export": ([("model/x.rs", "pub use crate::pricing::black_scholes; // facade-compat: pricing\n")], 0),
        "marked plain import is not exempt": ([("model/x.rs", "use crate::pricing::black_scholes; // facade-compat: pricing\n")], 1),
        "multiline import": ([("model/x.rs", "use crate::{\n    Options,\n    pricing::black_scholes,\n};\n")], 1),
        "nested brace import": ([err, ("model/x.rs", "use crate::{error::{strategies::StrategyError, DecimalError}, model::Options};\n")], 1),
        "qualified error path": ([err, ("model/x.rs", "use crate::error::strategies::StrategyError;\n")], 1),
        "qualified group": ([err, ("model/x.rs", "use crate::error::{strategies::StrategyError};\n")], 1),
        "multiline qualified group": ([err, ("model/x.rs", "use crate::error::{\n    strategies::StrategyError,\n};\n")], 1),
        "empty test module then import": ([("model/x.rs", "#[cfg(test)]\nmod tests {}\nuse crate::strategies::Strategy;\n")], 1),
        "test module file then import": ([("model/x.rs", "#[cfg(test)]\nmod tests;\nuse crate::strategies::Strategy;\n")], 1),
        "synthetic file": ([("chains/generators.rs", "use crate::simulation::WalkParams;\n")], 0),
        "deferred pair in its file": ([("pricing/unified.rs", "use crate::simulation::simulator::Simulator;\n")], 0),
        "deferred pair in another file": ([("pricing/other.rs", "use crate::simulation::simulator::Simulator;\n")], 1),
        # --- error-type resolution (#590)
        "bare error import": ([err, ("model/x.rs", "use crate::error::StrategyError;\n")], 1),
        "error import with alias": ([err, ("model/x.rs", "use crate::error::StrategyError as SE;\nfn f() -> SE { todo!() }\n")], 1),
        "error import in a group": ([err, ("model/x.rs", "use crate::error::{DecimalError, StrategyError};\n")], 1),
        "error path in expression position": ([err, ("model/x.rs", "fn f() { let _ = crate::error::StrategyError::A; }\n")], 1),
        # The consumer's module edge (pricing -> curves) is allowed and the
        # re-export itself carries the compat marker, so the single violation
        # can only come from resolving the re-exported type. A marked `pub use`
        # is still followed by the resolver: it keeps a path alive, it does not
        # hide the type from consumers.
        "error re-exported from another module": (
            [
                err,
                ("curves/mod.rs", "pub use crate::error::StrategyError; // facade-compat: strategies\n"),
                ("pricing/x.rs", "use crate::curves::StrategyError;\n"),
            ],
            1,
        ),
        "re-export of a re-export": (
            [
                err,
                ("curves/mod.rs", "pub use crate::error::StrategyError; // facade-compat: strategies\n"),
                ("surfaces/mod.rs", "pub use crate::curves::StrategyError; // facade-compat: strategies\n"),
                ("pricing/x.rs", "use crate::surfaces::StrategyError;\n"),
            ],
            1,
        ),
        "super import inside a submodule": (
            [
                err,
                ("model/leg/mod.rs", "use crate::error::StrategyError;\n"),
                ("model/leg/x.rs", "use super::StrategyError;\n"),
            ],
            1,
        ),
        "glob import from the error module": ([err, ("model/x.rs", "use crate::error::*;\nfn f() -> StrategyError { todo!() }\n")], 1),
        "glob import without using the name": ([err, ("model/x.rs", "use crate::error::*;\nfn f() -> u8 { 0 }\n")], 0),
        "foreign Error of the same name": (
            [("error/unified.rs", "pub enum Error { A }\n"), ("model/x.rs", "use std::io::Error;\nfn f() -> Error { todo!() }\n")],
            0,
        ),
        "allowed error direction": ([err, ("strategies/x.rs", "use crate::error::StrategyError;\n")], 0),
        "error module itself": ([err, ("model/x.rs", "use crate::error::{self};\n")], 0),
        "module alias": (
            [err, ("model/x.rs", "use crate::error as errors;\nfn f() -> errors::StrategyError { todo!() }\n")],
            1,
            "model/x.rs",
        ),
        "module alias through self": (
            [err, ("model/x.rs", "use crate::error::{self as errors};\nfn f() -> errors::strategies::StrategyError { todo!() }\n")],
            1,
            "model/x.rs",
        ),
        "uniform (bare-relative) re-export hop": (
            [
                err,
                ("curves/inner.rs", "pub use crate::error::StrategyError;\n"),
                ("curves/mod.rs", "pub use inner::StrategyError; // facade-compat: strategies\n"),
                ("pricing/x.rs", "use crate::curves::StrategyError;\n"),
            ],
            2,
            "pricing/x.rs",
        ),
        "private parent binding seen through super": (
            [
                err,
                ("model/position/mod.rs", "use crate::error::StrategyError;\npub mod child;\n"),
                ("model/position/child.rs", "use super::StrategyError;\nfn f() -> StrategyError { todo!() }\n"),
            ],
            1,
            "model/position/child.rs",
        ),
        "glob name shadowed by a local definition": (
            [err, ("model/x.rs", "use crate::error::*;\npub struct StrategyError;\nfn f() -> StrategyError { todo!() }\n")],
            0,
        ),
        "glob name shadowed by an explicit import": (
            [
                err,
                ("model/local.rs", "pub struct StrategyError;\n"),
                ("model/x.rs", "use crate::error::*;\nuse crate::model::local::StrategyError;\nfn f() -> StrategyError { todo!() }\n"),
            ],
            0,
        ),
        "alias over a foreign error of the same name": (
            [
                ("error/unified.rs", "pub enum Error { A }\n"),
                ("curves/alias.rs", "use std::io::Error;\npub type IoResult<T> = Result<T, Error>;\n"),
                ("pricing/x.rs", "use crate::curves::alias::IoResult;\nfn f() -> IoResult<u8> { todo!() }\n"),
            ],
            0,
        ),
        "alias over a qualified crate error": (
            [
                err,
                ("curves/alias.rs", "pub type StratResult<T> = Result<T, crate::error::strategies::StrategyError>;\n"),
                ("pricing/x.rs", "use crate::curves::alias::StratResult;\nfn f() -> StratResult<u8> { todo!() }\n"),
            ],
            2,
            "pricing/x.rs",
        ),
        "type alias over an error reaches its consumer": (
            [
                err,
                ("curves/alias.rs", "pub type StratResult<T> = Result<T, crate::error::StrategyError>;\n"),
                ("pricing/x.rs", "use crate::curves::alias::StratResult;\nfn f() -> StratResult<u8> { todo!() }\n"),
            ],
            2,
            "pricing/x.rs",
        ),
        "ambiguous error name takes every candidate": (
            [
                ("error/strategies.rs", "pub enum Kind { A }\n"),
                ("error/decimal.rs", "pub enum Kind { A }\n"),
                ("model/x.rs", "use crate::error::Kind;\n"),
            ],
            1,
            "model -> error/strategies",
        ),
        "alias defined in an allowed layer": (
            [err, ("strategies/alias.rs", "pub type StratResult<T> = Result<T, crate::error::StrategyError>;\n")],
            0,
        ),
    }
    failures = 0
    for name, case in cases.items():
        files, expected = case[0], case[1]
        needle = case[2] if len(case) > 2 else None
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "src"
            for rel, content in files:
                target = root / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            edges, _ = scan(root)
            found = violations_of(edges)
            got = len(found)
            ok = got == expected and (needle is None or any(needle in item for item in found))
            if not ok:
                failures += 1
            detail = f" [{needle}]" if needle else ""
            print(f"self-test {'ok' if ok else 'FAIL'}: {name}{detail} (expected {expected}, got {got})")
    # A deferred entry whose edge is gone must be reported as stale.
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp) / "src"
        (root / "model").mkdir(parents=True)
        (root / "model" / "x.rs").write_text("pub fn nothing() {}\n")
        _, present = scan(root)
        stale = [key for key in DEFERRED if key not in present]
        ok = len(stale) == len(DEFERRED)
        print(f"self-test {'ok' if ok else 'FAIL'}: stale deferred entries are detected ({len(stale)} of {len(DEFERRED)})")
        if not ok:
            failures += 1
    return 1 if failures else 0


def inventory(src: Path = SRC) -> int:
    """Print the deferred edges and the facade-compat lines as a table."""
    _, present = scan(src)
    print("| Edge | Files | Owner |")
    print("| --- | --- | --- |")
    for (source, target), (files, issue) in sorted(DEFERRED.items()):
        state = "" if (source, target) in present else " (stale)"
        print(f"| {source} -> {target}{state} | {', '.join(sorted(files))} | {issue} |")
    print()
    print("| File | Source layer | Compat target |")
    print("| --- | --- | --- |")
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        parts = rel.split("/")
        top = parts[0].removesuffix(".rs")
        if top == "error":
            source_layer = ERROR_FILE_LAYER.get(parts[1].removesuffix(".rs") if len(parts) > 1 else "mod", "facade")
        else:
            source_layer = LAYER_OF.get(top, "facade")
        for number, line in enumerate(path.read_text().splitlines(), 1):
            if MARKER in line:
                print(f"| {rel}:{number} | {source_layer} | {line.split(MARKER, 1)[1].strip()} |")
    if AMBIGUOUS_STEMS:
        print()
        print("| Ambiguous error type | Defined in |")
        print("| --- | --- |")
        for name, stems in sorted(AMBIGUOUS_STEMS.items()):
            print(f"| {name} | {', '.join(f'error/{stem}.rs' for stem in stems)} |")
    return 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    if "--inventory" in sys.argv:
        return inventory()
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
