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
* files listed in `SYNTHETIC_FILES` (empty since #524, market is a crate): the market-to-simulation edge that the
  `synthetic` feature gates (ADR-0003);
* edges listed in `DEFERRED`, scoped to the files that carry them: known
  violations whose removal is a breaking change batched behind the 0.22.0
  bump (ADR-0001, "Sequencing under the semver gate"). Each entry names the
  issue that removes it and every such line is annotated `// deferred edge`
  in the source. The same module pair in any other file is a violation. A
  deferred edge that no longer exists is reported so the list can be pruned.

The run also prints the number of `// facade-compat` lines per layer, so
marker creep is visible in the CI log.

Once a layer has its own workspace crate (roadmap M2 onwards), two more
checks apply, both read from `cargo metadata`:

* the crate graph: every `optionstratlib-*` package may depend only on the
  packages of the layers below it (`CRATE_LAYER`, `ALLOWED`), in any
  dependency kind, and never on the `optionstratlib` facade. Cargo then
  rejects at compile time any core type, error payload or public signature
  that names a higher layer, which is what the source scan can only
  approximate;
* the single definition: no file under the facade's `src/` may belong to a
  layer that a workspace crate now owns, so a moved module cannot grow a
  second copy in the facade.

* the foundational crates (`positive`, `expiration_date`, `financial_types`,
  `option_type`): every workspace package that names one asks for the same
  version requirement, the resolved graph holds a single version of each,
  and no component other than core (and, until its modules are extracted,
  the facade) depends on one directly, so every crate agrees on what
  `Positive` or `Side` is (ADR-0001 D8, #515).

* the forbidden packages: each extracted component's resolved normal tree,
  with default features and with all of them, holds none of the packages the
  ADR-0002 fixture table lists as absent (`FORBIDDEN_PACKAGES`, #517).

These read `cargo metadata` and `cargo tree`, so `make check-graph` needs a
Rust toolchain on the PATH; a failing cargo command fails the check.

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
    # The simulation-backed chain and series generators: market data built by
    # the simulation engine, so they sit above both until M5 moves them into
    # `optionstratlib-market` behind its `synthetic` feature (#524).
    "synthetic": "facade",
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
    "metrics": "math",  # depends only on CurveError/SurfaceError (M1 D2)
    "probability": "analytics",
    "strategies": "strategies",
    "backtesting": "backtest",
    "projections": "analytics",
    "graph": "visualization",
    "unified": "facade",
    "mod": "facade",
}

# `src/utils/<file>.rs` -> target crate layer (ADR-0001 D2, M1-09).
# `src/utils` is the one crate-level helper module, so every file in it
# carries an explicit owner rather than inheriting a blanket `core`. A new
# file that is not listed here fails the check, which is what keeps the
# module from drifting back into a catch-all. Domain-local helpers
# (`greeks/utils.rs`, `chains/utils.rs`, ...) are owned by their parent
# module and need no entry.
UTILS_FILE_LAYER = {
    "numeric": "core",
    "rng": "core",
    "time": "core",
    "traits": "core",
    "mod": "core",
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
    "facade": set(LAYER_OF.values()) | set(ERROR_FILE_LAYER.values()) | set(UTILS_FILE_LAYER.values()),
}

# A bare `crate::error::Name` import cannot be attributed to an error file
# without a symbol table, so `error` as a TARGET is accepted from every layer;
# qualified `crate::error::<file>::Name` references are checked against the
# file's layer. `error` alone (the module itself, `use crate::error::{self}`)
# names no type and stays allowed; every type reference is resolved to its
# file by `error_types` / `resolve_error_refs` below (#590).
ALWAYS_ALLOWED_TARGETS = {"error"}

# Facade files whose market-to-simulation edge is the `synthetic`-gated
# capability (ADR-0003, roadmap M1-15). Empty since #524: market is its own
# crate, so the crate graph forbids the edge outright, and the generators
# live in the facade-layer `synthetic` module, which may name both layers.
# `synthetic_gate_violations` still proves that module sits behind the
# feature. A path listed here that no longer exists is reported as stale.
SYNTHETIC_FILES: set[str] = set()

# `#[cfg(feature = "synthetic")]`, however the attribute is spaced.
SYNTHETIC_CFG_RE = re.compile(r'#\[cfg\(feature\s*=\s*"synthetic"\)\]')

# Modules whose minimal (non-`synthetic`) surface must name no simulation
# type, when market files sit in a scanned `src/` (the self-test fixtures).
# The market crate itself is held to it by the crate graph and the
# forbidden-package check (#524).
MINIMAL_MARKET_MODULES = ("chains", "series")

# (source module, target module) -> (files that may carry the edge, the issue
# that removes it). Scoped to files on purpose: a new file introducing the
# same module pair is a fresh violation, not tolerated debt. Every listed
# line is annotated `// deferred edge` in the source.
DEFERRED: dict[tuple[str, str], tuple[frozenset[str], str]] = {
    # `Simulate::simulate` returns `SimulationStatsResult`; `SimulationStats`
    # `impl BasicAble for Simulator/RandomWalk` lives in strategies.
    ("strategies", "simulation"): (frozenset({"strategies/simulation_impls.rs"}), "0.22.0 batch (#505)"),
    # `Strategable: ... + Graph` supertrait bound.
    ("strategies", "visualization"): (frozenset({"strategies/base.rs"}), "0.22.0 batch (#505)"),
}

# Workspace package -> layer (ADR-0001 D1). Every published package carries
# the `optionstratlib` prefix; one missing from this table fails the check.
# Packages without the prefix (`osl-example-*`, `osl-fixture-*`, the example
# crates) are consumers, not components, and are not checked.
CRATE_LAYER = {
    "optionstratlib-core": "core",
    "optionstratlib-math": "math",
    "optionstratlib-pricing": "pricing",
    "optionstratlib-simulation": "simulation",
    "optionstratlib-market": "market",
    "optionstratlib-analytics": "analytics",
    "optionstratlib-strategies": "strategies",
    "optionstratlib-backtest": "backtest",
    "optionstratlib-visualization": "visualization",
    "optionstratlib": "facade",
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
    # The facade's own error files, plus those of the extracted component
    # crates: the facade re-exports their types through `crate::error`, and
    # their file stems keep the layer `ERROR_FILE_LAYER` gives them (#524).
    error_files = sorted((src / "error").glob("*.rs")) if (src / "error").is_dir() else []
    error_files += sorted(src.parent.glob("crates/*/src/error/*.rs"))
    for path in error_files:
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
    # One fixed point over both passes: a `pub use` can make an alias
    # resolvable and an alias can make a later `pub use` resolvable, so the
    # two must be iterated together, not one after the other.
    progress = True
    while progress:
        progress = False
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
                                        changed = progress = True
                                continue
                            source = public if selector == 1 else {**public, **private}
                            stem = source.get((tuple(abs_path[:-1]), abs_path[-1]))
                            if stem is not None:
                                key = (module, alias or abs_path[-1])
                                if key not in table:
                                    table[key] = stem
                                    changed = progress = True
        # `pub type Alias<..> = .. SomeError ..` makes the alias a path to the
        # error type for every consumer of the alias. The right-hand side is
        # resolved through the defining file's own bindings, never by
        # spelling: `use std::io::Error; pub type IoResult<T> = Result<T,
        # Error>;` names a foreign type, not this crate's unified `Error`.
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
                    progress = True
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


def module_targets(text: str) -> list[str]:
    """Every `crate::<module>` reference in `text`, grouped forms expanded.

    Shared by the layer scan and the `synthetic` gate check so the two cannot
    disagree about what counts as a reference: a plain `crate::simulation::X`
    and a grouped `use crate::{simulation::X}` resolve to the same target.
    """
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
    return targets


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
        elif top == "utils":
            stem = parts[1].removesuffix(".rs") if len(parts) > 1 else "mod"
            source = f"utils/{stem}"
            source_layer = UTILS_FILE_LAYER.get(stem)
        else:
            source = top
            source_layer = LAYER_OF.get(top)
        if source_layer is None:
            print(
                f"unknown module {source!r} in {rel}; add it to LAYER_OF, "
                "ERROR_FILE_LAYER or UTILS_FILE_LAYER",
                file=sys.stderr,
            )
            sys.exit(2)
        text = "\n".join(production_lines(strip_comments(path.read_text())))
        targets: list[str] = list(module_targets(text))
        # Error types, by the file that defines them (#590).
        for stem in resolve_error_refs(text, module_path_of(rel), types, ambiguous, public, private, modules):
            if stem in ERROR_FILE_LAYER:
                targets.append(f"error/{stem}")
        for target in targets:
            if (
                target not in LAYER_OF
                and not target.startswith("error/")
                and not target.startswith("utils/")
            ):
                continue
            if target == source:
                continue
            edges.setdefault((source, target), []).append(rel)
    return edges, set(edges)


def normalise(module: str, sub: str | None) -> str:
    """`error`/`utils` plus a known file stem becomes `<module>/<stem>`."""
    if module == "error" and sub in ERROR_FILE_LAYER:
        return f"error/{sub}"
    if module == "utils" and sub in UTILS_FILE_LAYER:
        return f"utils/{sub}"
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
    if name.startswith("utils/"):
        return UTILS_FILE_LAYER[name.split("/", 1)[1]]
    return LAYER_OF[name]


STRING_RE = re.compile(r'"(?:[^"\\]|\\.)*"|\'(?:[^\'\\]|\\.)*\'')

MOD_DECL_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+(\w+)\s*;")

# An attribute line, including the opening line of a multi-line attribute.
ATTR_START_RE = re.compile(r"^\s*#!?\[")


def _code_only(line: str) -> str:
    """The line with string and char literals blanked out.

    Brace counting drives the scope tracker below, and a format placeholder
    inside an attribute (`#[error("... {reason}")]`) or a message body would
    otherwise open or close a scope that does not exist.
    """
    return STRING_RE.sub('""', line)


def _items(lines: list[str]):
    """Yield `(attributes, head_index, end_index)` per top-of-scope item.

    An "item" is anything an attribute can sit on: a `mod` declaration, a
    `fn`, an `impl`, a `struct`, an enum variant, a `use`. Attributes are
    attached to the item that *follows* them, never to a sliding window, so a
    gated sibling cannot lend its gate to the next one. The item's extent runs
    to the matching close brace when it opens one, and to the terminating `;`
    or `,` when it does not, so a reference anywhere inside a gated item's
    body counts as gated however deep it sits.
    """
    pending: list[str] = []
    pending_start: int | None = None
    i = 0
    n = len(lines)
    while i < n:
        raw = lines[i]
        line = _code_only(raw)
        if not line.strip():
            i += 1
            continue
        if ATTR_START_RE.match(line):
            if pending_start is None:
                pending_start = i
            # An attribute may span several lines; consume until its brackets
            # balance so the item head is the line after it, not inside it.
            depth = line.count("[") - line.count("]")
            attr = [raw]
            while depth > 0 and i + 1 < n:
                i += 1
                attr.append(lines[i])
                inner = _code_only(lines[i])
                depth += inner.count("[") - inner.count("]")
            pending.append("\n".join(attr))
            i += 1
            continue

        head = i
        # Walk to the end of this item.
        depth = line.count("{") - line.count("}")
        opened = "{" in line
        end = i
        if not opened and not line.rstrip().endswith((";", ",")):
            # A head that spans several lines before its brace or terminator.
            while end + 1 < n:
                end += 1
                inner = _code_only(lines[end])
                depth += inner.count("{") - inner.count("}")
                if "{" in inner:
                    opened = True
                    break
                if inner.rstrip().endswith((";", ",")) and depth <= 0:
                    break
        if opened:
            while depth > 0 and end + 1 < n:
                end += 1
                inner = _code_only(lines[end])
                depth += inner.count("{") - inner.count("}")

        yield pending, pending_start if pending_start is not None else head, head, end
        pending = []
        pending_start = None
        i = end + 1


def _gated_line_numbers(lines: list[str]) -> set[int]:
    """Indices of lines that sit inside a `synthetic`-gated item.

    Recursive, because an item nested in a gated one is gated too: the whole
    extent of a gated item is marked, and the contents of every other item are
    scanned for gated items of their own.
    """
    gated: set[int] = set()

    def walk(offset: int, body: list[str]) -> None:
        for attrs, attr_start, head, end in _items(body):
            if any(SYNTHETIC_CFG_RE.search(a) for a in attrs):
                gated.update(range(offset + attr_start, offset + end + 1))
                continue
            if end > head:
                walk(offset + head + 1, body[head + 1 : end])

    walk(0, lines)
    return gated


def gated_module_files(src: Path = SRC) -> set[str]:
    """Files whose `mod` declaration carries the `synthetic` gate.

    A file brought in by `#[cfg(feature = "synthetic")] mod generators;`
    compiles only under the feature, so every reference in it is gated. The
    declaration is matched through the item walker, so the attribute has to
    sit on *that* declaration and not merely near it.
    """
    gated: set[str] = set()
    for path in src.rglob("*.rs"):
        lines = strip_comments(path.read_text()).splitlines()
        gated_lines = _gated_line_numbers(lines)
        parent = path.parent.relative_to(src).as_posix()
        for i, line in enumerate(lines):
            match = MOD_DECL_RE.match(_code_only(line))
            if not match or i not in gated_lines:
                continue
            name = match.group(1)
            gated.add(f"{parent}/{name}.rs" if parent != "." else f"{name}.rs")
            gated.add(f"{parent}/{name}/mod.rs" if parent != "." else f"{name}/mod.rs")
    return gated


def synthetic_gate_violations(src: Path = SRC) -> list[str]:
    """Prove the market-to-simulation edge really is behind `synthetic`.

    `SYNTHETIC_FILES` says an edge is the optional market capability; this
    says it is *gated*. A production `crate::simulation` reference in market
    code (or in the market-owned `error/chains.rs`) counts as gated when the
    item carrying it sits under `#[cfg(feature = "synthetic")]`, either on
    the item itself, on an item enclosing it, or on the `mod` declaration that
    brings the whole file in. Anything else is in the minimal market surface
    and is reported, so "minimal market names no simulation type" is checked
    rather than asserted (roadmap M1-15).
    """
    gated_modules = gated_module_files(src)

    problems: list[str] = []
    # The facade's `synthetic` module holds the generators until M5 (#524):
    # it must be declared under the feature, which gates every file in it.
    if any(src.glob("synthetic/*.rs")) and "synthetic/mod.rs" not in gated_modules:
        problems.append('synthetic/mod.rs: `mod synthetic` must be declared under #[cfg(feature = "synthetic")]')
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        top = rel.split("/")[0]
        if top not in MINIMAL_MARKET_MODULES and rel != "error/chains.rs":
            continue
        if rel in gated_modules:
            continue
        lines = production_lines(strip_comments(path.read_text()))
        gated_lines = _gated_line_numbers(lines)
        for start, end in _ungated_spans(lines, gated_lines):
            span = lines[start : end + 1]
            text = "\n".join(span)
            # The same extraction the layer scan uses, so a grouped
            # `use crate::{simulation::X}` counts exactly as a plain
            # `use crate::simulation::X` does.
            names_simulation = "simulation" in module_targets(text)
            if not names_simulation and "SimulationError" not in text:
                continue
            for offset, line in enumerate(span):
                if "simulation" in line or "SimulationError" in line:
                    problems.append(f"{rel}:{start + offset + 1}: {line.strip()}")
    return problems


def _ungated_spans(lines: list[str], gated: set[int]) -> list[tuple[int, int]]:
    """Maximal runs of consecutive ungated line indices.

    References are resolved per run rather than per line, because a grouped
    or multi-line `use` spells one reference across several lines. A run never
    straddles a gated item, so a span is entirely inside or entirely outside
    the feature.
    """
    spans: list[tuple[int, int]] = []
    start: int | None = None
    for i in range(len(lines)):
        if i in gated:
            if start is not None:
                spans.append((start, i - 1))
                start = None
            continue
        if start is None:
            start = i
    if start is not None:
        spans.append((start, len(lines) - 1))
    return spans


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
        elif top == "utils":
            layer = UTILS_FILE_LAYER.get(parts[1].removesuffix(".rs") if len(parts) > 1 else "mod", "core")
        else:
            layer = LAYER_OF.get(top, "facade")
        n = sum(1 for line in path.read_text().splitlines() if MARKER in line)
        if n:
            counts[layer] = counts.get(layer, 0) + n
    return counts


def cargo_metadata(root: Path) -> dict:
    """The resolved `cargo metadata` of the workspace (members and every dependency)."""
    import json
    import subprocess

    out = subprocess.run(
        [
            "cargo", "metadata", "--format-version", "1",
            "--manifest-path", str(root / "Cargo.toml"),
        ],
        check=True,
        capture_output=True,
        text=True,
    )
    return json.loads(out.stdout)


def workspace_packages(metadata: dict) -> list[dict]:
    """Workspace members with their declared dependencies."""
    members = set(metadata["workspace_members"])
    return [p for p in metadata["packages"] if p["id"] in members]


# The standalone crates that define the shared newtypes and enums (ADR-0001
# D8). Every workspace package that names one must ask for the same version
# requirement, and the resolved graph must hold a single version of each, or
# two OptionStratLib crates could disagree on what `Positive` is (#515).
FOUNDATIONAL = ("positive", "expiration_date", "financial_types", "option_type")

# Components allowed a normal dependency on a foundational crate. ADR-0001 D8
# names core alone; every other component imports through
# `optionstratlib_core`. The facade keeps its direct dependencies while its
# own modules still write `use positive::...`: ADR-0001 D8 rewrites those
# imports as each module is extracted, and the entry goes with the last one.
FOUNDATIONAL_DEPENDENTS = {"optionstratlib-core", "optionstratlib"}


# External packages that must not appear in a component's resolved normal
# dependency tree (`cargo tree -p <crate> -e normal`). Core's list is the
# "must be absent" column of the ADR-0002 `osl-fixture-core-only` row; ADR-0002
# has no math row, so math's list is the `osl-fixture-pricing-only` row plus
# ADR-0002 §3 ("no visualization, no I/O") plus `plotters`. Other
# `optionstratlib-*` crates are excluded by `crate_graph_violations`, not
# here. Checked with default features
# and with `--all-features`, so neither an optional dependency nor a feature
# can bring one in (#517). `utoipa` is missing from both lists only because
# `expiration_date` 0.4.0 forces `positive/utoipa` on every build (#628); it
# goes back in once that is fixed upstream.
FORBIDDEN_PACKAGES: dict[str, frozenset[str]] = {
    "optionstratlib-core": frozenset({
        "statrs", "rayon", "csv", "zip", "tokio", "reqwest", "plotly", "plotly_static",
        "tracing-subscriber", "indicatif", "prettytable-rs",
    }),
    "optionstratlib-math": frozenset({
        "csv", "zip", "tokio", "reqwest", "plotly", "plotly_static", "plotters",
        "fantoccini", "webdriver", "tracing-subscriber", "indicatif", "prettytable-rs",
    }),
    # The "must be absent" column of ADR-0002's `osl-fixture-pricing-only`
    # row, plus the presentation crates math also excludes.
    "optionstratlib-pricing": frozenset({
        "csv", "zip", "tokio", "reqwest", "plotly", "plotly_static", "plotters",
        "fantoccini", "webdriver", "tracing-subscriber", "indicatif", "prettytable-rs",
    }),
    # ADR-0002 `osl-fixture-market-minimal` row. `csv` and `zip` are still
    # mandatory until #525 gates them behind `io`.
    "optionstratlib-market": frozenset({
        "tokio", "reqwest", "futures", "plotly", "plotly_static", "plotters",
        "fantoccini", "webdriver", "tracing-subscriber", "indicatif",
    }),
}

# Packages a component's optional features are allowed to bring in: removed
# from its forbidden set for the all-features tree only.
FEATURE_ALLOWED_PACKAGES: dict[str, frozenset[str]] = {
    # `async` = `tokio`-backed `*_async` readers and writers.
    "optionstratlib-market": frozenset({"tokio"}),
}


def resolved_tree(root: Path, crate: str, all_features: bool) -> set[str]:
    """Package names in `crate`'s normal dependency tree."""
    import subprocess

    command = [
        "cargo", "tree", "-p", crate, "-e", "normal", "--prefix", "none", "--format", "{p}",
        "--manifest-path", str(root / "Cargo.toml"),
    ]
    if all_features:
        command.append("--all-features")
    out = subprocess.run(command, check=True, capture_output=True, text=True)
    return {line.split()[0] for line in out.stdout.splitlines() if line.strip()}


def forbidden_package_violations(trees: dict[tuple[str, str], set[str]]) -> list[str]:
    """`(crate, feature set) -> resolved packages`, checked against FORBIDDEN_PACKAGES."""
    found = []
    for (crate, features), packages in sorted(trees.items()):
        forbidden = FORBIDDEN_PACKAGES.get(crate, frozenset())
        if features == "all features":
            forbidden = forbidden - FEATURE_ALLOWED_PACKAGES.get(crate, frozenset())
        for name in sorted(packages & forbidden):
            found.append(f"{crate} ({features}) resolves {name}")
    return found


# Allowed internal module edges inside an extracted crate, per top-level
# module (the file stem directly under `src/`). A module may always reference
# itself. Production code only: `#[cfg(test)]` items are skipped, as in the
# facade scan. In `optionstratlib-pricing`, `kernels` holds the formulas the
# pricing models and the Greeks share, so neither imports the other's
# helpers; the numerical Greeks re-price through `pricing`, never the
# reverse (#523). Like the facade scan, this reads `crate::` paths only: a
# relative `super::super::greeks::d1` import is not seen, so internal
# imports in these crates use `crate::` paths.
INTRA_CRATE_RULES: dict[str, dict[str, frozenset[str]]] = {
    "optionstratlib-pricing": {
        "error": frozenset({"error"}),
        "kernels": frozenset({"error"}),
        "pricing": frozenset({"kernels", "error"}),
        "greeks": frozenset({"kernels", "pricing", "error"}),
        "volatility": frozenset({"kernels", "pricing", "greeks", "error"}),
    },
}


def intra_crate_violations(crate: str, src: Path, rules: dict[str, frozenset[str]]) -> list[str]:
    """Internal edges of one crate that its INTRA_CRATE_RULES entry forbids."""
    found = []
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        top = rel.split("/")[0].removesuffix(".rs")
        if top == "lib":
            continue
        allowed = rules.get(top)
        if allowed is None:
            found.append(f"{crate}: module {top!r} ({rel}) has no INTRA_CRATE_RULES entry")
            continue
        text = "\n".join(production_lines(strip_comments(path.read_text(), exempt_marked=False)))
        for target in sorted(set(module_targets(text))):
            module = target.split("/")[0]
            if module not in rules or module == top or module in allowed:
                continue
            found.append(f"{crate}: {rel} ({top}) -> {module}")
    return found


def foundational_violations(metadata: dict) -> list[str]:
    """Misaligned requirements or duplicate resolved versions of a foundational crate."""
    found: list[str] = []
    for crate in FOUNDATIONAL:
        requirements: dict[str, list[str]] = {}
        for package in workspace_packages(metadata):
            for dep in package.get("dependencies", []):
                if dep["name"] == crate:
                    requirements.setdefault(dep["req"], []).append(package["name"])
        if len(requirements) > 1:
            listed = "; ".join(f"{req} in {', '.join(sorted(set(names)))}" for req, names in sorted(requirements.items()))
            found.append(f"{crate}: workspace packages ask for different versions ({listed})")
        versions = sorted({p["version"] for p in metadata["packages"] if p["name"] == crate})
        if len(versions) > 1:
            found.append(f"{crate}: resolved more than once ({', '.join(versions)})")
    for package in workspace_packages(metadata):
        name = package["name"]
        if not is_component(name) or name in FOUNDATIONAL_DEPENDENTS:
            continue
        for dep in package.get("dependencies", []):
            if dep["name"] in FOUNDATIONAL and (dep.get("kind") or "normal") == "normal":
                found.append(f"{name} -> {dep['name']}: import it through optionstratlib_core instead")
    return found


def is_component(name: str) -> bool:
    return name == "optionstratlib" or name.startswith("optionstratlib-")


def crate_graph_violations(packages: list[dict]) -> list[str]:
    """Reverse or unknown dependencies between OptionStratLib workspace crates.

    Every dependency kind counts: a dev-dependency on a higher layer would let
    a lower crate's own tests reach upward. The one optional edge is
    `market -> simulation`, accepted only as an optional dependency that the
    `synthetic` feature enables (ADR-0003).
    """
    found: list[str] = []
    for package in packages:
        name = package["name"]
        if not is_component(name):
            continue
        layer = CRATE_LAYER.get(name)
        if layer is None:
            found.append(f"{name}: workspace crate with no layer, add it to CRATE_LAYER")
            continue
        synthetic = package.get("features", {}).get("synthetic", [])
        for dep in package.get("dependencies", []):
            target = dep["name"]
            if not is_component(target):
                continue
            target_layer = CRATE_LAYER.get(target)
            kind = dep.get("kind") or "normal"
            if target_layer is None:
                found.append(f"{name} -> {target} ({kind}): unknown workspace crate")
                continue
            if target_layer == "facade" and layer != "facade":
                found.append(f"{name} -> {target} ({kind}): a component never depends on the facade")
                continue
            if target_layer in ALLOWED[layer]:
                continue
            if (
                (layer, target_layer) == ("market", "simulation")
                and dep.get("optional")
                and f"dep:{target}" in synthetic
            ):
                continue
            found.append(f"{name} ({layer}) -> {target} ({target_layer}, {kind})")
    return found


def file_layer(rel: str) -> str | None:
    """Target layer of a facade source file, as `scan` assigns it."""
    parts = rel.split("/")
    top = parts[0].removesuffix(".rs")
    if top in ("lib", "prelude"):
        return "facade"
    stem = parts[1].removesuffix(".rs") if len(parts) > 1 else "mod"
    if top == "error":
        return ERROR_FILE_LAYER.get(stem)
    if top == "utils":
        return UTILS_FILE_LAYER.get(stem)
    return LAYER_OF.get(top)


def facade_redefinitions(src: Path, packages: list[dict]) -> list[str]:
    """Facade files that belong to a layer a workspace crate already owns."""
    owners = {
        CRATE_LAYER[p["name"]]: p["name"]
        for p in packages
        if CRATE_LAYER.get(p["name"]) not in (None, "facade")
    }
    found = []
    for path in sorted(src.rglob("*.rs")):
        rel = path.relative_to(src).as_posix()
        layer = file_layer(rel)
        if layer in owners:
            found.append(f"src/{rel} belongs to layer {layer}, which {owners[layer]} owns")
    return found


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
        # --- per-file ownership of `src/utils` (M1-09)
        "utils file is scanned as its own owner": (
            [("utils/rng.rs", "use crate::strategies::Strategy;\n")], 1, "utils/rng -> strategies",
        ),
        "utils file keeps its allowed edges": ([("utils/numeric.rs", "use crate::model::Options;\n")], 0),
        "a utils target resolves to the owning file": (
            [("model/x.rs", "use crate::utils::rng::deterministic_rng;\n")], 0,
        ),
        "deferred pair in its file": (
            [("strategies/base.rs", "use crate::visualization::Graph;\n")], 0,
        ),
        "deferred pair in another file": (
            [("strategies/other.rs", "use crate::visualization::Graph;\n")], 1,
        ),
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
        "re-export of an alias resolves": (
            [
                err,
                ("strategies/mod.rs", "pub type StratResult<T> = Result<T, crate::error::StrategyError>;\n"),
                ("curves/mod.rs", "pub use crate::strategies::StratResult; // facade-compat: strategies\n"),
                ("pricing/x.rs", "use crate::curves::StratResult;\nfn f() -> StratResult<u8> { todo!() }\n"),
            ],
            1,
            "pricing/x.rs",
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
    # The synthetic gate: a simulation reference in market code counts only
    # when the feature attribute really carries it (M1-15).
    gate_cases = {
        "facade synthetic module ungated": (
            {"lib.rs": "pub mod synthetic;\n", "synthetic/mod.rs": "mod chains;\n", "synthetic/chains.rs": "use crate::simulation::X;\n"},
            1,
        ),
        "facade synthetic module gated": (
            {"lib.rs": '#[cfg(feature = "synthetic")]\npub mod synthetic;\n', "synthetic/mod.rs": "mod chains;\n", "synthetic/chains.rs": "use crate::simulation::X;\n"},
            0,
        ),
        "ungated variant in market error": (
            {"error/chains.rs": "pub enum ChainError {\n    Simulation(Box<crate::error::SimulationError>),\n}\n"},
            1,
        ),
        "variant gated on the item": (
            {"error/chains.rs": 'pub enum ChainError {\n    #[cfg(feature = "synthetic")]\n    Simulation(Box<crate::error::SimulationError>),\n}\n'},
            0,
        ),
        "file gated at its mod declaration": (
            {
                "chains/mod.rs": '#[cfg(feature = "synthetic")]\nmod generators;\n',
                "chains/generators.rs": "use crate::simulation::WalkParams;\n",
            },
            0,
        ),
        "ungated mod declaration": (
            {
                "chains/mod.rs": "mod generators;\n",
                "chains/generators.rs": "use crate::simulation::WalkParams;\n",
            },
            1,
        ),
        "test module is not the minimal surface": (
            {"chains/x.rs": "#[cfg(test)]\nmod t {\n    use crate::simulation::WalkParams;\n}\n"},
            0,
        ),
        "a module outside market is not checked": (
            {"strategies/x.rs": "use crate::simulation::WalkParams;\n"},
            0,
        ),
        # A gated sibling must not lend its gate to the next item: the
        # attribute belongs to the declaration it sits on, not to a window.
        "gated sibling does not gate the next mod declaration": (
            {
                "chains/mod.rs": '#[cfg(feature = "synthetic")]\nmod other;\nmod generators;\n',
                "chains/other.rs": "pub fn nothing() {}\n",
                "chains/generators.rs": "use crate::simulation::WalkParams;\n",
            },
            1,
        ),
        "gated sibling does not gate the next enum variant": (
            {
                "error/chains.rs": (
                    "pub enum ChainError {\n"
                    '    #[cfg(feature = "synthetic")]\n'
                    "    Gated(u8),\n"
                    "    Simulation(Box<crate::error::SimulationError>),\n"
                    "}\n"
                ),
            },
            1,
        ),
        # The gate covers the whole item, however deep the reference sits.
        "gated function body, reference far from the attribute": (
            {
                "chains/x.rs": (
                    '#[cfg(feature = "synthetic")]\n'
                    "fn build() {\n"
                    + "    let _padding = 0;\n" * 12
                    + "    let _ = crate::simulation::WalkParams::default();\n"
                    "}\n"
                ),
            },
            0,
        ),
        "an ungated function body is still reported however deep": (
            {
                "chains/x.rs": (
                    "fn build() {\n"
                    + "    let _padding = 0;\n" * 12
                    + "    let _ = crate::simulation::WalkParams::default();\n"
                    "}\n"
                ),
            },
            1,
        ),
        # The main scanner expands grouped imports, so the gate check must
        # see them too or `SYNTHETIC_FILES` would exempt an edge nothing
        # proved to be gated.
        "grouped import is a reference": (
            {"chains/x.rs": "use crate::{simulation::WalkParams};\n"},
            1,
        ),
        "grouped import inside a gated item": (
            {
                "chains/x.rs": '#[cfg(feature = "synthetic")]\nmod inner {\n    use crate::{simulation::WalkParams};\n}\n',
            },
            0,
        ),
        "multiline grouped import is a reference": (
            {
                "chains/x.rs": "use crate::{\n    model::Options,\n    simulation::WalkParams,\n};\n",
            },
            1,
        ),
        "a gated impl block gates its methods": (
            {
                "chains/x.rs": (
                    '#[cfg(feature = "synthetic")]\n'
                    "impl Chain {\n"
                    "    fn build(&self) {\n"
                    "        let _ = crate::simulation::WalkParams::default();\n"
                    "    }\n"
                    "}\n"
                ),
            },
            0,
        ),
    }
    for name, (files, expected) in gate_cases.items():
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "src"
            for rel, content in files.items():
                target = root / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            got = len(synthetic_gate_violations(root))
            ok = got == expected
            if not ok:
                failures += 1
            print(f"self-test {'ok' if ok else 'FAIL'}: synthetic gate, {name} (expected {expected}, got {got})")
    # --- workspace crate graph (M2-01, carrying #507 forward)
    def pkg(name: str, *deps: tuple, features: dict | None = None) -> dict:
        return {
            "name": name,
            "features": features or {},
            "dependencies": [
                {"name": d[0], "kind": d[1] if len(d) > 1 else None, "optional": d[2] if len(d) > 2 else False}
                for d in deps
            ],
        }

    crate_cases = {
        "core depends on nothing of ours": ([pkg("optionstratlib-core", ("serde",))], 0),
        "core depends on the facade": ([pkg("optionstratlib-core", ("optionstratlib",))], 1),
        "core dev-depends on the facade": ([pkg("optionstratlib-core", ("optionstratlib", "dev"))], 1),
        "core depends on math": ([pkg("optionstratlib-core", ("optionstratlib-math",))], 1),
        "math depends on pricing": ([pkg("optionstratlib-math", ("optionstratlib-pricing",))], 1),
        "math depends on visualization": ([pkg("optionstratlib-math", ("optionstratlib-visualization",))], 1),
        "math depends on core": ([pkg("optionstratlib-math", ("optionstratlib-core",))], 0),
        "facade depends on core": ([pkg("optionstratlib", ("optionstratlib-core",))], 0),
        "unknown component": ([pkg("optionstratlib-extra")], 1),
        "unknown dependency": ([pkg("optionstratlib-core", ("optionstratlib-extra",))], 1),
        "example consumer is not checked": ([pkg("examples_chain", ("optionstratlib",))], 0),
        "synthetic optional edge": (
            [pkg("optionstratlib-market", ("optionstratlib-simulation", None, True),
                 features={"synthetic": ["dep:optionstratlib-simulation"]})],
            0,
        ),
        "mandatory market -> simulation": ([pkg("optionstratlib-market", ("optionstratlib-simulation",))], 1),
    }
    for name, (packages, expected) in crate_cases.items():
        got = len(crate_graph_violations(packages))
        ok = got == expected
        if not ok:
            failures += 1
        print(f"self-test {'ok' if ok else 'FAIL'}: crate graph, {name} (expected {expected}, got {got})")

    def meta(members: list[dict], resolved: list[tuple[str, str]]) -> dict:
        for index, member in enumerate(members):
            member["id"] = f"member-{index}"
        return {
            "workspace_members": [m["id"] for m in members],
            "packages": members + [{"id": f"dep-{n}-{v}", "name": n, "version": v} for n, v in resolved],
        }

    def declares(name: str, *deps: tuple[str, str]) -> dict:
        return {"name": name, "dependencies": [{"name": d, "req": r} for d, r in deps]}

    foundational_cases = {
        "one requirement, one version": (
            meta([declares("optionstratlib-core", ("positive", "^0.7")), declares("optionstratlib", ("positive", "^0.7"))],
                 [("positive", "0.7.1")]),
            0,
        ),
        "two requirements": (
            meta([declares("optionstratlib-core", ("positive", "^0.7")), declares("examples_chain", ("positive", "^0.6"))],
                 [("positive", "0.7.1")]),
            1,
        ),
        "two resolved versions": (
            meta([declares("optionstratlib-core", ("positive", "^0.7"))], [("positive", "0.7.1"), ("positive", "0.6.3")]),
            1,
        ),
        "math depends on positive directly": (
            meta([declares("optionstratlib-core", ("positive", "^0.7")), declares("optionstratlib-math", ("positive", "^0.7"))],
                 [("positive", "0.7.1")]),
            1,
        ),
        "example consumer may depend on positive": (
            meta([declares("optionstratlib-core", ("positive", "^0.7")), declares("examples_chain", ("positive", "^0.7"))],
                 [("positive", "0.7.1")]),
            0,
        ),
        "unrelated duplicate is not ours to check": (
            meta([declares("optionstratlib-core", ("rand", "^0.10"))], [("rand", "0.10.3"), ("rand", "0.9.2")]),
            0,
        ),
    }
    for name, (metadata, expected) in foundational_cases.items():
        got = len(foundational_violations(metadata))
        ok = got == expected
        if not ok:
            failures += 1
        print(f"self-test {'ok' if ok else 'FAIL'}: foundational crates, {name} (expected {expected}, got {got})")

    forbidden_cases = {
        "clean math tree": ({("optionstratlib-math", "default"): {"rayon", "statrs", "optionstratlib-core"}}, 0),
        "math pulls plotly": ({("optionstratlib-math", "all features"): {"rayon", "plotly"}}, 1),
        "core pulls rayon": ({("optionstratlib-core", "default"): {"rayon"}}, 1),
        "math may use rayon": ({("optionstratlib-math", "default"): {"rayon"}}, 0),
        "math pulls csv through a feature": ({("optionstratlib-math", "all features"): {"csv"}}, 1),
        "unlisted crate is not checked": ({("examples_chain", "default"): {"plotly"}}, 0),
        "market tokio only with features": ({("optionstratlib-market", "all features"): {"tokio"}}, 0),
        "market tokio by default": ({("optionstratlib-market", "default"): {"tokio"}}, 1),
    }
    for name, (trees_case, expected) in forbidden_cases.items():
        got = len(forbidden_package_violations(trees_case))
        ok = got == expected
        if not ok:
            failures += 1
        print(f"self-test {'ok' if ok else 'FAIL'}: forbidden packages, {name} (expected {expected}, got {got})")

    # Error types defined in a component crate keep their file's layer (#524).
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp) / "src"
        files = {
            "src/error/simulation.rs": "pub enum SimulationError { A }\nimpl From<SimulationError> for crate::error::ChainError {}\n",
            "src/error/mod.rs": "pub use optionstratlib_market::error::ChainError;\n",
            "crates/optionstratlib-market/src/error/chains.rs": "pub enum ChainError { A }\n",
        }
        for rel, content in files.items():
            target = Path(tmp) / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)
        edges, _ = scan(root)
        got = len(violations_of(edges))
        ok = got == 1
        if not ok:
            failures += 1
        print(f"self-test {'ok' if ok else 'FAIL'}: a component crate's error type keeps its layer (expected 1, got {got})")

    pricing_rules = INTRA_CRATE_RULES["optionstratlib-pricing"]
    intra_cases = {
        "pricing uses kernels": ({"pricing/a.rs": "use crate::kernels::big_n;\n"}, 0),
        "pricing uses greeks": ({"pricing/a.rs": "use crate::greeks::big_n;\n"}, 1),
        "grouped pricing -> greeks": ({"pricing/a.rs": "use crate::{error::PricingError, greeks::d1};\n"}, 1),
        "greeks re-prices through pricing": ({"greeks/numerical.rs": "use crate::pricing::price_option_with;\n"}, 0),
        "kernels uses pricing": ({"kernels.rs": "use crate::pricing::black_scholes;\n"}, 1),
        "test-only edge": ({"pricing/a.rs": "#[cfg(test)]\nmod t {\n    use crate::greeks::delta;\n}\n"}, 0),
        "unknown module": ({"extra.rs": "fn f() {}\n"}, 1),
    }
    for name, (files, expected) in intra_cases.items():
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "src"
            for rel, content in files.items():
                target = root / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            got = len(intra_crate_violations("optionstratlib-pricing", root, pricing_rules))
            ok = got == expected
            if not ok:
                failures += 1
            print(f"self-test {'ok' if ok else 'FAIL'}: internal edges, {name} (expected {expected}, got {got})")

    redefinition_cases = {
        "facade model file once core is a crate": (["model/x.rs"], [pkg("optionstratlib-core")], 1),
        "facade core error file once core is a crate": (["error/decimal.rs"], [pkg("optionstratlib-core")], 1),
        "facade model file before extraction": (["model/x.rs"], [], 0),
        "facade pricing file while only core is a crate": (["pricing/x.rs"], [pkg("optionstratlib-core")], 0),
    }
    for name, (files, packages, expected) in redefinition_cases.items():
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp) / "src"
            for rel in files:
                target = root / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text("")
            got = len(facade_redefinitions(root, packages))
            ok = got == expected
            if not ok:
                failures += 1
            print(f"self-test {'ok' if ok else 'FAIL'}: single definition, {name} (expected {expected}, got {got})")

    # Every tolerated edge must name the issue that removes it, so M1 cannot
    # close with an undocumented production edge (roadmap M1-10).
    unowned = [f"{s} -> {d}" for (s, d), (_, owner) in DEFERRED.items() if not re.search(r"#\d+", owner)]
    ok = not unowned
    if not ok:
        failures += 1
        for item in unowned:
            print(f"  deferred edge without an owning issue: {item}")
    print(f"self-test {'ok' if ok else 'FAIL'}: every deferred edge names an owning issue ({len(DEFERRED)} entries)")
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
    stale_synthetic = sorted(f for f in SYNTHETIC_FILES if not (SRC / f).exists())
    if stale_synthetic:
        print("SYNTHETIC_FILES entries no longer present, prune them:")
        for item in stale_synthetic:
            print(f"  {item}")
        return 1
    if violations:
        print("forbidden module edges (see doc/DEPENDENCY-MATRIX.md, ADR-0001 D9):")
        for item in violations:
            print(f"  {item}")
        return 1
    ungated = synthetic_gate_violations()
    if ungated:
        print('market code names a simulation type outside `#[cfg(feature = "synthetic")]` (ADR-0003, M1-15):')
        for item in ungated:
            print(f"  {item}")
        return 1
    metadata = cargo_metadata(SRC.parent)
    packages = workspace_packages(metadata)
    # Report every crate-level rule in one run, then fail once.
    crate_rules = [
        ("foundational type crates must resolve once, at one requirement, and only core may "
         "depend on them (ADR-0001 D8, #515):", foundational_violations(metadata)),
        ("forbidden workspace crate dependencies (ADR-0001 D1/D9):", crate_graph_violations(packages)),
        ("facade files in a layer that a workspace crate owns (one canonical definition):",
         facade_redefinitions(SRC, packages)),
    ]
    for crate, rules in sorted(INTRA_CRATE_RULES.items()):
        package = next((p for p in packages if p["name"] == crate), None)
        crate_rules.append((
            f"forbidden internal module edges in {crate} (#523):",
            intra_crate_violations(crate, Path(package["manifest_path"]).parent / "src", rules)
            if package is not None
            else [f"{crate}: listed in INTRA_CRATE_RULES but not a workspace member"],
        ))
    trees = {
        (crate, label): resolved_tree(SRC.parent, crate, all_features)
        for crate in sorted(FORBIDDEN_PACKAGES)
        if any(p["name"] == crate for p in packages)
        for label, all_features in (("default", False), ("all features", True))
    }
    crate_rules.append((
        "component crates resolve a forbidden package (ADR-0002 fixture table, #517):",
        forbidden_package_violations(trees),
    ))
    failed = False
    for heading, items in crate_rules:
        if items:
            failed = True
            print(heading)
            for item in items:
                print(f"  {item}")
    if failed:
        return 1
    crates = sorted(p["name"] for p in packages if CRATE_LAYER.get(p["name"]) not in (None, "facade"))
    deferred_count = sum(1 for key in DEFERRED if key in present)
    marks = ", ".join(f"{layer}={n}" for layer, n in sorted(marked_lines().items())) or "none"
    print(f"OK: no forbidden module edge ({deferred_count} deferred edges tolerated; facade-compat lines per layer: {marks})")
    print(f"OK: workspace crate graph acyclic and layered (components: {', '.join(crates) or 'none'})")
    print(f"OK: foundational crates resolve once ({', '.join(FOUNDATIONAL)})")
    print(f"OK: internal module edges acyclic in {', '.join(sorted(INTRA_CRATE_RULES))}")
    print(f"OK: no forbidden package in {', '.join(sorted({c for c, _ in trees})) or 'any component'} (default and all features)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
