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
* Leaks: every Plotly run is watched while it runs. The process tree under
  the `compare` process (chromedriver, Chrome and its helpers) is sampled
  every `TRACK_INTERVAL_S`; whatever of that tree is still alive a second
  after `compare` exited, with the same command line, is reported as leaked
  and terminated by its exact PID. Attribution is by ancestry recorded
  while the export ran, not by command-line pattern, so a browser any other
  program runs at the same time (another worktree's tests, Selenium,
  Playwright, the user's Chrome) is never reported or touched. A run that
  fails is checked the same way.
* Sandbox (macOS, `--sandbox`): the native cold runs execute under
  `sandbox-exec` with network access and process creation denied, so a
  successful run proves the export opened no socket and spawned nothing.
* Leak scenarios (`--mode leaks`): the cases where an export does not get
  to close its WebDriver session. `concurrent`: several Plotly exports
  started at once, as separate processes (parallel test binaries or
  worktrees), sharing the default chromedriver port. `interrupted`: one
  export whose process receives SIGINT, SIGTERM or SIGKILL while the browser
  is up. Leftovers are found and terminated as above, two seconds later.

Usage:
    measure.py --native BIN --baseline BIN --out DIR [--cold N] [--warm N]
               [--plotly-warm N] [--sandbox] [--backends plotters,plotly]
               [--name results]
    measure.py --mode leaks --baseline BIN --out DIR [--repeat N]
    measure.py --self-test
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
import threading
import time
from pathlib import Path

CASES = ("payoff", "curves", "surface")
NATIVE_FORMATS = ("png", "svg", "svg-plain")
PLOTLY_FORMATS = ("png", "svg")
EXTENSIONS = {"png": "png", "svg": "svg", "svg-plain": "plain.svg"}
SANDBOX_PROFILE = "(version 1)(allow default)(deny network*)(deny process-fork)"
TRACK_INTERVAL_S = 0.1
ENV = {**os.environ, "RUST_LOG": "warn"}


def process_table() -> dict[int, tuple[int, str]]:
    """Every process of the machine: PID -> (parent PID, command line)."""
    out = subprocess.run(
        ["ps", "-axo", "pid=,ppid=,command="], capture_output=True, text=True, check=True
    ).stdout
    table = {}
    for line in out.splitlines():
        parts = line.split(None, 2)
        if len(parts) == 3:
            table[int(parts[0])] = (int(parts[1]), parts[2])
    return table


def descendants(roots: set[int], table: dict[int, tuple[int, str]]) -> dict[int, str]:
    """The processes below `roots` in `table`, by PID, with their command lines."""
    children: dict[int, list[int]] = {}
    for pid, (parent, _) in table.items():
        children.setdefault(parent, []).append(pid)
    found, pending = {}, list(roots)
    while pending:
        for child in children.get(pending.pop(), []):
            if child not in found:
                found[child] = table[child][1]
                pending.append(child)
    return found


class Tracker:
    """Records the process trees under some children while they run."""

    def __init__(self) -> None:
        self.seen: dict[int, str] = {}

    def sample(self, roots: set[int]) -> None:
        self.seen.update(descendants(roots, process_table()))

    def leftovers(self, settle_s: float) -> list[dict]:
        """Recorded processes still alive after `settle_s`, terminated by PID.

        A PID only counts while its command line is the one recorded, so a PID
        the system reused for something else is left alone.
        """
        time.sleep(settle_s)
        table = process_table()
        found = [{"pid": pid, "command": command[:160]}
                 for pid, command in self.seen.items()
                 if pid in table and table[pid][1] == command]
        for item in found:
            try:
                os.kill(item["pid"], signal.SIGTERM)
            except ProcessLookupError:
                pass
        if found:
            time.sleep(1.0)
        return found


def run_watched(commands: list[list[str]], tracker: Tracker | None,
                timeout_s: float = 300.0) -> list[dict]:
    """Runs `commands` at once; returns each one's status, stderr and wall time.

    The wall time is taken by a thread blocked on the process, so sampling
    the process tree meanwhile does not coarsen it. stderr goes to a file, not
    a pipe: a process the command leaves behind inherits the descriptor, and
    reading a pipe to its end would wait for that process too.
    """
    with tempfile.TemporaryDirectory(prefix="osl-920-stderr-") as logs:
        files = [open(Path(logs) / f"{index}.err", "w+") for index in range(len(commands))]
        try:
            started = time.perf_counter()
            children = [subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=log,
                                         env=ENV)
                        for command, log in zip(commands, files)]
            results: list[dict] = [{} for _ in children]

            def wait(index: int, child: subprocess.Popen) -> None:
                try:
                    status: int | str = child.wait(timeout=timeout_s)
                except subprocess.TimeoutExpired:
                    # Our own child, by its handle; what it started is
                    # cleaned up by the caller's tracker.
                    child.kill()
                    child.wait()
                    status = "timeout"
                results[index] = {"status": status,
                                  "millis": (time.perf_counter() - started) * 1000.0}

            waiters = [threading.Thread(target=wait, args=(index, child))
                       for index, child in enumerate(children)]
            for waiter in waiters:
                waiter.start()
            while any(waiter.is_alive() for waiter in waiters):
                if tracker is not None:
                    tracker.sample({child.pid for child in children})
                time.sleep(TRACK_INTERVAL_S)
            for waiter in waiters:
                waiter.join()
            for result, log in zip(results, files):
                log.seek(0)
                result["stderr"] = log.read()[-400:]
            return results
        finally:
            for log in files:
                log.close()


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
        target = workdir / f"cold-{backend}-{case}-{fmt}-{run}.{EXTENSIONS[fmt]}"
        command = [binary, "export", backend, case, fmt, str(target)]
        if sandbox:
            command = ["sandbox-exec", "-p", SANDBOX_PROFILE, *command]
        # Nothing to watch for Plotters: the native binary links no browser
        # driver, and the sandboxed runs prove it starts no process.
        tracker = Tracker() if backend == "plotly" else None
        try:
            (result,) = run_watched([command], tracker)
        finally:
            # Success, failure, timeout or exception alike.
            if tracker is not None:
                leaked += [{"run": run, **item} for item in tracker.leftovers(1.0)]
        if result["status"] != 0:
            failures.append({"run": run, "status": result["status"],
                             "stderr": result["stderr"]})
            continue
        timings.append(result["millis"])
        sizes.append(target.stat().st_size)
    return {"backend": backend, "case": case, "format": fmt, "sandboxed": sandbox,
            "millis": timings, "stats": stats(timings) if timings else None,
            "bytes": sizes[0] if sizes else None, "failures": failures, "leaked": leaked}


