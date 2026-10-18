#!/usr/bin/env python3
"""Time the native and the Plotly exports of #920 and check for leaked processes.

Two `compare` binaries are measured: the native one (built without
`plotly-baseline`, so `plotly_static` is not even linked) for Plotters, and
the baseline one (built with `plotly-baseline`) for Plotly.

* Cold: `compare export <backend> <case> <format> <file>` run `--cold` times
  per combination, each a fresh process; the wall time includes process
  start, renderer start-up, the export and shutdown.
* Warm: `compare repeat ... <iterations>` once per combination; the binary
  records each export's wall time inside one process. Plotly builds a new
  exporter (chromedriver + Chrome) per image either way, so its warm runs
  still pay the browser start-up.
* Leaks: before and after every Plotly run the chromedriver and headless
  Chrome processes are listed; a process that appeared during the run and
  is still alive a second after it exited is reported as leaked, then
  terminated by its exact PID. Processes that existed before the run are
  never touched.
* Sandbox (macOS, `--sandbox`): the native cold runs execute under
  `sandbox-exec` with network access and process creation denied, so a
  successful run proves the export opened no socket and spawned nothing.

* Leak scenarios (`--mode leaks`): the cases where an export does not get
  to close its WebDriver session. `concurrent`: several Plotly exports
  started at once, as separate processes (parallel test binaries or
  worktrees), sharing the default chromedriver port. `interrupted`: one
  export whose process receives SIGINT, SIGTERM or SIGKILL while the browser
  is up. What is still running two seconds later is recorded, then
  terminated by PID.

Usage:
    measure.py --native BIN --baseline BIN --out DIR [--cold N] [--warm N]
               [--plotly-warm N] [--sandbox] [--backends plotters,plotly]
               [--name results]
    measure.py --mode leaks --baseline BIN --out DIR [--repeat N]
"""

from __future__ import annotations

import argparse
import json
import os
import platform
import signal
import statistics
import subprocess
import sys
import tempfile
import time
from pathlib import Path

CASES = ("payoff", "curves", "surface")
NATIVE_FORMATS = ("png", "svg", "svg-plain")
PLOTLY_FORMATS = ("png", "svg")
SANDBOX_PROFILE = "(version 1)(allow default)(deny network*)(deny process-fork)"


def browser_processes() -> dict[int, str]:
    """The WebDriver processes an export can start, by PID.

    Only the chromedriver at `WEBDRIVER_PATH` and the Chrome main processes it
    launches (chromedriver adds `--test-type=webdriver`) count, so a browser
    another program runs at the same time (Playwright, the user's own Chrome)
    is never reported or touched. Killing a Chrome main process ends its
    helpers.
    """
    driver = os.environ.get("WEBDRIVER_PATH", "chromedriver")
    out = subprocess.run(
        ["ps", "-axo", "pid=,command="], capture_output=True, text=True, check=True
    ).stdout
    found = {}
    for line in out.splitlines():
        pid, _, command = line.strip().partition(" ")
        if command.startswith(driver) or "--test-type=webdriver" in command:
            found[int(pid)] = command[:160]
    return found


def stats(values: list[float]) -> dict:
    ordered = sorted(values)
    p90 = ordered[min(len(ordered) - 1, int(round(0.9 * (len(ordered) - 1))))]
    return {
        "n": len(values),
        "min": ordered[0],
        "median": statistics.median(ordered),
        "p90": p90,
        "max": ordered[-1],
        "mean": statistics.fmean(ordered),
    }


