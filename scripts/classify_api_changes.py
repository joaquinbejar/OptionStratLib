#!/usr/bin/env python3
"""Classify the public API differences against the published 0.21.3 (#557).

`report_api_changes.py --baseline v0.21.3` lists what `cargo-semver-checks`
sees as removed or reshaped; with the facade split into component crates the
list is long, and most of it is items that still exist, defined elsewhere.
This sorts every reported item into exactly one class, so a reviewer reads the
removals and nothing else:

* `same-path`: the old path still resolves today, because the facade
  re-exports the component's definition there;
* `facade-path`: the item is reachable from the facade at another path
  (a canonical module root instead of a file module, #550);
* `facade-reexport`: the old path is gone, but the facade reaches the item
  through the module root of the component that defines it (the destination
  is looked up in the checked-in snapshots `public-api/<crate>.txt` and
  compiled);
* `removed`: no snapshot has an item of that name and kind.

(A fifth class, a component-defined item that no facade path reaches, exists
in the code as `component-only`; the 0.22 run found none.)

The first class is decided by compiling `use <old path>;` against the current
facade with every feature, one line per item, so the answer comes from rustc
and not from a text match. The others are lookups.

Usage:
    classify_api_changes.py REPORT.log [REPORT.log ...]  > classification.json
    classify_api_changes.py --markdown classification.json
    classify_api_changes.py --self-test
"""

from __future__ import annotations

import json
import re
import subprocess
import sys
import tempfile
from collections import defaultdict
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FACADE_MODULES = (
    "model utils constants curves surfaces geometrics pricing greeks volatility simulation chains series "
    "analytics pnl risk metrics strategies backtesting visualization error prelude"
).split()
ITEM_RE = re.compile(r"^\s+(\w+): (.*)$")
# A definition line may start with attributes (`#[repr(u8)] pub enum ...`)
# and carry qualifiers before the kind (`pub async fn ...`).
SNAPSHOT_RE = re.compile(
    r"^(?:#\[[^\]]*\]\s*)*pub (?:async |const |unsafe )*(mod|struct|enum|trait|fn|type|const|static|union|macro) (\S+)"
)
KIND = {
    "struct_missing": "struct", "enum_missing": "enum", "trait_missing": "trait", "function_missing": "fn",
    "module_missing": "mod", "pub_module_level_const_missing": "const", "declarative_macro_missing": "macro",
}


def parse_report(text: str) -> list[tuple[str, str]]:
    """`(lint, path)` of every module-level item a report lists; variants are kept apart."""
    items = []
    for line in text.splitlines():
        match = ITEM_RE.match(line)
        if match:
            lint, rest = match.groups()
            items.append((lint, rest.split(" ")[-1] if " " in rest else rest))
    return items


def snapshot_index(root: Path = ROOT) -> dict[str, list[tuple[str, str, str]]]:
    """`name -> [(kind, crate, path)]` over the component snapshots."""
    index: dict[str, list[tuple[str, str, str]]] = defaultdict(list)
    snapshots = [root / "public-api" / "optionstratlib.txt", *(root / "public-api").glob("optionstratlib-*.txt")]
    for path in sorted(snapshots):
        crate = path.stem.replace("-", "_")
        for line in path.read_text().splitlines():
            match = SNAPSHOT_RE.match(line)
            if match and "::" in match.group(2):
                kind, full = match.groups()
                # A function line carries its arguments, a generic item its
                # parameters and a trait its supertraits after the path:
                # `...::name(arg:`, `...::Name<T`, `...::Name:`.
                full = re.split(r"[(<]", full, maxsplit=1)[0].rstrip(":")
                index[full.rsplit("::", 1)[1]].append((kind, crate, full))
    return index


def resolves(uses: list[str], features: list[str]) -> set[int]:
    """Indexes of the `use` lines that compile against the current facade."""
    with tempfile.TemporaryDirectory() as tmp:
        crate = Path(tmp)
        (crate / "src").mkdir()
        feature_list = ", ".join(f'"{f}"' for f in features)
        (crate / "Cargo.toml").write_text(
            f'[package]\nname = "classify"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\n'
            f'optionstratlib = {{ path = "{ROOT}", features = [{feature_list}] }}\n[workspace]\n'
        )
        # One module per line: two imports of the same final name (`risk` in
        # `metrics` and in `prelude`) would otherwise collide in one scope
        # and be reported as unresolved.
        body = "\n".join(f"mod m{n} {{ {statement} }}" for n, statement in enumerate(uses))
        (crate / "src" / "lib.rs").write_text("#![allow(unused_imports)]\n" + body + "\n")
        done = subprocess.run(
            ["cargo", "check", "--message-format=json", "-q"], cwd=crate, capture_output=True, text=True, check=False
        )
    failing: set[int] = set()
    for line in done.stdout.splitlines():
        try:
            message = json.loads(line).get("message") or {}
        except json.JSONDecodeError:
            continue
        if message.get("level") == "error":
            failing.update(span["line_start"] - 2 for span in message.get("spans", []) if span["file_name"].endswith("lib.rs"))
    if done.returncode != 0 and not failing:
        raise SystemExit(f"classification crate failed to build:\n{done.stderr[-2000:]}")
    return set(range(len(uses))) - failing


