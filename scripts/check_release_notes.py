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

Usage: scripts/check_release_notes.py [notes.md]
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


def check(notes):
    pairs = manifest_pairs(notes.read_text(encoding="utf-8"))
    if not pairs:
        raise SystemExit(f"{notes}: no toml example to check")
    target = Path(os.environ.get("CARGO_TARGET_DIR", ROOT / "target")) / "release-notes"
    environment = dict(os.environ, CARGO_TARGET_DIR=str(target))
    with tempfile.TemporaryDirectory(prefix="osl-release-notes-") as work:
        for number, (line, dependencies, program) in enumerate(pairs, start=1):
            crate = Path(work) / f"example-{number}"
            (crate / "src").mkdir(parents=True)
            manifest = (
                "[package]\n"
                f'name = "osl-release-notes-{number}"\n'
                'version = "0.0.0"\n'
                'edition = "2024"\n'
                "publish = false\n\n"
                f"{dependencies}\n"
                f"{patch_section()}"
            )
            (crate / "Cargo.toml").write_text(manifest, encoding="utf-8")
            (crate / "src" / "main.rs").write_text(program, encoding="utf-8")
            print(f"=== example {number} (line {line})", flush=True)
            subprocess.run(
                ["cargo", "run", "--quiet", "--manifest-path", str(crate / "Cargo.toml")],
                check=True,
                env=environment,
            )
    print(f"OK: {len(pairs)} manifest example(s) of {notes.relative_to(ROOT)} build and run")


def main():
    notes = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else DEFAULT_NOTES
    try:
        check(notes)
    except subprocess.CalledProcessError as error:
        raise SystemExit(f"example failed: {error}") from error


if __name__ == "__main__":
    main()