def warm(binary: str, backend: str, case: str, fmt: str, iterations: int,
         workdir: Path) -> dict:
    tracker = Tracker() if backend == "plotly" else None
    leaked: list[dict] = []
    try:
        (result,) = run_watched(
            [[binary, "repeat", backend, case, fmt, str(iterations), str(workdir)]], tracker)
    finally:
        if tracker is not None:
            leaked = tracker.leftovers(1.0)
    if result["status"] != 0:
        return {"backend": backend, "case": case, "format": fmt, "failed": True,
                "stderr": result["stderr"], "leaked": leaked}
    report = json.loads((workdir / f"repeat-{backend}-{case}-{fmt}.json").read_text())
    millis = report["millis"]
    return {"backend": backend, "case": case, "format": fmt, "millis": millis,
            "first": millis[0], "stats": stats(millis[1:] if len(millis) > 1 else millis),
            "leaked": leaked}


def concurrent(binary: str, processes: int, workdir: Path, run: int) -> dict:
    tracker = Tracker()
    commands = [[binary, "export", "plotly", "curves", "png",
                 str(workdir / f"concurrent-{run}-{index}.png")]
                for index in range(processes)]
    try:
        results = run_watched(commands, tracker)
    finally:
        leaked = tracker.leftovers(2.0)
    return {"scenario": "concurrent", "processes": processes, "run": run,
            "statuses": [{"status": r["status"], "stderr": r["stderr"][-300:]}
                         for r in results],
            "tracked": len(tracker.seen), "leaked": leaked}


