#!/usr/bin/env python3
"""Compare `cargo-semver-checks` reports against the register of authorised
incompatible changes (multi-crate roadmap M1-17, #592).

The manifests stay at 0.21.3 and nothing is published while the migration
runs, so the ordinary `semver` job compares against the published 0.21.3 and
refuses every removal. This check keeps that comparison *and* adds two more,
so that an authorised break is expected exactly once and anything else fails:

* **C1 cumulative**: baseline = the published `0.21.3`. Sees every break that
  a consumer of the released crate would see, and nothing that was added
  after it.
* **C2 controlled reference**: baseline = the newest commit carrying an
  `Accepted-Breaks:` trailer among the ancestors of the state *before* the
  change under test (ancestors of the pull request's base commit, or of
  `HEAD^` on a push to `main`, never the commit under test itself, which
  would compare it with itself). Until such a commit exists, the register's
  `initial_reference` tag is used.
* **C3 incremental**: baseline = the first parent of the synthetic merge
  commit that CI checks out for a pull request, current = that commit. Sees
  exactly the delta the pull request adds to its base, including the removal
  of an item that was added after 0.21.3 and therefore invisible to C1.

Each register entry declares, per check, which findings it expects, because
the three comparisons do not see the same set and the same change can be
reported by different lints against different baselines.

Verdicts, any of which fails the run:

* **unforeseen**: a finding no entry declares;
* **unapproved**: a finding declared by an entry whose `status` is not
  `approved` or `landed`;
* **stale**: an entry declares a finding for this check that the report does
  not contain;
* **misdeclared**: an entry declares the finding for a surface or a check
  that did not produce it.

Report parsing is fail-closed: an exit status other than 0 (no break) or 100
(breaks reported), a missing summary, a block count that disagrees with the
reported failure count, any truncation marker, an unknown lint or a tool
version other than the pinned one is an error, never "zero breaks".

Usage:

    check_accepted_breaks.py --self-test
    check_accepted_breaks.py --check C3 --surface default --baseline <rev>
    check_accepted_breaks.py --all            # every surface, C1, C2, C3

Requires Python 3.11 or newer (`tomllib`).
"""

from __future__ import annotations

import argparse
import re
import subprocess
import sys
from pathlib import Path

try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover - guarded at call time
    tomllib = None

ROOT = Path(__file__).resolve().parent.parent
REGISTER = ROOT / "public-api" / "accepted-breaks.toml"
FIXTURES = ROOT / "tests" / "fixtures" / "semver-reports"

CHECKS = ("C1", "C2", "C3")
TRAILER = "Accepted-Breaks:"

# Report grammar of the pinned version.
FAILURE_RE = re.compile(r"^--- failure ([a-z0-9_]+): ", re.M)
CHECKED_RE = re.compile(r"^\s*Checked \[[^\]]*\] (\d+) checks: (\d+) pass(?:, (\d+) fail)?", re.M)
SUMMARY_RE = re.compile(r"^\s*Summary ", re.M)
IMPL_VERSION_RE = re.compile(r"cargo-semver-checks/tree/v([0-9.]+)/")
TRUNCATION_RE = re.compile(r"^\s*(\.\.\.|and \d+ more|\[truncated\])", re.M)
# Item lines end in a source location, spelled in three ways by 0.50.0:
# `, previously in file <path>:<line>`, ` in file <path>:<line>` and
# ` in <path>:<line>`. Everything before it is the item, verbatim.
LOCATION_RE = re.compile(r"\s*,?\s*(?:previously\s+)?in\s+(?:file\s+)?\S+:\d+\s*$")


class ReportError(RuntimeError):
    """The report cannot be trusted; the caller must fail, not assume zero."""


def load_register(path: Path = REGISTER) -> dict:
    if tomllib is None:
        raise ReportError("Python 3.11 or newer is required to read the register (tomllib)")
    with path.open("rb") as handle:
        return tomllib.load(handle)


def known_lints(text: str) -> set[str]:
    """Every lint named by a report, taken from its own `--- failure` lines."""
    return set(FAILURE_RE.findall(text))


def parse_report(text: str, returncode: int, tool: str, lints: set[str]) -> set[tuple[str, str]]:
    """`(lint, item)` pairs, or raise. Never returns an empty set on doubt."""
    if returncode not in (0, 100):
        raise ReportError(f"cargo-semver-checks exited {returncode}, report not usable:\n{text[-800:]}")
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
    if len(blocks) != expected_failures:
        raise ReportError(f"report announces {expected_failures} failing checks but contains {len(blocks)} failure blocks")
    findings: set[tuple[str, str]] = set()
    for index, block in enumerate(blocks):
        lint = block.group(1)
        if lints and lint not in lints:
            raise ReportError(f"unknown lint {lint!r}; the register cannot have approved it")
        end = blocks[index + 1].start() if index + 1 < len(blocks) else len(text)
        body = text[block.end():end]
        failed_in = body.split("Failed in:", 1)
        if len(failed_in) != 2:
            raise ReportError(f"failure block for {lint!r} has no `Failed in:` list")
        items = 0
        for line in failed_in[1].splitlines():
            if not line.startswith("  ") or not line.strip():
                if items:
                    break
                continue
            item = LOCATION_RE.sub("", line.strip()).strip()
            findings.add((lint, item))
            items += 1
        if items == 0:
            raise ReportError(f"failure block for {lint!r} lists no item")
    if expected_failures and not findings:
        raise ReportError("report announces failures but no item could be parsed")
    return findings