def cold(binary: str, backend: str, case: str, fmt: str, runs: int, workdir: Path,
         sandbox: bool) -> dict:
    timings, failures, leaked, sizes = [], [], [], []
    for run in range(runs):
        # plotly_static writes the extension of the format it exports, so the
        # target carries it.
        extension = {"png": "png", "svg": "svg", "svg-plain": "plain.svg"}[fmt]
        target = workdir / f"cold-{backend}-{case}-{fmt}-{run}.{extension}"
        command = [binary, "export", backend, case, fmt, str(target)]
        if sandbox:
            command = ["sandbox-exec", "-p", SANDBOX_PROFILE, *command]
        before = browser_processes() if backend == "plotly" else {}
        started = time.perf_counter()
        result = subprocess.run(command, capture_output=True, text=True,
                                env={**os.environ, "RUST_LOG": "warn"})
        elapsed = (time.perf_counter() - started) * 1000.0
        if result.returncode != 0:
            failures.append({"run": run, "status": result.returncode,
                             "stderr": result.stderr[-400:]})
            continue
        timings.append(elapsed)
        sizes.append(target.stat().st_size)
        if backend == "plotly":
            time.sleep(1.0)
            after = browser_processes()
            for pid, command_line in after.items():
                if pid not in before:
                    leaked.append({"run": run, "pid": pid, "command": command_line})
                    try:
                        os.kill(pid, signal.SIGTERM)
                    except ProcessLookupError:
                        pass
    return {"backend": backend, "case": case, "format": fmt, "sandboxed": sandbox,
            "millis": timings, "stats": stats(timings) if timings else None,
            "bytes": sizes[0] if sizes else None, "failures": failures, "leaked": leaked}


def warm(binary: str, backend: str, case: str, fmt: str, iterations: int,
         workdir: Path) -> dict:
    before = browser_processes() if backend == "plotly" else {}
    result = subprocess.run(
        [binary, "repeat", backend, case, fmt, str(iterations), str(workdir)],
        capture_output=True, text=True, env={**os.environ, "RUST_LOG": "warn"})
    leaked = []
    if backend == "plotly":
        time.sleep(1.0)
        leaked = [{"pid": pid, "command": c} for pid, c in browser_processes().items()
                  if pid not in before]
        for item in leaked:
            try:
                os.kill(item["pid"], signal.SIGTERM)
            except ProcessLookupError:
                pass
    if result.returncode != 0:
        return {"backend": backend, "case": case, "format": fmt, "failed": True,
                "stderr": result.stderr[-400:], "leaked": leaked}
    report = json.loads((workdir / f"repeat-{backend}-{case}-{fmt}.json").read_text())
    millis = report["millis"]
    return {"backend": backend, "case": case, "format": fmt, "millis": millis,
            "first": millis[0], "stats": stats(millis[1:] if len(millis) > 1 else millis),
            "leaked": leaked}


def version(command: list[str]) -> str:
    try:
        return subprocess.run(command, capture_output=True, text=True).stdout.strip()
    except OSError as error:
        return f"unavailable: {error}"


def leftovers(before: dict[int, str]) -> list[dict]:
    """WebDriver processes started since `before`, terminated after recording."""
    time.sleep(2.0)
    found = [{"pid": pid, "command": command} for pid, command in browser_processes().items()
             if pid not in before]
    for item in found:
        try:
            os.kill(item["pid"], signal.SIGTERM)
        except ProcessLookupError:
            pass
    time.sleep(1.0)
    return found


def concurrent(binary: str, processes: int, workdir: Path, run: int) -> dict:
    before = browser_processes()
    children = [
        subprocess.Popen([binary, "export", "plotly", "curves", "png",
                          str(workdir / f"concurrent-{run}-{index}.png")],
                         stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True,
                         env={**os.environ, "RUST_LOG": "warn"})
        for index in range(processes)
    ]
    statuses = []
    for child in children:
        _, stderr = child.communicate(timeout=180)
        statuses.append({"status": child.returncode, "stderr": stderr[-300:]})
    return {"scenario": "concurrent", "processes": processes, "run": run,
            "statuses": statuses, "leaked": leftovers(before)}


def interrupted(binary: str, sig: signal.Signals, delay: float, workdir: Path,
                run: int) -> dict:
    before = browser_processes()
    child = subprocess.Popen([binary, "export", "plotly", "curves", "png",
                              str(workdir / f"interrupted-{run}.png")],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                             env={**os.environ, "RUST_LOG": "warn"})
    time.sleep(delay)
    started = {pid for pid in browser_processes() if pid not in before}
    child.send_signal(sig)
    child.wait(timeout=60)
    return {"scenario": "interrupted", "signal": sig.name, "delay_s": delay, "run": run,
            "status": child.returncode, "browser_up_at_signal": len(started),
            "leaked": leftovers(before)}


