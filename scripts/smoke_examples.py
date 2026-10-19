#!/usr/bin/env python3
"""Run every example binary and every Criterion bench once (#787).

Examples and benches that nothing runs rot: an import changes, a feature moves,
a program starts to return `Err`, and the next person to try it finds out. This
runs them all, so CI and the release gates can.

* Examples: every `bin` target of the workspace packages `examples_*` and
  `osl-example-direct-*`, built beforehand by `cargo build --workspace --bins`
  (`make smoke-examples` does it), run from the repository root with a
  timeout. A run passes with exit status 0 and no panic message; anything
  else fails it. The examples' PNG and SVG exports are drawn natively since
  #927, with no browser or WebDriver, so an export failure is a failure like
  any other: the `needs-webdriver` result and the `--export` switch that
  turned it into a failure are gone (#929).
* Benches: every `bench` target of the workspace packages, run once through
  `cargo bench -p <package> --bench <target> -- --test` (Criterion's test mode:
  each benchmark runs one iteration and must succeed).

Usage:
    smoke_examples.py [--timeout SECONDS] [--only PACKAGE ...]
    smoke_examples.py --benches [--timeout SECONDS]
    smoke_examples.py --list
    smoke_examples.py --markdown RESULTS.json [RESULTS.json ...]
    smoke_examples.py --self-test
"""

from __future__ import annotations

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
EXAMPLE_PREFIXES = ("examples_", "osl-example-direct-")
PANIC = re.compile(r"panicked at|thread '.*' panicked|stack overflow", re.I)


def metadata() -> dict:
    out = subprocess.run(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    )
    return json.loads(out.stdout)


def workspace_targets(data: dict, kind: str, prefixes: tuple[str, ...] | None) -> list[tuple[str, str]]:
    """`(package, target)` of every workspace target of `kind`, sorted."""
    return [(package, target) for package, target, _ in workspace_targets_with_features(data, kind, prefixes)]


def workspace_targets_with_features(
    data: dict, kind: str, prefixes: tuple[str, ...] | None
) -> list[tuple[str, str, list[str]]]:
    """`(package, target, required-features)` of every workspace target of `kind`, sorted."""
    members = set(data["workspace_members"])
    found = []
    for package in data["packages"]:
        if package["id"] not in members:
            continue
        if prefixes is not None and not package["name"].startswith(prefixes):
            continue
        found += [
            (package["name"], t["name"], list(t.get("required-features") or []))
            for t in package["targets"]
            if kind in t["kind"]
        ]
    return sorted(found)


def classify(returncode: int | str, log: str) -> str:
    """`pass`, `timeout` or `FAIL` for one finished run."""
    if returncode == "timeout":
        return "timeout"
    if PANIC.search(log):
        return "FAIL"
    if returncode == 0:
        return "pass"
    return "FAIL"


def run(command: list[str], timeout: int, log_path: Path) -> tuple[int | str, float]:
    started = time.time()
    with log_path.open("w") as log:
        process = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, start_new_session=True)
        try:
            code: int | str = process.wait(timeout=timeout)
        except subprocess.TimeoutExpired:
            # Stop the process group this run started, never anything else.
            os.killpg(process.pid, signal.SIGTERM)
            process.wait()
            code = "timeout"
    return code, round(time.time() - started, 1)


def smoke(items: list[tuple[str, str, list[str]]], timeout: int, out: Path) -> int:
    out.mkdir(parents=True, exist_ok=True)
    rows = []
    for package, target, command in items:
        code, seconds = run(command, timeout, out / f"{package}__{target}.log")
        log = (out / f"{package}__{target}.log").read_text(errors="replace")
        verdict = classify(code, log)
        tail = [line for line in log.splitlines() if line.strip()][-1:]
        rows.append({"package": package, "target": target, "result": verdict, "code": code, "seconds": seconds,
                     "tail": tail[0][-160:] if tail else ""})
        print(f"{verdict:16} {package} {target} ({seconds} s)", flush=True)
    (out / "results.json").write_text(json.dumps(rows, indent=1))
    bad = [r for r in rows if r["result"] != "pass"]
    print(f"{len(rows)} run: {len(rows) - len(bad)} pass, {len(bad)} fail")
    for row in bad:
        print(f"FAIL {row['package']} {row['target']}: exit {row['code']}: {row['tail']}")
    return 1 if bad else 0


def markdown(results: list[list[dict]], titles: list[str]) -> str:
    """One table per package: every target and its result, for the release evidence."""
    lines: list[str] = []
    for title, rows in zip(titles, results):
        lines += [f"## {title}", ""]
        packages: dict[str, list[dict]] = {}
        for row in rows:
            packages.setdefault(row["package"], []).append(row)
        for package, items in sorted(packages.items()):
            counts: dict[str, int] = {}
            for item in items:
                counts[item["result"]] = counts.get(item["result"], 0) + 1
            summary = ", ".join(f"{n} {name}" for name, n in sorted(counts.items()))
            lines += [f"### `{package}` ({summary})", "", "| Target | Result | Time |", "| --- | --- | ---: |"]
            lines += [f"| `{item['target']}` | {item['result']} | {item['seconds']} s |" for item in items]
            lines.append("")
    return "\n".join(lines).rstrip("\n") + "\n"


