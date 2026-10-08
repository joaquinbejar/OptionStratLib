#!/usr/bin/env python3
"""Run the 0.22 release gates and record the evidence (#557, ADR-0004 section 6).

Every gate is one shell command run from the repository root. For each one
this records the exact command, its exit status, its wall time, the number of
`warning:` lines it printed and the tail of its output, and writes the full
log to `target/release-gates/<id>.log`. `render` turns the recorded results
into the evidence file `docs/release/0.22/gates.md`, together with the
toolchain versions and the commit the gates ran on.

A gate passes only with exit status 0 and no compiler or tool warning (the
`ignoring test ... not included in the published package` notices of
`cargo package` are counted apart, because they describe package contents,
which #559 owns). The semver reports are informational: they list what 0.22
removes or reshapes against the published 0.21.3 and decide nothing (#606).

Usage:
    release_gates.py run [--only ID ...] [--skip ID ...]
    release_gates.py render
    release_gates.py list
    release_gates.py --self-test
"""

from __future__ import annotations

import argparse
import json
import re
import shutil
import subprocess
import sys
import time
from dataclasses import dataclass
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
OUT = ROOT / "target" / "release-gates"
RESULTS = OUT / "results.json"
EVIDENCE = ROOT / "docs" / "release" / "0.22" / "gates.md"
BASELINE = "v0.21.3"
# The report parser needs Python 3.11 or newer, as the Makefile's PYTHON311.
REPORT_PYTHON = "$(command -v python3.13 || command -v python3.12 || command -v python3.11 || echo python3)"
SURFACES = ("none", "default", "plotly", "static_export", "async", "all")

PACKAGE_NOTICE = re.compile(r"^warning: ignoring test `[^`]+` as `[^`]+` is not included in the published package")
WARNING = re.compile(r"^(\x1b\[[0-9;]*m)*warning(\[[^\]]*\])?(\x1b\[[0-9;]*m)*:", re.M)


@dataclass(frozen=True)
class Gate:
    id: str
    group: str
    command: str
    purpose: str
    informational: bool = False


GATES: tuple[Gate, ...] = (
    Gate("fmt", "Workspace", "make fmt-check", "rustfmt on the workspace and every consumer fixture"),
    Gate("lint", "Workspace",
         "make lint",
         "Clippy `-D warnings` with all features and with none; test-module and browser-test scanners; "
         "the facade capability matrix (`FACADE_FEATURE_SETS`: clippy, rustdoc `-D warnings` and tests of each "
         "capability alone and the documented pairs)"),
    Gate("clippy-all", "Workspace",
         "cargo clippy --workspace --all-targets --all-features -- -D warnings",
         "the verification command of the issue, run on its own"),
    Gate("test", "Workspace",
         "make test",
         "each component crate with default and all features, the cross-component suite, and the facade with "
         "default, `plotly` and `static_export,plotly`; unit, integration and doctests"),
    Gate("test-all", "Workspace", "cargo test --all-features --workspace", "every workspace test, doctests included"),
    Gate("rustdoc-all", "Workspace",
         'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps',
         "rustdoc with every feature, warnings and broken links denied"),
    Gate("rustdoc-default", "Workspace",
         'RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps',
         "rustdoc with default features only: the default-only documentation"),
    Gate("doc", "Workspace", "make doc", "the repository's documentation target"),
    Gate("build-release", "Workspace", "cargo build --workspace --release", "release build, zero warnings"),
    Gate("scan-banned", "Workspace", "make scan-banned", "no panicking or printing construct in library code"),
    Gate("float-boundary", "Workspace", "make check-float-boundary", "`f64` stays inside the numeric kernels"),
    Gate("public-api", "API", "make public-api-check",
         "the checked-in public API snapshots of the facade and the nine components"),
    Gate("api-report-selftest", "API", "make check-api-report",
         "the semver report parser's self-test, including that its default surface is the manifest default"),
    Gate("classify-selftest", "API", "python3 scripts/classify_api_changes.py --self-test",
         "the API classifier's self-test"),
    Gate("graph", "Architecture", "make check-graph",
         "layer DAG, error/utils partition, synthetic and Plotly gates, facade feature table, forbidden packages"),
    Gate("feature-trees", "Architecture", "make check-feature-trees",
         "resolved dependency graph of 15 facade surfaces and what each may resolve"),
    Gate("fixtures", "Consumers", "make check-fixtures", "resolved graph of every consumer fixture and direct example"),
    Gate("consumers-022", "Consumers", "make test-022-consumers",
         "every 0.22 consumer fixture, against the workspace and against the package archives"),
    Gate("direct-examples", "Consumers", "make test-direct-component-examples",
         "the direct-component examples: Clippy, tests, run"),
    Gate("direct-examples-packaged", "Consumers", "make check-direct-examples-packaged",
         "the direct-component examples built outside the repository against the packaged crates"),
    Gate("components", "Per crate", "make check-components",
         "each component alone with default, no and all features, named feature sets, Clippy, rustdoc "
         "`-D warnings` with and without features, package contents, and the packaged archives"),
    Gate("visualization", "Surfaces", "make check-visualization",
         "backend-neutral, `plotly` and `static_export` compiled, linted, tested and documented independently, "
         "for the crate and the facade"),
    *(
        Gate(f"semver-{surface}", "Informational",
             f"{REPORT_PYTHON} scripts/report_api_changes.py --surface {surface} --baseline {BASELINE}",
             f"cargo-semver-checks against the published 0.21.3, surface `{surface}`",
             informational=True)
        for surface in SURFACES
    ),
)


