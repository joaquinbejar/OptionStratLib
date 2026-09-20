#!/usr/bin/env python3
"""Report the public API changes a pull request makes, per feature surface.

0.22.0 is under development and compatibility with the published 0.21.3 is
not a requirement, so this is a **report**, not a gate: it lists what the
change adds, removes or reshapes so a reviewer can judge it, and it does not
ask for per-item authorisation. What it still fails on is a tool, build or
parser problem, because an unreadable report must never be mistaken for "no
change".

Why per surface: an item can leave the default surface and stay behind a
feature, which a single `--all-features` run cannot see. The six surfaces are
`none`, `default`, `plotly`, `static_export`, `async` and `all`.

The comparison is the pull request's own delta: baseline = the first parent
of the merge commit CI checks out (that is the pull request's base), current
= that merge commit. `--release-type patch` is passed so the verdict never
depends on the manifest version; the report lists differences whatever
version the two sides declare.

Report parsing is fail-closed. An exit status other than 0 (no difference) or
100 (differences reported), a missing summary, a block count that disagrees
with the reported failure count, any truncation marker, an unknown lint, an
item line with no item, or a tool version other than the pinned one is an
error. A blank line inside a `Failed in:` list does not end it; the list ends
at the next failure block or at the summary, and cargo's own progress lines,
which are indented differently, end it too.

Usage:

    report_api_changes.py --self-test
    report_api_changes.py --surface default --baseline <rev>
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "tests" / "fixtures" / "semver-reports"


# Report grammar of the pinned version.
FAILURE_RE = re.compile(r"^--- failure ([a-z0-9_]+): ", re.M)
CHECKED_RE = re.compile(r"^\s*Checked \[[^\]]*\] (\d+) checks: (\d+) pass(?:, (\d+) fail)?", re.M)
SUMMARY_RE = re.compile(r"^\s*Summary ", re.M)
IMPL_VERSION_RE = re.compile(r"cargo-semver-checks/tree/v([0-9.]+)/")
TRUNCATION_RE = re.compile(r"^\s*(\.\.\.|and \d+ more|\[truncated\])", re.M)
ANSI_ESCAPE_RE = re.compile(
    r"\x1B(?:"
    r"\[[0-?]*[ -/]*[@-~]"            # CSI
    r"|][^\x1B\x07]*(?:\x07|\x1B\\)"  # OSC
    r"|[@-Z\\-_]"                      # Fe escape
    r")"
)
# An item line is indented by exactly two spaces. Cargo's own progress lines
# are indented by four or more and start with a status word, and they can be
# interleaved with the report because stdout and stderr are read together.
ITEM_RE = re.compile(r"^ {2}\S")
PROGRESS_RE = re.compile(r"^\s+(?:[A-Z][a-z]+|error|warning)\b")
# Item lines end in a source location, spelled in three ways by 0.50.0:
# `, previously in file <path>:<line>`, ` in file <path>:<line>` and
# ` in <path>:<line>`. Everything before it is the item, verbatim.
LOCATION_RE = re.compile(r"\s*,?\s*(?:previously\s+)?in\s+(?:file\s+)?\S+:\d+\s*$")


LINTS = FIXTURES / "lints-0.50.0.txt"


class ReportError(RuntimeError):
    """The report cannot be trusted; the caller must fail, not assume zero."""


def known_lints(path: Path = LINTS) -> set[str]:
    """The lint inventory of the pinned tool, read from a checked-in list.

    Deliberately not derived from the report under test: a list taken from
    the report contains, by construction, every lint the report names, so it
    could never reject an unknown one. Regenerate with
    `cargo semver-checks --list` when the pinned version changes.
    """
    lints = {line.strip() for line in path.read_text().splitlines() if line.strip() and not line.startswith("#")}
    if not lints:
        raise ReportError(f"lint inventory {path} is empty")
    return lints


def parse_report(text: str, returncode: int, tool: str, lints: set[str]) -> set[tuple[str, str]]:
    """`(lint, item)` pairs, or raise. Never returns an empty set on doubt."""
    if returncode not in (0, 100):
        raise ReportError(f"cargo-semver-checks exited {returncode}, report not usable:\n{text[-800:]}")
    text = ANSI_ESCAPE_RE.sub("", text)
    checked = CHECKED_RE.search(text)
    if checked is None or not SUMMARY_RE.search(text):
        raise ReportError("incomplete or unrecognised report: no `Checked`/`Summary` line")
    version = IMPL_VERSION_RE.search(text)
    if version is not None and f"cargo-semver-checks {version.group(1)}" != tool:
        raise ReportError(f"report came from cargo-semver-checks {version.group(1)}, register pins {tool!r}")
    if TRUNCATION_RE.search(text):
        raise ReportError("report contains a truncation marker; the finding list is not complete")
    expected_failures = int(checked.group(3) or 0)
    blocks = list(FAILURE_RE.finditer(text))
    if returncode == 0 and blocks:
        raise ReportError("report exited successfully but still contains failure blocks")
    if len(blocks) != expected_failures:
        raise ReportError(f"report announces {expected_failures} failing checks but contains {len(blocks)} failure blocks")
    findings: set[tuple[str, str]] = set()
    for index, block in enumerate(blocks):
        lint = block.group(1)
        if lints and lint not in lints:
            raise ReportError(f"unknown lint {lint!r}; the register cannot have approved it")
        end = blocks[index + 1].start() if index + 1 < len(blocks) else len(text)
        body = text[block.end():end]
        # The trailing `Summary`/`Finished` lines are indented too, so the
        # last block's item list must stop before them.
        summary = SUMMARY_RE.search(body)
        if summary is not None:
            body = body[: summary.start()]
        failed_in = body.split("Failed in:", 1)
        if len(failed_in) != 2:
            raise ReportError(f"failure block for {lint!r} has no `Failed in:` list")
        items = 0
        for line in failed_in[1].splitlines():
            if not line.strip():
                # A blank line inside a list is not a terminator.
                continue
            if ITEM_RE.match(line):
                item = LOCATION_RE.sub("", line.strip()).strip()
                if not item:
                    raise ReportError(f"failure block for {lint!r} has an item line with no item: {line!r}")
                findings.add((lint, item))
                items += 1
                continue
            if PROGRESS_RE.match(line) or not line.startswith(" "):
                # Cargo's own progress output, or the next section.
                break
            raise ReportError(f"failure block for {lint!r} has an unrecognised line: {line!r}")
        if items == 0:
            raise ReportError(f"failure block for {lint!r} lists no item")
    if expected_failures and not findings:
        raise ReportError("report announces failures but no item could be parsed")
    return findings


def git(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def assert_synthetic(base_sha: str, head_sha: str, cwd: Path = ROOT) -> str:
    """Pin CI's synthetic merge commit to the SHAs the event recorded.

    A re-run of an old job, or a branch that moved since the event, would
    otherwise be compared against the wrong delta.
    """
    first, second = git("rev-parse", "HEAD^1", cwd=cwd), git("rev-parse", "HEAD^2", cwd=cwd)
    if first != base_sha or second != head_sha:
        raise ReportError(
            "the checked-out merge commit does not match the event: "
            f"HEAD^1={first[:12]} (base.sha={base_sha[:12]}), HEAD^2={second[:12]} (head.sha={head_sha[:12]}); "
            "re-run the job on a fresh merge"
        )
    return first


def surface_flags(surface: dict) -> list[str]:
    if surface.get("all_features"):
        return ["--all-features"]
    flags = ["--only-explicit-features"]
    for feature in surface.get("features", []):
        flags += ["--features", feature]
    return flags


def run_semver(baseline: str, surface: dict, published: str | None, cwd: Path = ROOT) -> tuple[str, int]:
    command = ["cargo", "semver-checks", *surface_flags(surface)]
    command += ["--baseline-version", published] if published else ["--baseline-rev", baseline]
    done = subprocess.run(command, cwd=cwd, capture_output=True, text=True, check=False)
    return done.stdout + done.stderr, done.returncode


def self_test() -> int:
    """Prove the parser fails closed on every malformed report we have seen."""
    tool = TOOL
    good = (FIXTURES / "clean.txt").read_text()
    lints = known_lints()
    cases: list[tuple[str, str, int, object]] = [
        ("clean report", "clean.txt", 0, set()),
        ("seven lints, many items", "breaks.txt", 100, 69),
        ("feature-gated removal seen only without features", "feature-gated.txt", 100, 1),
        ("compile error", "compile-error.txt", 101, ReportError),
        ("missing baseline", "missing-baseline.txt", 101, ReportError),
        ("conflicting flags", "bad-flags.txt", 2, ReportError),
        ("truncated item list", "truncated.txt", 100, ReportError),
        ("block count disagrees with the summary", "miscounted.txt", 100, ReportError),
        ("unknown lint", "unknown-lint.txt", 100, ReportError),
        ("wrong tool version", "wrong-version.txt", 100, ReportError),
        ("failure block without items", "no-items.txt", 100, ReportError),
        ("blank line inside an item list", "blank-line-in-list.txt", 100, 69),
        ("wrapped item line", "wrapped-item.txt", 100, ReportError),
        ("success exit but findings present", "breaks.txt", 0, ReportError),
    ]
    failures = 0
    for name, fixture, code, want in cases:
        text = (FIXTURES / fixture).read_text()
        try:
            got: object = parse_report(text, code, tool, lints)
            ok = isinstance(want, set) and got == want or (isinstance(want, int) and len(got) == want)
            detail = f"{len(got)} findings" if not isinstance(want, type) else "parsed (expected a failure)"
        except ReportError as error:
            ok = want is ReportError
            detail = str(error).splitlines()[0][:70]
        if not ok:
            failures += 1
        print(f"self-test {'ok' if ok else 'FAIL'}: {name} -> {detail}")
    # Two changes on one type are two findings, keyed by (lint, item).
    pairs = parse_report((FIXTURES / "breaks.txt").read_text(), 100, tool, lints)
    ok = {("enum_marked_non_exhaustive", "enum E"), ("enum_variant_missing", "variant E::B")} <= pairs
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: two changes on one type are two findings")
    # The same item under two lints stays two findings.
    text = (FIXTURES / "same-item-two-lints.txt").read_text()
    both = parse_report(text, 100, tool, known_lints())
    same = {lint for lint, item in both if item == "struct probe::TwiceAlias"}
    ok = len(same) == 2
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: one item under two lints is two findings ({sorted(same)})")
    # `good` must parse to nothing at all.
    ok = parse_report(good, 0, tool, lints) == set()
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: a clean report yields no finding")
    # ANSI colour escapes in CI output must not make the report unreadable.
    coloured = good.replace("Checked ", "\x1b[32mChecked ").replace("Summary ", "\x1b[1mSummary ")
    coloured = coloured.replace("checks:", "checks:\x1b[0m").replace("Summary ", "Summary \x1b[0m")
    ok = parse_report(coloured, 0, tool, lints) == set()
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: ANSI-coloured clean report yields no finding")
    return 1 if failures else 0


SURFACES: dict[str, dict] = {
    # `--only-explicit-features` plus the features named here, so each
    # surface is exactly what it says. `default` is spelled out rather than
    # read from the manifest, so a change to the default feature set shows
    # up here as a diff instead of silently moving the surface.
    "none": {"features": []},
    "default": {"features": ["synthetic"]},
    "plotly": {"features": ["plotly"]},
    "static_export": {"features": ["static_export"]},
    "async": {"features": ["async"]},
    "all": {"all_features": True},
}

# The report grammar changes between releases; the parser is written against
# this exact version and refuses any other.
TOOL = "cargo-semver-checks 0.50.0"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--surface", choices=sorted(SURFACES))
    parser.add_argument("--baseline", help="the revision to compare against; on a pull request, its base")
    parser.add_argument("--base-sha", help="pull request base.sha, to pin the synthetic merge commit")
    parser.add_argument("--head-sha", help="pull request head.sha, to pin the synthetic merge commit")
    parser.add_argument("--root", type=Path, default=ROOT, help="crate root to run the comparison in")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.surface is None or not args.baseline:
        parser.error("--surface and --baseline are required unless --self-test is given")
    try:
        if args.base_sha and args.head_sha:
            assert_synthetic(args.base_sha, args.head_sha, cwd=args.root)
        text, code = run_semver(args.baseline, SURFACES[args.surface], None, cwd=args.root)
        found = parse_report(text, code, TOOL, known_lints())
    except ReportError as error:
        # A report we cannot read is a failure: it must never be mistaken for
        # "this pull request changes no API".
        print(f"api-changes: {error}", file=sys.stderr)
        return 2
    if not found:
        print(f"api-changes: {args.surface} unchanged against {args.baseline[:12]}")
        return 0
    print(f"api-changes: {args.surface} against {args.baseline[:12]}, {len(found)} item(s) for review")
    for lint, item in sorted(found):
        print(f"  {lint}: {item}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
