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
* Leaks: every Plotly run starts in a process group of its own, which
  chromedriver and Chrome inherit, and its process tree is also sampled every
  `TRACK_INTERVAL_S` while it runs. Whatever of that group or tree is still
  alive a second after `compare` exited is reported as leaked and terminated
  (`killpg` on the run's own group, the PID for a sampled stray). Ownership
  comes from the group and the recorded ancestry, never from a command-line
  pattern, so a browser any other program runs at the same time (another
  worktree's tests, Selenium, Playwright, the user's Chrome) is never
  reported or touched. Failed, timed-out and interrupted runs are checked the
  same way, in `finally`.
* Sandbox (macOS, `--sandbox`): the native cold runs execute under
  `sandbox-exec` with network access and process creation denied, so a
  successful run proves the export opened no socket and spawned nothing.
* Leak scenarios (`--mode leaks`): the cases where an export does not get
  to close its WebDriver session. `concurrent`: several Plotly exports
  started at once, as separate processes (parallel test binaries or
  worktrees), sharing the default chromedriver port. `interrupted`: one
  export whose process receives SIGINT, SIGTERM or SIGKILL while the browser
  is up, and SIGINT to its whole process group, as Ctrl-C in a terminal
  sends. Leftovers are found and terminated as above, two seconds later.

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


def process_table() -> dict[int, tuple[int, int, str]]:
    """Every process of the machine: PID -> (parent PID, group ID, command line)."""
    out = subprocess.run(
        ["ps", "-axo", "pid=,ppid=,pgid=,command="], capture_output=True, text=True, check=True
    ).stdout
    table = {}
    for line in out.splitlines():
        parts = line.split(None, 3)
        if len(parts) == 4:
            table[int(parts[0])] = (int(parts[1]), int(parts[2]), parts[3])
    return table


def descendants(roots: set[int], table: dict[int, tuple[int, int, str]]) -> dict[int, str]:
    """The processes below `roots` in `table`, by PID, with their command lines."""
    children: dict[int, list[int]] = {}
    for pid, (parent, _, _) in table.items():
        children.setdefault(parent, []).append(pid)
    found, pending = {}, list(roots)
    while pending:
        for child in children.get(pending.pop(), []):
            if child not in found:
                found[child] = table[child][2]
                pending.append(child)
    return found


class Tracker:
    """Owns the processes an export starts, and finds the ones it leaves.

    Every child is started in a process group of its own
    (`start_new_session`), and chromedriver and Chrome inherit it, so
    membership survives the child exiting and its descendants being
    reparented, even before the first sample. The tree under each child is
    also sampled while it runs, for anything that leaves the group.
    """

    def __init__(self) -> None:
        self.groups: set[int] = set()
        self.seen: dict[int, str] = {}

    def adopt(self, child: subprocess.Popen) -> None:
        """Registers `child`, started with `start_new_session=True`."""
        self.groups.add(child.pid)

    def sample(self, roots: set[int]) -> None:
        self.seen.update(descendants(roots, process_table()))

    def leftovers(self, settle_s: float) -> list[dict]:
        """Owned processes still alive after `settle_s`, then terminated.

        Owned means: in one of the adopted groups, or sampled under a child
        and still running the command line it was sampled with (a PID the
        system reused for something else is left alone). Groups are signalled
        with `killpg`, stray sampled processes by PID; nothing else is.
        """
        time.sleep(settle_s)
        table = process_table()
        found = {pid: command for pid, (_, group, command) in table.items()
                 if group in self.groups}
        strays = {pid: command for pid, command in self.seen.items()
                  if pid not in found and pid in table and table[pid][2] == command}
        found.update(strays)
        for group in self.groups:
            try:
                os.killpg(group, signal.SIGTERM)
            except (ProcessLookupError, PermissionError):
                pass
        for pid in strays:
            try:
                os.kill(pid, signal.SIGTERM)
            except ProcessLookupError:
                pass
        if found:
            time.sleep(1.0)
        return [{"pid": pid, "command": command[:160]} for pid, command in found.items()]


def start_owned(command: list[str], tracker: Tracker | None, stderr) -> subprocess.Popen:
    """Starts `command` in a process group of its own, adopted by `tracker`."""
    child = subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=stderr, env=ENV,
                             start_new_session=True)
    if tracker is not None:
        tracker.adopt(child)
    return child


def stop(children: list[subprocess.Popen]) -> None:
    """Kills and reaps the children still running, by their handles."""
    for child in children:
        if child.poll() is None:
            child.kill()
    for child in children:
        child.wait()


