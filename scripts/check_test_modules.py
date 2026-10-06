#!/usr/bin/env python3
"""Report test sources that no module declaration reaches (#633).

A Rust test target compiles only the files its root reaches through `mod`
declarations. A file under the target's directory that nothing declares
never compiles, so its tests silently never run; `tests/unit/simulation/
model_and_randomwalk_tests.rs` sat like that from #349 until #633.

Roots checked:

* the facade's multi-file test targets, `tests/unit/mod.rs` and
  `tests/property/mod.rs`;
* every `crates/*/tests/<dir>/main.rs` (a multi-file integration target);
* `tests/workspace/src/lib.rs`, the workspace integration package.

From each root, `mod name;` resolves to `name.rs` or `name/mod.rs` next to a
`mod.rs` / `main.rs` / `lib.rs` declarer, or under `<declarer stem>/` for any
other file, as rustc does. `#[path]` attributes are not used in this
repository and are not followed. Inline `mod name { ... }` blocks declare no
file. Every `.rs` under the root's directory that is not reached is reported.

Usage: scripts/check_test_modules.py [--self-test]
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
MOD_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", re.M)
LINE_COMMENT_RE = re.compile(r"//.*$", re.M)
BLOCK_COMMENT_RE = re.compile(r"/\*.*?\*/", re.S)


def declared_children(path: Path) -> list[Path]:
    """Files a source file declares with `mod name;`."""
    text = BLOCK_COMMENT_RE.sub("", LINE_COMMENT_RE.sub("", path.read_text()))
    if path.name in ("mod.rs", "main.rs", "lib.rs"):
        base = path.parent
    else:
        base = path.parent / path.stem
    found = []
    for name in MOD_RE.findall(text):
        for candidate in (base / f"{name}.rs", base / name / "mod.rs"):
            if candidate.is_file():
                found.append(candidate)
                break
    return found


def unreachable_files(root_file: Path) -> list[Path]:
    """`.rs` files under `root_file`'s directory that its module tree misses."""
    reached: set[Path] = set()
    stack = [root_file]
    while stack:
        current = stack.pop()
        if current in reached:
            continue
        reached.add(current)
        stack.extend(declared_children(current))
    return sorted(p for p in root_file.parent.rglob("*.rs") if p not in reached)


def roots(base: Path) -> list[Path]:
    found = [base / "tests" / "unit" / "mod.rs", base / "tests" / "property" / "mod.rs"]
    found += sorted(base.glob("crates/*/tests/*/main.rs"))
    found.append(base / "tests" / "workspace" / "src" / "lib.rs")
    return [p for p in found if p.is_file()]


def check(base: Path) -> list[str]:
    problems = []
    for root_file in roots(base):
        for orphan in unreachable_files(root_file):
            problems.append(
                f"{orphan.relative_to(base)}: no `mod` declaration reaches it from "
                f"{root_file.relative_to(base)}, so it never compiles"
            )
    return problems


def self_test() -> int:
    cases = {
        "declared file and directory module": (
            {"tests/unit/mod.rs": "mod a;\nmod b;\n", "tests/unit/a.rs": "", "tests/unit/b/mod.rs": "mod c;\n", "tests/unit/b/c.rs": ""},
            0,
        ),
        "undeclared directory": (
            {"tests/unit/mod.rs": "mod a;\n", "tests/unit/a.rs": "", "tests/unit/sim/x_tests.rs": ""},
            1,
        ),
        "commented-out declaration does not count": (
            {"tests/unit/mod.rs": "// mod a;\n", "tests/unit/a.rs": ""},
            1,
        ),
        "non-mod.rs declarer resolves under its stem": (
            {"tests/property/mod.rs": "mod group;\n", "tests/property/group.rs": "mod inner;\n", "tests/property/group/inner.rs": ""},
            0,
        ),
        "crate integration target": (
            {"crates/x/tests/integration/main.rs": "mod a;\n", "crates/x/tests/integration/a.rs": "", "crates/x/tests/integration/b.rs": ""},
            1,
        ),
    }
    failures = 0
    for name, (files, expected) in cases.items():
        with tempfile.TemporaryDirectory() as tmp:
            base = Path(tmp)
            for rel, content in files.items():
                target = base / rel
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(content)
            got = len(check(base))
            ok = got == expected
            failures += 0 if ok else 1
            print(f"self-test {'ok' if ok else 'FAIL'}: {name} (expected {expected}, got {got})")
    return 1 if failures else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    problems = check(ROOT)
    if problems:
        print("test sources that never compile (#633):")
        for item in problems:
            print(f"  {item}")
        return 1
    print(f"OK: every test source is reached from its target root ({len(roots(ROOT))} roots)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