def tool_versions() -> list[tuple[str, str]]:
    def run(*command: str) -> str:
        try:
            out = subprocess.run(command, capture_output=True, text=True, cwd=ROOT, check=False)
        except OSError as error:
            return f"unavailable ({error})"
        return (out.stdout or out.stderr).strip().splitlines()[0] if (out.stdout or out.stderr).strip() else "unknown"

    pins = {}
    try:
        text = subprocess.run(["make", "--no-print-directory", "print-public-api-pins"], capture_output=True,
                              text=True, cwd=ROOT, check=False).stdout
        for line in text.splitlines():
            if "=" in line:
                key, value = line.split("=", 1)
                pins[key.strip()] = value.strip()
    except OSError:
        pass
    nightly = pins.get("public_api_nightly", "")
    return [
        ("rustc", run("rustc", "--version")),
        ("cargo", run("cargo", "--version")),
        ("rustfmt", run("cargo", "fmt", "--version")),
        ("clippy", run("cargo", "clippy", "--version")),
        ("nightly for public-api", run("rustc", f"+{nightly}", "--version") if nightly else "unknown"),
        ("cargo-public-api", run("cargo", "public-api", "--version")),
        ("cargo-semver-checks", run("cargo", "semver-checks", "--version")),
        ("python3", run("python3", "--version")),
        ("python3 for the report", run(shutil.which("python3.13") or shutil.which("python3.12") or "python3", "--version")),
        ("host", run("uname", "-srm")),
    ]


def count_warnings(text: str) -> tuple[int, int]:
    """`(warnings, package notices)` in a log: package notices are not compiler warnings."""
    notices = sum(1 for line in text.splitlines() if PACKAGE_NOTICE.match(line))
    total = len(WARNING.findall(text))
    return total - notices, notices


def run_gate(gate: Gate) -> dict:
    OUT.mkdir(parents=True, exist_ok=True)
    log = OUT / f"{gate.id}.log"
    started = time.time()
    with log.open("w") as handle:
        completed = subprocess.run(
            gate.command, shell=True, cwd=ROOT, stdout=handle, stderr=subprocess.STDOUT, check=False
        )
    seconds = round(time.time() - started)
    text = log.read_text(errors="replace")
    warnings, notices = count_warnings(text)
    tail = [line for line in text.splitlines() if line.strip()][-3:]
    return {
        "id": gate.id,
        "command": gate.command,
        "returncode": completed.returncode,
        "seconds": seconds,
        "warnings": warnings,
        "package_notices": notices,
        "tail": tail,
    }


def verdict(result: dict, gate: Gate) -> str:
    if gate.informational:
        return "reported" if result["returncode"] in (0, 100) else "tool failure"
    if result["returncode"] != 0:
        return "FAIL"
    return "pass" if result["warnings"] == 0 else "FAIL (warnings)"


def load_results() -> dict[str, dict]:
    return json.loads(RESULTS.read_text()) if RESULTS.is_file() else {}


def git(*args: str) -> str:
    return subprocess.run(["git", *args], capture_output=True, text=True, cwd=ROOT, check=False).stdout.strip()