def run_watched(commands: list[list[str]], tracker: Tracker | None,
                timeout_s: float = 300.0) -> list[dict]:
    """Runs `commands` at once; returns each one's status, stderr and wall time.

    The wall time is taken by a thread blocked on the process, so sampling
    the process tree meanwhile does not coarsen it. stderr goes to a file, not
    a pipe: a process the command leaves behind inherits the descriptor, and
    reading a pipe to its end would wait for that process too. Children are
    registered as they start, so if a later one fails to start, or anything
    raises, the started ones are killed and reaped; what they left behind is
    the caller's tracker's to clean.
    """
    with tempfile.TemporaryDirectory(prefix="osl-920-stderr-") as logs:
        files = []
        children: list[subprocess.Popen] = []
        try:
            started = time.perf_counter()
            for index, command in enumerate(commands):
                files.append(open(Path(logs) / f"{index}.err", "w+"))
                children.append(start_owned(command, tracker, files[-1]))
            results: list[dict] = [{} for _ in children]

            def wait(index: int, child: subprocess.Popen) -> None:
                try:
                    status: int | str = child.wait(timeout=timeout_s)
                except subprocess.TimeoutExpired:
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
            stop(children)
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


def interrupted(binary: str, sig: signal.Signals, to_group: bool, delay_s: float,
                workdir: Path, run: int) -> dict:
    """One export interrupted by `sig` while its browser is up.

    `to_group=False` signals the exporting process only, as a test harness,
    a timeout or `kill <pid>` does. `to_group=True` signals its whole process
    group, chromedriver and Chrome included, as Ctrl-C in a terminal does.
    """
    tracker = Tracker()
    child = start_owned([binary, "export", "plotly", "curves", "png",
                         str(workdir / f"interrupted-{run}.png")], tracker, subprocess.DEVNULL)
    try:
        deadline = time.perf_counter() + delay_s
        while time.perf_counter() < deadline and child.poll() is None:
            tracker.sample({child.pid})
            time.sleep(TRACK_INTERVAL_S)
        tracker.sample({child.pid})
        up_at_signal = len(tracker.seen)
        if to_group:
            os.killpg(child.pid, sig)
        else:
            child.send_signal(sig)
        child.wait(timeout=60)
    finally:
        stop([child])
        leaked = tracker.leftovers(2.0)
    return {"scenario": "interrupted", "signal": sig.name,
            "target": "process group" if to_group else "exporting process",
            "delay_s": delay_s, "run": run, "status": child.returncode,
            "browser_up_at_signal": up_at_signal, "leaked": leaked}


def self_test() -> int:
    """Leak attribution without a browser: only the run's own processes.

    Foreign processes started before and during a run stand in for another
    worktree's Chrome and must survive. Owned leftovers must be found and
    terminated when the "export" fails after a delay, when it fails before
    the first sample (its child is already reparented by then), and when it
    times out; and when the second of two commands cannot start, the first
    must be killed and reaped.
    """
    python = sys.executable
    sleeper = [python, "-c", "import time; time.sleep(30)"]
    leave_behind = f"'{python}' -c 'import time; time.sleep(30)' & {{delay}} exit 3"
    failures: list[str] = []
    foreign = [subprocess.Popen(sleeper)]

    def check(name: str, commands: list[list[str]], expect_status, expect_leaks: int,
              timeout_s: float = 300.0) -> None:
        tracker = Tracker()
        result = None
        try:
            foreign.append(subprocess.Popen(sleeper))
            (result,) = run_watched(commands, tracker, timeout_s=timeout_s)
        finally:
            leaked = tracker.leftovers(0.5)
        if result is None or result["status"] != expect_status:
            failures.append(f"{name}: status {result and result['status']}, "
                            f"expected {expect_status}")
        if len(leaked) != expect_leaks:
            failures.append(f"{name}: {len(leaked)} leftovers, expected {expect_leaks}")

    try:
        check("late failure", [["/bin/sh", "-c", leave_behind.format(delay="sleep 0.5;")]],
              3, 1)
        check("immediate failure", [["/bin/sh", "-c", leave_behind.format(delay="")]], 3, 1)
        check("timeout", [sleeper], "timeout", 0, timeout_s=0.5)
        # Reparented before any sample: only the process group can attribute it.
        orphaning = Tracker()
        parent = start_owned(["/bin/sh", "-c", leave_behind.format(delay="")], orphaning,
                             subprocess.DEVNULL)
        parent.wait()
        if len(orphaning.leftovers(0.5)) != 1:
            failures.append("reparented before any sample: leftover not attributed")
        first: list[subprocess.Popen] = []
        original = subprocess.Popen

        def recording_popen(*args, **kwargs):
            child = original(*args, **kwargs)
            first.append(child)
            return child

        subprocess.Popen = recording_popen
        try:
            run_watched([sleeper, ["/nonexistent/osl-920-command"]], Tracker())
            failures.append("start failure: no exception raised")
        except OSError:
            pass
        finally:
            subprocess.Popen = original
        if not first or first[0].poll() is None:
            failures.append("start failure: the first child was left running")
        for process in foreign:
            if process.poll() is not None:
                failures.append("a foreign process was touched")
    finally:
        stop(foreign)
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
                results["scenarios"].append(
                    interrupted(args.baseline, sig, False, 0.8, workdir, run))
            print(f"interrupted SIGINT to the group (Ctrl-C) run {run}", file=sys.stderr)
            results["scenarios"].append(
                interrupted(args.baseline, signal.SIGINT, True, 0.8, workdir, run))
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
