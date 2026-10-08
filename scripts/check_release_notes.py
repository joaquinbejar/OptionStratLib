#!/usr/bin/env python3
"""Build and run every manifest example of the 0.22 release notes (#562).

The notes show minimal `Cargo.toml` dependency sections, each followed by a
Rust program. For every `toml` block this builds a standalone crate, outside
the repository, whose manifest carries that block exactly as shown plus a
`[patch.crates-io]` pointing the facade and every component at this
checkout (the crates are not published yet), with the following `rust` block
as `src/main.rs`, and runs it. A `toml` block not followed by a `rust` block,
or a notes file without any, fails, so an example cannot silently lose its
check. The doctests of the facade (`ReleaseNotesDoctests`) compile the same
programs with every feature on; this check proves the feature selection
shown is enough.

A program that writes `dec!` must have `rust_decimal` in its manifest
(#777): the prelude re-exports `rust_decimal_macros::dec`, which expands to
`::rust_decimal` paths, so the consumer depends on `rust_decimal` itself.
Every pair is held to that rule before anything builds, and `--self-test`
proves the rule against the compiler: a facade-only manifest whose program
writes `dec!` fails to compile without `rust_decimal` and compiles with it.

Usage: scripts/check_release_notes.py [notes.md]
       scripts/check_release_notes.py --self-test
Environment: CARGO_TARGET_DIR (default: target) holds the build under
`release-notes/`.
"""

import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
DEFAULT_NOTES = ROOT / "docs" / "release" / "0.22" / "RELEASE-NOTES.md"
COMPONENTS = (
    "core",
    "math",
    "pricing",
    "simulation",
    "market",
    "analytics",
    "strategies",
    "backtest",
    "visualization",
)
FENCE = re.compile(r"^```(\w*)\s*$")
USES_DEC = re.compile(r"\bdec!")
DECLARES_RUST_DECIMAL = re.compile(r"^\s*rust_decimal\s*=", re.MULTILINE)
# The cheapest facade build: core only. Its prelude still re-exports `dec!`.
SELF_TEST_FACADE = 'optionstratlib = { version = "0.22.0", default-features = false }\n'
SELF_TEST_RUST_DECIMAL = 'rust_decimal = "1.43"\n'
SELF_TEST_PROGRAM = """use optionstratlib::prelude::*;

fn main() {
    let rate: Decimal = dec!(0.05);
    assert!(rate > Decimal::ZERO);
}
"""


def code_blocks(text):
    """Yields `(language, body, line)` for every fenced block."""
    lines = text.split("\n")
    index = 0
    while index < len(lines):
        opening = FENCE.match(lines[index])
        if opening is None:
            index += 1
            continue
        start = index
        index += 1
        body = []
        while index < len(lines) and lines[index].strip() != "```":
            body.append(lines[index])
            index += 1
        if index == len(lines):
            raise SystemExit(f"unterminated code block at line {start + 1}")
        yield opening.group(1), "\n".join(body) + "\n", start + 1
        index += 1


def manifest_pairs(text):
    """Pairs each `toml` block with the `rust` block right after it."""
    blocks = list(code_blocks(text))
    pairs = []
    for position, (language, body, line) in enumerate(blocks):
        if language != "toml":
            continue
        following = blocks[position + 1] if position + 1 < len(blocks) else None
        if following is None or following[0] != "rust":
            raise SystemExit(f"line {line}: the toml block has no rust program after it")
        pairs.append((line, body, following[1]))
    return pairs


def patch_section():
    entries = [f'optionstratlib = {{ path = "{ROOT}" }}']
    for component in COMPONENTS:
        path = ROOT / "crates" / f"optionstratlib-{component}"
        entries.append(f'optionstratlib-{component} = {{ path = "{path}" }}')
    return "[patch.crates-io]\n" + "\n".join(entries) + "\n"


def missing_rust_decimal(dependencies, program):
    """True when `program` writes `dec!` and `dependencies` lacks `rust_decimal`."""
    return USES_DEC.search(program) is not None and DECLARES_RUST_DECIMAL.search(dependencies) is None


def cargo_environment():
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "release-notes"
    return dict(os.environ, CARGO_TARGET_DIR=str(target))


