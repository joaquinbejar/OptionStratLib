## About the commit gated

The run above is of `1a21b724`: `main` at `68b12789` plus this branch's two
script commits (the runner and the `report_api_changes.py` fix). The branch
was rebased afterwards, onto `f2bea813` (#779), so that SHA is no longer in
its history; the files changed after the run are this documentation, the
classifier's handling of bare-name constants and the new `release-gates`
Makefile target, none of which a gate reads. The `release-notes` gate (#562, merged
after the run) ran on `9d962a36`, the rebased head, and is the only row from it. This is the evidence for the
state of `main` on 2026-10-08, not the release candidate: the candidate is
frozen by the owner (#560) and gets a complete rerun of `make release-gates`
on its commit, which rewrites this file.

## Findings and fixes of this run

- `make check-api-report` failed on `main`: the parser's self-test compares
  `SURFACES["default"]` of `scripts/report_api_changes.py` with the root
  `Cargo.toml` default, and #549 added `schema` to the latter. The surface
  list now carries `schema`, with a comment saying why `none`, `plotly`,
  `static_export` and `async` carry no derives. The same self-test is the
  first step of the API CHANGES workflow, so that step fails on every pull
  request that carries no such fix.
- `cargo package` prints one notice per test or bench target it cannot ship,
  for example ``warning: ignoring test `convergence` as `tests/convergence.rs`
  is not included in the published package``: 53 distinct notices since
  #559 (49 from the component manifests, 4 from the facade's), from the
  `include` lists (`src/**/*`, manifest, README, licence), which exclude
  `tests/` and `benches/`. They are cargo notices about package
  contents, not compiler or tool warnings, and the run counts them apart
  ("package notices"); the gates that package the crates (`components`,
  `consumers-022`, `direct-examples-packaged`) pass otherwise. Package contents
  and metadata belong to #559, which keeps them on purpose: an archive ships
  only `src/`, the manifest, the README and the licence (the facade also the
  two `docs/*.md` its rustdoc includes), while the integration tests and
  benches read repository data and stay in the repository and CI. The facade
  prints the same kind of notice for its own `[[test]]` and `[[bench]]`
  targets (`tests`, `property_tests`, `prelude`, `benches`). They are accepted
  cargo notices, not warnings to fix; `docs/release/0.22/packages.md` gives
  the per-package count and why neither shipping nor suppressing them is
  right.

## Facade capability matrix

`make lint` runs, for every entry of `FACADE_FEATURE_SETS` in the Makefile,
Clippy, `RUSTDOCFLAGS="-D warnings" cargo doc` and `cargo test` of the facade
with `--no-default-features --features <set>`: `math`, `schema`, `pricing`,
`market`, `io`, `async`, `synthetic`, `analytics`, `strategies`, `simulation`,
`market,simulation`, `analytics,simulation`, `strategies,simulation`,
`backtest`, `visualization` and `market,synthetic`, after the plain
`--no-default-features` test. `make test` runs the facade with the default
set, `plotly` and `static_export,plotly`; `make check-visualization` compiles,
lints, tests and documents the backend-neutral, `plotly` and `static_export`
surfaces apart, for the crate and for the facade. `make check-components`
tests, lints and documents each component with its default, no and all
features, and with its named feature sets (`optionstratlib-market:io`,
`:async`, `:synthetic`, `optionstratlib-visualization:plotly`).

## Not gates of this run

`make coverage` and `cargo audit` run in CI (`code_coverage.yml`,
`audit.yml`); the PNG/SVG export tests need a matching browser and run weekly
(`static_export.yml`, `make test-export`); benchmarks are not a gate. The
semver reports are informational (#606) and classified in
`api-classification.md`. Nothing is published, tagged or version-bumped by
this run (#560, #561 stay with the owner).

## Reproduce

```sh
python3 scripts/release_gates.py run      # every gate, logs in target/release-gates/
python3 scripts/release_gates.py render   # rewrites this file
```