def classify(items: list[tuple[str, str]], features: list[str]) -> list[dict]:
    index = snapshot_index()
    checked = [(lint, path) for lint, path in items if lint in KIND and path.startswith("optionstratlib::") or lint == "declarative_macro_missing"]
    uses = [f"use {path if path.startswith('optionstratlib::') else 'optionstratlib::' + path};" for _, path in checked]
    ok = resolves(uses, features)
    # Same-named items at the root of each facade module.
    alternatives = []
    for _, path in checked:
        name = path.rsplit("::", 1)[-1]
        alternatives += [f"use optionstratlib::{module}::{name};" for module in FACADE_MODULES]
    found = resolves(alternatives, features)
    result = []
    for n, (lint, path) in enumerate(checked):
        name = path.rsplit("::", 1)[-1]
        top = path.split("::")[1] if "::" in path else path
        row = {"lint": lint, "path": path, "area": top, "name": name}
        if n in ok:
            row["class"] = "same-path"
        else:
            roots = [FACADE_MODULES[i] for i in range(len(FACADE_MODULES))
                     if n * len(FACADE_MODULES) + i in found and FACADE_MODULES[i] != "prelude"]
            where = [f"{crate}::{full.split('::', 1)[1]}" for kind, crate, full in index.get(name, [])
                     if kind == KIND.get(lint, kind)]
            if roots:
                row["class"], row["now"] = "facade-path", [f"optionstratlib::{roots[0]}::{name}"]
            elif where:
                row["class"], row["now"] = "component", sorted(set(where))[:3]
            else:
                row["class"] = "removed"
        result.append(row)
    # A component-only item may still be reachable from the facade, through
    # its module root: `optionstratlib_pricing::pricing::x::Name` is
    # `optionstratlib::pricing::x::Name`. Compile each candidate.
    candidates = []
    for row in result:
        if row["class"] == "component":
            for full in row["now"]:
                crate, rest = full.split("::", 1)
                candidates.append((row, f"optionstratlib::{rest}", full))
    reachable = resolves([f"use {facade};" for _, facade, _ in candidates], features)
    for n, (row, facade, full) in enumerate(candidates):
        if n in reachable and "facade" not in row:
            row["facade"] = facade
    for row in result:
        if row["class"] == "component":
            row["class"] = "facade-reexport" if "facade" in row else "component-only"
    return result


def markdown(rows: list[dict]) -> str:
    counts: dict[tuple[str, str], int] = defaultdict(int)
    for row in rows:
        counts[(row["area"], row["class"])] += 1
    classes = ("same-path", "facade-path", "facade-reexport", "component-only", "removed")
    if not any(row["class"] == "component-only" for row in rows):
        classes = tuple(c for c in classes if c != "component-only")
    lines = ["| Old path area | " + " | ".join(classes) + " | total |", "| --- | " + " | ".join("---:" for _ in classes) + " | ---: |"]
    for area in sorted({row["area"] for row in rows}):
        cells = [counts[(area, c)] for c in classes]
        lines.append(f"| `optionstratlib::{area}` | " + " | ".join(str(c) for c in cells) + f" | {sum(cells)} |")
    totals = [sum(counts[(a, c)] for a in {r['area'] for r in rows}) for c in classes]
    lines.append("| **total** | " + " | ".join(f"**{t}**" for t in totals) + f" | **{sum(totals)}** |")
    return "\n".join(lines) + "\n"


def self_test() -> int:
    failures = 0

    def check(name: str, got: object, expected: object) -> None:
        nonlocal failures
        ok = got == expected
        failures += 0 if ok else 1
        print(f"self-test {'ok' if ok else 'FAIL'}: {name} (expected {expected!r}, got {got!r})")

    report = "api-changes: all against v0.21.3, 3 item(s) for review\n  struct_missing: struct optionstratlib::a::B\n  enum_variant_added: enum variant optionstratlib::a::E::V\n  declarative_macro_missing: macro nz\n"
    check("report lines parse", parse_report(report), [
        ("struct_missing", "optionstratlib::a::B"), ("enum_variant_added", "optionstratlib::a::E::V"),
        ("declarative_macro_missing", "nz"),
    ])
    rows = [{"area": "a", "class": "removed"}, {"area": "a", "class": "same-path"}]
    check("the table counts per area and class", markdown(rows).count("| `optionstratlib::a` | 1 | 0 | 0 | 1 | 2 |"), 1)
    index = snapshot_index()
    check("a function is indexed by its name, not its arguments",
          any(kind == "fn" for kind, _, _ in index.get("black_scholes", [])), True)
    check("an attribute before `pub` does not hide a definition",
          any(kind == "enum" for kind, _, _ in index.get("TraceMode", [])), True)
    check("an async function is indexed",
          any(kind == "fn" for kind, _, _ in index.get("read_ohlcv_from_zip_async", [])), True)
    check("a generic type is indexed by its name",
          any(kind == "struct" for kind, _, _ in index.get("Simulator", [])), True)
    return 1 if failures else 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    if args[:1] == ["--markdown"]:
        print(markdown(json.loads(Path(args[1]).read_text())), end="")
        return 0
    items: list[tuple[str, str]] = []
    for name in args:
        items += parse_report(Path(name).read_text())
    rows = classify(sorted(set(items)), ["plotly", "static_export", "async"])
    print(json.dumps(rows, indent=1))
    return 0


if __name__ == "__main__":
    sys.exit(main())