def git(*args: str, cwd: Path = ROOT) -> str:
    return subprocess.run(["git", *args], cwd=cwd, capture_output=True, text=True, check=True).stdout.strip()


def reference_commit(before: str, register: dict, cwd: Path = ROOT) -> str:
    """Newest `Accepted-Breaks:` commit among the ancestors of `before`.

    `before` is the state the change under test starts from: a pull request's
    base commit, or `HEAD^` on a push. The commit under test is never a
    candidate, so an integrating push cannot be compared with itself.
    """
    out = subprocess.run(
        ["git", "log", "--format=%H", f"--grep=^{TRAILER}", before],
        cwd=cwd, capture_output=True, text=True, check=False,
    ).stdout.split()
    if out:
        return out[0]
    return git("rev-parse", register["register"]["initial_reference"], cwd=cwd)


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


def expected(register: dict, check: str, surface: str, landed_only: bool) -> tuple[set[tuple[str, str]], set[tuple[str, str]]]:
    """`(expected, unapproved)` findings for one check and surface."""
    ready: set[tuple[str, str]] = set()
    pending: set[tuple[str, str]] = set()
    for entry in register.get("break", []):
        if surface not in entry.get("surfaces", []):
            continue
        for finding in entry.get("findings", []):
            if finding.get("check") != check:
                continue
            key = (finding["lint"], finding["item"])
            approved = entry.get("status") in ("approved", "landed")
            if approved and (entry.get("status") == "landed" or not landed_only):
                ready.add(key)
            else:
                pending.add(key)
    return ready, pending


def verdicts(found: set, ready: set, pending: set, check: str, surface: str) -> list[str]:
    problems = []
    for lint, item in sorted(found - ready - pending):
        problems.append(f"unforeseen break on {surface}/{check}: {lint} {item}")
    for lint, item in sorted(found & pending):
        problems.append(f"unapproved break on {surface}/{check}: {lint} {item} (entry is not approved)")
    for lint, item in sorted(ready - found):
        problems.append(f"stale entry on {surface}/{check}: {lint} {item} is declared but not reported")
    return problems


def self_test() -> int:
    """Prove the parser fails closed on every malformed report we have seen."""
    register = load_register()
    tool = register["register"]["tool"]
    good = (FIXTURES / "clean.txt").read_text()
    lints = known_lints((FIXTURES / "breaks.txt").read_text())
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
        ("success exit but findings present", "clean.txt", 0, set()),
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
    both = parse_report(text, 100, tool, known_lints(text))
    same = {lint for lint, item in both if item == "struct probe::TwiceAlias"}
    ok = len(same) == 2
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: one item under two lints is two findings ({sorted(same)})")
    # An empty register expects nothing, so any finding is unforeseen.
    problems = verdicts(pairs, set(), set(), "C1", "default")
    ok = len(problems) == len(pairs)
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: every finding is unforeseen against an empty register ({len(problems)})")
    # `good` must parse to nothing at all.
    ok = parse_report(good, 0, tool, lints) == set()
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: a clean report yields no finding")
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--check", choices=CHECKS)
    parser.add_argument("--surface")
    parser.add_argument("--baseline", help="rev for C2/C3; ignored for C1")
    parser.add_argument("--base-sha", help="pull request base.sha, to pin the synthetic merge commit")
    parser.add_argument("--head-sha", help="pull request head.sha, to pin the synthetic merge commit")
    parser.add_argument("--landed-only", action="store_true", help="on `main`: only `landed` entries may be expected")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    register = load_register()
    surfaces = {surface["id"]: surface for surface in register["register"]["surfaces"]}
    if args.surface not in surfaces or args.check is None:
        parser.error("--check and --surface are required unless --self-test is given")
    try:
        if args.base_sha and args.head_sha:
            assert_synthetic(args.base_sha, args.head_sha)
        published = register["register"]["published"] if args.check == "C1" else None
        text, code = run_semver(args.baseline or "", surfaces[args.surface], published)
        found = parse_report(text, code, register["register"]["tool"], known_lints(text))
    except ReportError as error:
        print(f"accepted-breaks: {error}", file=sys.stderr)
        return 2
    ready, pending = expected(register, args.check, args.surface, args.landed_only)
    problems = verdicts(found, ready, pending, args.check, args.surface)
    for problem in problems:
        print(f"accepted-breaks: {problem}")
    if problems:
        return 1
    print(f"accepted-breaks OK: {args.surface}/{args.check} matches the register ({len(found)} authorised findings)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
