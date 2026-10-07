#!/usr/bin/env python3
"""Run every 0.22 consumer fixture outside workspace feature unification.

Every directory under `fixtures/consumers/` is a real downstream crate,
excluded from the root workspace (ADR-0004 section 3), that depends on the
facade or on component crates with the manifest a 0.22 user would write. For
each one this runs, with a target directory of its own
(`target/fixtures/<scenario>`) so nothing the workspace or another fixture
enables can widen its graph:

    cargo clippy --manifest-path <fixture>/Cargo.toml --all-targets <flags> -- -D warnings
    cargo test   --manifest-path <fixture>/Cargo.toml <flags>

once with the fixture's default features and, for a fixture that declares a
`[features]` table of its own, again with `--no-default-features` and with
`--all-features`, so its minimal and its full surface are both exercised.
The dependency graph assertions (`expect.toml`) are `scripts/check_fixtures.py`;
`make test-022-consumers` runs both (#552).

Usage:
    test_consumers.py [scenario ...]   run every fixture, or the named ones
    test_consumers.py --list           print the plan without running it
    test_consumers.py --self-test      check the plan derivation
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "fixtures" / "consumers"
TARGET = ROOT / "target" / "fixtures"

# A `[features]` table header on a line of its own.
FEATURES_TABLE_RE = re.compile(r"^\s*\[features\]\s*$", re.M)


def feature_sets(manifest_text: str) -> list[list[str]]:
    """The cargo flag sets a fixture runs under.

    Every fixture runs with its defaults. One with features of its own also
    runs with none and with all of them.
    """
    sets: list[list[str]] = [[]]
    if FEATURES_TABLE_RE.search(manifest_text):
        sets += [["--no-default-features"], ["--all-features"]]
    return sets


def fixtures(names: list[str]) -> list[Path]:
    found = sorted(path.parent for path in FIXTURES.glob("*/Cargo.toml"))
    if not names:
        return found
    by_name = {path.name: path for path in found}
    unknown = [name for name in names if name not in by_name]
    if unknown:
        raise SystemExit(f"unknown fixture(s): {', '.join(unknown)}")
    return [by_name[name] for name in names]


def plan(scenario: Path) -> list[list[str]]:
    manifest = scenario / "Cargo.toml"
    commands: list[list[str]] = []
    for flags in feature_sets(manifest.read_text()):
        base = ["--manifest-path", str(manifest)]
        commands.append(["cargo", "clippy", *base, "--all-targets", *flags, "--", "-D", "warnings"])
        commands.append(["cargo", "test", *base, *flags])
    return commands


def self_test() -> int:
    cases = {
        '[package]\nname = "a"\n[dependencies]\nx = "1"\n': [[]],
        '[package]\nname = "a"\n\n[features]\nschema = ["x/schema"]\n': [
            [],
            ["--no-default-features"],
            ["--all-features"],
        ],
        # A table merely named like it does not count.
        '[package]\nname = "a"\n[dependencies]\nx = { version = "1", features = ["io"] }\n': [[]],
    }
    failures = 0
    for text, expected in cases.items():
        got = feature_sets(text)
        ok = got == expected
        failures += not ok
        print(f"self-test {'ok' if ok else 'FAIL'}: {expected} (got {got})")
    return 1 if failures else 0


def main() -> int:
    args = sys.argv[1:]
    if "--self-test" in args:
        return self_test()
    listing = "--list" in args
    names = [arg for arg in args if not arg.startswith("--")]

    selected = fixtures(names)
    if not selected:
        raise SystemExit(f"no fixture under {FIXTURES.relative_to(ROOT)}")
    for scenario in selected:
        env = dict(os.environ, CARGO_TARGET_DIR=str(TARGET / scenario.name))
        for command in plan(scenario):
            shown = " ".join(command).replace(str(ROOT) + "/", "")
            print(f"[{scenario.name}] {shown}", flush=True)
            if listing:
                continue
            if subprocess.run(command, cwd=ROOT, env=env, check=False).returncode != 0:
                print(f"FAILED: {scenario.name}: {shown}", file=sys.stderr)
                return 1
    if not listing:
        print(f"OK: {len(selected)} consumer fixture(s) linted and tested on their own")
    return 0


if __name__ == "__main__":
    sys.exit(main())
