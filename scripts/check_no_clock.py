#!/usr/bin/env python3
"""Prove the library builds and tests with the wall clock off (#944).

With the `clock` feature off, no package in the build may enable `chrono/clock`,
`expiration_date/clock` or `option_type/clock`, so `Utc::now` does not exist. For
every configuration below this script asks `cargo tree -e features -i chrono`,
the acceptance check of #944, over all targets and with dev-dependencies, because
features unify: a dev-dependency that enables the clock would make the test run
prove nothing. Each crate also has a `#[cfg(not(feature = "clock"))]` test that
fails if the clock turns out to be on (`tests/no_clock.rs`).

The default build must keep the clock, so that is asserted too.

Features are not the whole story: `std::time` needs none, so this also scans the
production sources. `utils/time.rs` is the one file that may read the clock, with
`Utc::now` or `Local::now`. `SystemTime::now` is nowhere, and `Instant::now` only
where the same function measures a duration with `.elapsed()`. The scan reads text:
`disallowed-methods` in `clippy.toml` is what resolves an alias or a call split across
lines. An `Instant` used as a seed passes both: that one is not caught.

Usage:
    check_no_clock.py               assert the trees, then lint, build, test and document
    check_no_clock.py --trees       assert the trees only
    check_no_clock.py --self-test   prove the tree assertion
"""

from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

FACADE = "optionstratlib"
COMPONENTS = [
    "optionstratlib-core",
    "optionstratlib-math",
    "optionstratlib-pricing",
    "optionstratlib-simulation",
    "optionstratlib-market",
    "optionstratlib-analytics",
    "optionstratlib-strategies",
    "optionstratlib-backtest",
    "optionstratlib-visualization",
]

# Every optional feature of a component, on top of the bare crate.
COMPONENT_FEATURES = {
    "optionstratlib-core": "schema",
    "optionstratlib-math": "schema",
    "optionstratlib-pricing": "schema",
    "optionstratlib-simulation": "schema",
    "optionstratlib-market": "io,async,synthetic,schema",
    "optionstratlib-analytics": "schema",
    "optionstratlib-strategies": "schema",
    "optionstratlib-backtest": "schema",
    "optionstratlib-visualization": "plotly,static_export",
}

# The facade with no capability, with each one alone, and with all of them.
# Each capability is built and tested by `make lint` as well (FACADE_FEATURE_SETS).
EVERYTHING = "visualization,synthetic,io,async,schema,plotly,static_export"
FACADE_TREES = [
    "",
    "math",
    "schema",
    "pricing",
    "market",
    "io",
    "async",
    "synthetic",
    "analytics",
    "strategies",
    "simulation",
    "backtest",
    "visualization",
    "plotly",
    "static_export",
    EVERYTHING,
]
FACADE_TESTS = ["", "pricing", "strategies", "backtest", EVERYTHING]
# With the defaults off and `clock` named, as the migration note tells a consumer to do:
# nothing else lints or tests these, and each default build has every capability on.
CLOCK_NAMED = ["clock", "clock,backtest"]

# What `cargo tree -e features -i chrono` prints when something turned it on.
LEAK_RE = re.compile(
    r'(?:chrono|expiration_date|option_type) feature "(?:clock|now|wasmbind)"|iana-time-zone'
)


# A read of the clock in production code. `Instant` is monotonic and never tells the
# time, so it is held to the narrower rule in `clock_reads`.
CLOCK_READ_RE = re.compile(r"\b(?:Utc|Local)\s*>?\s*::\s*now\b")
SYSTEM_TIME_RE = re.compile(r"\bSystemTime\s*>?\s*::\s*now\b|\bUNIX_EPOCH\s*\.\s*elapsed\b")
INSTANT_RE = re.compile(r"(?:let\s+(?:mut\s+)?(\w+)\s*=\s*)?(?:std::time::)?\bInstant::now\s*\(")
# The only file that may read the clock (`wall_clock` and the local date helper).
CLOCK_FILE = "crates/optionstratlib-core/src/utils/time.rs"


def production_lines(text: str) -> list[tuple[int, str]]:
    """The numbered lines outside comments and `#[cfg(test)]` items."""
    out: list[tuple[int, str]] = []
    lines = text.splitlines()
    i = 0
    while i < len(lines):
        stripped = lines[i].strip()
        if stripped == "#[cfg(test)]" or stripped.startswith("#[cfg(all(test,"):
            depth = 0
            started = False
            i += 1
            while i < len(lines):
                opens = lines[i].count("{")
                started = started or opens > 0
                depth += opens - lines[i].count("}")
                if (started and depth <= 0) or (not started and lines[i].rstrip().endswith(";")):
                    break
                i += 1
            i += 1
            continue
        if not stripped.startswith("//"):
            out.append((i + 1, lines[i]))
        i += 1
    return out


