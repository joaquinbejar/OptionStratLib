#!/usr/bin/env python3
"""Report tests that start a browser without being `#[ignore]`d (#724).

PNG / SVG export goes through `plotly_static`, which spawns chromedriver and a
headless Chrome, and `show()` hands a chart to the default browser. A test
that does either inside `make test` or CI leaves browser processes behind on
every run, so those tests are `#[ignore]`d and run only by `make test-visual`
/ `make test-export`. Five `curves.rs` tests slipped past that rule and kept
exporting PNG on every run until #724; this script keeps it from happening
again.

A test is a `fn` whose attribute block carries `#[test]` or `#[tokio::test]`.
It is reported when the block has no `#[ignore]` and the function body calls
one of:

* `.write_png(` / `.write_svg(`
* `.write_image(` / `write_image_with_exporter(`
* `render(OutputType::Png(` / `render(OutputType::Svg(`
* `.show()`

A call that provably never reaches a browser (for example a write to a path
`prepare_file_path` rejects first) carries a
`// browser-test: allow -- <reason>` marker on the same line. A marker without
a reason is itself reported.

Scanned: `src/`, `tests/`, `benches/`, and every `crates/*/src` and
`crates/*/tests`.

Usage: scripts/check_browser_tests.py [--self-test]
"""

from __future__ import annotations

import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

BROWSER_CALL_RE = re.compile(
    r"\.write_png\("
    r"|\.write_svg\("
    r"|\.write_image\("
    r"|\bwrite_image_with_exporter\("
    r"|\brender\(\s*OutputType::(?:Png|Svg)\("
    r"|\.show\(\)"
)
TEST_ATTR_RE = re.compile(r"#\[\s*(?:tokio::)?test\b")
IGNORE_ATTR_RE = re.compile(r"#\[\s*ignore\b")
FN_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?(?:async\s+)?fn\s+(\w+)")
MARKER = "browser-test: allow"
MARKER_OK_RE = re.compile(r"browser-test: allow -- \S")
LINE_COMMENT_RE = re.compile(r"//.*$")
STRING_RE = re.compile(r'"(?:\\.|[^"\\])*"')


def code_of(line: str) -> str:
    """The line without string literals and its trailing `//` comment."""
    return LINE_COMMENT_RE.sub("", STRING_RE.sub('""', line))


def scan_file(path: Path) -> list[str]:
    """Violations in one source file, as `path:line: message` strings."""
    lines = path.read_text().splitlines()
    rel = path.relative_to(ROOT) if path.is_relative_to(ROOT) else path
    problems: list[str] = []

    for number, line in enumerate(lines, start=1):
        if MARKER in line and not MARKER_OK_RE.search(line):
            problems.append(f"{rel}:{number}: '{MARKER}' marker without a reason")

    attrs: list[str] = []
    index = 0
    while index < len(lines):
        stripped = lines[index].strip()
        if stripped.startswith("#["):
            attrs.append(stripped)
            index += 1
            continue
        fn = FN_RE.match(lines[index])
        if fn and any(TEST_ATTR_RE.search(a) for a in attrs):
            ignored = any(IGNORE_ATTR_RE.search(a) for a in attrs)
            end = body_end(lines, index)
            if not ignored:
                for number in range(index, end + 1):
                    raw = lines[number]
                    if BROWSER_CALL_RE.search(code_of(raw)) and not MARKER_OK_RE.search(raw):
                        problems.append(
                            f"{rel}:{number + 1}: test `{fn.group(1)}` starts a browser "
                            "but is not #[ignore]d"
                        )
            index = end + 1
            attrs = []
            continue
        if stripped and not stripped.startswith("//"):
            attrs = []
        index += 1
    return problems


def body_end(lines: list[str], start: int) -> int:
    """Index of the line that closes the function opened at `start`."""
    depth = 0
    opened = False
    for index in range(start, len(lines)):
        code = code_of(lines[index])
        depth += code.count("{") - code.count("}")
        if "{" in code:
            opened = True
        if opened and depth <= 0:
            return index
    return len(lines) - 1


def sources(root: Path) -> list[Path]:
    """Every Rust file under the scanned directories of `root`."""
    dirs = [root / "src", root / "tests", root / "benches"]
    for crate in sorted((root / "crates").glob("*")):
        dirs += [crate / "src", crate / "tests"]
    return sorted(p for d in dirs if d.is_dir() for p in d.rglob("*.rs"))


def self_test() -> int:
    """Prove the scanner reports what it must and nothing else."""
    cases = {
        "bad_png.rs": ("#[test]\nfn t() {\n    g.write_png(p).ok();\n}\n", 1),
        "bad_render.rs": (
            "#[test]\n#[cfg(feature = \"x\")]\nfn t() {\n    g.render(OutputType::Svg(&p));\n}\n",
            1,
        ),
        "bad_show.rs": ("#[tokio::test]\nasync fn t() {\n    g.show();\n}\n", 1),
        "ignored.rs": (
            "#[test]\n#[ignore = \"browser\"]\nfn t() {\n    g.write_svg(p).ok();\n}\n",
            0,
        ),
        "marked.rs": (
            "#[test]\nfn t() {\n    g.write_png(bad); // browser-test: allow -- rejected path\n}\n",
            0,
        ),
        "marker_no_reason.rs": (
            "#[test]\nfn t() {\n    g.write_png(bad); // browser-test: allow\n}\n",
            2,
        ),
        "not_a_test.rs": ("fn write(g: &G) {\n    g.write_png(p).ok();\n}\n", 0),
        "comment_and_string.rs": (
            "#[test]\nfn t() {\n    // g.write_png(p)\n    let s = \".write_png(\";\n}\n",
            0,
        ),
        "construct_only.rs": (
            "#[test]\nfn t() {\n    let o = OutputType::Png(&p);\n}\n",
            0,
        ),
        "next_fn_not_leaked.rs": (
            "#[test]\n#[ignore]\nfn a() {\n}\n\n#[test]\nfn b() {\n    g.write_png(p);\n}\n",
            1,
        ),
    }
    failed = 0
    with tempfile.TemporaryDirectory() as tmp:
        for name, (text, expected) in cases.items():
            path = Path(tmp) / name
            path.write_text(text)
            got = scan_file(path)
            if len(got) != expected:
                failed += 1
                print(f"self-test {name}: expected {expected} problem(s), got {got}")
    if failed:
        return 1
    print(f"browser-tests: self-test OK ({len(cases)} cases)")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    problems = [p for path in sources(ROOT) for p in scan_file(path)]
    if problems:
        print("Tests that start a browser must be #[ignore]d (run them with make test-visual):")
        for problem in problems:
            print(f"  {problem}")
        return 1
    print("browser-tests: OK")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