def timing(args: argparse.Namespace, out: Path, environment: dict) -> int:
    backends = set(args.backends.split(","))
    results = {"environment": environment, "cold": [], "warm": []}
    with tempfile.TemporaryDirectory(prefix="osl-920-") as scratch:
        workdir = Path(scratch)
        for case in CASES:
            if "plotters" in backends:
                for fmt in NATIVE_FORMATS:
                    print(f"cold plotters {case} {fmt}", file=sys.stderr)
                    results["cold"].append(cold(args.native, "plotters", case, fmt, args.cold,
                                                workdir, args.sandbox))
            if "plotly" in backends:
                for fmt in PLOTLY_FORMATS:
                    print(f"cold plotly {case} {fmt}", file=sys.stderr)
                    results["cold"].append(cold(args.baseline, "plotly", case, fmt, args.cold,
                                                workdir, False))
        for case in CASES:
            if "plotters" in backends:
                for fmt in NATIVE_FORMATS:
                    print(f"warm plotters {case} {fmt}", file=sys.stderr)
                    results["warm"].append(warm(args.native, "plotters", case, fmt, args.warm,
                                                workdir))
            if "plotly" in backends:
                for fmt in PLOTLY_FORMATS:
                    print(f"warm plotly {case} {fmt}", file=sys.stderr)
                    results["warm"].append(warm(args.baseline, "plotly", case, fmt,
                                                args.plotly_warm, workdir))
    target = out / f"{args.name}.json"
    target.write_text(json.dumps(results, indent=2))
    leaks = sum(len(r["leaked"]) for r in results["cold"] + results["warm"])
    failures = sum(len(r["failures"]) for r in results["cold"])
    print(f"wrote {target}; failures={failures} leaked={leaks}", file=sys.stderr)
    return 0


def leaks(args: argparse.Namespace, out: Path, environment: dict) -> int:
    results = {"environment": environment, "scenarios": []}
    with tempfile.TemporaryDirectory(prefix="osl-920-leaks-") as scratch:
        workdir = Path(scratch)
        for run in range(args.repeat):
            for processes in (2, 4):
                print(f"concurrent {processes} run {run}", file=sys.stderr)
                results["scenarios"].append(concurrent(args.baseline, processes, workdir, run))
            for sig in (signal.SIGINT, signal.SIGTERM, signal.SIGKILL):
                print(f"interrupted {sig.name} run {run}", file=sys.stderr)
                results["scenarios"].append(interrupted(args.baseline, sig, 0.8, workdir, run))
    target = out / "leaks.json"
    target.write_text(json.dumps(results, indent=2))
    total = sum(len(item["leaked"]) for item in results["scenarios"])
    print(f"wrote {target}; leaked processes={total}", file=sys.stderr)
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__,
                                     formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--mode", choices=("timing", "leaks"), default="timing")
    parser.add_argument("--native")
    parser.add_argument("--baseline", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--name", default="results")
    parser.add_argument("--backends", default="plotters,plotly")
    parser.add_argument("--cold", type=int, default=10)
    parser.add_argument("--warm", type=int, default=200)
    parser.add_argument("--plotly-warm", type=int, default=10)
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument("--sandbox", action="store_true")
    args = parser.parse_args()
    if args.mode == "timing" and "plotters" in args.backends and not args.native:
        parser.error("--native is required to time plotters")

    out = Path(args.out)
    out.mkdir(parents=True, exist_ok=True)
    environment = {
        "platform": platform.platform(),
        "machine": platform.machine(),
        "rustc": version(["rustc", "--version"]),
        "chrome": version([os.environ.get("BROWSER_PATH", "google-chrome"), "--version"]),
        "chromedriver": version([os.environ.get("WEBDRIVER_PATH", "chromedriver"), "--version"]),
        "git": version(["git", "rev-parse", "HEAD"]),
        "mode": args.mode,
        "sandboxed_native_cold_runs": args.sandbox,
        "cold_runs": args.cold,
        "warm_iterations": args.warm,
        "plotly_warm_iterations": args.plotly_warm,
    }
    if args.mode == "leaks":
        return leaks(args, out, environment)
    return timing(args, out, environment)


if __name__ == "__main__":
    sys.exit(main())
