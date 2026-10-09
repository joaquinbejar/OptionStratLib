# OptionStratLib 0.22: architecture and adoption guide

OptionStratLib 0.22 is a redesign. The single crate of 0.21 is now nine
component crates and a facade, the public API was reshaped around one owner
per concept, and several results changed because their models were fixed.
**0.22 promises no compatibility with 0.21**: code, feature lists and
serialized data written for 0.21 may need the changes this guide describes.
Adopt 0.22 as a new API, using the replacement workflow of each section.

The [CHANGELOG](https://github.com/joaquinbejar/OptionStratLib/blob/main/CHANGELOG.md)
lists every change with its issue; this guide groups the ones you meet when
moving an application. Every Rust example here compiles and runs as a
doctest of the `optionstratlib` facade (`cargo test --doc`), so it matches
the released API.

## 1. The workspace

Each component crate owns one layer, and a layer depends only on the layers
below it (ADR-0001 D9, enforced by `make check-graph`):

```text
core (model, utils, constants)
  <- math (curves, surfaces, geometrics)
       <- pricing (pricing, greeks, volatility)
            <- simulation
            <- market (chains, series)        market -> simulation only with `synthetic`
                 <- analytics (analytics, pnl, risk, metrics)
                      <- strategies
                           <- backtest        (also -> simulation)
                                <- visualization
```

The [ownership map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
gives, for every concept, its defining crate and module, the direct import,
the facade path and the feature it needs. A facade path and the component
path are the same type, never a wrapper, so code may mix them.

## 2. Facade or components, and feature selection

Depend on the **facade** (`optionstratlib`) for the whole library behind
one version and one prelude. Its default enables every capability plus
`io`, `synthetic` and `schema`, so a plain dependency enables every
capability 0.21's default build had (not every 0.21 item: section 4 lists
the removals):

```toml
[dependencies]
optionstratlib = "0.22.0"
rust_decimal = "1.43"  # for the prelude's `dec!`, see section 3
```

Narrow it with `default-features = false` and the capabilities you use.
Each capability implies the layers it is built on (the table below):

```toml
[dependencies]
# pricing, greeks and volatility; no market data, I/O, async or charts
optionstratlib = { version = "0.22.0", default-features = false, features = ["pricing"] }
rust_decimal = "1.43"  # for the prelude's `dec!`
```

Or depend on **component crates** directly for the smallest graph. Their
default features are empty; opt into `io`, `async`, `synthetic`, `schema`,
`plotly` or `static_export` explicitly:

```toml
[dependencies]
optionstratlib-core = "0.22.0"
optionstratlib-pricing = "0.22.0"
```

| Crate | Own features | Facade feature |
| --- | --- | --- |
| `optionstratlib-core` | `schema` | always |
| `optionstratlib-math` | `schema` | `math` |
| `optionstratlib-pricing` | `schema` | `pricing` (implies `math`) |
| `optionstratlib-simulation` | `schema` | `simulation` (implies `pricing`) |
| `optionstratlib-market` | `io`, `async`, `synthetic`, `schema` | `market` (implies `pricing`); `io`, `async`, `synthetic` forward to the market features |
| `optionstratlib-analytics` | `schema` | `analytics` (implies `market`) |
| `optionstratlib-strategies` | `schema` | `strategies` (implies `analytics`) |
| `optionstratlib-backtest` | `schema` | `backtest` (implies `strategies` and `simulation`) |
| `optionstratlib-visualization` | `plotly`, `static_export` | `visualization` (implies `backtest`); `plotly`, `static_export` forward to it |

The facade also has `schema` (forwarded to every enabled component, adding
no component). The reserved `parallel` feature is retired (#832): `rayon` is
a mandatory dependency, and a manifest that names `parallel` must drop it.
`static_export` no longer implies `async` (#833): a manifest that used the
market `*_async` wrappers through `static_export` alone adds `async`.

**Minimal consumers.** [`examples/direct`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct)
has one runnable program per capability on component crates alone, each
with its facade equivalent. The consumer fixtures under
[`fixtures/consumers`](https://github.com/joaquinbejar/OptionStratLib/tree/main/fixtures/consumers)
build the facade and the components with one feature set each and assert
which packages their graph resolves; `make test-022-consumers` runs them.

## 3. Imports

**Canonical paths (#550).** Each item has one canonical path: the flat path
of its module. An item defined in a submodule and re-exported by its parent
is canonical at the parent, `optionstratlib::pricing::black_scholes` rather
than `optionstratlib::pricing::black_scholes_model::black_scholes`; the
submodules stay public as documentation anchors. Paths kept only for 0.21
are gone. Errors are flat in `optionstratlib::error`:
`optionstratlib::error::decimal`, `error::trade`, `error::curves`,
`error::pricing`, `error::simulation` and `error::unified` are removed, so
import `optionstratlib::error::{DecimalError, PricingError, ..}`. The
aggregate `optionstratlib::error::Error` exists only with the
`visualization` feature (on by default); a narrower build such as
`features = ["pricing"]` returns the layer error, `PricingError` there.
The detail enums (`...Kind`) stay in their kind module
(`error::position`, `error::greeks`, `error::chains`, `error::probability`,
`error::strategies`), and `AdjustmentError` moved from `strategies` to
`optionstratlib::error`.

**The prelude is small (#551).** `optionstratlib::prelude` now holds the
domain types, the extension traits whose methods you call on library types
(`OptionPricing`, `Greeks`, `Profit`, the strategy traits, `Graph`, ...)
and the entry type of each capability, gated by its feature. It has no
globs, no free functions, no errors and no standard-library items: import
`black_scholes`, `delta`, `generator_optionchain`, `Error`, `PricingError`,
`GraphData`, `std::path::Path` and the rest from their modules. The
prelude's own docs list every item with its reason.

**`dec!` needs `rust_decimal` in your manifest (#777).** The prelude
re-exports `rust_decimal_macros::dec`, which checks its literal at compile
time and expands to `::rust_decimal` paths, so a crate that writes `dec!`
depends on `rust_decimal` itself, next to the facade (the manifests in
section 2 do). Without it the build fails with ``cannot find `rust_decimal` ``.
`Decimal` alone needs nothing extra. `make check-release-notes` proves both
halves against the compiler.

**Pricing methods come from a trait (#499).** `Options` lost its inherent
`calculate_price_*`, `time_value` and `calculate_implied_volatility`; the
same calls work with `OptionPricing` in scope, which the prelude provides:

```rust
use optionstratlib::error::Error;
use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
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
    // `OptionPricing` carries the pricing methods, `Greeks` the Greeks; the
    // aggregate `error::Error` (only with `visualization`) takes either
    // error through `?`. A `pricing`-only build would return `PricingError`
    // and map the `GreeksError` of `delta` into it.
    let price = call.calculate_price_black_scholes()?;
    assert!((price - dec!(4.76)).abs() < dec!(0.01));
    assert!(call.delta()? > dec!(0.7));
    Ok(())
}
```

**Logging.** `setup_logger` and `setup_logger_with_level` are removed and
the library no longer depends on `tracing-subscriber`: install a subscriber
in your binary, for example `tracing_subscriber::fmt().init()`.

## 4. API redesigns and their replacement workflows

### Stochastic pricing takes your generator (#638, #743, #755)

No public pricing function draws from the thread RNG implicitly. Monte
Carlo, telegraph, `TelegraphProcess`, `simulate_returns` and
`simulate_heston_volatility` take a trailing `rng`: pass
`deterministic_rng(seed)` for reproducible results or `&mut rand::rng()` for
fresh draws. The telegraph pricer is now a Monte Carlo expectation over
`no_paths` paths (`pricing::TELEGRAPH_PATHS` is the trait's default), and
its regime switches between two volatility levels given as
`RegimeVolatility`; `RegimeVolatility::constant(sigma)` keeps one level.

```rust
use optionstratlib::prelude::*;
use optionstratlib::pricing::{RegimeVolatility, monte_carlo_option_pricing, telegraph};
use optionstratlib::utils::deterministic_rng;
use std::num::NonZeroUsize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let call = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    );
    let steps = NonZeroUsize::new(20).ok_or("steps")?;
    let paths = NonZeroUsize::new(500).ok_or("paths")?;

    // The same seed gives the same price.
    let first = monte_carlo_option_pricing(&call, steps, paths, &mut deterministic_rng(7))?;
    let again = monte_carlo_option_pricing(&call, steps, paths, &mut deterministic_rng(7))?;
    assert_eq!(first, again);

    // Telegraph: transition rates, then the two regime volatilities.
    let volatility = RegimeVolatility::new(pos_or_panic!(0.15), pos_or_panic!(0.25))?;
    let price = telegraph(
        &call,
        steps,
        paths,
        Some(dec!(0.5)),
        Some(dec!(0.5)),
        volatility,
        &mut deterministic_rng(7),
    )?;
    assert!(price > Decimal::ZERO);
    Ok(())
}
```

### Seeded walks: `WalkParams::seed` (#539)

`WalkParams` has a `seed: Option<u64>` field, so every struct literal names
it. `seed: None` keeps the unseeded behaviour; `Some(seed)` makes every
built-in stochastic walk, the walk drivers and `Simulator` reproducible.
Historical walks replay their prices and ignore it.

```rust
use optionstratlib::prelude::*;
use optionstratlib::simulation::generator_positive;

/// A walker that keeps every default kernel.
#[derive(Clone)]
struct Gbm;
impl WalkTypeAble<Positive, Positive> for Gbm {}

fn walk(seed: u64) -> Result<RandomWalk<Positive, Positive>, Box<dyn std::error::Error>> {
    let params = WalkParams {
        size: 30,
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walk_type: WalkType::GeometricBrownian {
            dt: pos_or_panic!(1.0 / 365.0),
            drift: dec!(0.05),
            volatility: pos_or_panic!(0.2),
        },
        walker: Box::new(Gbm),
        seed: Some(seed),
    };
    Ok(RandomWalk::new("gbm".to_string(), &params, generator_positive)?)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let first = walk(42)?;
    let again = walk(42)?;
    let last = |w: &RandomWalk<Positive, Positive>| w.last().map(|step| *step.y.value());
    assert_eq!(last(&first), last(&again));
    Ok(())
}
```

### Strategies validate what they build (#696, #706)

Every strategy constructor and builder returns
`StrategyError::InvalidStrategy` instead of a strategy that fails its own
`validate()`: inverted or equal strikes, a short leg with a zero premium, a
zero quantity, strike or underlying price, or an empty symbol are rejected.
An optimizer seed (`get_best_area`, `get_best_ratio`) must be a valid
strategy too: give it ordered strikes and a non-zero short premium.

- **Put spreads take textbook legs.** `BullPutSpread::get_strategy` takes
  the long put at the lower strike and the short put at the higher one (a
  credit spread), `BearPutSpread::get_strategy` the reverse. In 0.21 each
  built the other's payoff under its own name.
- **Butterflies are 1/2/1.** `LongButterflySpread` and
  `ShortButterflySpread` carry twice the wing quantity on the body, and
  their builders reject one contract per leg.
- **`CallButterfly` is removed.** It was long the lower strike and short the
  middle and upper ones, a ladder. The textbook long call butterfly is
  `LongButterflySpread`; the 1x1x1 ladder is `BullCallLadder`. No alias is
  kept, so code naming the old type fails to compile.
- **`Strategy::max_profit` / `max_loss` are `Option<Positive>`** (#661),
  not `Option<f64>`.

```rust
use optionstratlib::error::StrategyError;
use optionstratlib::prelude::*;
use optionstratlib::strategies::base::StrategyType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // A long call butterfly 90/100/110: the body is built with two contracts.
    let butterfly = LongButterflySpread::new(
        "XYZ".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(90.0),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(12.0),
        pos_or_panic!(5.0),
        pos_or_panic!(1.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )?;
    let contracts: Positive = butterfly
        .get_positions()?
        .iter()
        .map(|position| position.option.quantity)
        .sum();
    assert_eq!(contracts, pos_or_panic!(4.0));

    // A bull call spread with its strikes inverted is rejected, not built.
    let inverted = BullCallSpread::new(
        "XYZ".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(105.0),
        pos_or_panic!(95.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(1.5),
        pos_or_panic!(6.5),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    );
    assert!(matches!(inverted, Err(StrategyError::InvalidStrategy { .. })));

    // `CallButterfly` no longer names a strategy.
    assert!("CallButterfly".parse::<StrategyType>().is_err());
    assert!("BullCallLadder".parse::<StrategyType>().is_ok());
    Ok(())
}
```

### Optimizer searches report their outcome (#791, #793)

`Optimizable::find_optimal`, `get_best_area` and `get_best_ratio` return
`Result<(), StrategyError>`; in 0.21 they returned `()` and only logged a
failed search. Add `?` at each call. A search that finds a candidate picks
the same legs as in 0.21.

- **No candidate is an error.** An empty filtered chain, or one where every
  combination is discarded, returns `StrategyError::NoValidCandidate {
  strategy }` and leaves the strategy exactly as it was. Match it where an
  empty search is expected and the seed should be kept.
- **No search is an error.** `LongCall`, `LongPut`, `ShortCall`, `ShortPut`,
  `CoveredCall`, `ProtectivePut` and `Collar` return
  `StrategyError::OperationError(OperationErrorKind::NotSupported { .. })`.
- **`CustomStrategy`** returns the error of a position it cannot update from
  the chain (a strike without the quote a leg needs) and of the break-even
  recomputation of its best positions, with the strategy unchanged.

```rust
use optionstratlib::error::StrategyError;
use optionstratlib::prelude::*;
use optionstratlib::strategies::base::StrategyType;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut spread = BullCallSpread::default();
    // A chain with no strikes has no candidate: the search says so and the
    // seed is kept.
    let empty = OptionChain::new("XYZ", Positive::HUNDRED, "2030-01-18".to_string(), None, None);
    match spread.get_best_area(&empty, FindOptimalSide::All) {
        Ok(()) => { /* `spread` holds the best legs */ }
        Err(StrategyError::NoValidCandidate { strategy }) => {
            assert_eq!(strategy, StrategyType::BullCallSpread);
        }
        Err(e) => return Err(e.into()),
    }

    // A single leg has no search.
    let mut call = LongCall::default();
    assert!(call.get_best_ratio(&empty, FindOptimalSide::All).is_err());
    Ok(())
}
```

### Contract multiplier and position sizing (#733, #760, #731)

`Options` has `contract_size: Positive` (units of the underlying per
contract, 1 by default). Payoffs, premiums, costs, P&L, Greeks, margin and
the strategies' break-evens and extremes scale by
`quantity × contract_size`; prices from the pricing models stay per unit,
and position fees stay per contract. `Trade` and `pnl::Transaction` carry
their own `contract_size`, and strategies expose `get_contract_size` /
`set_contract_size` through `BasicAble`. A struct literal `Options { .. }`
or `Trade { .. }` must name the field (`contract_size: Positive::ONE`
keeps 0.21 results); `Options::new` keeps its signature. `Trade::new` and
`Trade::set_timestamp` return a `Result` (#771): a timestamp outside the
`i64` nanosecond range is a `TradeError::ArithmeticOverflow`.

`CoveredCall`, `Collar` and `ProtectivePut` size their option legs in shares
with option fees per share (#731): in 0.21 the first two used a hundredth of
an option per share, so their payoffs were neither capped nor floored. Pass
the number of shares the option covers and fees per share.

```rust
use optionstratlib::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One standard equity contract: 100 units of the underlying.
    let call = Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(105.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
    .with_contract_size(Positive::HUNDRED);
    assert_eq!(call.position_size()?, Positive::HUNDRED);
    // Five points in the money on one contract of 100 units.
    assert_eq!(call.payoff()?, dec!(500));
    Ok(())
}
```

### Analytics inputs and results (#656, #619, #623, #664, #829)

- `PriceTrend` has private `Decimal` fields: build it with
  `PriceTrend::new(drift_rate, confidence)?`, which rejects a confidence
  outside `[0, 1]`.
- The probability kernels take a required `VolatilityAdjustment`; they no
  longer price at a hidden 0.2 volatility.
- `ProbabilityAnalysis::expected_value` is a signed `Decimal`: a strategy
  that loses on average reports a negative value instead of zero.
- The price-probability threshold includes the lognormal `-sigma^2 / 2`
  term, so the risk-neutral `P(S_T < K)` is `N(-d2)`. Every probability for
  `sigma > 0`, and every strategy probability and expected value built on
  it, changes.
- `RNDAnalysis::calculate_rnd` / `calculate_skew`, `RNDResult::new` and
  `RNDStatistics::new` return the analytics-owned `error::RNDError` instead
  of the market's `ChainError`. `EmptyDensities` and `EmptySkewData` moved
  from `ChainError` to `RNDError`; a chain failure (no ATM volatility, for
  instance) arrives as `RNDError::Chain`, and a `?` into a function that
  returns `ChainError` becomes a `?` into `RNDError` (`From<ChainError>`
  exists) or into the facade's `error::Error`.
- `RNDParameters` has no `interpolation_points`: `calculate_rnd` never read
  it. Drop it from the literal; no result changes.

```rust
use optionstratlib::analytics::{PriceTrend, VolatilityAdjustment, calculate_single_point_probability};
use optionstratlib::prelude::*;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let volatility = VolatilityAdjustment {
        base_volatility: pos_or_panic!(0.2),
        std_dev_adjustment: Positive::ZERO,
    };
    let expiry = ExpirationDate::Days(pos_or_panic!(30.0));
    // Risk-neutral: P(S_T < 105) = N(-d2) = 0.79043 (S = 100, r = 5%).
    let (below, _) = calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(105.0),
        volatility,
        None,
        &expiry,
        Some(dec!(0.05)),
    )?;
    assert!((below.to_dec() - dec!(0.79043)).abs() < dec!(0.0001));

    // An upward trend lowers the probability of finishing below 105.
    let trend = PriceTrend::new(dec!(0.3), dec!(0.9))?;
    let (below_with_trend, _) = calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(105.0),
        volatility,
        Some(trend),
        &expiry,
        Some(dec!(0.05)),
    )?;
    assert!(below_with_trend < below);
    Ok(())
}
```

### Market data (#512, #525, #537, #642, #643)

- File I/O (`OptionChain::{load_from_csv, save_to_csv, load_from_json,
  save_to_json}`, the OHLCV ZIP reader) needs `io`; the `*_async` wrappers
  need `async`.
- The simulation-backed generators `chains::generator_optionchain` and
  `series::generator_optionseries` need `synthetic`. A simulation failure
  arrives as `ChainError::Generator`, whose source downcasts to
  `SimulationError`. `chains::generator_positive` is gone; use
  `simulation::generator_positive`.
- `OptionChain::strike_price_range_vec` takes and returns `Positive`.
- `OptionChain::show()` is removed: print chains with
  `visualization::terminal::ChainReport` (`print_table`, `render_table`).

### Payoffs, rates and FX (#637, #709, #720)

- `Payoff::payoff` returns `OptionsResult<Decimal>`, and `PayoffInfo`'s
  spot fields are `Positive`.
- A negative risk-free rate is valid everywhere.
- Garman–Kohlhagen reads a signed foreign rate from
  `ExoticParams::foreign_rate` when set, falling back to `dividend_yield`.
  An `ExoticParams` literal must name the field or end with
  `..ExoticParams::default()`.

```rust
use optionstratlib::model::option::ExoticParams;
use optionstratlib::prelude::*;
use optionstratlib::pricing::garman_kohlhagen;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // EUR/CHF-style call with a negative foreign rate:
    // S = 1.00, K = 0.98, r_d = 2%, r_f = -0.75%, sigma = 10%, T = 0.5.
    let call = Options::new(
        OptionType::European,
        Side::Long,
        "EURCHF".to_string(),
        pos_or_panic!(0.98),
        ExpirationDate::Days(pos_or_panic!(182.5)),
        pos_or_panic!(0.1),
        Positive::ONE,
        Positive::ONE,
        dec!(0.02),
        OptionStyle::Call,
        Positive::ZERO,
        Some(ExoticParams {
            foreign_rate: Some(dec!(-0.0075)),
            ..ExoticParams::default()
        }),
    );
    let price = garman_kohlhagen(&call)?;
    assert!((price - dec!(0.047738205471098)).abs() < dec!(0.000000001));
    Ok(())
}
```

### Backtests and terminal output (#538, #546, #677, #691)

Backtesting is `optionstratlib-backtest` (facade `backtest`), and terminal
tables belong to `optionstratlib-visualization`. The `print_*` methods of
the backtest types are gone: bring
`visualization::terminal::SimulationReport` into scope and call
`print_summary` / `print_individual_results`, or the `render_*` forms that
return the tables as a `String`. `SimulationStats::max_profit`, `max_loss`
and `avg_holding_period` are replaced by `statistics()?` (`best_pnl`,
`worst_pnl`, `average_holding_period`), and its errors are
`BacktestError`. A complete backtest, as the facade consumer fixture runs
it:

```rust
use optionstratlib::prelude::*;
use optionstratlib::simulation::generator_positive;
use optionstratlib::visualization::terminal::SimulationReport;

/// A walker that keeps every default: a historical walk replays its prices.
#[derive(Clone)]
struct Replay;
impl WalkTypeAble<Positive, Positive> for Replay {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Long 100 call, 30 days, premium 5, open and close fees 0.5 each.
    let call = LongCall::new(
        "XYZ".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.20),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )?;
    let prices = [100.0, 105.0, 110.0, 115.0, 120.0]
        .into_iter()
        .map(Positive::new)
        .collect::<Result<Vec<_>, _>>()?;
    let params = WalkParams {
        size: prices.len(),
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices,
            symbol: Some("XYZ".to_string()),
        },
        walker: Box::new(Replay),
        seed: None,
    };
    let simulator = Simulator::new("replay".to_string(), 2, &params, generator_positive)?;
    let stats = call.simulate(&simulator, ExitPolicy::Expiration)?;

    // In the money at expiry: 20 - premium 5 - fees 1 = 14 per walk.
    assert_eq!(stats.average_pnl, dec!(14));
    assert!(stats.render_summary()?.contains("SIMULATION SUMMARY"));
    Ok(())
}
```

The full version, with the report assertions, is
[`fixtures/consumers/headless-full/tests/workflow.rs`](https://github.com/joaquinbejar/OptionStratLib/blob/main/fixtures/consumers/headless-full/tests/workflow.rs).

### Charts (#542, #543, #690)

Charts are `optionstratlib-visualization` (facade `visualization`); the
`Graph` contract needs no backend, `plotly` adds rendering and
`static_export` adds PNG/SVG export. `Plottable` has no `Error` type; drop
`type Error = ..;` from an implementation. `impl_graph_for_payoff_strategy!`
names everything it expands to through `$crate` paths, so the caller no
longer imports `Graph`, `GraphData` and the rest for it.

## 5. Serialized data

0.22 reads and writes JSON with the same `serde` model, except as listed
below. Strategies derive `Deserialize`, so a strategy document is not
re-validated when read: the constructors, the builders and
`StrategyRequest::get_strategy` validate (#696), a deserialized strategy
does not until you call `validate()`.

| Data | 0.22 behaviour | What to do |
| --- | --- | --- |
| `StrategyType` / `StrategyRequest` with `"CallButterfly"` | No longer deserializes; `"CallButterfly".parse::<StrategyType>()` fails | Write `"BullCallLadder"` (the former ladder) or `"LongButterflySpread"` (the butterfly) |
| `StrategyRequest` for `BullPutSpread` / `BearPutSpread` with the 0.21 leg order | `get_strategy` returns an error instead of the other spread's payoff (#696): a bull put spread is long the lower strike and short the higher one, a bear put spread the reverse | Swap the legs of the stored request, or rebuild through the constructors |
| `StrategyRequest` or document for `LongButterflySpread` / `ShortButterflySpread` with one contract per leg | `get_strategy` rejects the request (#706); a deserialized document reads but fails `validate()` | Rebuild through the constructors, which put twice the wing quantity on the body, or call `validate()` on what you read |
| `CoveredCall`, `Collar` documents | Deserialize without re-validation, and their 0.21 option quantities (a hundredth of an option per share) now mean shares (#731) | Rebuild through the constructors with the shares covered and fees per share, then `validate()` |
| `Options` | Gains `contract_size` (`#[serde(default)]` = 1) | Old documents read as one-unit contracts |
| `Trade`, `Transaction` | Gain `contract_size` (`#[serde(default)]` = 1); `premium` is per unit of the underlying, fees per contract | Old documents read as one-unit contracts; for larger contracts store the per-unit premium and set `contract_size` |
| `ExoticParams` | Gains `foreign_rate` (`null` when unset) | Old documents read as `None` |
| `OptionSeries` | Keys are read back as absolute dates (`ExpirationDate::DateTime` at 18:30 UTC); only `YYYY-MM-DD` keys are accepted | Rewrite other key formats as `YYYY-MM-DD`; match `DateTime` or use `get_days()` |
| `ToSchema` (OpenAPI) | Derived only with the `schema` feature (on in the facade default) | Enable `schema` on component crates that need it |

## 6. Results that change

These fixes change numbers 0.21 returned; each is documented with its
reference values in the CHANGELOG:

- probability kernels: `-sigma^2 / 2` term (#664), no hidden 0.2 volatility
  (#619), no floor on an inverted CDF difference (#570);
- expected values: signed, without the extra trend scaling (#623);
- American pricing: early exercise and `Side` in Barone-Adesi-Whaley and
  the binomial tree, including the tree root (#648, #708, #716);
- barriers follow Reiner-Rubinstein (#646), fixed-strike lookbacks
  Conze-Viswanathan (#647), a gap put prices as a put (#649), quanto reads
  the foreign rate and Kirk's spread the second dividend yield (#650);
- supplied-path Monte Carlo discounts at the risk-free rate (#651), and the
  implied-volatility solvers report targets that have no implied volatility
  (#652);
- telegraph: Monte Carlo expectation and regime volatility (#743, #755);
  the path-based pricers include the dividend yield in the drift (#756);
- walks: telegraph switching probability `1 - e^(-lambda dt)` (#683),
  jump-diffusion `lambda dt` (#684), Heston normal Wiener increments (#742);
- P&L: mark-to-market scales with quantity (#725), covered strategies
  mark to market (#728) and size in shares (#731);
- put spreads chart and price their textbook payoff (#696), butterflies
  their 1/2/1 payoff (#706);
- `decimal_to_f64` is correctly rounded (#670);
- `Options::payoff` and the P&L built on it are each family's terminal payoff,
  the value its pricing kernel gives at `T = 0`, signed by the side, and the
  vanilla intrinsic value is the exact `Decimal` difference (#844).

## 7. Where to go next

- [0.22.0 release notes](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/release/0.22/RELEASE-NOTES.md):
  the release at a glance, with checked minimal manifests.
- [Ownership map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md):
  where every concept lives.
- [`examples/direct`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct):
  one program per capability on component crates.
- [CHANGELOG](https://github.com/joaquinbejar/OptionStratLib/blob/main/CHANGELOG.md):
  every change, with its issue and migration note.
- The facade's [crate documentation](https://docs.rs/optionstratlib) and each
  component's, which link back here.