def self_test() -> int:
    failures = 0

    def check(name: str, got: object, expected: object) -> None:
        nonlocal failures
        ok = got == expected
        failures += 0 if ok else 1
        print(f"self-test {'ok' if ok else 'FAIL'}: {name} (expected {expected!r}, got {got!r})")

    check("a clean exit passes", classify(0, "INFO done"), "pass")
    check("a panic fails even with status 0", classify(0, "thread 'main' panicked at src/x.rs:1:1"), "FAIL")
    check("an error exit fails", classify(1, "Error: ConstructionError(..)"), "FAIL")
    check("a timeout is reported", classify("timeout", ""), "timeout")
    # Since #929 no export failure waits for a driver: the old browser
    # exporter's message and the native exporter's typed errors both fail.
    browser_log = "Error: Render(\"Failed to write PNG after 3 attempts: Failed to create StaticExporter\")"
    check("the old exporter's failure fails", classify(1, browser_log), "FAIL")
    native_log = 'Error: Graph(IncompleteSurface { name: "Volatility Surface", expected: 1980, found: 45 })'
    check("a native export error fails", classify(1, native_log), "FAIL")
    check("a run that times out is a timeout whatever it logged", classify("timeout", native_log), "timeout")
    data = {
        "workspace_members": ["a", "b", "c"],
        "packages": [
            {"id": "a", "name": "examples_x", "targets": [{"name": "one", "kind": ["bin"]}, {"name": "lib", "kind": ["lib"]}]},
            {"id": "b", "name": "osl-example-direct-y", "targets": [{"name": "osl-example-direct-y", "kind": ["bin"]}]},
            {"id": "c", "name": "optionstratlib", "targets": [{"name": "benches", "kind": ["bench"]}]},
            {"id": "d", "name": "examples_outside", "targets": [{"name": "z", "kind": ["bin"]}]},
        ],
    }
    check("example binaries are found by prefix and kind",
          workspace_targets(data, "bin", EXAMPLE_PREFIXES), [("examples_x", "one"), ("osl-example-direct-y", "osl-example-direct-y")])
    rows = [{"package": "p", "target": "a", "result": "pass", "seconds": 1.5},
            {"package": "p", "target": "b", "result": "FAIL", "seconds": 0.2}]
    check("the evidence table groups targets by package",
          "### `p` (1 FAIL, 1 pass)" in markdown([rows], ["Examples"]) and "| `b` | FAIL | 0.2 s |" in markdown([rows], ["Examples"]), True)
    gated = {"workspace_members": ["a"], "packages": [{"id": "a", "name": "m", "targets": [
        {"name": "io", "kind": ["bench"], "required-features": ["io", "x"]}, {"name": "plain", "kind": ["bench"]}]}]}
    check("a bench keeps its required features",
          workspace_targets_with_features(gated, "bench", None), [("m", "io", ["io", "x"]), ("m", "plain", [])])
    check("bench targets are found in every workspace package", workspace_targets(data, "bench", None), [("optionstratlib", "benches")])
    return 1 if failures else 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--benches", action="store_true")
    parser.add_argument("--timeout", type=int, default=300)
    parser.add_argument("--only", nargs="*", default=[])
    parser.add_argument("--list", action="store_true")
    parser.add_argument("--markdown", nargs="+", metavar="RESULTS.json", help="print the evidence tables of recorded runs")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if args.markdown:
        runs = [json.loads(Path(name).read_text()) for name in args.markdown]
        print(markdown(runs, [Path(name).parent.name.capitalize() for name in args.markdown]), end="")
        return 0
    data = metadata()
    target_dir = Path(data["target_directory"])
    out = target_dir / "smoke"
    if args.benches:
        # A bench with `required-features` (`chains_io` needs `io`) is run with
        # them, or cargo refuses it.
        items = [(p, t, ["cargo", "bench", "-p", p, "--bench", t, *(["--features", ",".join(f)] if f else []), "--", "--test"])
                 for p, t, f in workspace_targets_with_features(data, "bench", None)]
    else:
        items = [(p, t, [str(target_dir / "debug" / t)])
                 for p, t in workspace_targets(data, "bin", EXAMPLE_PREFIXES) if not args.only or p in args.only]
    if args.list:
        for package, target, _ in items:
            print(package, target)
        return 0
    if not args.benches:
        missing = [t for _, t, command in items if not Path(command[0]).is_file()]
        if missing:
            print(f"not built: {', '.join(missing)}; run `cargo build --workspace --bins` first", file=sys.stderr)
            return 1
    return smoke(items, args.timeout, out / ("benches" if args.benches else "examples"))


if __name__ == "__main__":
    sys.exit(main())