def code_of(line: str) -> str:
    """A line without its trailing comment. A `//` inside a string, a URL say, is code."""
    in_string = False
    index = 0
    while index < len(line):
        char = line[index]
        if in_string:
            if char == "\\":
                index += 1
            elif char == '"':
                in_string = False
        elif char == '"':
            in_string = True
        elif line.startswith("//", index):
            return line[:index]
        index += 1
    return line


def clock_reads(path: str, text: str) -> list[str]:
    """Where one production source reads the clock against the rule."""
    found = []
    lines = production_lines(text)
    for index, (number, line) in enumerate(lines):
        code = code_of(line)
        if SYSTEM_TIME_RE.search(code) or (CLOCK_READ_RE.search(code) and path != CLOCK_FILE):
            found.append(f"{path}:{number}: {line.strip()}")
        instant = INSTANT_RE.search(code)
        if instant:
            name = instant.group(1)
            rest = "\n".join(following for _, following in lines[index + 1 : index + 60])
            if name is None or f"{name}.elapsed()" not in rest:
                found.append(f"{path}:{number}: `Instant::now` that measures no duration: {line.strip()}")
    return found


def check_sources() -> list[str]:
    found = []
    sources = [*ROOT.glob("crates/*/src/**/*.rs"), *ROOT.glob("src/**/*.rs")]
    for source in sorted(sources):
        found += clock_reads(str(source.relative_to(ROOT)), source.read_text(encoding="utf-8"))
    return found


def leaks(tree: str) -> list[str]:
    """The lines of a tree that show the clock enabled."""
    return [line for line in tree.splitlines() if LEAK_RE.search(line)]


def run(*args: str) -> subprocess.CompletedProcess[str]:
    return subprocess.run(args, cwd=ROOT, capture_output=True, text=True)


def tree(package: str, features: str, *, default: bool = False) -> str:
    command = ["cargo", "tree", "--color", "never", "-e", "features", "-i", "chrono", "--target", "all"]
    command += ["-p", package]
    if not default:
        command.append("--no-default-features")
    if features:
        command += ["--features", features]
    result = run(*command)
    if result.returncode != 0:
        raise SystemExit(f"cargo tree failed for {package} [{features}]:\n{result.stderr}")
    return result.stdout


def describe(package: str, features: str) -> str:
    return f"{package} --no-default-features" + (f" --features {features}" if features else "")


def check_trees() -> list[str]:
    problems = [f"reads the clock outside `{CLOCK_FILE}`: {found}" for found in check_sources()]
    configurations = [(package, "") for package in COMPONENTS]
    configurations += [(package, COMPONENT_FEATURES[package]) for package in COMPONENTS]
    configurations += [(FACADE, features) for features in FACADE_TREES]
    for package, features in configurations:
        lines = leaks(tree(package, features))
        if lines:
            problems.append(
                f"{describe(package, features)} enables the clock. "
                f"`cargo tree -e features -i chrono` shows:\n  " + "\n  ".join(lines)
            )
    for package in [FACADE, *(p for p in COMPONENTS if p != "optionstratlib-math")]:
        if not leaks(tree(package, "", default=True)):
            problems.append(f"{package} lost the clock in its default build")
    return problems


def cargo(*args: str) -> None:
    print("+ cargo " + " ".join(args), flush=True)
    env = dict(os.environ, LOGLEVEL="WARN")
    if subprocess.run(["cargo", *args], cwd=ROOT, env=env).returncode != 0:
        raise SystemExit(f"cargo {' '.join(args)} failed")


def build_and_test() -> None:
    configurations = [(package, "") for package in COMPONENTS]
    configurations += [
        (package, COMPONENT_FEATURES[package])
        for package in ("optionstratlib-market", "optionstratlib-visualization")
    ]
    configurations += [(FACADE, features) for features in FACADE_TESTS]
    for package, features in configurations:
        flags = ["-p", package, "--no-default-features"]
        if features:
            flags += ["--features", features]
        cargo("clippy", *flags, "--all-targets", "--", "-D", "warnings")
        cargo("build", *flags)
        cargo("test", *flags)
        doc = subprocess.run(
            ["cargo", "doc", "--no-deps", *flags],
            cwd=ROOT,
            env=dict(os.environ, RUSTDOCFLAGS="-D warnings"),
        )
        if doc.returncode != 0:
            raise SystemExit(f"cargo doc {' '.join(flags)} failed")


