# 0.22.0 package contents and metadata

Evidence for M8-03 (#559): what each of the ten published packages ships,
the metadata crates.io will show, and the proof that the archives build on
their own. Nothing here is published, tagged or version-bumped.

The ten packages are the facade `optionstratlib` and its nine components:
`optionstratlib-core`, `-math`, `-pricing`, `-simulation`, `-market`,
`-analytics`, `-strategies`, `-backtest` and `-visualization`.

## Reproduce

```sh
make check-packages          # contents and metadata (scripts/check_packages.py)
make check-package-archives  # build, docs and doc tests from the unpacked archives,
                             # then the declared rust-version (scripts/check_package_archives.sh)
python3 scripts/check_packages.py --report   # the tables below
```

Both run in the Components workflow, after `check-components`. The archive
check shares its packaging with `check-direct-examples-packaged` and
`check-022-consumers-packaged` (`scripts/package_archives.sh`): the ten crates
are packaged in one `cargo package` run, so their path dependencies resolve
from each other, and unpacked outside the repository.

Recorded on `f2bea813` plus this change, with cargo 1.99.0 (stable) and Rust
1.89.0 for the minimum-version check.

## Contents

`cargo package --list` per package, and the archive sizes `cargo package`
reports:

| Package | Files | Archive | Categories | Keywords |
| --- | ---: | --- | --- | --- |
| `optionstratlib` | 13 | 353.7KiB (98.9KiB compressed) | finance, data-structures | finance, options, trading |
| `optionstratlib-core` | 39 | 674.2KiB (135.7KiB compressed) | finance, data-structures | finance, options, trading, decimal |
| `optionstratlib-math` | 42 | 729.5KiB (146.6KiB compressed) | finance, mathematics | finance, curves, interpolation, decimal |
| `optionstratlib-pricing` | 48 | 1.3MiB (261.0KiB compressed) | finance, mathematics | finance, options, pricing, greeks |
| `optionstratlib-simulation` | 24 | 421.4KiB (84.4KiB compressed) | finance, mathematics, simulation | finance, options, simulation, random-walk, monte-carlo |
| `optionstratlib-market` | 25 | 835.3KiB (147.7KiB compressed) | finance, data-structures | finance, options, option-chain, market-data |
| `optionstratlib-analytics` | 56 | 861.4KiB (141.3KiB compressed) | finance, mathematics | finance, options, pnl, risk, analytics |
| `optionstratlib-strategies` | 51 | 2.3MiB (368.6KiB compressed) | finance, mathematics | finance, options, strategies, spreads, trading |
| `optionstratlib-backtest` | 16 | 193.0KiB (43.0KiB compressed) | finance, mathematics, simulation | finance, options, backtesting, simulation, strategy |
| `optionstratlib-visualization` | 33 | 311.0KiB (75.1KiB compressed) | finance, visualization | finance, options, visualization, charts, plotly |

- `optionstratlib`: 4 files under `src/`, plus `Cargo.toml`, `Cargo.toml.orig`,
  `Cargo.lock`, `.cargo_vcs_info.json`, `LICENSE`, `README.md`,
  `docs/migration-0.22.md`, `docs/ownership.md` and
  `docs/release/0.22/RELEASE-NOTES.md` (the crate docs include the README,
  the migration guide and the release notes; the ownership map is the one
  the READMEs link).
- Each component: its `src/` tree, plus `Cargo.toml`, `Cargo.toml.orig`,
  `Cargo.lock`, `.cargo_vcs_info.json`, `LICENSE` and `README.md`.
  `optionstratlib-market`, `-analytics` and `-strategies` also ship
  `testdata/SP500-18-oct-2024-5781.88.json`, the option chain their unit tests
  read, so `cargo test` passes from the archive (#558, `publish-dry-run.md`).
  `make check-packages` checks it is the repository's `examples/Chains/` copy.

No archive carries `Draws/`, `target/`, `doc/`, `.issues/`, `.github/`,
`scripts/`, `fixtures/`, `public-api/`, tests, benches, examples, the
Makefile, `rust-toolchain.toml`, `CHANGELOG.md` or `README.tpl`;
`scripts/check_packages.py` fails if one does.

## Metadata

Every package inherits the workspace values: version `0.22.0` (lockstep,
ADR-0001 D1), edition `2024`, `rust-version = "1.89"`, license `MIT`
(with `LICENSE` beside each manifest), `readme = "README.md"`, and repository
and homepage `https://github.com/joaquinbejar/OptionStratLib`. No package sets
`documentation`, so crates.io links docs.rs. Categories are crates.io slugs
(at most five) and keywords follow the crates.io rules (at most five, at most
20 characters, ASCII starting with a letter). Every path dependency also
carries a version requirement, and every dependency on a sibling package
requires `^0.22.0`.

Each description names the crate's responsibility; a component's ends with
the facade feature that re-exports it, and the facade's names the nine
component crates. Each README has the same: a "Place in the workspace"
section with its dependencies, the layers it must not depend on, and its
facade paths and feature (#554). The README links to the ownership map and
the migration guide are absolute GitHub links to tracked files.

## Features

| Package | Features | Optional dependencies |
| --- | --- | --- |
| `optionstratlib` | `analytics`, `async`, `backtest`, `default`, `io`, `market`, `math`, `parallel`, `plotly`, `pricing`, `schema`, `simulation`, `static_export`, `strategies`, `synthetic`, `visualization` | the eight components above core |
| `optionstratlib-core` | `default`, `schema` | `utoipa` |
| `optionstratlib-math` | `default`, `schema` | `utoipa` |
| `optionstratlib-pricing` | `default`, `schema` | `utoipa` |
| `optionstratlib-simulation` | `default`, `schema` | `utoipa` |
| `optionstratlib-market` | `async`, `default`, `io`, `schema`, `synthetic` | `csv`, `optionstratlib-simulation`, `tokio`, `utoipa`, `zip` |
| `optionstratlib-analytics` | `default`, `schema` | `utoipa` |
| `optionstratlib-strategies` | `default`, `schema` | `utoipa` |
| `optionstratlib-backtest` | `default`, `schema` | `utoipa` |
| `optionstratlib-visualization` | `default`, `plotly`, `static_export` | `plotly` |

Every `dep:` entry names an optional dependency, every `x/feature` a
dependency and every `x?/feature` an optional one; every optional dependency
is enabled by a feature; no feature is named to subtract. Each row of
section 2 of `docs/ownership.md` names a facade feature that enables the
component feature in its last column (`io`, `async`, `synthetic`, `plotly`,
`static_export`, `schema`). `parallel` is the reserved facade feature the
map documents; it enables nothing.

## Unpacked archives

`scripts/check_package_archives.sh` unpacks the ten archives into a scratch
workspace outside the repository whose `[patch.crates-io]` resolves each
crate's `0.22.0` dependencies on its siblings to the unpacked copies, stamps
the unpacked files with the current time (the archives carry a fixed 2006
mtime, which let a kept build directory pass for up to date, #558), and
runs:

| Step | Result |
| --- | --- |
| `cargo build --workspace --all-features` | ok |
| `cargo check -p <crate> --no-default-features`, each of the ten | ok |
| `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps` | ok, no warning |
| `cargo test --workspace --all-features --doc` | ok: 270 doc tests pass, 0 fail (ignored ones are `ignore`/`no_run` blocks) |
| `cargo +1.89 check --workspace --all-features` | ok |
| `cargo +1.89 check --workspace --no-default-features` | ok |

Doc tests per crate (passed, with the ignored count in parentheses): facade
31, core 62 (2), math 29 (1), pricing 46, simulation 9 (14), market 16,
analytics 44 (39), strategies 22 (4), backtest 2 (1), visualization 9 (1).
Each archive is also built and tested alone, with its own unit tests, in
`make publish-dry-run` (#558, `publish-dry-run.md`).
The direct-component examples (`make check-direct-examples-packaged`, #555)
and the 0.22 consumer fixtures (`make check-022-consumers-packaged`, #552)
build and pass their tests against the same archives.

## Package notices

`cargo package` prints one notice per test or bench target a package cannot
ship (``warning: ignoring test `convergence` as `tests/convergence.rs` is
not included in the published package``): 53 in all.

| Package | Notices | Targets |
| --- | ---: | --- |
| `optionstratlib` | 4 | tests `tests`, `property_tests`, `prelude`; bench `benches` |
| `optionstratlib-core` | 4 | integration tests |
| `optionstratlib-math` | 4 | integration tests |
| `optionstratlib-pricing` | 10 | integration tests |
| `optionstratlib-simulation` | 4 | integration tests |
| `optionstratlib-market` | 6 | integration tests |
| `optionstratlib-analytics` | 13 | integration tests |
| `optionstratlib-strategies` | 3 | integration tests |
| `optionstratlib-backtest` | 2 | integration tests |
| `optionstratlib-visualization` | 3 | integration tests |

They are accepted, not fixed, because both ways of removing them are worse:

- **Shipping `tests/` and `benches/`** would publish tests that fail from the
  archive: they read repository data that is not in it
  (for example the option chains under `examples/Chains/`, which sit outside
  every package), and the benches need the workspace's Criterion setup. A
  crates.io user gains nothing from them, and the archives grow.
- **Suppressing the notice** has no cargo switch. The only way to stop it is
  to remove the test and bench targets from the manifest (`autotests =
  false`, `autobenches = false` and no `[[test]]` / `[[bench]]`), which
  would stop them from running in the repository and CI.

`cargo package` drops the targets from the published manifest, so the
archives build cleanly (see "Unpacked archives"). `scripts/release_gates.py`
counts these notices apart from compiler and tool warnings
(`docs/release/0.22/gates-notes.md`).

## Findings and changes

1. **The facade archive carried local tooling and repository-only files.**
   Its unanchored `include` patterns (`README.md`, `Cargo.toml`, `Makefile`,
   `rust-toolchain.toml`, `Docker/**`, `examples/**/*.rs`, `tests/**/*.rs`,
   `benches/**/*`) shipped the Makefile, the toolchain file, two nested
   READMEs (`examples/direct/README.md`,
   `tests/fixtures/semver-reports/README.md`), 16 integration-test files and
   11 bench files: 43 files before, 13 after. The tests read repository data
   (`examples/Chains/*.json`) that is not in the archive, and the Makefile
   drives scripts that are not either. The facade now includes, anchored at
   its root, only `src/`, the `docs/*.md` files and the release notes its
   rustdoc includes or its README links, `Cargo.toml`, `README.md` and
   `LICENSE`, as the components already did
   (whose patterns are now anchored the same way). The tests and benches stay
   in the repository and in CI; `cargo package` drops their targets from the
   published manifest.
2. **No package declared a `rust-version`, and the documented 1.88 was not
   enough.** The code needs 1.88 (`utoipa` 6, the 2024 let-chains), but the
   dependency requirements need 1.89: `uuid` 1.27 and `statrs` 0.19.1 (with
   `nalgebra` 0.35 and `wide` 1.7) declare `rust-version = "1.89"`, so
   `cargo +1.88 check` of the unpacked archives fails to resolve. The
   workspace now declares `rust-version = "1.89"`, every package inherits it,
   the README and crate docs say 1.89, and the archive check proves it with
   Rust 1.89 (installed with `rustup toolchain install 1.89 --profile
   minimal`; CI installs it the same way). Lowering the `uuid` and `statrs`
   requirements to keep 1.88 would be a dependency change and was not made.
3. **The descriptions did not place the crates.** Each component description
   now ends with the facade feature that re-exports it, and the facade's
   names its nine component crates instead of a generic summary.

Everything else held: lockstep `0.22.0`, edition, license, readme,
repository, homepage, valid categories and keywords, versioned path
dependencies, and additive features that match the ownership map.
