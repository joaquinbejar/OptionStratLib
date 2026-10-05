#!/usr/bin/env python3
"""Fail when a component crate's public API gains an `f64` (#522).

Money, strikes, premia, P&L and public rates cross the public boundary of the
extracted crates as `rust_decimal::Decimal` or a validated newtype such as
`Positive`; `f64` belongs inside numerical kernels. This check reads the
checked-in `cargo public-api` snapshots of the component crates
(`public-api/optionstratlib-*.txt`) and fails on every line that mentions
`f64` unless it is

* an error diagnostic: an item whose own path lies under a crate's `error`
  module (a field or constructor of an error type, carrying the offending
  input or a non-finite intermediate that has no `Decimal` form). A
  function elsewhere that merely *returns* an error type is not exempt, or
* listed in `public-api/float-boundary-allowlist.txt`, where each entry is a
  reviewed, pre-existing `f64` item with the reason it is allowed.

An allowlist entry whose line no longer appears in any snapshot is reported
as stale, so the list only shrinks. The facade snapshot is not checked here:
its remaining upper-layer modules are audited as they are extracted.

Run `make check-float-boundary`; `make public-api-check` runs it too.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SNAPSHOTS = sorted((ROOT / "public-api").glob("optionstratlib-*.txt"))
ALLOWLIST = ROOT / "public-api" / "float-boundary-allowlist.txt"
F64 = re.compile(r"\bf64\b")
ERROR_PATH = re.compile(r"^optionstratlib_[a-z]+::error::")
# The item's own path: what follows `pub [fn|const|static|type] ` up to the
# parameter list, the field's type or a generic list.
ITEM_PATH = re.compile(r"^pub (?:(?:unsafe |async |const )*fn |const |static |type )?([A-Za-z0-9_:]+)")


def is_error_item(line: str) -> bool:
    """True when the snapshot line declares an item inside an `error` module."""
    match = ITEM_PATH.match(line.strip())
    return bool(match and ERROR_PATH.match(match.group(1)))


def allowlist(path: Path) -> dict[str, str]:
    """`line -> reason` from `<line> # <reason>` entries; blank and `#` lines skipped."""
    entries: dict[str, str] = {}
    if not path.exists():
        return entries
    for raw in path.read_text().splitlines():
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        line, sep, reason = raw.partition(" # ")
        if not sep or not reason.strip():
            raise SystemExit(f"{path.name}: entry without a reason: {raw!r}")
        entries[line.strip()] = reason.strip()
    return entries


def violations(lines: list[str], allowed: dict[str, str]) -> list[str]:
    return [
        line for line in lines
        if F64.search(line) and not is_error_item(line) and line.strip() not in allowed
    ]


def stale_entries(allowed: dict[str, str], seen: set[str]) -> list[str]:
    return sorted(line for line in allowed if line not in seen)


def self_test() -> int:
    allowed = {"pub fn a::utils::approx_equal(f64, f64) -> bool": "tolerance comparison"}
    cases = {
        "decimal signature": (["pub fn a::price(&Options) -> Decimal"], 0),
        "new f64 result": (["pub fn a::price(&Options) -> f64"], 1),
        "error diagnostic": (["pub optionstratlib_pricing::error::PricingError::NonFinite::value: f64"], 0),
        "allowlisted": (["pub fn a::utils::approx_equal(f64, f64) -> bool"], 0),
        "word boundary": (["pub fn a::to_f64_points(&self) -> Vec<Decimal>"], 0),
        "returns an error type, takes f64": (
            ["pub fn optionstratlib_core::model::decimal::f64_to_decimal(f64) -> Result<Decimal, optionstratlib_core::error::DecimalError>"],
            1,
        ),
        "error constructor": (["pub fn optionstratlib_pricing::error::pricing::PricingError::non_finite(&'static str, f64) -> Self"], 0),
    }
    failures = 0
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "allow.txt"
        path.write_text("# comment\n\npub fn a::b(f64) -> bool # a reason\n")
        ok = allowlist(path) == {"pub fn a::b(f64) -> bool": "a reason"}
        failures += not ok
        print(f"self-test {'ok' if ok else 'FAIL'}: allowlist parsing")
        path.write_text("pub fn a::b(f64) -> bool\n")
        try:
            allowlist(path)
            ok = False
        except SystemExit:
            ok = True
        failures += not ok
        print(f"self-test {'ok' if ok else 'FAIL'}: allowlist entry without a reason is refused")
    stale = stale_entries({"pub fn gone(f64) -> bool": "x"}, {"pub fn kept(f64) -> bool"})
    ok = stale == ["pub fn gone(f64) -> bool"]
    failures += not ok
    print(f"self-test {'ok' if ok else 'FAIL'}: stale allowlist entry reported")
    for name, (lines, expected) in cases.items():
        got = len(violations(lines, allowed))
        ok = got == expected
        failures += not ok
        print(f"self-test {'ok' if ok else 'FAIL'}: {name} (expected {expected}, got {got})")
    return 1 if failures else 0


def main() -> int:
    if "--self-test" in sys.argv:
        return self_test()
    allowed = allowlist(ALLOWLIST)
    seen: set[str] = set()
    found: list[str] = []
    for snapshot in SNAPSHOTS:
        lines = snapshot.read_text().splitlines()
        seen.update(line.strip() for line in lines)
        found += [f"{snapshot.name}: {line}" for line in violations(lines, allowed)]
    stale = stale_entries(allowed, seen)
    if stale:
        print("float-boundary allowlist entries no longer in any snapshot; remove them:")
        for line in stale:
            print(f"  {line}")
    if found:
        print("public f64 outside error diagnostics and the reviewed allowlist (#522):")
        for line in found:
            print(f"  {line}")
        print("Use Decimal or a validated newtype at the public boundary, or add a")
        print("reviewed entry with its reason to public-api/float-boundary-allowlist.txt.")
        return 1
    if stale:
        return 1
    print(f"OK: no unreviewed public f64 in {', '.join(s.stem for s in SNAPSHOTS)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
