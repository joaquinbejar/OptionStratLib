# OptionStratLib 0.22.0 release notes

> **Draft.** Nothing described here is tagged, released or published yet.
> The crates.io and docs.rs links below resolve once the owner publishes
> 0.22.0, and the "Measured baseline" section is filled only after #563
> validates its measurements.

OptionStratLib 0.22.0 splits the library into nine component crates behind
the `optionstratlib` facade, gives every public concept one owning crate and
one canonical path, and fixes a set of numerical models whose results
change. **It is a breaking release with no compatibility promise toward
0.21.3 or earlier**: source, feature lists and some serialized data need the
changes the [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
describes, each with its replacement workflow.

- [Migration guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md):
  the replacement workflow for every redesign.
- [Ownership map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md):
  the defining crate, direct import, facade path and feature of every
  public concept.
- [CHANGELOG](https://github.com/joaquinbejar/OptionStratLib/blob/main/CHANGELOG.md):
  every change with its issue, reference values and migration note.
- [`examples/direct`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct):
  one runnable program per capability on component crates alone.

## Packages

Ten packages, all at **0.22.0**, listed in dependency order: each depends
only on packages above it (the market crate depends on simulation only
through its `synthetic` feature).

| Package | Owns | crates.io | docs.rs |
| --- | --- | --- | --- |
| `optionstratlib-core` | Domain model: options, positions, legs, trades, checked `Decimal` helpers and their errors | [0.22.0](https://crates.io/crates/optionstratlib-core/0.22.0) | [docs](https://docs.rs/optionstratlib-core/0.22.0) |
| `optionstratlib-math` | Curves, surfaces, interpolation and geometry on `Decimal` coordinates | [0.22.0](https://crates.io/crates/optionstratlib-math/0.22.0) | [docs](https://docs.rs/optionstratlib-math/0.22.0) |
| `optionstratlib-pricing` | Pricing models, Greeks and implied volatility | [0.22.0](https://crates.io/crates/optionstratlib-pricing/0.22.0) | [docs](https://docs.rs/optionstratlib-pricing/0.22.0) |
| `optionstratlib-simulation` | Random walks, stochastic processes, simulators, exit policies, path statistics | [0.22.0](https://crates.io/crates/optionstratlib-simulation/0.22.0) | [docs](https://docs.rs/optionstratlib-simulation/0.22.0) |
| `optionstratlib-market` | Option chains and series: building, parsing, CSV/JSON/ZIP I/O | [0.22.0](https://crates.io/crates/optionstratlib-market/0.22.0) | [docs](https://docs.rs/optionstratlib-market/0.22.0) |
| `optionstratlib-analytics` | P&L, risk, metrics, probability kernels, chain metrics and projections | [0.22.0](https://crates.io/crates/optionstratlib-analytics/0.22.0) | [docs](https://docs.rs/optionstratlib-analytics/0.22.0) |
| `optionstratlib-strategies` | Spreads, butterflies, condors, straddles, strangles, custom strategies, delta neutrality | [0.22.0](https://crates.io/crates/optionstratlib-strategies/0.22.0) | [docs](https://docs.rs/optionstratlib-strategies/0.22.0) |
| `optionstratlib-backtest` | Strategy backtests over simulated paths, run statistics and reports | [0.22.0](https://crates.io/crates/optionstratlib-backtest/0.22.0) | [docs](https://docs.rs/optionstratlib-backtest/0.22.0) |
| `optionstratlib-visualization` | Chart models, the `Graph` contract, terminal reports, optional Plotly rendering and static export | [0.22.0](https://crates.io/crates/optionstratlib-visualization/0.22.0) | [docs](https://docs.rs/optionstratlib-visualization/0.22.0) |
| `optionstratlib` | The facade: every component behind one version, one prelude and capability features | [0.22.0](https://crates.io/crates/optionstratlib/0.22.0) | [docs](https://docs.rs/optionstratlib/0.22.0) |

## Architecture

The components form a layered graph; a crate depends only on its own layer
and the ones below it, and `make check-graph` enforces it on every pull
request (ADR-0001 D9):

```text
core
  <- math
       <- pricing
            <- simulation
            <- market            market -> simulation only with `synthetic`
                 <- analytics
                      <- strategies
                           <- backtest        (also -> simulation)
                                <- visualization
```

A facade path and the component path name the same type, never a wrapper,
so code may mix `optionstratlib::model::Options` and
`optionstratlib_core::model::Options`.

## API policy

- **One owner per concept.** Every public type and capability has one
  defining crate, recorded in the ownership map and checked against the
  public-API snapshots by `make check-graph`. Wrappers, aliases and adapters
  kept only for 0.21 paths are removed (#550).
- **One canonical path.** The flat path of a module is canonical; defining
  submodules stay public as documentation anchors. Errors are flat in each
  crate's `error` module (`optionstratlib::error` on the facade); the
  detail enums stay in their kind modules (#550, #556).
- **A small, explicit prelude.** `optionstratlib::prelude` holds the domain
  types, the extension traits whose methods you call, and one entry type per
  capability, each behind its feature: 86 named items, no globs, no free
  functions, no errors (#551). The component crates `core` and `math` have
  no prelude, by decision (#518).
- **`Decimal` and validated newtypes at public boundaries.** Monetary values
  are `rust_decimal::Decimal` or `Positive`; `f64` stays inside numeric
  kernels, guarded by `make check-float-boundary` (#522).
- **Libraries do not print or install loggers.** Terminal output lives only
  in `optionstratlib-visualization` (#546); `setup_logger` is gone and no
  crate depends on `tracing-subscriber` (#545).

## Facade defaults and component selection

The facade's default enables every capability plus `io`, `synthetic` and
`schema`, so a plain dependency builds every capability 0.21's default build
had. Narrow it with `default-features = false` and the capabilities you use,
or depend on component crates, whose default features are empty.

| Facade feature | Adds | Implies |
| --- | --- | --- |
| `math` | `optionstratlib-math` | |
| `pricing` | `optionstratlib-pricing` | `math` |
| `market` | `optionstratlib-market` (no I/O, no generators) | `pricing` |
| `analytics` | `optionstratlib-analytics` | `market` |
| `strategies` | `optionstratlib-strategies` | `analytics` |
| `simulation` | `optionstratlib-simulation` | `pricing` |
| `backtest` | `optionstratlib-backtest` | `strategies`, `simulation` |
| `visualization` | `optionstratlib-visualization`, the aggregate `error::Error` | `backtest` |
| `plotly` | Plotly rendering | `visualization` |
| `static_export` | PNG/SVG export | `plotly`, `async` |
| `io` | CSV, JSON and ZIP readers and writers of market data | `market` |
| `async` | `tokio`-backed `*_async` market I/O | `market`, `io` |
| `synthetic` | Simulation-backed chain and series generators | `market`, `simulation` |
| `schema` | `utoipa::ToSchema` derives on every enabled component (adds no component) | |
| `parallel` | Reserved; enables nothing in 0.22 | |

Default: `pricing`, `market`, `analytics`, `strategies`, `simulation`,
`backtest`, `visualization`, `synthetic`, `io`, `schema`.

Component features: every component except visualization has `schema`;
`optionstratlib-market` also has `io`, `async` (implies `io`) and
`synthetic`; `optionstratlib-visualization` has `plotly` and
`static_export`.

### Minimal manifests

Each manifest below is followed by a program that uses it. Both are
checked: `make check-release-notes` builds and runs every pair as a
standalone crate with exactly the `[dependencies]` shown, and the programs
also run as doctests of the facade (`cargo test --doc`). Each lists
`rust_decimal`: the `dec!` macro, which the facade prelude re-exports from
`rust_decimal_macros`, expands to `rust_decimal` paths, so a crate that
writes `dec!` depends on `rust_decimal` itself.

The whole library through the facade:

```toml
[dependencies]
optionstratlib = "0.22.0"
rust_decimal = "1.43"
```

```rust
use optionstratlib::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Hull's worked example: S = 42, K = 40, r = 10%, sigma = 20%, T = 0.5.
    let call = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(40.0),
        ExpirationDate::Days(pos_or_panic!(182.5)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(42.0),
        dec!(0.10),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let price = call.calculate_price_black_scholes()?;
    assert!((price - dec!(4.76)).abs() < dec!(0.01));
    Ok(())
}
```

Pricing only, through the facade: no market data, I/O, simulation or
charts resolve.

```toml
[dependencies]
optionstratlib = { version = "0.22.0", default-features = false, features = ["pricing"] }
rust_decimal = "1.43"
```

```rust
use optionstratlib::prelude::*;
use optionstratlib::pricing::black_scholes;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let put = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        pos_or_panic!(40.0),
        ExpirationDate::Days(pos_or_panic!(182.5)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(42.0),
        dec!(0.10),
        OptionStyle::Put,
        Positive::ZERO,
        None,
    );
    // Hull's put on the same inputs: 0.81.
    let price = black_scholes(&put)?;
    assert!((price - dec!(0.81)).abs() < dec!(0.01));
    assert!(put.delta()? < Decimal::ZERO);
    Ok(())
}
```

The same capability on component crates, with no facade:

```toml
[dependencies]
optionstratlib-core = "0.22.0"
optionstratlib-pricing = "0.22.0"
rust_decimal = "1.43"
rust_decimal_macros = "1.40"
```

```rust
use optionstratlib_core::model::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
use optionstratlib_pricing::greeks::Greeks;
use optionstratlib_pricing::pricing::black_scholes;
use rust_decimal_macros::dec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let call = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        Positive::new(40.0)?,
        ExpirationDate::Days(Positive::new(182.5)?),
        Positive::new(0.2)?,
        Positive::ONE,
        Positive::new(42.0)?,
        dec!(0.10),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let price = black_scholes(&call)?;
    assert!((price - dec!(4.76)).abs() < dec!(0.01));
    assert!(call.delta()? > dec!(0.7));
    Ok(())
}
```

Strategies without simulation, backtesting or charts:

```toml
[dependencies]
optionstratlib = { version = "0.22.0", default-features = false, features = ["strategies"] }
rust_decimal = "1.43"
```

```rust
use optionstratlib::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Bull call spread 95/105: long the lower strike at 6.5, short the
    // higher one at 1.5, no fees.
    let spread = BullCallSpread::new(
        "XYZ".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(6.5),
        pos_or_panic!(1.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )?;
    // Width 10 less the net debit 5.
    assert_eq!(spread.get_max_profit()?, pos_or_panic!(5.0));
    assert_eq!(spread.get_max_loss()?, pos_or_panic!(5.0));
    Ok(())
}
```

More scenarios (market data, analytics, simulation, backtesting, charts) are
in [`examples/direct`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct)
and the consumer fixtures under
[`fixtures/consumers`](https://github.com/joaquinbejar/OptionStratLib/tree/main/fixtures/consumers),
which `make test-022-consumers` builds and runs feature set by feature set.

## Changes

Each item names its issue; the CHANGELOG has the full entry and the
migration guide the replacement workflow.

### Source

- **Crate split.** core (#514), math (#516), pricing (#521), market (#524),
  analytics (#529), strategies (#531), simulation (#536), backtest (#538) and
  visualization (#542) are their own crates; the facade re-exports them.
- **Paths and prelude.** Canonical flat paths and the removal of the
  duplicate error modules (`error::decimal`, `trade`, `curves`, `pricing`,
  `simulation`, `unified`) (#550); a minimal explicit prelude (#551);
  `AdjustmentError` moved to the strategies crate's `error` as a `thiserror`
  error (#556); the strategies module no longer re-exports analytics, P&L or
  Greeks items (#530); `test_strategy_traits!` is no longer public (#534).
- **Errors.** Errors wrap only lower layers (#511); math errors lose their
  rendering variant (#517); `CurveError` and `SurfaceError` gain `Generator`
  and `LegAble::pnl_at_price` / `Position::pnl_at_expiration` report
  `PositionError` (#507); `SimulationStats` reports `BacktestError`
  (#677); a failed numeric step
  in a pricing kernel is a typed `PricingError` instead of a substituted
  value (#639); Heston and telegraph walk overflow is a `SimulationError`
  instead of a panic (#686).
- **Capabilities move to traits.** `Options` lost its inherent pricing
  methods; use `OptionPricing` and `Greeks` (#499). `Strategable` no longer
  requires `Graph`, and `Simulator` / `RandomWalk` no longer implement
  `BasicAble` (#658).
- **Seeded randomness.** Monte Carlo, telegraph, `TelegraphProcess`,
  `simulate_returns` and `simulate_heston_volatility` take the caller's
  generator (#638); `WalkParams` has `seed: Option<u64>` (#539).
- **Telegraph pricer.** A Monte Carlo expectation over `no_paths` paths
  driven by a normal shock (#743), switching between the two volatility
  levels of `RegimeVolatility` (#755).
- **Strategies.** Constructors and builders reject strategies that fail
  their own `validate()` (#696); the put spreads take textbook legs (#696);
  butterflies are 1/2/1, `CallButterfly` is removed and the 1x1x1 ladder is
  `BullCallLadder` (#706); `max_profit` / `max_loss` are `Option<Positive>`
  (#661); covered strategies size their option legs in shares with fees per
  share (#731) and handle partial cover (#765).
- **Contract multiplier.** `Options`, `Trade`, `pnl::Transaction` and the
  strategies carry `contract_size` (#733, #760); `Trade::cost`, `income` and
  `net` are checked and fallible, and `PnL` converts from a `Trade` with
  `TryFrom` (#765).
- **Analytics.** `PriceTrend` has private fields and `PriceTrend::new`
  (#656); the probability kernels take a required `VolatilityAdjustment`
  (#619); `expected_value` is a signed `Decimal` (#623).
- **Market data.** `strike_price_range_vec` works in `Positive` (#642);
  `OptionChain::show()` is removed and `Display for OptionChain` writes a
  plain-text table (#546); `chains::generator_positive` is gone (#512).
- **Pricing inputs.** `Payoff::payoff` returns `OptionsResult<Decimal>` and
  `PayoffInfo` takes `Positive` spots (#637); Garman-Kohlhagen reads
  `ExoticParams::foreign_rate` (#720); a negative risk-free rate is valid
  (#709); `uncertain_volatility_bounds` returns signed `Decimal` bounds
  (#715).
- **Backtests and charts.** `SimulationStats` sums `PnL::total_pnl` and
  reports through `statistics()` (#691, #677); the `print_*` methods move to
  `visualization::terminal::SimulationReport` (#546); `Plottable` has no
  `Error` type (#543).
- **Logging.** `setup_logger` and `setup_logger_with_level` are removed; the
  library no longer depends on `tracing-subscriber` (#506, #545).

### Features

- The facade routes every capability through a feature (#528, #549), as in
  the table above; `pricing` alone resolves no market, I/O, async or
  visualization package.
- Market file I/O needs `io` and the `*_async` wrappers need `async`
  (#525).
- The simulation-backed generators need `synthetic`, the only
  market-to-simulation edge (#512). It is on by default; a build with
  `default-features = false` must list it.
- `ToSchema` derives need `schema`; without it no component resolves
  `utoipa` (#628).
- Component crates have empty default features.

### Serialization

- `"CallButterfly"` no longer deserializes as a `StrategyType` (#706).
- `StrategyRequest`s in the 0.21 put-spread leg order (#696) or with one
  contract per butterfly leg (#706) are rejected by `get_strategy`.
- Strategies derive `Deserialize` and are not re-validated when read; a
  deserialized `CoveredCall` or `Collar` from 0.21 reads its option
  quantity as shares (#731).
- `Options`, `Trade` and `Transaction` gain `contract_size`
  (`#[serde(default)]` = 1) (#733, #760).
- `ExoticParams` gains `foreign_rate` (#720).
- `OptionSeries` reads its `YYYY-MM-DD` keys as `ExpirationDate::DateTime`
  at 18:30 UTC and accepts no other key format (#643).

The guide's [serialized-data
table](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md#5-serialized-data)
gives what to do for each.

### Toolchain and dependencies

- **Rust 1.88 or newer**, edition 2024, stable toolchain: `utoipa` 6 needs
  1.88. The manifests declare no `rust-version`.
- The public API exposes `utoipa` 6, `positive` 0.7, `expiration_date` 0.4,
  `option_type` 0.4 and `financial_types` 0.3; a consumer naming their types
  must use the same lines. The facade itself no longer depends on
  `positive`: `Positive` comes from `optionstratlib-core` (#556).
- No crate below visualization resolves `prettytable-rs`, `indicatif` or
  `pretty-simple-display` (#546).

### Operations

- No library code writes to stdout or installs a global subscriber: install
  your own `tracing` subscriber, and print tables through
  `visualization::terminal` (#545, #546).
- Default builds are synchronous; `tokio` resolves only with `async` (and so
  with `static_export`) (#525, #549).
- Repository gates that guard the release: `make check-graph` (layers,
  error and utils partition, ownership map), `make check-feature-trees`,
  `make check-fixtures`, `make check-components` (each crate on its own),
  `make test-022-consumers`, `make test-direct-component-examples`,
  `make check-direct-examples-packaged`, `make check-float-boundary` and
  `make public-api-check`. The API review compares each pull request with
  its own base on six feature surfaces instead of a 0.21.3 compatibility
  gate (#606).
- CI and `make coverage` require `cargo-tarpaulin` >= 0.37.5.

## Results that change

0.22 fixes models whose 0.21 results were wrong. The changes below move
numbers; each CHANGELOG entry gives the reference values and tests. The
other entries of this release change API shape, and each CHANGELOG entry
says whether any value moves.

- **Probability:** the price-probability threshold includes the lognormal
  `-sigma^2 / 2` term, so the risk-neutral `P(S_T < K)` is `N(-d2)`; every
  probability with `sigma > 0`, and the strategy probabilities and expected
  values built on it, change (#664). No hidden 0.2 volatility (#619); an
  inverted CDF difference is an error instead of zero (#570).
- **Expected value:** signed, without the extra trend scaling (#623).
- **American options:** Barone-Adesi-Whaley and the binomial tree honour
  early exercise and `Side` (#648), including at the tree root (#708,
  #716).
- **Exotics:** barriers follow Reiner-Rubinstein (#646), fixed-strike
  lookbacks Conze-Viswanathan (#647), a gap put prices as a put (#649),
  quanto reads the foreign rate and Kirk's spread the second dividend yield
  (#650), a compound at its own expiry values the underlying at its payoff
  (#639), Garman-Kohlhagen uses `foreign_rate` when set (#720).
- **Monte Carlo and paths:** supplied-path Monte Carlo discounts at the
  risk-free rate (#651); path-based pricers include the dividend yield in
  the drift (#756); the telegraph pricer is a Monte Carlo expectation with
  regime volatility (#743, #755).
- **Walks:** telegraph switching probability `1 - e^(-lambda dt)` (#683),
  jump-diffusion `lambda dt` (#684), Heston normal Wiener increments (#742).
- **Implied volatility:** targets with no implied volatility are reported as
  errors (#652).
- **P&L and strategies:** mark-to-market scales with quantity (#725);
  covered strategies mark to market (#728), size in shares (#731) and
  report partial cover correctly (#765); put spreads price their textbook
  payoff (#696) and butterflies their 1/2/1 payoff (#706);
  `uncertain_volatility_bounds` gives a short position its negative bounds
  instead of `(0, 0)` (#715).
- **Conversions:** `decimal_to_f64` is correctly rounded (#670).

## Measured baseline

> **Placeholder: not filled in.** Build-time, dependency-count and runtime
> comparisons with 0.21 are added here only after #563 (M8-07) validates
> the measurements. Until then this release makes no performance or build
> size claim.