def lint_and_test_clock_named() -> None:
    for features in CLOCK_NAMED:
        flags = ["-p", FACADE, "--no-default-features", "--features", features]
        cargo("clippy", *flags, "--all-targets", "--", "-D", "warnings")
        cargo("test", *flags)


def self_test() -> int:
    clean = """chrono v0.4.45
\u251c\u2500\u2500 chrono feature "serde"
\u2514\u2500\u2500 chrono feature "std"
    \u2514\u2500\u2500 expiration_date v0.6.0
"""
    leaked = """chrono v0.4.45
\u251c\u2500\u2500 chrono feature "clock"
\u2502   \u2514\u2500\u2500 expiration_date feature "clock"
\u2514\u2500\u2500 chrono feature "std"
"""
    cases = {
        "a tree without the clock is clean": (leaks(clean), 0),
        "`clock` and what enables it are reported": (leaks(leaked), 2),
        "`now` alone is a leak": (leaks('chrono feature "now"'), 1),
        "`wasmbind` alone is a leak": (leaks('chrono feature "wasmbind"'), 1),
        "option_type forwards it": (leaks('option_type feature "clock"'), 1),
        "a package reading the time zone is a leak": (leaks("iana-time-zone v0.1.65"), 1),
        "another crate's `clock` is not ours": (leaks('tokio feature "clock"'), 0),
    }
    other = "crates/x/src/y.rs"
    cases |= {
        "`Utc::now` elsewhere": (clock_reads(other, "fn f() { let t = Utc::now(); }"), 1),
        "`chrono::Utc::now` elsewhere": (clock_reads(other, "let t = chrono::Utc::now();"), 1),
        "`Local::now` elsewhere": (clock_reads(other, "let t = Local::now();"), 1),
        "`SystemTime::now` elsewhere": (clock_reads(other, "let t = SystemTime::now();"), 1),
        "`std::time::SystemTime::now`": (clock_reads(other, "let t = std::time::SystemTime::now();"), 1),
        "`Utc::now` in the clock file": (clock_reads(CLOCK_FILE, "fn f() { Utc::now() }"), 0),
        "`SystemTime::now` in the clock file": (clock_reads(CLOCK_FILE, "let t = SystemTime::now();"), 1),
        "passed as a function": (clock_reads(other, "let t = x.unwrap_or_else(SystemTime::now);"), 1),
        "`Utc::now` passed as a function": (clock_reads(other, "let t = x.unwrap_or_else(Utc::now);"), 1),
        "`UNIX_EPOCH.elapsed`": (clock_reads(other, "let t = UNIX_EPOCH.elapsed();"), 1),
        "a qualified path": (clock_reads(other, "let t = <SystemTime>::now();"), 1),
        "spaced": (clock_reads(other, "let t = SystemTime :: now();"), 1),
        "after a URL on the line": (clock_reads(other, 'let u = "http://x"; let t = Utc::now();'), 1),
        "a URL alone is not a read": (clock_reads(other, 'let u = "http://x";'), 0),
        "a comment is not a read": (clock_reads(other, "// Utc::now()"), 0),
        "a trailing comment is not a read": (clock_reads(other, "let x = 1; // Utc::now()"), 0),
        "a test module is not production": (
            clock_reads(other, "#[cfg(test)]\nmod t {\n    fn f() { Utc::now(); }\n}\n"),
            0,
        ),
        "a gated test module is not production": (
            clock_reads(other, '#[cfg(all(test, feature = "clock"))]\nmod t {\n    fn f() { Utc::now(); }\n}\n'),
            0,
        ),
        "production after a test module": (
            clock_reads(other, "#[cfg(test)]\nmod t {}\nfn f() { Utc::now(); }\n"),
            1,
        ),
        "an `Instant` measuring a duration": (
            clock_reads(other, "let started = Instant::now();\nwork();\nlet d = started.elapsed();\n"),
            0,
        ),
        "an `Instant` kept as a time": (clock_reads(other, "let started = Instant::now();\nkeep(started);\n"), 1),
        "an `Instant` with no binding": (clock_reads(other, "keep(Instant::now());\n"), 1),
    }
    failed = [name for name, (found, want) in cases.items() if len(found) != want]
    for name in failed:
        print(f"self-test failed: {name}")
    if not failed:
        print(f"OK: {len(cases)} self-test cases")
    return 1 if failed else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    problems = check_trees()
    if problems:
        print("\n".join(problems))
        return 1
    print("OK: no configuration without `clock` enables `chrono/clock`, and the default builds keep it")
    if "--trees" not in sys.argv:
        build_and_test()
        lint_and_test_clock_named()
        print("OK: every configuration without `clock` lints, builds, tests and documents")
    return 0


if __name__ == "__main__":
    sys.exit(main())
