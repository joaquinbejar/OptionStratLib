#!/usr/bin/env python3
"""Check the consumer fixtures' resolved dependency graphs (ADR-0004 section 3).

Every directory under `fixtures/consumers/` is a real crate excluded from the
root workspace, and every directory under `examples/direct/` a workspace member
(the direct-component examples, #555, named `direct-<scenario>` here), each
with an `expect.toml` holding two lists: `present` (packages its normal graph
must resolve) and `absent` (packages it must not). For each one this resolves

    cargo tree --manifest-path fixtures/consumers/<scenario>/Cargo.toml \\
      -e normal --prefix none

anchors every name on the package (`csv` never matches `csv-core`), fails on
a missing `present` or a resolved `absent` package, and prints the resolved
package count in the M0 baseline format (`sed 's/ (\\*)$//' | sort -u | wc -l`)
as a recorded, not asserted, measurement.

Usage: scripts/check_fixtures.py [scenario ...]   (default: every fixture)
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "fixtures" / "consumers"
EXAMPLES = ROOT / "examples" / "direct"
EXAMPLE_PREFIX = "direct-"


def discover(fixtures: Path = FIXTURES, examples: Path = EXAMPLES) -> dict[str, Path]:
    """Every checked crate by name: fixtures as `<dir>`, examples as `direct-<dir>`."""
    found: dict[str, Path] = {}
    for root, prefix in ((fixtures, ""), (examples, EXAMPLE_PREFIX)):
        if root.is_dir():
            for path in sorted(root.iterdir()):
                if (path / "expect.toml").is_file():
                    found[f"{prefix}{path.name}"] = path
    return found


def expectations(path: Path) -> dict[str, list[str]]:
    """The `present` and `absent` string lists of an `expect.toml`.

    A minimal reader for exactly this shape (two arrays of strings, comments
    allowed), so the check needs no third-party TOML parser on Python < 3.11.
    """
    text = "\n".join(line.split("#", 1)[0] for line in path.read_text().splitlines())
    lists: dict[str, list[str]] = {}
    for key in ("present", "absent"):
        match = re.search(rf"^\s*{key}\s*=\s*\[(.*?)\]", text, re.S | re.M)
        if match is None:
            raise SystemExit(f"{path}: missing `{key} = [...]`")
        lists[key] = re.findall(r'"([^"]+)"', match.group(1))
    return lists


def resolved(manifest: Path) -> set[str]:
    out = subprocess.run(
        ["cargo", "tree", "--manifest-path", str(manifest), "-e", "normal", "--prefix", "none"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    return {line.removesuffix(" (*)").strip() for line in out.splitlines() if line.strip()}


def check(label: str, scenario: Path) -> list[str]:
    expect = expectations(scenario / "expect.toml")
    lines = resolved(scenario / "Cargo.toml")
    names = {line.split(" ")[0] for line in lines}
    problems = [f"{label}: did not resolve {name}" for name in expect["present"] if name not in names]
    problems += [f"{label}: resolved {name}" for name in expect["absent"] if name in names]
    print(f"{label}: {len(lines)} resolved package entries ({len(names)} distinct packages)")
    return problems


def self_test() -> int:
    import tempfile

    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "expect.toml"
        path.write_text('# c\npresent = ["a", "b"] # x\nabsent = [\n  "c",\n  "d",\n]\n')
        ok = expectations(path) == {"present": ["a", "b"], "absent": ["c", "d"]}
    print(f"self-test {'ok' if ok else 'FAIL'}: expect.toml parsing")

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        for rel in ("fixtures/a", "fixtures/b", "examples/a", "examples/c"):
            (root / rel).mkdir(parents=True)
            (root / rel / "expect.toml").write_text("present = []\nabsent = []\n")
        (root / "fixtures" / "no-expectation").mkdir()
        names = sorted(discover(root / "fixtures", root / "examples"))
        missing = sorted(discover(root / "nowhere", root / "examples"))
    found = names == ["a", "b", "direct-a", "direct-c"] and missing == ["direct-a", "direct-c"]
    print(f"self-test {'ok' if found else 'FAIL'}: fixtures and examples are found under distinct names")
    return 0 if ok and found else 1


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    wanted = [a for a in sys.argv[1:] if not a.startswith("--")]
    scenarios = discover()
    if wanted:
        missing = set(wanted) - set(scenarios)
        if missing:
            raise SystemExit(f"unknown fixture(s): {', '.join(sorted(missing))}")
        scenarios = {name: path for name, path in scenarios.items() if name in wanted}
    problems = [problem for label, scenario in scenarios.items() for problem in check(label, scenario)]
    for problem in problems:
        print(f"FAIL: {problem}")
    if problems:
        return 1
    print(f"OK: {len(scenarios)} consumer fixture and example graph(s) match their expectations")
    return 0


if __name__ == "__main__":
    sys.exit(main())
