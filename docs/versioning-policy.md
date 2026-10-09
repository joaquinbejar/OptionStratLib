# Versioning policy for 0.22.x patch releases

**Status:** Proposed, awaiting the owner's decision (#834). Nothing below is in
force until it is accepted; until then the 0.22 lockstep rule of ADR-0001
applies unchanged.

## Context

OptionStratLib 0.22 ships ten crates.io packages: the facade `optionstratlib`
and nine components (`optionstratlib-core`, `-math`, `-pricing`,
`-simulation`, `-market`, `-analytics`, `-strategies`, `-backtest`,
`-visualization`). For 0.22.0 they move in **lockstep**:

- ADR-0001 rejected per-crate versioning for 0.22; the roadmap leaves it open
  "after the architecture is stable".
- `make check-packages` (`scripts/check_packages.py`, PR #786) enforces it:
  every package is exactly `0.22.0` (`VERSION`), and every path dependency on
  a sibling requires that version.
- `scripts/release_gates.py` records one evidence file per release,
  `docs/release/0.22/gates.md`, for the whole set.

The first 0.22.x patch will be the first time a fix touches fewer than ten
crates. This document decides what that release looks like before it is
needed.

## Options

### A. Strict lockstep for every release

Every 0.22.x release republishes all ten crates at the same version, even the
ones whose source did not change.

- Simple to reason about: one version names the whole set, and the existing
  checks hold as they are (only `VERSION` changes).
- Republishes unchanged crates: crates.io and every downstream lockfile see
  nine "new" versions that contain nothing new, and a consumer that depends
  only on `optionstratlib-core` gets a patch bump with no change.

### B. Lockstep minor, independent patch (recommended)

The **minor** line stays shared: every crate of the 0.22 family is
`0.22.N` for some `N`, and a breaking change opens 0.23 for all ten crates at
once. Within the line, a **patch** release publishes only the crates whose
published source changed, each with its own next patch number.

- A fix in `optionstratlib-pricing` ships `optionstratlib-pricing 0.22.1`;
  the other crates stay at `0.22.0`.
- Sibling requirements become `"^0.22"` (any 0.22.x) instead of the exact
  lockstep version, so Cargo resolves the newest compatible patch for
  consumers of the facade without republishing it.
- A dependent crate is republished only when it **needs** the fix to be
  correct (it raises its requirement, e.g. `"^0.22.1"`) or its own source
  changed.
- The facade is republished only when its own source, features or minimum
  requirements change.

### C. Fully independent versioning

Each crate follows its own semver line (`optionstratlib-pricing` may reach
0.24 while `-core` stays at 0.22).

- Maximum flexibility, but the facade then pins a matrix of versions, the
  ownership map (`docs/ownership.md`) has to track per-crate lines, and the
  "one version names the family" guarantee of ADR-0001 is gone. Not justified
  while the crates are released together from one repository.

## Proposed decision: B

1. **Shared minor line.** All ten crates share `0.MINOR`; a breaking change in
   any crate bumps the minor of all ten, published together (lockstep as for
   0.22.0).
2. **Independent patch.** A 0.22.x release publishes only the crates whose
   packaged source changed since their last release, each at its own next
   patch. A patch carries no public API change other than an addition that
   `cargo semver-checks` classifies as minor-compatible; anything else waits
   for the next minor.
3. **Sibling requirements** are `"^0.MINOR"` by default. A crate that depends
   on a fix raises its requirement to that patch (`"^0.22.1"`) and is
   published in the same release.
4. **Publication order** stays the dependency order of #560, restricted to the
   crates in the release.
5. **Evidence.** Each patch release records its gates in
   `docs/release/0.22.N/gates.md`, naming the crates published and the
   version of every crate the facade resolves at that commit.
6. **0.23 resets the family**: every crate is published at `0.23.0`.

## Changes once accepted

These are not part of this draft; they follow the owner's decision:

- `scripts/check_packages.py`: replace the exact `VERSION` check with "every
  package is on the `0.22` line" and "every sibling requirement is `^0.22` or
  a `^0.22.N` no newer than the sibling's own version"; keep the other checks.
- `scripts/release_gates.py`: take the list of crates being released and the
  release version from the command line, and write the evidence to the
  release's own directory.
- `make check-packages` self-test: add a mixed-patch fixture (`-pricing`
  at 0.22.1, the rest at 0.22.0) that must pass, and one with a sibling on
  0.23 that must fail.
- `CLAUDE.md` / release command: document the patch procedure.
