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
reported failure count, any truncation marker, an unknown lint, an item line
with no item (a wrapped line) or a tool version other than the pinned one is
an error, never "zero breaks". A blank line inside a `Failed in:` list does
not end it; the list ends at the next failure block or at the summary.

What the script enforces about authorisation, and what it does not: it
checks the register's structure (an approved entry carries an approval
reference pointing at the register issue, a decision, a migration note, at
least one surface and one declared finding, and no stored commit SHA) and,
with `--verify-approvals`, that each approval comment exists and was written
by the register's `owner`. It cannot stop the owner's account from being
used by someone else; that is a procedural control, and `CODEOWNERS` with a
second identity is the technical alternative.

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
ANSI_ESCAPE_RE = re.compile(r"\x1b\[[0-9;]*[A-Za-z]")
# An item line is indented by exactly two spaces. Cargo's own progress lines
# are indented by four or more and start with a status word, and they can be
# interleaved with the report because stdout and stderr are read together.
ITEM_RE = re.compile(r"^ {2}\S")
PROGRESS_RE = re.compile(r"^\s+(?:[A-Z][a-z]+|error|warning)\b")
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


LINTS = FIXTURES / "lints-0.50.0.txt"


def known_lints(_text: str | None = None, path: Path = LINTS) -> set[str]:
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


def reference_commit(before: str, register: dict, cwd: Path = ROOT) -> str:  # noqa: D401
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


def landing_commit(entry_id: str, cwd: Path = ROOT) -> str | None:
    """The commit on the current history whose trailer announces this entry."""
    out = subprocess.run(
        ["git", "log", "--format=%H%x00%B%x00", f"--grep=^{TRAILER}", "HEAD"],
        cwd=cwd, capture_output=True, text=True, check=False,
    ).stdout
    for chunk in out.split("\x00\n"):
        if not chunk.strip():
            continue
        sha, _, message = chunk.partition("\x00")
        for line in message.splitlines():
            if line.startswith(TRAILER) and entry_id in [part.strip() for part in line[len(TRAILER):].split(",")]:
                return sha.strip()
    return None


def in_delta(entry: dict, baseline: str, check: str, pr: int | None, cwd: Path = ROOT) -> bool:
    """Is this entry's break inside the delta the comparison measures?

    C1 measures against the published crate, which contains no landing at
    all, so every authorised break is expected there for ever. C2 and C3
    measure against a commit: a break that landed before it is already part
    of the baseline and must NOT be expected again, which is what lets an
    ordinary pull request stay green without a label once a break has landed.
    """
    if check == "C1":
        return True
    landed = landing_commit(entry.get("id", ""), cwd=cwd)
    if landed is None:
        # Not on this history yet: it is landing in this pull request.
        return pr is not None and (entry.get("landed") or {}).get("pr") == pr
    reachable = subprocess.run(
        ["git", "merge-base", "--is-ancestor", landed, baseline],
        cwd=cwd, capture_output=True, text=True, check=False,
    ).returncode == 0
    return not reachable


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


def authorised(entry: dict, register: dict) -> str | None:
    """Why this entry may not be treated as authorised, or `None` when it may."""
    if entry.get("status") not in ("approved", "landed"):
        return f"status is {entry.get('status')!r}"
    reference = (entry.get("approval") or {}).get("ref", "")
    issue = register.get("register", {}).get("issue")
    if not reference:
        return "no approval reference"
    if issue and f"/{issue}#issuecomment-" not in reference:
        return f"approval reference does not point at a comment on issue #{issue}"
    if not entry.get("decision") or not entry.get("migration"):
        return "no decision or no migration note"
    return None