def write_crate(work, name, dependencies, program):
    """Writes a standalone crate with `dependencies` verbatim; returns its manifest."""
    crate = Path(work) / name
    (crate / "src").mkdir(parents=True)
    manifest = (
        "[package]\n"
        f'name = "{name}"\n'
        'version = "0.0.0"\n'
        'edition = "2024"\n'
        "publish = false\n\n"
        f"[dependencies]\n{dependencies}\n"
        f"{patch_section()}"
    )
    (crate / "Cargo.toml").write_text(manifest, encoding="utf-8")
    (crate / "src" / "main.rs").write_text(program, encoding="utf-8")
    return crate / "Cargo.toml"


def dependency_lines(block):
    """The body of a `[dependencies]` block, without its header."""
    lines = block.split("\n")
    if not lines or lines[0].strip() != "[dependencies]":
        raise SystemExit("a manifest example must start with `[dependencies]`")
    if any(line.strip().startswith("[") for line in lines[1:]):
        raise SystemExit("a manifest example may hold only `[dependencies]`")
    return "\n".join(lines[1:])


def check(notes):
    pairs = manifest_pairs(notes.read_text(encoding="utf-8"))
    if not pairs:
        raise SystemExit(f"{notes}: no toml example to check")
    for line, dependencies, program in pairs:
        if missing_rust_decimal(dependencies, program):
            raise SystemExit(
                f"line {line}: the program writes `dec!` but the manifest has no "
                "`rust_decimal` dependency, which `dec!` expands to (#777)"
            )
    environment = cargo_environment()
    with tempfile.TemporaryDirectory(prefix="osl-release-notes-") as work:
        for number, (line, dependencies, program) in enumerate(pairs, start=1):
            manifest = write_crate(
                work, f"osl-release-notes-{number}", dependency_lines(dependencies.rstrip("\n")), program
            )
            print(f"=== example {number} (line {line})", flush=True)
            subprocess.run(
                ["cargo", "run", "--quiet", "--manifest-path", str(manifest)],
                check=True,
                env=environment,
            )
    print(f"OK: {len(pairs)} manifest example(s) of {notes.relative_to(ROOT)} build and run")


def self_test():
    """Proves the `dec!` rule against the compiler (#777)."""
    without = SELF_TEST_FACADE
    with_rust_decimal = SELF_TEST_FACADE + SELF_TEST_RUST_DECIMAL
    if not missing_rust_decimal(without, SELF_TEST_PROGRAM):
        raise SystemExit("self-test: the rule accepted `dec!` without `rust_decimal`")
    if missing_rust_decimal(with_rust_decimal, SELF_TEST_PROGRAM):
        raise SystemExit("self-test: the rule refused `dec!` with `rust_decimal`")
    environment = cargo_environment()
    with tempfile.TemporaryDirectory(prefix="osl-release-notes-self-test-") as work:
        failing = write_crate(work, "osl-dec-without-rust-decimal", without, SELF_TEST_PROGRAM)
        result = subprocess.run(
            ["cargo", "check", "--quiet", "--manifest-path", str(failing)],
            env=environment,
            capture_output=True,
            text=True,
        )
        if result.returncode == 0:
            raise SystemExit(
                "self-test: a facade-only manifest compiled `dec!` without `rust_decimal`; "
                "the documented requirement (#777) no longer holds, update the docs and this rule"
            )
        if "rust_decimal" not in result.stderr:
            sys.stderr.write(result.stderr)
            raise SystemExit("self-test: the build without `rust_decimal` failed for another reason")
        passing = write_crate(work, "osl-dec-with-rust-decimal", with_rust_decimal, SELF_TEST_PROGRAM)
        subprocess.run(
            ["cargo", "check", "--quiet", "--manifest-path", str(passing)],
            check=True,
            env=environment,
        )
    print("OK: self-test: `dec!` fails without `rust_decimal` and compiles with it")


def main():
    try:
        if sys.argv[1:] == ["--self-test"]:
            self_test()
            return
        notes = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else DEFAULT_NOTES
        check(notes)
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"example failed: {error}") from error


if __name__ == "__main__":
    main()
