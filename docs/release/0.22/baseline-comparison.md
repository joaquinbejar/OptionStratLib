# 0.22 against the 0.21.3 baseline: dependencies, build time, artifacts

M8-07 (#563). The Milestone 0 baseline (M0-01, #490) recorded what a
consumer of 0.21.3 resolves and compiles. This document re-runs the same
commands on one host and one toolchain for 0.21.3 and for the 0.22 workspace,
so every delta below compares like with like. The raw logs, the scripts and
the resolved package lists are in
[`baseline-comparison/`](baseline-comparison).

## Revisions and host

| | 0.21.3 | 0.22 |
| --- | --- | --- |
| Commit | `80efdc02656b8df198a654b71a628ca292023ae5` (the M0 base) | `d274e8f7a5cb48a7a0fad5202daf1cac4ed0e1e4` (`origin/main`, 2026-10-08) |
| Lockfile | fresh resolution 2026-10-08, 528 packages, SHA-256 `3dfe332356e062165627c8833b2ca7277bac2ca736ff4fe60841f2bd631dd13f` | fresh resolution 2026-10-08, 522 packages, SHA-256 `d827698d7efb577396757c0c32f60d37b8b2f4333a42aa0796c8111a2c59d33f` |
| Production / test lines under `src/` (M0's `awk` split) | 77,518 / 113,653 | 88,229 / 125,427 (the facade's `src/` and every `crates/*/src/`) |

- **Host:** flumix, x86_64, 16 threads, 31 GiB RAM, Ubuntu 24.04 (Linux
  6.8.0-139). Its Docker containers stayed up but idle (load average 0.00
  before the run), no other compilation ran, and one idle login shell held no
  CPU.
- **Toolchain:** `rustc 1.99.0 (b940084d7 2026-09-28)`, `cargo 1.99.0
  (5f94df478 2026-08-27)`, the stable channel both `rust-toolchain.toml`
  files name. 0.21.3 builds on 1.99 unchanged, so no older toolchain was
  needed.
- **Measured:** 2026-10-08, 20:08 to 20:44 UTC.

Both lockfiles are fresh resolutions made minutes apart (`cargo
generate-lockfile`), so transitive versions come from the same registry state.
`Cargo.lock` is not tracked in either revision.

## Method

The M0-01 script (`doc/BASELINE.md`, appendix), generalised to one
package/feature profile per line
([`timing-563.sh`](baseline-comparison/timing-563.sh)), with M0's cache
policy:

- `cargo fetch` before the first sample: the registry cache is warm and no
  download is inside a measurement.
- **Clean:** `cargo clean` immediately before each sample, so every package
  of the profile compiles inside it. flumix sets no `build.build-dir`, so
  `target/` holds everything.
- **Incremental:** one warm `cargo check`, then `touch` of a source file
  before each timed `cargo check`.
- Debug profile, `--quiet`, wall clock from Python `time.time()`.
- **Repetitions:** three per cell, reported as median and range. The four
  cells behind the headline comparison (default and all-features facade,
  clean build and clean check, both revisions) got three more samples in a
  second pass that alternated the two revisions sample by sample
  ([`interleave-563.sh`](baseline-comparison/interleave-563.sh)), six in all.
- **Package counts:** M0's command, `cargo tree -q <profile> -e normal
  --prefix none | sed 's/ (\*)$//' | sort -u | wc -l` (name/version pairs;
  two versions of one crate count twice)
  ([`counts-563.sh`](baseline-comparison/counts-563.sh)). The lists are in
  `counts-0213/` and `counts-main/`.
- **Static export:** the `static_export` feature pulls in `plotly_static`,
  whose build script needs an installed browser or webdriver, and flumix has
  neither. Every all-features build of both revisions therefore ran with
  `WEBDRIVER_PATH=/usr/bin/true BROWSER_PATH=/usr/bin/true`. With
  `WEBDRIVER_PATH` pointing at an existing file the script skips browser
  detection and the driver download, so no network access is timed. This
  only affects compiling, not the measured code.

## Profiles

0.21.3 offers two surfaces: its `default = []` (every mandatory dependency,
already headless) and `--all-features` (`plotly`, `static_export`, `async`).
A 0.21.3 consumer that needed only pricing, or only market data, still took
the whole crate, so **each focused 0.22 profile is compared with the 0.21.3
default**. No 0.21.3 profile is narrower.

| Profile | 0.22 command | 0.21.3 counterpart |
| --- | --- | --- |
| core | `cargo build -p optionstratlib-core` | default (no core-only build existed) |
| pricing | `cargo build -p optionstratlib-pricing` | default |
| market | `cargo build -p optionstratlib-market` (no `io`, no `synthetic`) | default |
| simulation | `cargo build -p optionstratlib-simulation` | default |
| analytics | `cargo build -p optionstratlib-analytics` | default |
| headless facade | `cargo build -p optionstratlib --no-default-features --features backtest`: every computational layer, no visualization, I/O, generators or schema | default |
| 0.22 default | `cargo build -p optionstratlib` (adds visualization without Plotly, `io`, `synthetic` and `schema`; this is itself headless, with no Plotly or static export) | default |
| all features | `cargo build -p optionstratlib --all-features` | `--all-features` |

The component crates' default features are empty. The facade's capability
features resolve the same graph as the component they name; the consumer
fixtures assert that (`make check-fixtures`).

## Resolved packages

| Profile | 0.21.3 pairs | 0.22 pairs | Delta |
| --- | ---: | ---: | ---: |
| core | 131 | 41 | -90 (-69 %) |
| pricing | 131 | 62 | -69 (-53 %) |
| market | 131 | 63 | -68 (-52 %) |
| simulation | 131 | 63 | -68 (-52 %) |
| analytics | 131 | 65 | -66 (-50 %) |
| headless facade | 131 | 69 | -62 (-47 %) |
| 0.22 default | 131 | 124 | -7 (-5 %) |
| all features | 284 | 264 | -20 (-7 %) |

Package names (one per crate): 0.21.3 129 default and 260 all-features;
0.22 40, 61, 62, 62, 64, 68, 123 and 242 in the order above.

The dependency-tree success metrics hold independently of any timing; these
names are read from the lists in `counts-main/`:

| Profile | Resolves | Does not resolve |
| --- | --- | --- |
| core | `optionstratlib-core` and its domain types | any other OptionStratLib crate, `utoipa`, `csv`, `zip`, `plotly`, `tokio`, `reqwest`, `prettytable-rs`, `indicatif` |
| pricing | core, math, pricing | chains/series (`optionstratlib-market`), strategies, simulation, backtest, visualization, `csv`, `zip`, `tokio` |
| market | core, math, pricing, market | `optionstratlib-simulation` (only with `synthetic`), `csv`, `zip` (only with `io`), `tokio` |
| simulation | core, math, pricing, simulation | strategies, backtest, market, visualization |
| analytics | core, math, pricing, market, analytics | strategies, simulation, backtest, visualization |
| 0.22 default | the whole domain, `csv`, `zip`, `prettytable-rs`, `utoipa` | `plotly`, `plotly_static`, `tokio`, `reqwest` |

The 0.21.3 default resolves `csv`, `zip`, `prettytable-rs`, `indicatif` and
`utoipa` unconditionally.

**Counts against M0.** M0 recorded 131 default and 285 all-features pairs for
0.21.3 on macOS with the lockfile archived as
`doc/baseline/Cargo.lock.0.21.3-2026-09-19`. On flumix that same lockfile gives
131 and **287**, and today's fresh lockfile 131 and 284. `cargo tree` resolves
for the host target only, so Linux and macOS differ by their
platform-specific crates, and the 2026-09-19 to 2026-10-08 registry drift
accounts for the rest. Only the same-host, same-day 284 is compared above.

The issue's fixture commands (`cargo tree --all-features -e normal --prefix
none | sort -u | wc -l` inside the fixture, which keeps the `(*)` duplicate
lines and counts the fixture's own direct dependencies) print 86 for
`pricing-only` and 93 for `simulation-only`. They are not comparable with the
pairs above, only with themselves over time.

## Build time

Medians in seconds, with the range and the raw samples. "Clean" is `cargo
clean` before every sample.

| Profile | Clean check | Range | Clean build | Range | Samples |
| --- | ---: | --- | ---: | --- | ---: |
| 0.21.3 default | 17.5 | 17.0 to 18.4 | 23.4 | 22.1 to 24.7 | 6 |
| 0.21.3 all features | 42.8 | 41.6 to 44.1 | 55.2 | 52.7 to 78.3 | 6 |
| core | 4.1 | 3.8 to 4.1 | 4.9 | 4.8 to 4.9 | 3 |
| pricing | 8.7 | 8.6 to 8.8 | 11.2 | 10.8 to 11.3 | 3 |
| market | 9.2 | 9.1 to 9.3 | 11.7 | 11.3 to 11.8 | 3 |
| simulation | 9.1 | 8.9 to 9.1 | 11.1 | 10.9 to 11.6 | 3 |
| analytics | 9.6 | 9.5 to 9.7 | 12.1 | 12.0 to 12.1 | 3 |
| headless facade | 11.3 | 11.3 to 11.4 | 15.1 | 15.1 to 15.4 | 3 |
| 0.22 default | 15.7 | 15.2 to 16.0 | 21.3 | 20.9 to 22.0 | 6 |
| 0.22 all features | 34.5 | 34.2 to 35.0 | 42.8 | 42.0 to 43.7 | 6 |

Deltas of the medians against the 0.21.3 counterpart:

| Profile | Clean check | Clean build |
| --- | ---: | ---: |
| core | -77 % | -79 % |
| pricing | -50 % | -52 % |
| market | -47 % | -50 % |
| simulation | -48 % | -53 % |
| analytics | -45 % | -48 % |
| headless facade | -35 % | -35 % |
| 0.22 default | -10 % | -9 % |
| all features | -19 % | -22 % |

Raw samples, in run order:

- 0.21.3 default, build: 22.1, 23.7, 23.2, 22.5, 24.1, 24.7; check: 17.2,
  18.4, 17.8, 17.8, 17.0, 17.2.
- 0.21.3 all features, build: 56.3, 55.9, 78.3, 54.6, 52.7, 53.5; check:
  43.9, 44.1, 43.4, 41.6, 41.7, 42.1. The 78.3 is a single outlier: the other
  five samples, three of them in the alternated second pass, sit between 52.7
  and 56.3. The median absorbs it, and the delta is -22 % with or without it.
- 0.22 default, build: 21.4, 20.9, 22.0, 20.9, 21.7, 21.2; check: 15.2, 15.4,
  16.0, 15.6, 15.8, 15.8.
- 0.22 all features, build: 42.0, 42.5, 42.9, 42.7, 43.7, 43.4; check: 34.4,
  34.5, 34.2, 34.6, 34.5, 35.0.
- The focused profiles' samples are in `timing-main.log`.

**Incremental check** (`touch`, warm cache, three samples):

| Profile | File touched | Median (s) | Range (s) |
| --- | --- | ---: | --- |
| 0.21.3 default | `src/lib.rs` (all code) | 2.0 | 1.5 to 3.2 |
| 0.21.3 all features | `src/lib.rs` | 4.0 | 3.9 to 4.0 |
| core, pricing, market, simulation, analytics | the crate's own `src/lib.rs` | 0.2 to 0.3 | 0.2 to 0.3 |
| headless facade | `src/lib.rs` / `crates/optionstratlib-core/src/lib.rs` | 0.1 / 1.2 | 0.1 / 1.2 |
| 0.22 default | `src/lib.rs` / `crates/optionstratlib-core/src/lib.rs` | 0.1 / 1.6 | 0.1 / 1.6 to 3.6 |
| 0.22 all features | `src/lib.rs` / `crates/optionstratlib-core/src/lib.rs` | 2.7 / 2.7 | 2.7 / 2.7 to 2.9 |

In 0.21.3, touching `src/lib.rs` re-checks all of the library's code. In 0.22
the facade's `src/lib.rs` holds only re-exports, so the 0.22 rows also touch
core's `lib.rs`, the crate every layer depends on: the worst single-file case.
The all-features rows of both revisions carry a fixed cost of about 2 s
outside the library. `plotly_static`'s build script declares
`rerun-if-changed` on `~/.local/bin`, which does not exist on flumix, so Cargo
re-runs it on every check (see the confounders).

## Artifacts (debug)

`target/` is the whole directory after the clean build. "Library rlibs" is
the sum of the OptionStratLib crates' own `.rlib` files in `target/debug/deps`;
in 0.21.3 that is the single `liboptionstratlib.rlib`.

| Profile | Library rlibs (B) | `target/` (B) | `target/` delta |
| --- | ---: | ---: | ---: |
| 0.21.3 default | 170,318,516 | 1,098,900,306 | |
| 0.21.3 all features | 182,731,560 | 2,510,987,106 | |
| core | 7,627,740 | 222,470,046 | -80 % |
| pricing | 57,608,004 | 514,120,925 | -53 % |
| market | 71,251,786 | 555,997,431 | -49 % |
| simulation | 61,430,588 | 530,666,748 | -52 % |
| analytics | 82,873,132 | 593,379,300 | -46 % |
| headless facade | 161,646,408 | 823,125,967 | -25 % |
| 0.22 default | 204,703,478 | 1,126,396,796 | +2.5 % |
| 0.22 all features | 208,996,724 | 2,390,464,034 | -4.8 % |

The library's own rlibs grow (+20 % default, +14 % all features against
0.21.3). That is consistent with 14 % more production code and per-crate
metadata, and with the 0.22 default compiling `io`, `synthetic` and `schema`
code. The totals fall because fewer dependencies are compiled.

## Confounders

- **Host and toolchain differ from M0.** M0 ran on an Apple M4 Max with Rust
  1.98.1. Its absolute times (17.5 s default build) are not compared with
  these; both revisions were re-measured here instead.
- **Different code, not only different packaging.** 0.22 has 14 % more
  production lines, fixes many numerical models, adds `io`, `synthetic`,
  `schema` and terminal reports to its default, and drops
  `indicatif`. A delta is the effect of the whole release, not of the crate
  split alone. The focused profiles are the comparison the split exists for:
  a focused 0.22 consumer compiles a subset that no 0.21.3 consumer could
  select.
- **0.22 default is not 0.21.3 default.** The 0.22 default enables
  visualization (without Plotly), `io`, `synthetic` and `schema`, a superset
  of 0.21.3's default, and still resolves fewer packages and compiles
  faster; its `target/` is 2.5 % larger. A like-for-like
  capability match would sit between the headless facade and the 0.22
  default.
- **`plotly_static` build script.** The browser stub applies to both
  revisions. The missing `~/.local/bin` adds the same ~2 s re-run to every
  all-features incremental check in both, so the all-features incremental
  numbers measure that script as much as the library.
- **Single host, one evening.** Six samples for the headline cells, three for
  the rest. Every clean timing lies within 6 % of its median, with two
  exceptions: the 0.21.3 outlier described with the raw samples, and core's
  fastest check (3.8 s, 7 % under a 4.1 s median).
- **Debug profile only**, as M0. Release builds were not measured.

## Conclusions the data supports

- Every focused 0.22 profile resolves about half the packages of the 0.21.3
  crate (41 to 69 against 131) and checks from clean in 23 % to 65 % of its
  time, with the excluded capabilities verified absent from each graph.
- The 0.22 facade's default resolves 124 pairs against 131, and checks and
  builds about 10 % faster, while compiling more features.
- With all features, 0.22 resolves 264 pairs against 284, and checks 19 % and
  builds 22 % faster.
- Incremental checks after a one-file change inside a component take 0.1 to
  0.3 s in that component, against 2.0 s for the 0.21.3 crate. A change to
  core, which every layer depends on, still takes 1.2 to 1.6 s through the
  facade.

These are measurements of the two releases on one machine. They do not
isolate how much of each delta comes from the crate split rather than from
the other changes listed above.
