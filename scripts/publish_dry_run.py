#!/usr/bin/env python3
"""Package and publish-dry-run the ten published crates, then build and test
every archive on its own (#558, M8-02).

Nothing is uploaded: every `cargo publish` this script runs carries
`--dry-run`, and the script refuses to run a publish command without it.

Steps, in the publication order (core first, the facade last):

1. `cargo package -p <each>`: the ten archives, each verified by cargo. The
   components are not on crates.io yet, so one invocation packages them
   together and cargo resolves each crate's siblings from the archives it
   has just made rather than from the registry.
2. The size and SHA-256 of each archive.
3. `cargo publish --dry-run --workspace`: the publication of the workspace's
   publishable packages, in dependency order, short of the upload.
4. `cargo publish --dry-run -p <each>`, one at a time: the dry-run a release
   will repeat once each crate's dependencies are on crates.io. Today only
   `optionstratlib-core` resolves; every other crate stops on a sibling that
   is not published yet, which is recorded as `waiting on <sibling>`. Any
   other failure fails the run.
5. Every archive on its own: unpacked outside the repository into a
   workspace that holds only that crate, with a `[patch.crates-io]` entry
   for each sibling it depends on (directly or transitively) pointing at
   that sibling's unpacked archive, never at the repository. In it,
   `cargo build` and `cargo test` (unit and doc tests) run with the default
   features, with none, and with every feature but `static_export`, whose
   build script needs a browser.

A warning fails the run, except the two notices packaging produces on
purpose: `ignoring test/benchmark ... not included in the published package`
(#559 keeps tests and benches out of the archives) and `aborting upload due to
dry run`. Logs go to `<target>/publish-dry-run/`.

Usage:
    publish_dry_run.py [--allow-dirty] [--skip-isolated]
    publish_dry_run.py --self-test

`--allow-dirty` passes the flag to cargo for a working copy with
uncommitted changes; release evidence is taken without it, from a committed
tree.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import tarfile
import tempfile
import time
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
VERSION = "0.22.0"
# Publication order: every crate after the ones it depends on, the facade last.
PACKAGES = (
    "optionstratlib-core",
    "optionstratlib-math",
    "optionstratlib-pricing",
    "optionstratlib-simulation",
    "optionstratlib-market",
    "optionstratlib-analytics",
    "optionstratlib-strategies",
    "optionstratlib-backtest",
    "optionstratlib-visualization",
    "optionstratlib",
)
# Built in every feature set but this one: `plotly_static`'s build script
# needs an installed browser and webdriver.
EXCLUDED_FEATURE = "static_export"

ALLOWED_NOTICE = re.compile(
    r"^warning: (ignoring (test|benchmark) `[^`]+` as `[^`]+` is not included in the published package"
    r"|aborting upload due to dry run)$"
)
WARNING = re.compile(r"^warning(\[[^\]]*\])?:")
WAITING = re.compile(r"no matching package named `(optionstratlib[a-z-]*)` found")


def warnings_in(output: str) -> list[str]:
    """The warning lines of `output` that are not an allowed packaging notice."""
    return [
        line
        for line in output.splitlines()
        if WARNING.match(line) and not ALLOWED_NOTICE.match(line)
    ]


def notices_in(output: str) -> int:
    return sum(1 for line in output.splitlines() if ALLOWED_NOTICE.match(line))


class Run:
    """Runs commands, logs each one, and remembers the failures."""

    def __init__(self, logs: Path) -> None:
        self.logs = logs
        self.failures: list[str] = []
        self.count = 0

    def cargo(self, label: str, args: list[str], cwd: Path, env: dict[str, str] | None = None,
              may_fail: bool = False) -> tuple[int, str]:
        if args[:1] == ["publish"] and "--dry-run" not in args:
            raise SystemExit(f"refusing to run `cargo {' '.join(args)}` without --dry-run")
        self.count += 1
        log = self.logs / f"{self.count:03d}-{re.sub(r'[^A-Za-z0-9.-]+', '_', label)}.log"
        started = time.time()
        proc = subprocess.run(
            ["cargo", *args], cwd=cwd, env={**os.environ, **(env or {})},
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True,
        )
        elapsed = time.time() - started
        log.write_text(f"$ cargo {' '.join(args)}\n(cwd {cwd})\n\n{proc.stdout}\n[exit {proc.returncode}]\n")
        stray = warnings_in(proc.stdout)
        status = "ok" if proc.returncode == 0 else f"exit {proc.returncode}"
        print(f"  {label}: {status}, {elapsed:.0f}s, {notices_in(proc.stdout)} packaging notices", flush=True)
        if stray:
            self.failures.append(f"{label}: {len(stray)} unexpected warning(s), first: {stray[0]}")
            print(f"    unexpected: {stray[0]}", flush=True)
        if proc.returncode != 0 and not may_fail:
            self.failures.append(f"{label}: exit {proc.returncode} (log {log.relative_to(ROOT)})")
        return proc.returncode, proc.stdout


def metadata() -> dict:
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps"],
        cwd=ROOT, check=True, stdout=subprocess.PIPE, text=True,
    ).stdout
    return json.loads(out)


def sibling_closure(meta: dict) -> dict[str, list[str]]:
    """For each published package, the published siblings it depends on, transitively."""
    direct = {
        p["name"]: {d["name"] for d in p["dependencies"] if d["name"] in PACKAGES}
        for p in meta["packages"]
        if p["name"] in PACKAGES
    }
    closure: dict[str, list[str]] = {}
    for name in PACKAGES:
        seen: set[str] = set()
        todo = list(direct[name])
        while todo:
            dep = todo.pop()
            if dep not in seen:
                seen.add(dep)
                todo.extend(direct[dep])
        closure[name] = [p for p in PACKAGES if p in seen]
    return closure


def feature_sets(meta: dict, name: str) -> list[tuple[str, list[str]]]:
    package = next(p for p in meta["packages"] if p["name"] == name)
    features = sorted(f for f in package["features"] if f != "default")
    if EXCLUDED_FEATURE in features:
        kept = [f for f in features if f != EXCLUDED_FEATURE]
        everything = (f"all but {EXCLUDED_FEATURE}", ["--features", ",".join(kept)])
    else:
        everything = ("all", ["--all-features"])
    return [("default", []), ("none", ["--no-default-features"]), everything]


def fresh_mtimes(directory: Path) -> None:
    """Stamps every unpacked file with the current time.

    `cargo package` writes every file with the same fixed mtime (2006), and a
    workspace member's artifacts are named from its path relative to the
    workspace root, which is the same on every run. Without a fresh mtime
    cargo would take the artifacts of an earlier run as up to date and test
    stale code.
    """
    now = time.time()
    for path in directory.rglob("*"):
        if path.is_file():
            os.utime(path, (now, now))


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--allow-dirty", action="store_true")
    parser.add_argument("--skip-isolated", action="store_true")
    parser.add_argument("--self-test", action="store_true")
    args = parser.parse_args()
    if args.self_test:
        return self_test()

    meta = metadata()
    target = Path(meta["target_directory"])
    out = target / "publish-dry-run"
    logs = out / "logs"
    logs.mkdir(parents=True, exist_ok=True)
    for old in logs.glob("*.log"):
        old.unlink()
    publishable = sorted(p["name"] for p in meta["packages"] if p["publish"] is None)
    if publishable != sorted(PACKAGES):
        print(f"publishable packages {publishable} are not the ten published crates", file=sys.stderr)
        return 1
    dirty = ["--allow-dirty"] if args.allow_dirty else []
    run = Run(logs)
    head = subprocess.run(["git", "rev-parse", "HEAD"], cwd=ROOT, stdout=subprocess.PIPE, text=True).stdout.strip()
    print(f"commit {head}, {subprocess.run(['cargo', '--version'], stdout=subprocess.PIPE, text=True).stdout.strip()}")

    print("== 1. cargo package (verified), dependency order")
    selection = [arg for name in PACKAGES for arg in ("-p", name)]
    run.cargo("package", ["package", *selection, *dirty], ROOT)

    print("== 2. archives")
    archives = {name: target / "package" / f"{name}-{VERSION}.crate" for name in PACKAGES}
    for name, path in archives.items():
        if path.exists():
            print(f"  {path.name}  {path.stat().st_size} B  sha256 {sha256(path)}")
        else:
            run.failures.append(f"missing archive {path.name}")

    print("== 3. cargo publish --dry-run --workspace")
    run.cargo("publish-workspace", ["publish", "--dry-run", "--workspace", *dirty], ROOT)

    print("== 4. cargo publish --dry-run -p <each>, dependency order")
    for name in PACKAGES:
        code, output = run.cargo(f"publish-{name}", ["publish", "--dry-run", "-p", name, *dirty], ROOT, may_fail=True)
        if code == 0:
            print(f"    {name}: dry-run passes against crates.io")
            continue
        missing = WAITING.search(output)
        if missing:
            print(f"    {name}: waiting on {missing.group(1)} (not on crates.io yet)")
        else:
            run.failures.append(f"publish-{name}: failed for a reason other than an unpublished sibling")

    if not args.skip_isolated and not run.failures:
        print("== 5. each archive on its own (build and test)")
        closure = sibling_closure(meta)
        env = {"CARGO_TARGET_DIR": str(out / "isolated-target")}
        with tempfile.TemporaryDirectory(prefix="osl-publish-dry-run-") as scratch:
            for name in PACKAGES:
                work = Path(scratch) / name
                work.mkdir()
                for member in [name, *closure[name]]:
                    with tarfile.open(archives[member]) as tar:
                        tar.extractall(work)
                fresh_mtimes(work)
                patches = "".join(
                    f'{sib} = {{ path = "{sib}-{VERSION}" }}\n' for sib in closure[name]
                )
                (work / "Cargo.toml").write_text(
                    "[workspace]\n"
                    'resolver = "3"\n'
                    f'members = ["{name}-{VERSION}"]\n'
                    + (f"\n[patch.crates-io]\n{patches}" if patches else "")
                )
                print(f"  -- {name} (siblings from archives: {', '.join(closure[name]) or 'none'})")
                manifest = ["--manifest-path", str(work / "Cargo.toml")]
                for label, flags in feature_sets(meta, name):
                    run.cargo(f"{name}-build-{label}", ["build", *manifest, *flags], work, env)
                    run.cargo(f"{name}-test-{label}", ["test", *manifest, *flags], work, env)

    if run.failures:
        print("FAILED:")
        for failure in run.failures:
            print(f"  {failure}")
        return 1
    print(f"OK: logs in {logs.relative_to(ROOT) if logs.is_relative_to(ROOT) else logs}")
    return 0


def self_test() -> int:
    failures = 0

    def check(name: str, got: object, want: object) -> None:
        nonlocal failures
        if got != want:
            failures += 1
            print(f"FAIL {name}: got {got!r}, want {want!r}")

    out = "\n".join([
        "warning: ignoring test `convergence` as `tests/convergence.rs` is not included in the published package",
        "warning: ignoring benchmark `math` as `benches/math.rs` is not included in the published package",
        "warning: aborting upload due to dry run",
        "warning: unused import: `Foo`",
        "warning[E0001]: something",
        "   Compiling x",
    ])
    check("packaging notices are allowed", notices_in(out), 3)
    check("other warnings are kept", len(warnings_in(out)), 2)
    check(
        "an unpublished sibling is recognised",
        WAITING.search("no matching package named `optionstratlib-core` found").group(1),
        "optionstratlib-core",
    )
    meta = {"packages": [
        {"name": n, "features": {"default": [], "schema": [], **({"plotly": [], "static_export": []}
                                                                 if n == "optionstratlib" else {})},
         "dependencies": [{"name": d} for d in PACKAGES[: PACKAGES.index(n)][-1:]]}
        for n in PACKAGES
    ]}
    check("the closure is transitive", sibling_closure(meta)["optionstratlib-math"], ["optionstratlib-core"])
    check("the facade closes over every component", len(sibling_closure(meta)["optionstratlib"]), 9)
    check(
        "static_export is left out",
        feature_sets(meta, "optionstratlib")[2],
        (f"all but {EXCLUDED_FEATURE}", ["--features", "plotly,schema"]),
    )
    check("a crate without it builds all features", feature_sets(meta, "optionstratlib-core")[2],
          ("all", ["--all-features"]))
    try:
        Run(Path(tempfile.gettempdir())).cargo("publish", ["publish", "-p", "x"], ROOT)
        failures += 1
        print("FAIL a publish without --dry-run must be refused")
    except SystemExit:
        pass
    print("self-test: ok" if failures == 0 else f"self-test: {failures} failure(s)")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
