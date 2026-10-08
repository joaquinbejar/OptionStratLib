# Public concept ownership map (0.22)

Which crate owns each public concept of OptionStratLib 0.22, where to import
it from, and which feature it needs. One defining owner per concept: the
facade (`optionstratlib`) re-exports, it does not define, except where a row
says *facade*.

This file is checked: `scripts/check_ownership_map.py` (run by
`make check-graph`) parses every table below and fails when a listed path no
longer exists, when a facade path is not proved identical to its defining
item by a compiled test, when a feature disagrees with the manifests, or when
the public API gains a module, error or root item that no row covers. Edit
the tables with the code; the check names the row that drifted.

## How to read it

- **Defining module** is where the item lives and what a direct dependency on
  that component imports: `optionstratlib-pricing = "0.22"` gives
  `optionstratlib_pricing::pricing::black_scholes`.
- **Facade path** is the same item through `optionstratlib`. The two paths
  are the same type (`TypeId`), never a wrapper: `tests/unit/canonical_paths_test.rs`
  proves it for every row, and `tests/prelude/main.rs` for every prelude item.
- **Facade feature** is the `optionstratlib` feature that makes the facade path
  exist (`always` for the core paths). The facade default enables every
  capability plus `io`, `synthetic` and `schema`; component crates have empty
  defaults (ADR-0002).