def render() -> str:
    results = load_results()
    meta = results.get("_meta", {})
    lines = [
        "# 0.22 release gates: evidence",
        "",
        "Generated by `scripts/release_gates.py render` from a run of `scripts/release_gates.py run` (#557).",
        "Nothing here is hand-edited except the classification sections below the tables.",
        "",
        "## Run",
        "",
        f"- Commit gated: `{meta.get('commit', 'unknown')}` ({meta.get('subject', '')})",
        f"- Working tree at the start: {meta.get('dirty', 'unknown')}",
        f"- Started: {meta.get('started', 'unknown')}",
        "",
        "## Toolchains",
        "",
        "| Tool | Version |",
        "| --- | --- |",
        *(f"| {name} | `{version}` |" for name, version in meta.get("tools", [])),
        "",
    ]
    groups: dict[str, list[Gate]] = {}
    for gate in GATES:
        groups.setdefault(gate.group, []).append(gate)
    for group, gates in groups.items():
        lines += [f"## {group}", "", "| Gate | Command | Result | Time | Warnings | What it covers |", "| --- | --- | --- | --- | --- | --- |"]
        for gate in gates:
            result = results.get(gate.id)
            if result is None:
                lines.append(f"| `{gate.id}` | `{gate.command}` | not run | | | {gate.purpose} |")
                continue
            warnings = str(result["warnings"])
            if result["package_notices"]:
                warnings += f" (+{result['package_notices']} package notices)"
            lines.append(
                f"| `{gate.id}` | `{gate.command}` | {verdict(result, gate)} | {result['seconds']} s | {warnings} | {gate.purpose} |"
            )
        lines.append("")
    return "\n".join(lines).rstrip("\n") + "\n"


def self_test() -> int:
    failures = 0

    def check(name: str, got: object, expected: object) -> None:
        nonlocal failures
        ok = got == expected
        failures += 0 if ok else 1
        print(f"self-test {'ok' if ok else 'FAIL'}: {name} (expected {expected!r}, got {got!r})")

    notice = "warning: ignoring test `a` as `tests/a.rs` is not included in the published package"
    check("package notices are counted apart", count_warnings(f"{notice}\n{notice}\n"), (0, 2))
    check("a compiler warning counts", count_warnings(f"warning: unused variable: `x`\n{notice}\n"), (1, 1))
    check("a lint warning counts", count_warnings("warning[E0xxx]: something\n"), (1, 0))
    check("command echoes are not warnings", count_warnings('RUSTDOCFLAGS="-D warnings" cargo doc\n'), (0, 0))
    check("a coloured warning counts", count_warnings("\x1b[33mwarning\x1b[0m: x\n"), (1, 0))
    gate = Gate("g", "G", "true", "p")
    info = Gate("i", "I", "true", "p", informational=True)
    result = {"returncode": 0, "warnings": 0}
    check("a clean exit passes", verdict(result, gate), "pass")
    check("a warning fails a gate", verdict({"returncode": 0, "warnings": 2}, gate), "FAIL (warnings)")
    check("a failing exit fails a gate", verdict({"returncode": 1, "warnings": 0}, gate), "FAIL")
    check("status 100 is a report", verdict({"returncode": 100, "warnings": 0}, info), "reported")
    check("another status is a tool failure", verdict({"returncode": 2, "warnings": 0}, info), "tool failure")
    check("gate ids are unique", len({g.id for g in GATES}), len(GATES))
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("action", nargs="?", choices=("run", "render", "list"))
    parser.add_argument("--only", nargs="*", default=[])
    parser.add_argument("--skip", nargs="*", default=[])
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.action == "list":
        for gate in GATES:
            print(f"{gate.id:26} {gate.command}")
        return 0
    if args.action == "render":
        EVIDENCE.parent.mkdir(parents=True, exist_ok=True)
        EVIDENCE.write_text(render())
        print(f"wrote {EVIDENCE.relative_to(ROOT)}")
        return 0
    if args.action != "run":
        parser.print_help()
        return 2
    results = load_results()
    results["_meta"] = results.get("_meta") or {
        "commit": git("rev-parse", "HEAD"),
        "subject": git("log", "-1", "--format=%s"),
        "dirty": "clean" if not git("status", "--porcelain") else "had uncommitted changes",
        "started": time.strftime("%Y-%m-%d %H:%M:%S %z"),
        "tools": tool_versions(),
    }
    selected = [g for g in GATES if (not args.only or g.id in args.only) and g.id not in args.skip]
    for gate in selected:
        print(f"=== {gate.id}: {gate.command}", flush=True)
        results[gate.id] = run_gate(gate)
        print(f"    {verdict(results[gate.id], gate)} in {results[gate.id]['seconds']} s, "
              f"{results[gate.id]['warnings']} warning(s)", flush=True)
        OUT.mkdir(parents=True, exist_ok=True)
        RESULTS.write_text(json.dumps(results, indent=1))
    failed = [g.id for g in selected if not g.informational and verdict(results[g.id], g) != "pass"]
    broken = [g.id for g in selected if g.informational and verdict(results[g.id], g) != "reported"
              and results[g.id]["returncode"] != 0]
    if failed or broken:
        print(f"FAILED: {', '.join(failed + broken)}")
        return 1
    print("all selected gates passed")
    return 0


if __name__ == "__main__":
    sys.exit(main())