def expected(
    register: dict, check: str, surface: str, landed_only: bool,
    baseline: str = "", pr: int | None = None, cwd: Path = ROOT, scope: bool = True,
) -> tuple[set[tuple[str, str]], set[tuple[str, str]], dict[tuple[str, str], str], set[tuple[str, str]]]:
    """`(expected, unapproved, elsewhere, tolerated)` for one check and surface.

    `elsewhere` holds findings the register declares for *other* checks or
    surfaces: reported here, they are misdeclared, not unforeseen, and the
    message says which entry to correct. `tolerated` holds authorised breaks
    that landed before this comparison's baseline: the comparison may or may
    not report them, and neither is a problem.
    """
    ready: set[tuple[str, str]] = set()
    pending: set[tuple[str, str]] = set()
    elsewhere: dict[tuple[str, str], str] = {}
    tolerated: set[tuple[str, str]] = set()
    for entry in register.get("break", []):
        here = surface in entry.get("surfaces", [])
        refusal = authorised(entry, register)
        # A break that landed before this comparison's baseline is already
        # part of it; expecting it again would fail every later pull request.
        within = True if not scope else in_delta(entry, baseline, check, pr, cwd=cwd)
        for finding in entry.get("findings", []):
            key = (finding["lint"], finding["item"])
            if not here or finding.get("check") != check:
                elsewhere.setdefault(key, f"{entry.get('id', '?')} declares it for {finding.get('check')}/{entry.get('surfaces', [])}")
                continue
            if refusal is None and (entry.get("status") == "landed" or not landed_only) and within:
                ready.add(key)
            elif refusal is None and not within:
                # Authorised and already inside the baseline: the comparison
                # need not report it, and may.
                tolerated.add(key)
            else:
                pending.add(key)
    for key in ready | pending | tolerated:
        elsewhere.pop(key, None)
    return ready, pending, elsewhere, tolerated


def verdicts(
    found: set, ready: set, pending: set, check: str, surface: str,
    elsewhere: dict | None = None, tolerated: set | None = None,
) -> list[str]:
    elsewhere = elsewhere or {}
    tolerated = tolerated or set()
    problems = []
    for lint, item in sorted(found - ready - pending - set(elsewhere) - tolerated):
        problems.append(f"unforeseen break on {surface}/{check}: {lint} {item}")
    for lint, item in sorted(found & set(elsewhere)):
        problems.append(f"misdeclared break on {surface}/{check}: {lint} {item} ({elsewhere[(lint, item)]})")
    for lint, item in sorted(found & pending):
        problems.append(f"unapproved break on {surface}/{check}: {lint} {item} (entry is not approved)")
    for lint, item in sorted(ready - found):
        problems.append(f"stale entry on {surface}/{check}: {lint} {item} is declared but not reported")
    return problems


def self_approval(register: dict, against: str, path: Path, cwd: Path = ROOT) -> list[str]:
    """Refuse a pull request that authorises its own break.

    Only the owner may move an entry to `approved`, in a change of its own.
    A landing pull request may fill `landed.pr` and flip `approved` to
    `landed`, nothing else.
    """
    if tomllib is None:
        return ["Python 3.11 or newer is required to read the register (tomllib)"]
    relative = path.relative_to(cwd) if path.is_absolute() else path
    done = subprocess.run(["git", "show", f"{against}:{relative}"], cwd=cwd, capture_output=True, text=True, check=False)
    before = tomllib.loads(done.stdout) if done.returncode == 0 else {"break": []}
    previous = {entry.get("id"): entry for entry in before.get("break", [])}
    problems = []
    for entry in register.get("break", []):
        name = entry.get("id")
        was = previous.get(name)
        status = entry.get("status")
        if status in ("approved", "landed") and (was is None or was.get("status") == "proposed"):
            problems.append(
                f"register: {name} is authorised by this change itself; approval belongs to a separate change by the owner"
            )
        if was is not None and status == "landed" and was.get("status") == "landed":
            continue
        if was is not None and (entry.get("approval") or {}) != (was.get("approval") or {}) and status != "proposed":
            problems.append(f"register: {name} changes its approval reference in the same change that uses it")
    return problems


def validate_register(register: dict) -> list[str]:
    """Structural rules the register must satisfy before any comparison counts.

    What this enforces in code: an approved or landed entry carries an
    approval reference pointing at the register issue, a migration note, a
    decision, at least one declared finding and at least one surface, and no
    entry stores a commit SHA (the landing commit is resolved from the
    `Accepted-Breaks:` trailer). What it does **not** enforce: that the
    approval comment was written by the owner. Reading it needs the network,
    so `--verify-approvals` does that in CI; without it the owner rule is a
    procedural control, checked by human review.
    """
    problems: list[str] = []
    meta = register.get("register", {})
    issue = meta.get("issue")
    for entry in register.get("break", []):
        name = entry.get("id", "<entry without id>")
        if not entry.get("id"):
            problems.append("register: an entry has no id")
        if entry.get("status") not in ("proposed", "approved", "landed"):
            problems.append(f"register: {name} has status {entry.get('status')!r}")
        if not entry.get("surfaces"):
            problems.append(f"register: {name} declares no surface")
        if not entry.get("findings"):
            problems.append(f"register: {name} declares no finding")
        for finding in entry.get("findings", []):
            if finding.get("check") not in CHECKS or not finding.get("lint") or not finding.get("item"):
                problems.append(f"register: {name} has a malformed finding {finding!r}")
        if entry.get("status") in ("approved", "landed"):
            reference = (entry.get("approval") or {}).get("ref", "")
            if not reference:
                problems.append(f"register: {name} is {entry['status']} without an approval reference")
            elif issue and f"/{issue}#" not in reference:
                problems.append(f"register: {name} approval reference does not point at issue #{issue}: {reference}")
            if not entry.get("migration"):
                problems.append(f"register: {name} is {entry['status']} without a migration note")
            if not entry.get("decision"):
                problems.append(f"register: {name} is {entry['status']} without a decision reference")
        if (entry.get("landed") or {}).get("sha"):
            problems.append(f"register: {name} stores a commit SHA; the landing commit is resolved from the trailer")
    return problems