def interrupted(binary: str, sig: signal.Signals, delay_s: float, workdir: Path,
                run: int) -> dict:
    tracker = Tracker()
    child = subprocess.Popen([binary, "export", "plotly", "curves", "png",
                              str(workdir / f"interrupted-{run}.png")],
                             stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, env=ENV)
    try:
        deadline = time.perf_counter() + delay_s
        while time.perf_counter() < deadline and child.poll() is None:
            tracker.sample({child.pid})
            time.sleep(TRACK_INTERVAL_S)
        tracker.sample({child.pid})
        up_at_signal = len(tracker.seen)
        child.send_signal(sig)
        child.wait(timeout=60)
    finally:
        if child.poll() is None:
            child.kill()
            child.wait()
        leaked = tracker.leftovers(2.0)
    return {"scenario": "interrupted", "signal": sig.name, "delay_s": delay_s, "run": run,
            "status": child.returncode, "browser_up_at_signal": up_at_signal,
            "leaked": leaked}


def self_test() -> int:
    """Leak attribution without a browser: only the run's own tree is touched.

    A foreign process started before the run and one started during it stand
    in for another worktree's Chrome; the "export" is a shell that starts a
    long-lived child and exits with an error, the way a failed export leaves
    a browser behind. The leftover must be found and terminated, the foreign
    processes must survive, and a timed-out run must be killed and cleaned.
    """
    sleeper = [sys.executable, "-c", "import time; time.sleep(30)"]
    foreign_before = subprocess.Popen(sleeper)
    failures = []
    try:
        tracker = Tracker()
        foreign_during = None
        try:
            command = ["/bin/sh", "-c",
                       f"'{sys.executable}' -c 'import time; time.sleep(30)' & sleep 0.5; exit 3"]
            foreign_during = subprocess.Popen(sleeper)
            (result,) = run_watched([command], tracker)
        finally:
            leaked = tracker.leftovers(0.5)
        if result["status"] != 3:
            failures.append(f"failed export status {result['status']}, expected 3")
        if len(leaked) != 1:
            failures.append(f"expected exactly the one leaked child, found {leaked}")
        for name, process in (("before", foreign_before), ("during", foreign_during)):
            if process is None or process.poll() is not None:
                failures.append(f"foreign process started {name} the run was touched")
        timeout_tracker = Tracker()
        try:
            (timed,) = run_watched([sleeper], timeout_tracker, timeout_s=0.5)
        finally:
            timeout_tracker.leftovers(0.2)
        if timed["status"] != "timeout":
            failures.append(f"timed-out run reported {timed['status']}")
        if foreign_during is not None:
            foreign_during.kill()
            foreign_during.wait()
    finally:
        foreign_before.kill()
        foreign_before.wait()
    for failure in failures:
        print(f"self-test: {failure}", file=sys.stderr)
    print("self-test: " + ("FAILED" if failures else "OK"), file=sys.stderr)
    return 1 if failures else 0


def version(command: list[str]) -> str:
    try:
        return subprocess.run(command, capture_output=True, text=True).stdout.strip()
    except OSError as error:
        return f"unavailable: {error}"


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
    parser.add_argument("--self-test", action="store_true")
    parser.add_argument("--native")
    parser.add_argument("--baseline")
    parser.add_argument("--out")
    parser.add_argument("--name", default="results")
    parser.add_argument("--backends", default="plotters,plotly")
    parser.add_argument("--cold", type=int, default=10)
    parser.add_argument("--warm", type=int, default=200)
    parser.add_argument("--plotly-warm", type=int, default=10)
    parser.add_argument("--repeat", type=int, default=3)
    parser.add_argument("--sandbox", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()
    if not args.baseline or not args.out:
        parser.error("--baseline and --out are required")
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
        "track_interval_s": TRACK_INTERVAL_S,
    }
    if args.mode == "leaks":
        return leaks(args, out, environment)
    return timing(args, out, environment)


if __name__ == "__main__":
    sys.exit(main())
