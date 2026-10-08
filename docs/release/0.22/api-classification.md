# 0.22 public API and semver diff against 0.21.3: classification

Review evidence for #557, not a gate: compatibility with the published 0.21.3
is not a requirement of 0.22 (#606). Produced by
`scripts/report_api_changes.py --surface <surface> --baseline v0.21.3` for the
six surfaces (`none`, `default`, `plotly`, `static_export`, `async`, `all`;
raw output and exit status in `gates.md`, `semver-*` rows) and sorted by
`scripts/classify_api_changes.py`, on the commit named in `gates.md`.

## What the reports are

`cargo-semver-checks` 0.50.0 reports removals and incompatible reshaping, not
additions: the `all` surface lists 1,166 items: 1,163 module-level items (including 8
macros and 26 constants) missing at their 0.21.3 path, and 3 variants added to
an exhaustive enum. An item that moved to
a component crate and is re-exported at the same facade path is still reported,
because the tool compares the defining crate. The additions reach the reviewer
through the checked-in snapshots (`make public-api-check`): 26,983 lines for
the single 0.21.3 crate against 15,501 across the facade and the nine
component crates now.

## Method

Every reported item is sorted into exactly one class, by compiling
`use <old path>;` against the current facade with every feature (rustc, one
module per line) and by looking the item up in the snapshots (the tool
reports constants by bare name; those are looked up in the snapshots and
their facade path compiled):

| Class | Meaning | Action for a user |
| --- | --- | --- |
| `same-path` | the old path still resolves; the item is a re-export of the component's definition | none |
| `facade-path` | the old path is gone; the same-named item is at the root of another facade module (the file module became private, #550; the prelude was minimized, #551) | change the import to the canonical path |
| `facade-reexport` | the old path is gone; the facade reaches the item through the module root of the component that defines it | change the import to the canonical path |
| `component-only` | defined in a component crate and no facade path reaches it | none in 0.22 (zero items) |
| `removed` | no snapshot has an item of that name and kind | see the table below |

## Result (surface `all`)

| Old path area | same-path | facade-path | facade-reexport | removed | total |
| --- | ---: | ---: | ---: | ---: | ---: |
| `optionstratlib::(constants, bare name)` | 0 | 0 | 25 | 1 | 26 |
| `optionstratlib::Options` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::assert_decimal_eq` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::backtesting` | 38 | 0 | 0 | 0 | 38 |
| `optionstratlib::chains` | 18 | 4 | 0 | 0 | 22 |
| `optionstratlib::constants` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::curves` | 7 | 1 | 1 | 0 | 9 |
| `optionstratlib::d2f` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::d2fu` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::error` | 48 | 8 | 3 | 1 | 60 |
| `optionstratlib::f2d` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::f2du` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::geometrics` | 24 | 2 | 0 | 0 | 26 |
| `optionstratlib::greeks` | 41 | 0 | 0 | 0 | 41 |
| `optionstratlib::impl_graph_for_payoff_strategy` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::metrics` | 109 | 0 | 0 | 0 | 109 |
| `optionstratlib::model` | 51 | 1 | 1 | 0 | 53 |
| `optionstratlib::nz` | 1 | 0 | 0 | 0 | 1 |
| `optionstratlib::pnl` | 17 | 0 | 0 | 0 | 17 |
| `optionstratlib::prelude` | 68 | 300 | 30 | 6 | 404 |
| `optionstratlib::pricing` | 77 | 0 | 2 | 4 | 83 |
| `optionstratlib::risk` | 4 | 0 | 0 | 0 | 4 |
| `optionstratlib::series` | 4 | 0 | 0 | 0 | 4 |
| `optionstratlib::simulation` | 27 | 2 | 0 | 0 | 29 |
| `optionstratlib::strategies` | 136 | 11 | 0 | 5 | 152 |
| `optionstratlib::surfaces` | 4 | 1 | 0 | 0 | 5 |
| `optionstratlib::test_strategy_traits` | 0 | 0 | 0 | 1 | 1 |
| `optionstratlib::utils` | 16 | 8 | 3 | 7 | 34 |
| `optionstratlib::visualization` | 21 | 0 | 0 | 0 | 21 |
| `optionstratlib::volatility` | 15 | 1 | 0 | 0 | 16 |
| **total** | **734** | **339** | **65** | **25** | **1163** |

`prelude` is the largest area by far: 0.21 exported everything from five
globs, and #551 replaced them with an explicit, minimized prelude. Of its 404
reported items, 68 still resolve there, 330 are at a canonical facade path
(the removal table in the CHANGELOG, "Removed from `optionstratlib::prelude`",
lists the import to use) and 6 were removed with their owners (below).

## Removed items (25), each with its reason

| Removed | Replacement / reason | Issue, CHANGELOG entry |
| --- | --- | --- |
| `pricing::PricingEngine`, `pricing::price_option`, `pricing::unified::*` (and their `prelude` paths) | one generic engine parameterised by its Monte Carlo pricer: `GenericPricingEngine<M>`, `ClosedFormEngine`, `price_option_with` | #598, #508: "The pricing dispatcher becomes the generic engine" |
| `utils::logger`, `setup_logger`, `setup_logger_with_level` (and `prelude::setup_logger`) | a library installs no global subscriber; call `tracing_subscriber::fmt()...init()` from the binary | #506, #545: "`setup_logger` and `setup_logger_with_level` are gone" |
| `utils::others` | split into `utils::numeric` and `utils::rng`, each with one owner | #506: "`src/utils` carries a per-file owner" |
| `utils::file` | `prepare_file_path` lives at `visualization::prepare_file_path` | #506: "`prepare_file_path` moved to `visualization`" |
| `error::trade` | flat canonical path `error::TradeError` | #550: "Canonical 0.22 error paths" |
| `strategies::CallButterfly`, `strategies::call_butterfly`, `CALL_BUTTERFLY_DESCRIPTION` (and `prelude::CallButterfly`) | butterflies are textbook 1/2/1; the 1x1x1 call ladder it was is `BullCallLadder` | #706: "`CallButterfly` is removed" |
| `strategies::graph` | the empty module kept after the `Graph` impls moved to `visualization::strategies` | #658, #505: "`Strategable` no longer requires `Graph`" |
| `strategies::macros`, `test_strategy_traits!` | no longer public API; the conformance tests moved to the crates they test | #534: "`test_strategy_traits!` is no longer public API" |

## Reshaping

- `optionstratlib::error::Error` gains the variants `Adjustment`, `Backtest`
  and `Projection` (reported as `enum_variant_added`: the enum is exhaustive,
  so an exhaustive `match` needs new arms). They carry the errors of the
  crates that now own those types; see the CHANGELOG entries for #550 and #542.
- `optionstratlib::error::Error` needs `visualization` (it wraps `GraphError`),
  so the `none` and `async` surfaces report the enum itself as missing.
  The CHANGELOG entry for #542 records it.

## Differences between surfaces

Items present only under `all` are feature-gated, as intended (gaps are
listed against the `all` report): `visualization::{make_scatter, make_surface,
pick_color, to_plotly_mode}` and `prelude::make_surface` need `plotly`
(absent from `none`, `default` and `async`), and
`utils::read_ohlcv_from_zip_async` needs `async` (absent from `none`,
`default` and `plotly`). The `default` surface equals `all` minus those.
`ToSchema` impls are not reported by this tool; `schema` is a default feature
(#549), so `none`, `plotly`, `static_export` and `async` carry no derives, by
design.

## Reproduce

```sh
python3.13 scripts/report_api_changes.py --surface all --baseline v0.21.3 > all.log
python3 scripts/classify_api_changes.py all.log > classification.json
python3 scripts/classify_api_changes.py --markdown classification.json
```