def verify_approvals(register: dict) -> list[str]:
    """Check each approved entry's comment exists and was written by the owner.

    Needs network and `gh`; CI runs it, a local run may skip it.
    """
    meta = register["register"]
    problems: list[str] = []
    for entry in register.get("break", []):
        if entry.get("status") not in ("approved", "landed"):
            continue
        reference = (entry.get("approval") or {}).get("ref", "")
        comment_id = reference.rsplit("-", 1)[-1] if "#issuecomment-" in reference else ""
        if not comment_id.isdigit():
            problems.append(f"register: {entry.get('id')} approval reference is not a comment URL: {reference}")
            continue
        done = subprocess.run(
            ["gh", "api", f"repos/{meta.get('repo', 'joaquinbejar/OptionStratLib')}/issues/comments/{comment_id}",
             "--jq", ".user.login + \"\\n\" + .body"],
            capture_output=True, text=True, check=False,
        )
        if done.returncode != 0:
            problems.append(f"register: {entry.get('id')} approval comment {comment_id} could not be read")
            continue
        author, _, body = done.stdout.partition("\n")
        if author.strip() != meta.get("owner"):
            problems.append(f"register: {entry.get('id')} approval comment was written by {author.strip()!r}, not the owner")
        elif entry.get("id") not in body:
            problems.append(f"register: {entry.get('id')} is not named in its approval comment")
    return problems


