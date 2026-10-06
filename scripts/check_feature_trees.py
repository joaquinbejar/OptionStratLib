#!/usr/bin/env python3
"""Pin the dependency graph of the minimal market surface and of `synthetic`.

Roadmap M1-15 asks for cargo-tree fixtures that assert the expected
difference between the market surface without `synthetic` and with it. Each
surface gets its own fixture holding the whole graph, so a change that adds,
removes or re-parents a dependency shows up even when it lands on both
surfaces at once; the difference between the two is derived from them and
printed. Since #536 extracted `optionstratlib-simulation` that difference is
the edge to the simulation crate and its own graph, and that *is* the
assertion: enabling `synthetic` adds exactly that crate. When #537 moves the
generators behind market's own `synthetic` feature, the same fixtures carry
the new edge without the check having to change.

Normalisation, and what it costs:

* `--target all` resolves every target's dependencies, so the fixture is the
  same on macOS, Linux and Windows. Without it the graph is the host's and a
  developer machine and CI disagree.
* The graph is stored as a sorted `parent -> child` edge list plus one
  `package [features]` line per package, so a dependency that `synthetic`
  promotes from transitive to direct is a new edge, and a feature it turns on
  for a package both surfaces already share is a changed feature line. Either
  one fails the check.
* Nodes are package names without versions. `Cargo.lock` is not committed, so
  a version recorded here would be whatever the resolver picked that day and
  an unrelated upstream patch release would fail CI for every contributor.
  The cost is real: a version bump, and two versions of one package
  collapsing into one node, are invisible to this check. Pinning those needs
  a committed lockfile, which is a packaging decision, tracked in #616.

The `visualization`, `plotly` and `static_export` surfaces also carry
dependency assertions that do not depend on the fixtures, so an
`--update` run cannot record a regression (#544): no surface below `plotly`
resolves a Plotly, image-export, WebDriver or async-runtime package or the
visualization crate where it does not belong, `plotly` resolves Plotly and no
export stack, and `static_export` resolves all of it.

Usage:
    check_feature_trees.py              compare against the fixtures
    check_feature_trees.py --update     rewrite the fixtures
    check_feature_trees.py --self-test  prove the dependency assertions
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

# CI sets `CARGO_TERM_COLOR=always`, so `--color never` alone is not enough on
# every cargo version; the output is stripped as well, or a coloured `(*)`
# marker would be read as a package's feature list.
ANSI_RE = re.compile(r"\x1b\[[0-9;]*m")
FIXTURES = ROOT / "tests" / "fixtures" / "feature-trees"

# name -> the cargo feature flags that select the surface. `market` is a facade
# capability (ADR-0002), so the minimal market surface names it explicitly:
# `--no-default-features` alone builds no market at all.
SURFACES = {
    "minimal": ["--no-default-features", "--features", "market"],
    "synthetic": ["--no-default-features", "--features", "synthetic"],
    # `backtest` enables strategies and simulation and no market feature
    # beyond the minimal one: in particular not `synthetic` (#541).
    "backtest": ["--no-default-features", "--features", "backtest"],
    # `visualization` adds the chart crate on top of `backtest` and no Plotly,
    # image-export or async package: those come only with `plotly` and
    # `static_export` (#542).
    "visualization": ["--no-default-features", "--features", "visualization"],
    # `plotly` adds Plotly itself; `static_export` adds the image exporter, the
    # WebDriver client and the async runtime on top (#544).
    "plotly": ["--no-default-features", "--features", "plotly"],
    "static_export": ["--no-default-features", "--features", "static_export"],
}

# What each surface's graph must and must not resolve, whatever its fixture
# says. `EXPORT_STACK` is the image exporter and its WebDriver client,
# `RUNTIME` the async runtime and HTTP client under it, `VISUALIZATION_CRATE`
# the chart crate: all absent from every surface that does not ask for them.
# `plotly` may resolve `tokio` here, and only as a child of
# `wasm-bindgen-futures`: Plotly's own `wasm32` dependency names it, and
# `--target all` resolves that target too (`ONLY_VIA`). `check-graph`
# resolves the host and asserts `plotly` alone adds no `tokio` or `reqwest`
# there.
PLOTLY = {"plotly"}
EXPORT_STACK = {"plotly_static", "fantoccini", "webdriver"}
RUNTIME = {"tokio", "reqwest"}
VISUALIZATION_CRATE = {"optionstratlib-visualization"}
BACKEND_RULES: dict[str, tuple[set[str], set[str]]] = {
    "minimal": (set(), PLOTLY | EXPORT_STACK | RUNTIME | VISUALIZATION_CRATE),
    "synthetic": (set(), PLOTLY | EXPORT_STACK | RUNTIME | VISUALIZATION_CRATE),
    "backtest": (set(), PLOTLY | EXPORT_STACK | RUNTIME | VISUALIZATION_CRATE),
    "visualization": (VISUALIZATION_CRATE, PLOTLY | EXPORT_STACK | RUNTIME),
    "plotly": (VISUALIZATION_CRATE | PLOTLY, EXPORT_STACK | {"reqwest"}),
    "static_export": (VISUALIZATION_CRATE | PLOTLY | EXPORT_STACK | RUNTIME, set()),
}

# Packages a surface tolerates only as a child of the named parents.
ONLY_VIA: dict[str, dict[str, set[str]]] = {
    "plotly": {"tokio": {"wasm-bindgen-futures"}},
}


HEADER = """# Dependency graph of the `{surface}` facade surface, resolved with
# `cargo tree --target all -e no-dev`: a sorted `parent -> child` edge list
# over package names, then the features enabled on each package. Generated by
# `make feature-trees-update`; see scripts/check_feature_trees.py for why
# versions are not recorded.
"""


def edges(flags: list[str]) -> list[str]:
    """The edges and per-package features `cargo tree` resolves for a surface."""
    result = subprocess.run(
        [
            "cargo", "tree", "--color", "never", "--target", "all",
            "-e", "no-dev", "--prefix", "depth", "--format", "{p} {f}", *flags,
        ],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        print(result.stderr, file=sys.stderr)
        raise SystemExit(f"cargo tree failed for {' '.join(flags)}")

    found: set[str] = set()
    features: dict[str, set[str]] = {}
    stack: list[str] = []
    for line in ANSI_RE.sub("", result.stdout).splitlines():
        line = line.rstrip()
        if not line or not line[0].isdigit():
            continue
        digits = len(line) - len(line.lstrip("0123456789"))
        depth = int(line[:digits])
        # `{p} {f}` gives "name v1.2.3 [(...)] [feat,feat]", where the
        # parenthesised field is a local path, `(proc-macro)` or `(*)` for an
        # already-shown subtree. Those are dropped, along with the version
        # (see the module docstring); whatever is left after the version is
        # the feature list. Dropping the path matters: for the root package
        # it is this checkout's absolute path.
        fields = [f for f in line[digits:].split() if not f.startswith("(")]
        name = fields[0]
        feats: set[str] = set()
        if len(fields) > 2:
            feats = {f for f in fields[2].split(",") if f}
        del stack[depth:]
        stack.append(name)
        if depth:
            found.add(f"{stack[depth - 1]} -> {name}")
        features.setdefault(name, set()).update(feats)
    graph = sorted(found)
    graph.append("")
    graph.extend(
        f"{name} [{','.join(sorted(feats))}]" for name, feats in sorted(features.items())
    )
    return graph


def packages_of(graph: list[str]) -> set[str]:
    """The package names in a graph, from its `name [features]` lines."""
    return {line.split(" [")[0] for line in graph if line and " -> " not in line and not line.startswith("#")}


def backend_violations(surface: str, graph: list[str]) -> list[str]:
    """Packages a surface must resolve and does not, or must not and does."""
    required, forbidden = BACKEND_RULES.get(surface, (set(), set()))
    names = packages_of(graph)
    problems = [f"{surface}: does not resolve {name}" for name in sorted(required - names)] + [
        f"{surface}: resolves {name}" for name in sorted(forbidden & names)
    ]
    for name, parents in sorted(ONLY_VIA.get(surface, {}).items()):
        incoming = {line.split(" -> ")[0] for line in graph if line.endswith(f" -> {name}")}
        problems += [f"{surface}: {parent} brings {name}" for parent in sorted(incoming - parents)]
    return problems


def self_test() -> int:
    """Prove the dependency assertions see a regression a regenerated fixture would hide."""
    base = ["a -> optionstratlib-core", "", "optionstratlib [backtest]", "optionstratlib-core []"]

    def graph(*extra: str) -> list[str]:
        return base + [f"{name} []" for name in extra]

    cases = {
        "headless surface is clean": ("minimal", graph(), 0),
        "headless surface resolves plotly": ("minimal", graph("plotly"), 1),
        "headless surface resolves the visualization crate": ("backtest", graph("optionstratlib-visualization"), 1),
        "headless surface resolves tokio": ("synthetic", graph("tokio"), 1),
        "visualization is clean": ("visualization", graph("optionstratlib-visualization"), 0),
        "visualization lost its crate": ("visualization", graph(), 1),
        "visualization resolves plotly": ("visualization", graph("optionstratlib-visualization", "plotly"), 1),
        "plotly is clean": ("plotly", graph("optionstratlib-visualization", "plotly"), 0),
        "plotly lost plotly": ("plotly", graph("optionstratlib-visualization"), 1),
        "plotly resolves the exporter": (
            "plotly",
            graph("optionstratlib-visualization", "plotly", "plotly_static", "fantoccini"),
            2,
        ),
        "plotly may resolve tokio for wasm": (
            "plotly",
            graph("optionstratlib-visualization", "plotly", "tokio", "wasm-bindgen-futures")
            + ["wasm-bindgen-futures -> tokio"],
            0,
        ),
        "plotly resolves tokio through another parent": (
            "plotly",
            graph("optionstratlib-visualization", "plotly", "tokio", "wasm-bindgen-futures")
            + ["wasm-bindgen-futures -> tokio", "plotly_x -> tokio"],
            1,
        ),
        "plotly resolves tokio through the facade": (
            "plotly",
            graph("optionstratlib-visualization", "plotly", "tokio") + ["optionstratlib -> tokio"],
            1,
        ),
        "plotly resolves reqwest": ("plotly", graph("optionstratlib-visualization", "plotly", "reqwest"), 1),
        "visualization resolves tokio": ("visualization", graph("optionstratlib-visualization", "tokio"), 1),
        "static_export is complete": (
            "static_export",
            graph("optionstratlib-visualization", "plotly", *sorted(EXPORT_STACK | RUNTIME)),
            0,
        ),
        "static_export lost the runtime": (
            "static_export",
            graph("optionstratlib-visualization", "plotly", "plotly_static", "fantoccini", "webdriver", "reqwest"),
            1,
        ),
        "static_export lost the exporter": (
            "static_export",
            graph("optionstratlib-visualization", "plotly", "fantoccini", "webdriver", "reqwest", "tokio"),
            1,
        ),
        "an edge is not a package": ("minimal", ["optionstratlib -> plotly", "", "optionstratlib []"], 0),
    }
    failures = 0
    for name, (surface, case, expected) in cases.items():
        got = len(backend_violations(surface, case))
        ok = got == expected
        failures += 0 if ok else 1
        print(f"self-test {'ok' if ok else 'FAIL'}: {name} (expected {expected}, got {got})")
    surfaces = set(SURFACES) == set(BACKEND_RULES)
    failures += 0 if surfaces else 1
    print(f"self-test {'ok' if surfaces else 'FAIL'}: every surface has a backend rule and the reverse")
    return 1 if failures else 0


def render(surface: str, graph: list[str]) -> str:
    return HEADER.format(surface=surface) + "\n".join(graph) + "\n"


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    update = "--update" in sys.argv
    FIXTURES.mkdir(parents=True, exist_ok=True)
    graphs = {name: edges(flags) for name, flags in SURFACES.items()}

    # Checked before any fixture is written, so `--update` cannot record a
    # graph that breaks the gate.
    problems = [problem for surface, graph in graphs.items() for problem in backend_violations(surface, graph)]
    for problem in problems:
        print(f"backend gate: {problem}", file=sys.stderr)
    if problems:
        return 1

    failures = 0
    for surface, graph in graphs.items():
        path = FIXTURES / f"{surface}.txt"
        rendered = render(surface, graph)
        if update:
            path.write_text(rendered)
            print(f"updated {path.relative_to(ROOT)} ({len(graph)} lines)")
            continue
        if not path.exists():
            print(f"missing fixture {path.relative_to(ROOT)}; run 'make feature-trees-update'", file=sys.stderr)
            failures += 1
            continue
        if path.read_text() != rendered:
            expected = set(path.read_text().splitlines())
            print(f"the {surface!r} surface's dependency graph drifted:", file=sys.stderr)
            for line in sorted(set(graph) - expected):
                print(f"  + {line}", file=sys.stderr)
            for line in sorted(expected - set(graph) - {""} ):
                if not line.startswith("#"):
                    print(f"  - {line}", file=sys.stderr)
            print("If this is intentional, run 'make feature-trees-update' and commit the diff.", file=sys.stderr)
            failures += 1

    if update:
        return 0
    if failures:
        return 1

    added = sorted(set(graphs["synthetic"]) - set(graphs["minimal"]))
    removed = sorted(set(graphs["minimal"]) - set(graphs["synthetic"]))
    detail = "nothing (the feature gates source, not crates)"
    if added or removed:
        detail = f"{len(added)} line(s) added, {len(removed)} removed: " + ", ".join(added + removed)
    print(
        f"OK: all {len(graphs)} surface dependency graphs match their fixtures "
        f"({len(graphs['minimal'])} lines); `synthetic` changes {detail}"
    )
    print("OK: Plotly, the export stack and the visualization crate appear only on the surfaces that ask for them")
    return 0


if __name__ == "__main__":
    sys.exit(main())