- **Canonical path policy** (#550, owner decision on #752): an item defined in
  a submodule and re-exported by its parent is canonical at the parent
  (`optionstratlib::pricing::black_scholes`). The defining submodules
  (`pricing::black_scholes_model`, `strategies::long_call`, …) stay public as
  documentation anchors; new code and docs use the flat path. Paths kept only
  for 0.21 were removed in 0.22.
- **0.22 status**: `public` for every row below. Private in 0.22: the
  per-file error modules that held only a flat-exported type (`error::decimal`,
  `error::trade`, `error::curves`, `error::pricing`, `error::simulation`,
  `error::unified` and the other per-file error modules), and anything not
  listed here or in the component API snapshots under `public-api/`.

## 1. Capability modules

| Concept group | Defining module | Facade path | Example item | Facade feature | 0.22 status |
| --- | --- | --- | --- | --- | --- |
| Domain model: options, positions, trades, legs, payoffs, decimal helpers | `optionstratlib_core::model` | `optionstratlib::model` | `Position` | always | public |
| Utilities: time frames, RNG seeding, numeric helpers, `Len` | `optionstratlib_core::utils` | `optionstratlib::utils` | `TimeFrame` | always | public |
| Constants | `optionstratlib_core::constants` | `optionstratlib::constants` | `PI` | always | public |
| Curves (2D containers, interpolation, statistics) | `optionstratlib_math::curves` | `optionstratlib::curves` | `Curve` | math | public |
| Surfaces (3D containers) | `optionstratlib_math::surfaces` | `optionstratlib::surfaces` | `Surface` | math | public |
| Geometry: construction, interpolation, intersection | `optionstratlib_math::geometrics` | `optionstratlib::geometrics` | `ConstructionParams` | math | public |
| Pricing models (Black-Scholes, binomial, Monte Carlo, telegraph, exotics) | `optionstratlib_pricing::pricing` | `optionstratlib::pricing` | `BinomialPricingParams` | pricing | public |
| Greeks | `optionstratlib_pricing::greeks` | `optionstratlib::greeks` | `GreeksSnapshot` | pricing | public |
| Volatility: implied-volatility solvers, estimators, smiles | `optionstratlib_pricing::volatility` | `optionstratlib::volatility` | `VolatilitySmile` | pricing | public |
| Simulation: random walks, stochastic processes, simulators, exit policies | `optionstratlib_simulation::simulation` | `optionstratlib::simulation` | `ExitPolicy` | simulation | public |
| Market data: option chains | `optionstratlib_market::chains` | `optionstratlib::chains` | `OptionChain` | market | public |
| Market data: option series | `optionstratlib_market::series` | `optionstratlib::series` | `OptionSeries` | market | public |
| Analytics: probability kernels, profit ranges, projections, RND | `optionstratlib_analytics::analytics` | `optionstratlib::analytics` | `PriceTrend` | analytics | public |
| Profit and loss | `optionstratlib_analytics::pnl` | `optionstratlib::pnl` | `PnL` | analytics | public |
| Risk: SPAN margin, risk metrics | `optionstratlib_analytics::risk` | `optionstratlib::risk` | `SPANMargin` | analytics | public |
| Option-chain metrics (curve and surface traits) | `optionstratlib_analytics::metrics` | `optionstratlib::metrics` | `SmileDynamicsCurve` | analytics | public |
| Strategies, strategy traits, delta neutrality, strategy probability | `optionstratlib_strategies::strategies` | `optionstratlib::strategies` | `BullCallSpread` | strategies | public |
| Backtesting over simulated paths | `optionstratlib_backtest::backtesting` | `optionstratlib::backtesting` | `GeneralPerformanceMetrics` | backtest | public |
| Visualization: chart data, `Graph`, terminal reports | `optionstratlib_visualization::visualization` | `optionstratlib::visualization` | `GraphData` | visualization | public |

## 2. Feature-gated capabilities inside a module

These add items to a module above; the facade feature forwards to the
component feature.

| Capability | Defining item | Facade path | Facade feature | Component feature |
| --- | --- | --- | --- | --- |
| CSV, JSON and ZIP readers and writers of chains and OHLCV candles | `optionstratlib_market::chains::csv` | `optionstratlib::chains::csv` | io | `optionstratlib-market/io` |
| `*_async` wrappers over the market I/O | `optionstratlib_market::chains::chain::OptionChain::load_from_csv_async` | `optionstratlib::chains::OptionChain::load_from_csv_async` | async | `optionstratlib-market/async` |
| Simulation-backed chain generator | `optionstratlib_market::chains::generator_optionchain` | `optionstratlib::chains::generator_optionchain` | synthetic | `optionstratlib-market/synthetic` |
| Simulation-backed series generator | `optionstratlib_market::series::generator_optionseries` | `optionstratlib::series::generator_optionseries` | synthetic | `optionstratlib-market/synthetic` |
| Plotly rendering and trace builders | `optionstratlib_visualization::visualization::make_scatter` | `optionstratlib::visualization::make_scatter` | plotly | `optionstratlib-visualization/plotly` |
| PNG and SVG export | `optionstratlib_visualization::visualization::Graph::write_png` | `optionstratlib::visualization::Graph::write_png` | static_export | `optionstratlib-visualization/static_export` |
| `utoipa::ToSchema` derives on domain types | `optionstratlib_core::model::Options` | `optionstratlib::model::Options` | schema | `optionstratlib-core/schema` |

`parallel` is a reserved facade feature that enables nothing in 0.22.

## 3. Errors

Every concrete error has exactly one owner: the crate whose code raises it.
The canonical path is flat, `<crate>::error::<Error>` and
`optionstratlib::error::<Error>`. The `...Kind` detail enums stay in the kind
module named in the last column (their names collide across crates if
flattened); the facade re-exports those modules as `optionstratlib::error::<module>`.
The one exception is `OperationErrorKind`, the operation-level kind several
errors share: core exports it flat, at `optionstratlib_core::error::OperationErrorKind`
and `optionstratlib::error::OperationErrorKind`.

| Error | Owner package | Direct import | Facade path | Facade feature | Kind module |
| --- | --- | --- | --- | --- | --- |
| `DecimalError` | `optionstratlib-core` | `optionstratlib_core::error::DecimalError` | `optionstratlib::error::DecimalError` | always | — |
| `OptionsError` | `optionstratlib-core` | `optionstratlib_core::error::OptionsError` | `optionstratlib::error::OptionsError` | always | — |
| `PositionError` | `optionstratlib-core` | `optionstratlib_core::error::PositionError` | `optionstratlib::error::PositionError` | always | `position` |
| `TradeError` | `optionstratlib-core` | `optionstratlib_core::error::TradeError` | `optionstratlib::error::TradeError` | always | — |
| `CurveError` | `optionstratlib-math` | `optionstratlib_math::error::CurveError` | `optionstratlib::error::CurveError` | math | — |
| `SurfaceError` | `optionstratlib-math` | `optionstratlib_math::error::SurfaceError` | `optionstratlib::error::SurfaceError` | math | — |
| `InterpolationError` | `optionstratlib-math` | `optionstratlib_math::error::InterpolationError` | `optionstratlib::error::InterpolationError` | math | — |
| `MetricsError` | `optionstratlib-math` | `optionstratlib_math::error::MetricsError` | `optionstratlib::error::MetricsError` | math | — |
| `PricingError` | `optionstratlib-pricing` | `optionstratlib_pricing::error::PricingError` | `optionstratlib::error::PricingError` | pricing | — |
| `GreeksError` | `optionstratlib-pricing` | `optionstratlib_pricing::error::GreeksError` | `optionstratlib::error::GreeksError` | pricing | `greeks` |
| `VolatilityError` | `optionstratlib-pricing` | `optionstratlib_pricing::error::VolatilityError` | `optionstratlib::error::VolatilityError` | pricing | — |
| `SimulationError` | `optionstratlib-simulation` | `optionstratlib_simulation::error::SimulationError` | `optionstratlib::error::SimulationError` | simulation | — |
| `ChainError` | `optionstratlib-market` | `optionstratlib_market::error::ChainError` | `optionstratlib::error::ChainError` | market | `chains` |
| `OhlcvError` | `optionstratlib-market` | `optionstratlib_market::error::OhlcvError` | `optionstratlib::error::OhlcvError` | market | — |
| `ProbabilityError` | `optionstratlib-analytics` | `optionstratlib_analytics::error::ProbabilityError` | `optionstratlib::error::ProbabilityError` | analytics | `probability` |
| `ProjectionError` | `optionstratlib-analytics` | `optionstratlib_analytics::error::ProjectionError` | `optionstratlib::error::ProjectionError` | analytics | — |
| `RNDError` | `optionstratlib-analytics` | `optionstratlib_analytics::error::RNDError` | `optionstratlib::error::RNDError` | analytics | — |
| `TransactionError` | `optionstratlib-analytics` | `optionstratlib_analytics::error::TransactionError` | `optionstratlib::error::TransactionError` | analytics | — |
| `StrategyError` | `optionstratlib-strategies` | `optionstratlib_strategies::error::StrategyError` | `optionstratlib::error::StrategyError` | strategies | `strategies` |
| `AdjustmentError` | `optionstratlib-strategies` | `optionstratlib_strategies::error::AdjustmentError` | `optionstratlib::error::AdjustmentError` | strategies | — |
| `BacktestError` | `optionstratlib-backtest` | `optionstratlib_backtest::error::BacktestError` | `optionstratlib::error::BacktestError` | backtest | — |
| `GraphError` | `optionstratlib-visualization` | `optionstratlib_visualization::error::GraphError` | `optionstratlib::error::GraphError` | visualization | — |
| `Error` (aggregate over the errors above) | `optionstratlib` (facade) | `optionstratlib::error::Error` | `optionstratlib::error::Error` | visualization | — |

The `...Result<T>` aliases sit next to their error, flat: `DecimalResult`,
`OptionsResult`, `CurvesResult`, `PricingResult`, `GreeksResult`,
`SimulationResult`, `ProbabilityResult`, `StrategyResult`.

## 4. Facade root items

| Item | Defining path | Facade path | Facade feature |
| --- | --- | --- | --- |
| `Options` | `optionstratlib_core::model::Options` | `optionstratlib::Options` | always |
| `ExpirationDate` | `optionstratlib_core::model::ExpirationDate` | `optionstratlib::ExpirationDate` | always |
| `OptionStyle` | `optionstratlib_core::model::OptionStyle` | `optionstratlib::OptionStyle` | always |
| `OptionType` | `optionstratlib_core::model::OptionType` | `optionstratlib::OptionType` | always |
| `RainbowType` | `optionstratlib_core::model::RainbowType` | `optionstratlib::RainbowType` | always |
| `Side` | `optionstratlib_core::model::Side` | `optionstratlib::Side` | always |
| `assert_decimal_eq!` | `optionstratlib_core::assert_decimal_eq` | `optionstratlib::assert_decimal_eq` | always |
| `d2f!` | `optionstratlib_core::d2f` | `optionstratlib::d2f` | always |
| `d2fu!` | `optionstratlib_core::d2fu` | `optionstratlib::d2fu` | always |
| `f2d!` | `optionstratlib_core::f2d` | `optionstratlib::f2d` | always |
| `f2du!` | `optionstratlib_core::f2du` | `optionstratlib::f2du` | always |
| `nz!` | `optionstratlib_core::nz` | `optionstratlib::nz` | always |
| `impl_graph_for_payoff_strategy!` | `optionstratlib_visualization::impl_graph_for_payoff_strategy` | `optionstratlib::impl_graph_for_payoff_strategy` | visualization |
| `VERSION`, `version()` | `optionstratlib` (facade; each component has its own `VERSION`) | `optionstratlib::VERSION` | always |

## 5. Externally owned foundational types

Defined by separately published crates of the same owner, never duplicated
in the workspace. Only `optionstratlib-core` depends on them and re-exports
them (ADR-0001 D8); every other component, and the facade, imports them
through core (the facade's tests and benches name `positive` as a
dev-dependency, to prove the single `Positive` type). The canonical core
path is `optionstratlib_core::model::<Type>`; `model::types` also holds the
types that have no flat path.

| Type | Owning crate | Core re-export | Facade path |
| --- | --- | --- | --- |
| `Positive`, `PositiveError`, `is_positive`, `DAYS_IN_A_YEAR` (core re-exports it as `optionstratlib_core::constants::DAYS_IN_A_YEAR`), and the macros `pos_or_panic!`, `spos!`, `assert_pos_relative_eq!` (at the core root, `optionstratlib_core::pos_or_panic`, and in `optionstratlib::prelude`) | `positive` | `optionstratlib_core::model::Positive` | `optionstratlib::model::Positive` |
| `ExpirationDate`, `ExpirationDateError` | `expiration_date` | `optionstratlib_core::model::ExpirationDate` | `optionstratlib::model::ExpirationDate` |
| `OptionStyle`, `Side`, and `Action`, `UnderlyingAssetType` (these two only under `model::types`) | `financial_types` | `optionstratlib_core::model::OptionStyle` | `optionstratlib::model::OptionStyle` |
| `OptionType`, `RainbowType`, and the sub-enums and `OptionBasicType` (only under `model::types`) | `option_type` | `optionstratlib_core::model::OptionType` | `optionstratlib::model::OptionType` |

## 6. The prelude

`optionstratlib::prelude` is a convenience, never an owner: each item is a
re-export of a canonical path above, gated by the same feature. Its module
docs list every item with its reason and feature, and `tests/prelude/main.rs`
proves each one is its defining item on every feature surface.

## 7. Where new code goes

Put a new item in the lowest layer that can own it (ADR-0001 D9:
core ← math ← pricing ← simulation / market ← analytics ← strategies ←
backtest ← visualization), give it a flat re-export in its module, add a row
here if it is a new concept group, error or root item, and extend
`tests/unit/canonical_paths_test.rs` with its facade path. `make check-graph`
enforces the layering and this map.
