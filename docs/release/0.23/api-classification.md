# 0.23 public API and semver diff against 0.22: classification

Review evidence for #938, not a gate. Two sources:

1. **The facade**, `optionstratlib`: `scripts/report_api_changes.py --surface
   <surface> --baseline v0.22.2` on the six surfaces (`none`, `default`,
   `plotly`, `static_export`, `async`, `all`; raw output and exit status in
   `gates.md`, `semver-*` rows).
2. **Each component crate** against its published version on crates.io:
   `cargo semver-checks -p <crate> --all-features --release-type patch`
   (cargo-semver-checks 0.50.0). `--release-type patch` makes any
   incompatible change a reported failure whatever the manifest version
   says. The facade report cannot see these: it compares the items a crate
   defines, and the facade re-exports `GraphError` from
   `optionstratlib-visualization`.

Both report removals and incompatible reshaping, not additions; additions
reach the reviewer through the checked-in snapshots (`make
public-api-check`), which #933 updated for the new `GraphError` variants.

## Facade

| Surface | Result |
| --- | --- |
| `none`, `default`, `plotly`, `static_export`, `async`, `all` | no removal or incompatible reshaping against `v0.22.2` |

## Components

| Crate | Baseline (crates.io) | Checks | Findings |
| --- | --- | --- | --- |
| `optionstratlib-core` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-math` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-pricing` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-simulation` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-market` | 0.22.2 | 223 pass, 31 skip | none |
| `optionstratlib-analytics` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-strategies` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-backtest` | 0.22.1 | 223 pass, 31 skip | none |
| `optionstratlib-visualization` | 0.22.1 | 222 pass, 1 fail, 31 skip | `enum_marked_non_exhaustive`: `GraphError` |

## Classification

| Finding | Class | Where it is documented | Action for a user |
| --- | --- | --- | --- |
| `GraphError` marked `#[non_exhaustive]` (`crates/optionstratlib-visualization/src/error/graph.rs`), with fourteen new variants | intended, breaking | CHANGELOG `[Unreleased]`, breaking changes; `docs/migration-0.23.md`; ADR-0005 decision 5; release notes "Who has to change something" | add a wildcard arm to every `match` on `GraphError` |

The new variants themselves are not reported separately: adding a variant to
a `#[non_exhaustive]` enum is compatible, and marking the enum is the one
breaking step. Behaviour changes with no signature change (images drawn
from `graph_data()`/`graph_config()` rather than `to_plot()`, typed errors
for data the browser repaired, SVG text as outlines) are not visible to
`cargo-semver-checks`; they are listed in the CHANGELOG and the migration
guide.

Under `docs/versioning-policy.md` a breaking change in any crate moves the
whole family to the next minor line, which is why all ten crates are at
0.23.0.