def self_test() -> int:
    """Prove the parser fails closed on every malformed report we have seen."""
    register = load_register()
    tool = register["register"]["tool"]
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
    both = parse_report(text, 100, tool, known_lints())
    same = {lint for lint, item in both if item == "struct probe::TwiceAlias"}
    ok = len(same) == 2
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: one item under two lints is two findings ({sorted(same)})")
    # An empty register expects nothing, so any finding is unforeseen.
    problems = verdicts(pairs, set(), set(), "C1", "default", {}, set())
    ok = len(problems) == len(pairs)
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: every finding is unforeseen against an empty register ({len(problems)})")
    # Register rules: an approved entry needs an approval reference, a
    # migration and a decision, and may not store a SHA.
    bad = {
        "register": {"issue": 592, "owner": "someone"},
        "break": [
            {"id": "AB-X", "status": "approved", "surfaces": ["all"],
             "findings": [{"check": "C1", "lint": "function_missing", "item": "function x"}],
             "landed": {"sha": "deadbeef"}},
        ],
    }
    problems = validate_register(bad)
    ok = any("approval reference" in p for p in problems) and any("commit SHA" in p for p in problems) \
        and any("migration" in p for p in problems) and any("decision" in p for p in problems)
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: an approved entry without approval, migration or decision is rejected ({len(problems)} problems)")
    good_register = {
        "register": {"issue": 592, "owner": "someone"},
        "break": [
            {"id": "AB-Y", "status": "approved", "surfaces": ["all"], "decision": "D4", "migration": "call the trait",
             "approval": {"ref": "https://github.com/o/r/issues/592#issuecomment-1"},
             "findings": [{"check": "C1", "lint": "function_missing", "item": "function x"}]},
        ],
    }
    ok = validate_register(good_register) == []
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: a well-formed approved entry passes the register rules")
    # A proposed entry can never make a check pass.
    proposed = {"register": {"issue": 592}, "break": [dict(good_register["break"][0], status="proposed")]}
    ready, pending, elsewhere, tolerated = expected(proposed, "C1", "all", False, scope=False)
    problems = verdicts({("function_missing", "function x")}, ready, pending, "C1", "all", elsewhere, tolerated)
    ok = ready == set() and any("unapproved" in p for p in problems)
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: a proposed entry never makes a check pass")
    # A finding declared for another check or surface is misdeclared, not unforeseen.
    ready, pending, elsewhere, tolerated = expected(good_register, "C3", "all", False, scope=False)
    problems = verdicts({("function_missing", "function x")}, ready, pending, "C3", "all", elsewhere, tolerated)
    ok = len(problems) == 1 and "misdeclared" in problems[0]
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: a finding declared for another check is misdeclared ({problems})")
    # `landed-only` refuses an approved-but-not-landed entry on `main`.
    ready, _, _, _ = expected(good_register, "C1", "all", True, scope=False)
    ok = ready == set()
    failures += 0 if ok else 1
    print(f"self-test {'ok' if ok else 'FAIL'}: --landed-only expects nothing from an approved-but-unlanded entry")
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


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--verify-approvals", action="store_true", help="read every approval comment and check its author is the register owner (needs gh and network)")
    parser.add_argument("--check", choices=CHECKS)
    parser.add_argument("--surface")
    parser.add_argument("--baseline", help="rev for C3; for C2 it overrides the resolved reference")
    parser.add_argument("--before", help="C2: the state before the change under test (base.sha on a PR, HEAD^ on a push); the reference is the newest Accepted-Breaks: commit among its ancestors")
    parser.add_argument("--base-sha", help="pull request base.sha, to pin the synthetic merge commit")
    parser.add_argument("--head-sha", help="pull request head.sha, to pin the synthetic merge commit")
    parser.add_argument("--landed-only", action="store_true", help="on `main`: only `landed` entries may be expected")
    parser.add_argument("--pr", type=int, help="the pull request under test; an entry landing here must name it in `landed.pr`")
    parser.add_argument("--against", help="the base commit whose register this one is compared with, to refuse a pull request that approves its own entries")
    parser.add_argument("--register", type=Path, default=REGISTER, help="register path (the sequence fixture points this at its own probe)")
    parser.add_argument("--root", type=Path, default=ROOT, help="crate root to run the comparisons in")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    register = load_register(args.register)
    if args.verify_approvals:
        problems = validate_register(register) + verify_approvals(register)
        for problem in problems:
            print(f"accepted-breaks: {problem}")
        if problems:
            return 1
        print("accepted-breaks OK: every approved entry carries an owner approval")
        return 0
    surfaces = {surface["id"]: surface for surface in register["register"]["surfaces"]}
    if args.surface not in surfaces or args.check is None:
        parser.error("--check and --surface are required unless --self-test is given")
    try:
        if args.base_sha and args.head_sha:
            assert_synthetic(args.base_sha, args.head_sha, cwd=args.root)
        # C1 compares with the published crate; a probe that was never
        # published (the sequence fixture) falls back to its reference tag,
        # which plays the same role there.
        published = register["register"].get("published") if args.check == "C1" else None
        if args.check == "C1" and not published:
            args.baseline = args.baseline or git("rev-parse", register["register"]["initial_reference"], cwd=args.root)
        baseline = args.baseline or ""
        if args.check == "C2" and not baseline:
            if not args.before:
                parser.error("--check C2 needs --before (or an explicit --baseline)")
            baseline = reference_commit(args.before, register, cwd=args.root)
            print(f"accepted-breaks: C2 reference {baseline[:12]} (before {args.before[:12]})")
        text, code = run_semver(baseline, surfaces[args.surface], published, cwd=args.root)
        found = parse_report(text, code, register["register"]["tool"], known_lints())
    except ReportError as error:
        print(f"accepted-breaks: {error}", file=sys.stderr)
        return 2
    problems = validate_register(register)
    if args.against:
        problems += self_approval(register, args.against, args.register, cwd=args.root)
    ready, pending, elsewhere, tolerated = expected(
        register, args.check, args.surface, args.landed_only,
        baseline=baseline, pr=args.pr, cwd=args.root,
    )
    problems += verdicts(found, ready, pending, args.check, args.surface, elsewhere, tolerated)
    for problem in problems:
        print(f"accepted-breaks: {problem}")
    if problems:
        return 1
    print(f"accepted-breaks OK: {args.surface}/{args.check} matches the register ({len(found)} authorised findings)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
