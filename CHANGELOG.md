# Changelog

All notable changes to **OptionStratLib** are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed — breaking

- **`Plottable` has no `Error` type, and the chart data of every graph
  adapter is pinned** (#543). Graph behaviour has left the lower layers
  (#542, #658); this finishes M6-02.
  - `Plottable::Error` is removed. Nothing read it: building a plot cannot
    fail, and rendering and export return `GraphError` whatever the plotted
    type, so the `CurveError` / `SurfaceError` it named for `Curve`,
    `Vec<Curve>` and `Surface` never came out of the plot path. Migration:
    delete the `type Error = ..;` line from a `Plottable` impl.
  - A new golden test, `optionstratlib-visualization`'s
    `tests/graph_data_golden_test.rs`, pins the `graph_data` and
    `graph_config` of every adapter: `Options`, `Position`, `Curve`,
    `Vec<Curve>`, `Surface`, `PlotBuilder`, `RandomWalk`, `Simulator` and
    the 22 concrete strategies. The golden file was generated on `624eeda2`,
    before #542 moved the adapters out of the facade, and the extracted
    crate reproduces it byte for byte, so the move changed no point of any
    chart. The strategies that `StrategyRequest` builds are charted from
    their positions through `StrategyConstructor`, which is the workflow for
    rendering a built strategy now that `Strategable` has no `Graph`
    supertrait. The test adds `serde_json`, already a workspace dependency,
    as a dev-dependency of the crate.
  - The crate docs list each adapter with its defining crate, and show the
    builder-to-chart workflow as a doctest: dispatch on `strategy_type` to
    the concrete type, or keep a `Box<dyn Chartable>` with
    `trait Chartable: Strategable + Graph`. The `curves` and `surfaces`
    module docs no longer describe `plotters`, `Vec<Surface>`, shading
    helpers or a `SurfaceError` plot path. References in the simulation and
    strategies crates that still placed `Graph` in the facade now point at
    `optionstratlib-visualization`.

- **Visualization is its own crate, `optionstratlib-visualization`** (#542).
  `visualization` (the `Graph` contract, `GraphData`, `GraphConfig`,
  `Series2D`, `Surface3D`, styles, `PlotBuilder` / `Plottable`, the Plotly
  trace builders and the `Graph` implementations for `Options`, `Position`,
  `Curve`, `Surface`, `RandomWalk`, `Simulator` and every concrete strategy)
  and `GraphError` move to `crates/optionstratlib-visualization`, the leaf of
  the workspace: it depends on core, math, pricing, simulation, market and
  strategies, and no other crate depends on it. Its features are `plotly`
  (`dep:plotly` without `static_export_default`) and `static_export`
  (`plotly` plus `plotly/static_export_default`), so `plotly` alone no longer
  resolves `plotly_static`, `fantoccini`, `webdriver`, `tokio` or `reqwest`;
  the workspace `plotly` dependency drops `static_export_default` for the
  same reason (ADR-0002 section 3). The facade re-exports `visualization`,
  `GraphError` and `impl_graph_for_payoff_strategy!` behind a new
  `visualization` feature (`dep:optionstratlib-visualization`, `backtest`; in
  `default`), `plotly` now implies `visualization` and forwards
  `optionstratlib-visualization/plotly`, and `static_export` forwards
  `optionstratlib-visualization/static_export`; the facade no longer depends
  on `plotly` itself. The unified `error::Error` wraps `GraphError`, so it
  and the `prelude` chart items (`Graph`, `GraphData`, `Series2D`,
  `Surface3D`, `TraceMode`, `Plottable`, `Error`, `GraphError`) need
  `visualization` instead of `backtest`; the facade unit suite declares
  `required-features = ["visualization"]`. Changes to the graph contract:
  - **One `Graph` trait on every feature surface.** There were two
    definitions, one without `plotly` and one with it; there is now one,
    whose required items (`graph_data`, with `graph_config` provided) are the
    same everywhere. `plotly` adds the provided `to_plot`, `write_html`,
    `show`, `render` and `to_interactive_html`, `static_export` adds
    `write_png` and `write_svg`; no implementation changes with a feature.
  - **`OutputType::Png` and `OutputType::Svg` exist without `plotly`**
    (ADR-0002 section 4 forbids feature-gated variants). Behaviour change:
    `Graph::render` with a PNG or SVG target in a build without
    `static_export` returns `GraphError::Render` instead of returning `Ok`
    without writing anything.
  - `optionstratlib::visualization::file` is gone; `prepare_file_path` stays
    at `visualization::prepare_file_path`, its one path.
  The visualization unit tests, the facade's `tests/unit/visualization`
  suite, `tests/unit/error/graph_test.rs` and the strategy panic-freedom
  property (which charts strategies) moved into the crate's tests, on
  component paths only. `make check-graph` forbids plotting, image-export,
  async and I/O packages in a featureless visualization build and allows
  `plotly` and the static-export packages only under their features, with
  crate-graph and forbidden-package self-tests; a new `visualization` facade
  surface fixture (`make check-feature-trees`) pins the headless graph;
  `check-components` (including `--features plotly` alone), `make test`,
  `make doc`, `make test-visual`, the public-API snapshots, the facade
  capability matrix (`visualization` alone) and CI cover the new crate.
  `Series2D::line_width` and the `VisPoint2D` / `VisPoint3D` widths are
  pixel sizes and join the reviewed `f64` allowlist.

- **`WalkParams` has a `seed: Option<u64>` field** (#539). Every struct
  literal now names it; `seed: None` keeps the previous behaviour exactly:
  the built-in walk kernels draw from the thread RNG, through the same
  generator, in the same order, so an unseeded walk is what it was. The
  field is the seed-ownership point of the simulation boundary (see Added).
  Migrate `WalkParams { size, init_step, walk_type, walker }` to
  `WalkParams { size, init_step, walk_type, walker, seed: None }`.

- **`SimulationStats` reports `BacktestError` and reuses the backtest
  adapter's projection** (#677). `SimulationStats::update` and
  `SimulationStats::update_outcome` return `Result<(), BacktestError>`
  instead of `Result<(), SimulationError>`: a running P&L total that leaves
  the `Decimal` range is `BacktestError::Decimal` (labelled
  `backtesting::stats::total_pnl`), and a run counter that cannot advance is
  the new `BacktestError::CounterOverflow { counter }` (built with
  `BacktestError::counter_overflow`) instead of
  `SimulationError::InvalidParameters`, since the counters belong to the
  backtest. `update` no longer copies the result into a `PathOutcome` field
  by field; it takes `From<&SimulationResult> for PathOutcome` and replaces
  only `pnl` with `result.pnl.realized`. That P&L is the documented
  difference from `SimulationStatsResult`, which sums `PnL::total_pnl`. The
  two are equal for every result the library builds: an early exit has no
  unrealized leg, an expiry has a zero one. They differ only for a
  caller-built result with a non-zero unrealized leg. Unit tests pin both
  sides of that difference and the agreement on library-shaped results.
  `print_summary` now prints the header `SIMULATION SUMMARY` instead of
  `SHORT PUT SIMULATION SUMMARY`. The new `SimulationStats::statistics()`
  returns `Result<PathStatistics, BacktestError>`, computed by
  `PathStatistics::from_outcomes` over the accepted outcomes, so the
  accumulator's aggregate comes from the same owner as
  `SimulationStatsResult`'s (the tests pin the two equal). `print_summary`
  still prints its own running figures. No figure changes: every `update`
  that succeeded before gives the same counters, totals, extremes, average
  holding period and printed summary.

- **Backtesting is its own crate, `optionstratlib-backtest`** (#538).
  `backtesting` (`Simulate` and its single-leg implementations, the adapters
  from the simulation engine's `PathOutcome` / `PathStatistics` to
  `SimulationResult` / `SimulationStatsResult`, run statistics, report types
  and performance metrics) and `BacktestError` move to
  `crates/optionstratlib-backtest`, which depends on core, pricing,
  simulation, analytics and strategies and on no math, chain I/O, plotting or
  async crate. The generic engine types stay `optionstratlib-simulation`'s
  and are reused, not duplicated. The facade re-exports `backtesting` and
  `BacktestError` behind a new `backtest` feature (`dep:optionstratlib-backtest`,
  `strategies`, `simulation`; in `default`, implied by `plotly`); the
  still-local visualization module, the unified `error::Error` and their
  `prelude` items now need `backtest`, and the facade test suites and benches
  declare `required-features = ["backtest"]`. Behaviour change:
  `simulate_single_leg` (and every `Simulate::simulate` built on it) no
  longer draws an `indicatif` progress bar; it reports each evaluated path as
  a `tracing` debug event and the end of the run as an info event (ADR-0002
  section 6), so `indicatif` leaves the facade and the backtest graph. The
  results are unchanged: the 60 unit tests moved with their code and the
  single-leg golden regression (`tests/golden/single_leg_simulation.json`)
  moved into the crate's tests unchanged. `prettytable-rs` stays in the
  backtest crate for `SimulationStatsResult::print_summary` until M6-05, as
  in market; the facade drops its own direct `prettytable-rs`, `indicatif`
  and normal `uuid` dependencies (`uuid` is a dev-dependency for one test).
  With the facade no longer enabling `prettytable-rs/win_crlf`, the chain and
  statistics tables print LF line endings on Windows too.
  `make check-graph` forbids plotting, I/O, async and `indicatif` packages in
  backtest, with crate-graph and forbidden-package self-tests;
  `check-components`, `make test`, `make doc`, the public-API snapshots, the
  facade capability matrix (`backtest` alone) and CI cover the new crate.

- **`PriceTrend` has private `Decimal` fields and a validating constructor**
  (#656). `drift_rate` (annual drift as a fraction, any sign) and
  `confidence` were public `f64` fields that the kernels checked on every
  call; they are now private `Decimal`s read through `drift_rate()` and
  `confidence()`, and `PriceTrend::new(drift_rate, confidence)` returns
  `Result<PriceTrend, ProbabilityError>`, rejecting a confidence outside
  `[0, 1]` with `ProbabilityCalculationErrorKind::TrendError` at
  construction instead of at evaluation. Migrate
  `PriceTrend { drift_rate: 0.1, confidence: 0.95 }` to
  `PriceTrend::new(dec!(0.1), dec!(0.95))?`. The kernels convert each field
  to its nearest `f64` with the new `decimal_to_f64_correctly_rounded` and
  still multiply in `f64`, so `calculate_single_point_probability`,
  `calculate_price_probability`, `ProfitLossRange::calculate_probability`
  and `ProbabilityAnalysis::expected_value` return the same values as before
  for a trend written with the digits of the former `f64` literals, at any
  number of decimal places. A caller holding a computed `f64` keeps identical
  results by converting it with `Decimal::from_f64_retain`, not
  `Decimal::from_f64` or `f64_to_decimal`, which round to fewer digits
  (`0.1 + 0.2` becomes `0.3`). Two inputs are no longer expressible: a
  confidence outside `[0, 1]`, which `ProbabilityAnalysis::expected_value`
  used to answer through its zero-volatility early return and which is now
  refused at construction, and a drift outside the `Decimal` range (about
  ±7.9e28, finite, at most 28 decimal places). `PriceTrend` also derives
  `PartialEq` and `Eq`. The four float-boundary allowlist entries go.

- **Simulation is its own crate, `optionstratlib-simulation`** (#536).
  `simulation` (random walks, stochastic processes, steps, the walk driver,
  `RandomWalk`, `Simulator`, the Ornstein-Uhlenbeck process, exit policies
  and the generic `PathEvaluator` / `PathOutcome` / `PathStatistics`) and
  `SimulationError` (with `SimulationResult` and the `error::simulation`
  module) move to `crates/optionstratlib-simulation`, which depends on core
  and pricing and on no market, analytics, strategy, backtesting, plotting,
  I/O or terminal-table crate. Formulas, seeds, results and serialized forms
  are unchanged. What does change:
  - The facade `simulation` feature becomes
    `["dep:optionstratlib-simulation", "pricing"]` and re-exports the module
    and the errors from the crate; `market`, `analytics`, `strategies` or
    `pricing` alone resolve no simulation crate. Backtesting, visualization,
    the unified `error::Error` and the `synthetic` generators stay in the
    facade with their current gates.
  - `impl From<SimulationError> for ChainError` is removed: both types now
    belong to other crates, so the facade cannot implement it. The
    `synthetic` generators still return a simulation failure as
    `ChainError::Generator` whose source downcasts to `SimulationError`; a
    caller that converted by hand writes `ChainError::generator(err)`. The
    conversion comes back in `optionstratlib-market` behind its `synthetic`
    feature when the generators move there (#537).
  - `optionstratlib-simulation` derives `utoipa::ToSchema` (`WalkType`,
    `ExitPolicy`) only under its `schema` feature; the facade enables it.
    Foundational types are imported through `optionstratlib-core`
    (ADR-0001 D8).
  - Unit tests move with their files. `tests/unit/pricing/unified_pricing_test.rs`
    moves to `crates/optionstratlib-simulation/tests`. The volatility
    panic-freedom properties move to
    `crates/optionstratlib-pricing/tests/volatility_panic_freedom_test.rs` and
    the Ornstein-Uhlenbeck one to
    `crates/optionstratlib-simulation/tests/ou_panic_freedom_test.rs`, so
    each runs without the facade. The deterministic `RampWalker` stays with
    the walk driver tests and the facade's generator tests keep their own
    copy until #537.
  - `check-graph` forbids in simulation the packages of ADR-0002's
    simulation-only absent list (`prettytable-rs` included) except
    `utoipa`, which only the `schema` feature reaches, plus the plotting and
    presentation crates, with crate graph,
    forbidden-package and error-layer self-tests; `check-components`,
    `make test`, `make doc`, CI (a simulation-only facade build), the
    public-API snapshots and the float gate cover the crate. The `synthetic`
    feature-tree fixture now records the simulation crate as a real edge.

- **`test_strategy_traits!` is no longer public API; the strategy
  integration tests move to the crates they test** (#534).
  - The macro was `#[macro_export]`ed from `optionstratlib-strategies` and
    re-exported at the facade root (`optionstratlib::test_strategy_traits!`),
    although it only generates a `#[cfg(test)]` module of
    `static_assertions` checks. It is now a crate-private, test-only macro
    of `optionstratlib-strategies` (`#[cfg(test)] mod macros`), and the
    `strategies::macros` module and both public paths are removed. Every
    concrete strategy still runs the same trait conformance test under the
    same name. A downstream crate that used it asserts the traits itself,
    e.g. `static_assertions::assert_impl_all!(MyStrategy: Strategies,
    Validable, Optimizable, ...)`.
  - The strategy suites of the facade's `tests/unit/strategies` (custom,
    single-leg, protective put, no-lower-break-even, `delta`, `simple`,
    `optimal`, `optimal_center`) move to
    `crates/optionstratlib-strategies/tests/integration`, and the strategy
    properties of `tests/property/strategies_panic_freedom_test.rs` to
    `crates/optionstratlib-strategies/tests/panic_freedom_test.rs`; they
    import the component paths and run without the facade, and the
    optimisation suites read their chain fixtures from the workspace root.
    The probability kernel, SPAN margin and P&L primitive properties of that
    file move to `crates/optionstratlib-analytics/tests/panic_freedom_test.rs`.
    `proptest`, already a workspace dependency, becomes a dev-dependency of
    both crates.
  - The strategy Greeks against per-leg pricing Greeks regression
    (`greeks_side_sign_test`, #428) moves to the new `osl-workspace-tests`
    member under `tests/workspace` (ADR-0004 section 8): unpublished, sources
    under `src/`, with core, pricing and strategies declared as direct
    dev-dependencies and no facade. `make test-workspace-integration` runs
    it; `make test` and the Components workflow call it.
  - What needs visualization stays in the facade: the strategy `Graph` test
    (now `tests/unit/visualization/strategy_graph_test.rs`) and the
    price-range walk property, which draws the payoff chart. No regression
    data, tolerance, case count or test name changes.

- **`Strategy::max_profit` and `Strategy::max_loss` are `Option<Positive>`**
  (#661), not `Option<f64>`: they are monetary amounts, and the public
  boundary carries them as validated non-negative values like the typed
  accessors `get_max_profit` / `get_max_loss` already did. Build them with
  `Positive::new(x)` / `Positive::new_decimal(d)` (both return
  `Result<Positive, PositiveError>`) or `pos_or_panic!(x)`. `Display` still
  rounds them to cents, half to even, as the `f64` fields printed
  (`Max Profit: $11.00` for 10.999); `Debug` prints them like the other
  `Positive` fields (`Some(8)` where it printed `Some(8.0)`). The two
  float-boundary allowlist entries go.

- **Strategies are their own crate, `optionstratlib-strategies`** (#531).
  `strategies` and `StrategyError` (with the `error::strategies` module and
  its `From` impls, `From<StrategyError> for ProbabilityError` included)
  move to `crates/optionstratlib-strategies`, which depends on core, pricing,
  market and analytics and on no math, simulation, backtesting or plotting
  crate. Results and serialized forms are unchanged. What does change:
  - The facade reaches them through a new `strategies` feature (implies
    `analytics`, in `default`; `plotly` implies it). `strategies` alone
    builds the module, `StrategyError`, the `test_strategy_traits!` macro
    and the strategy `prelude` items without simulation. The code still in
    the facade (backtesting, visualization, the unified `error::Error`) now
    needs `strategies` and `simulation`, and the facade test suites and
    benches declare `required-features = ["strategies", "simulation"]`.
  - `strategies::FindOptimalSide` and `strategies::utils::FindOptimalSide`
    are removed: the enum is market-owned and keeps one public path,
    `chains::utils::FindOptimalSide` (`optionstratlib_market::chains::utils`
    in the component crate). The `prelude` still exports it, now from that
    path and under `market`. No other alias was left in the strategies
    layer after #530 and #658.
  - `optionstratlib-strategies` derives `utoipa::ToSchema` only under its
    `schema` feature; the facade enables it. The facade drops its
    `itertools` dependency, whose only user was the strategy combination
    search.
  - The strategy unit tests move with their files; the PMCC optimiser tests
    that `io` gated in the facade always run there, reading the chain
    fixture from the workspace root (market's `io` is a dev-dependency).
    The integration suites under `tests/unit/strategies` and
    `tests/property` stay in the facade until M4-06 (#534).
    `check-graph` forbids the analytics package set in strategies, and
    `check-components`, `make test`, `make doc`, the public-API snapshots
    and the float gate cover it.

- **`Strategable` no longer requires `Graph`, and `Simulator` / `RandomWalk`
  no longer implement `BasicAble`** (#658). These were the last two reverse
  edges of the strategies layer (`strategies -> visualization`,
  `strategies -> simulation`); with them gone the `DEFERRED` table of the
  boundary checker is empty and `make check-graph` tolerates no edge, which
  lets strategies become a crate that depends on neither (M4-03, #531).
  - `Graph` is dropped from the `Strategable` supertraits. Every concrete
    strategy still implements `Graph` in `visualization::strategies`, and
    charts render as before. Generic code that charts through a
    `Strategable` bound states it: `S: Strategable + Graph`. A
    `Box<dyn Strategable>` (what `StrategyRequest::get_strategy` returns)
    can no longer be charted directly. Build the concrete strategy and chart
    that: match on `request.strategy_type` and call the type's
    `StrategyConstructor::get_strategy(&request.positions)`, e.g.
    `IronCondor::get_strategy(&request.positions)?.write_html(path)`. To
    keep a trait object, define `trait Chartable: Strategable + Graph {}`
    with a blanket impl for `T: Strategable + Graph` and box into
    `Box<dyn Chartable>`.
  - The `test_strategy_traits!` macro no longer asserts `Graph`; the
    visualization layer's tests assert it for every strategy.
  - `impl BasicAble for Simulator<X, Y>` and `impl BasicAble for
    RandomWalk<X, Y>` are removed. They only forwarded to the inherent
    `get_title`, which stays: call `simulator.get_title()` /
    `walk.get_title()` (it returns `&str`; add `.to_string()` where the
    trait's `String` was used). The trait's other methods leave with them;
    on these types they only reached the defaults (empty collections, an
    `OperationNotSupported` error from the setters, or the `one_option`
    panic), so nothing usable is lost.
  - The empty `strategies::graph` module, kept only so that path stayed
    valid after the `Graph` impls moved to `visualization::strategies`
    (#505), is removed; the impls and `impl_graph_for_payoff_strategy!` are
    unchanged.

- **The strategies module no longer re-exports analytics, P&L or Greeks
  items** (#530). Each item keeps one public path, its owning layer's; the
  types, functions and results are unchanged. Removed paths and their
  replacements:
  - `strategies::probabilities::{PriceTrend, VolatilityAdjustment,
    calculate_price_probability, calculate_single_point_probability}`:
    use `analytics::{PriceTrend, VolatilityAdjustment,
    calculate_price_probability, calculate_single_point_probability}`.
  - `strategies::probabilities::ProfitRangeProbability`: use
    `analytics::ProfitRangeProbability`.
  - `strategies::DeltaAdjustment` and
    `strategies::delta_neutral::DeltaAdjustment`: use `pnl::DeltaAdjustment`
    (its `SameSize` payload is `pnl::DeltaAdjustmentSameSize`, as before).
  - `strategies::DELTA_THRESHOLD` and
    `strategies::delta_neutral::DELTA_THRESHOLD`: use
    `greeks::DELTA_THRESHOLD` (also in the `prelude`, unchanged).
  `strategies::probabilities` keeps `ProbabilityAnalysis` and
  `StrategyProbabilityAnalysis`, and `DeltaNeutrality` still returns
  `pnl::DeltaAdjustment`, so strategy adjustment P&L stays on the strategy
  layer. The audit of `optionstratlib-analytics` found no public signature,
  trait bound or error variant naming a strategy type; `check-graph` now
  self-tests that an analytics dependency on `optionstratlib-strategies`,
  `-backtest`, `-visualization` or `-simulation`, of any kind, is reported.

- **P&L, risk, metrics and strategy-neutral analytics are their own crate,
  `optionstratlib-analytics`** (#529). `analytics`, `pnl`, `risk`, `metrics`
  and their errors (`ProbabilityError` with the `error::probability` module,
  `ProjectionError`, `TransactionError`) move to
  `crates/optionstratlib-analytics`, which depends on core, math, pricing and
  market and on no strategy, simulation, backtesting or plotting crate.
  Results and serialized forms are unchanged. What does change:
  - The facade reaches them through a new `analytics` feature (implies
    `market`, in `default`; `plotly` implies it). `analytics` alone builds
    the four modules, their errors and their `prelude` items (`PnL`,
    `PnLCalculator`, `BasicCurves`, `BasicSurfaces`, the metric traits,
    `ProbabilityError`, `TransactionError`) without strategies or
    simulation. The code still in the facade (strategies, backtesting,
    visualization, the unified `error::Error`) now needs `analytics` and
    `simulation`, and the facade test suites and benches declare
    `required-features = ["analytics", "simulation"]`.
  - `ProbabilityAnalysis`, `StrategyProbabilityAnalysis` and
    `From<StrategyError> for ProbabilityError` stay with the strategies; the
    eleven `ProbabilityError` tests that convert a `StrategyError` move next
    to that impl. The test-only `flat_volatility_0_2` helper is no longer
    visible to the facade, whose one user spells the same value out.
  - `optionstratlib-analytics` derives `utoipa::ToSchema` only under its
    `schema` feature; the facade enables it. The facade drops its
    `lazy_static` dependency, whose only user was `pnl`.
  - The analytics-only suites (probability and adjustment fixtures, P&L
    traits, `TransactionError`, and the eight chain metric and projection
    suites) move into the crate, with fixture paths anchored at the
    workspace root; the JSON-fixture unit tests that `io` gated in the facade
    always run there (market's `io` is a dev-dependency). `check-graph`
    forbids the minimal-market package set in analytics, and
    `check-components`, `make test`, `make doc`, the public-API snapshots and
    the float gate cover it (`PriceTrend::{drift_rate, confidence}`
    allowlisted as pre-existing dimensionless kernel inputs, until #656
    removed them).

- **The facade routes `math`, `pricing`, `market` and `simulation` through
  features** (#528, ADR-0002 Decision 2). `optionstratlib-math`,
  `optionstratlib-pricing` and `optionstratlib-market` become optional
  dependencies of the facade, enabled by features of the same name
  (`pricing` implies `math`, `market` implies `pricing`); `simulation`
  implies `pricing` only. `synthetic`, `io` and `async` now imply `market`,
  and `synthetic` and `plotly` also imply `simulation`. The default enables
  all of them, so the default surface is unchanged. With
  `default-features = false` the facade is the core layer alone: add
  `features = ["pricing"]` for pricing, Greeks and volatility without market,
  I/O, async or charts, or `["market"]` for chains and series without I/O or
  simulation. Until analytics, pnl, risk, metrics, strategies, backtesting
  and visualization leave the facade (M4 to M6) they, the unified
  `error::Error` and their `prelude` items need both `market` and
  `simulation`; every other `prelude` group follows its own capability. The
  facade test suites and benches declare `required-features = ["market",
  "simulation"]`, `make lint` runs Clippy for each capability on its own, CI
  builds the pricing-only and market-only facades, and the minimal market
  feature-tree fixture is now resolved with `--features market`. Two
  consumer fixtures, `fixtures/consumers/facade-pricing` and
  `facade-market`, use the facade with one capability each through the
  `prelude` and the canonical paths, prove those paths are the component
  items, and pin their graphs (`make check-fixtures`,
  `make check-consumer-facade`, `make test-consumer-facade`, all in CI). A stale
  note claiming the analytics items kept historical `chains` paths is gone.

- **Market file I/O sits behind an `io` feature** (#525, ADR-0003).
  `optionstratlib-market` gains `io` (`csv`, `zip`) and `async` now implies
  it. Behind `io`:
  - `OptionChain::{save_to_csv, load_from_csv, save_to_json, load_from_json}`;
  - the OHLCV reader (`chains::csv`, `OhlcvCandle`, `read_ohlcv_from_zip`);
  - `OhlcvError` and `From<csv::Error> for ChainError`.

  The `*_async` wrappers stay behind `async`. Without `io` the market crate
  resolves no `csv`, `zip` or `tokio`; serde (de)serialization works either
  way. `OhlcvError` itself is always available (only its `From<ZipError>`
  needs `io`), so no public error enum changes shape with a feature
  (ADR-0002 section 4). The facade gains a default-on `io` feature, so its
  default surface is unchanged, and its `async` implies `io`. With
  `default-features = false` the facade's graph shrinks from 165 to 123
  packages, and market alone from 119 to 76. Its unused direct `csv` and `zip`
  dependencies go, and `prettytable-rs` is taken without its CSV feature,
  which nothing used. `make check-graph` pins the market tree for no
  features, `io`, `async` and all features, and `make check-components`
  builds, lints and tests market under `io` and `async` alone. Tests that
  need file I/O run only with `io`; the facade's tests now build and pass
  with no default features, and `make lint` also runs Clippy that way.

- **Option chains and series are their own crate, `optionstratlib-market`**
  (#524). `chains`, `series`, `ChainError` and `OhlcvError` move to
  `crates/optionstratlib-market`, which depends on core, math and pricing and
  on no analytics, strategy, simulation, backtesting or plotting crate. The
  facade re-exports both modules and the errors; the facade `async` feature
  forwards to the market crate's `async` (the `tokio`-backed `*_async`
  readers and writers). Serialized forms and results are unchanged. What does
  change:
  - The chain and series generators driven by a random walk need the
    simulation engine; since #537 they live in `optionstratlib-market`
    behind its `synthetic` feature, at the 0.21 facade paths
    `optionstratlib::chains::generator_optionchain` and
    `optionstratlib::series::generator_optionseries`. The prelude exports
    them as before.
  - `ChainError::Simulation(SimulationError)` becomes
    `ChainError::Generator(Box<dyn Error + Send + Sync>)`: a generator's own
    error, typed, downcastable to `SimulationError`. Market names no
    simulation type without `synthetic`. `ChainError::generator(err)` builds
    it, and `From<SimulationError> for ChainError` is in market under
    `synthetic` (#537).
  - The 14 inherent projection methods analytics added to `OptionChain`
    (`gamma_curve`, `delta_curve`, `vega_curve`, `theta_curve`,
    `vanna_curve`, `veta_curve`, `charm_curve`, `color_curve`,
    `veta_time_surface`, `theta_time_surface`, `charm_time_surface`,
    `color_time_surface`, `vanna_surface`, `vomma_surface`) become the
    analytics trait `OptionChainProjections`; import it to call them. It is
    not in the prelude: there `theta_curve`, `charm_curve` and `color_curve`
    resolve to the `ThetaCurve`, `CharmCurve` and `ColorCurve` metrics, which
    a new test pins to the same points as the projections.
  - The compatibility re-export `chains::{RNDAnalysis, RNDParameters,
    RNDResult}` is removed; use `optionstratlib::analytics::rnd`.
  - `OptionChain` and `OptionSeries` document their 0.22 serialization
    contract, with new round-trip and invalid-key tests. An `OptionSeries`
    round trip can move each expiry key one day earlier today (#643).
  - New market API the facade needs: `OptionChain::set_expiration_date` (a
    `#[doc(hidden)]` test seam), `OptionChainBuildParams::set_expiration_date`,
    `OptionSeriesBuildParams::{chain_params, series, set_series}`, and the
    now public `OptionData::{get_option, valid_call, valid_put}`,
    `OptionChain::filter_option_data` and `chains::UpdateFromOptionData`.
  - `optionstratlib-market` derives `utoipa::ToSchema` only under its
    `schema` feature; the facade enables it.
  - The market-only suites (`panic_freedom`, `quote_invariants` and four
    `tests/unit/chain` files) and the chain tests move into the crate; the
    projection tests move to analytics. `check-components`, the public-API
    snapshots and the forbidden-package check cover market (`tokio` only
    under `async`; `csv` and `zip` are gated by #525).

- **Pricing, Greeks and volatility are their own crate,
  `optionstratlib-pricing`** (#521). `pricing`, `greeks`, `volatility` and
  their errors (`PricingError`, `PricingResult`, `GreeksError`,
  `VolatilityError`) move to `crates/optionstratlib-pricing`, which depends
  only on `optionstratlib-core`, `optionstratlib-math` and general numeric
  crates (`statrs`, `rayon`, `rand`, `rand_distr`, `num-traits`).
  The facade re-exports the three modules and the errors, so
  `optionstratlib::pricing::black_scholes` and the other existing paths
  still resolve; formulas, tolerances and results are unchanged. What does
  change:
  - `pricing::{Payoff, PayoffInfo}`, kept only so a 0.21 path resolved, are
    removed; the payoff contracts are core's
    (`optionstratlib::model::payoff::{Payoff, PayoffInfo}`), and the prelude
    exports them from there. `pricing::Profit` stays.
  - `optionstratlib-pricing` derives `utoipa::ToSchema` only under its
    `schema` feature; the facade enables it.
  - `optionstratlib_core::constants` re-exports `DAYS_IN_A_YEAR`, which
    pricing reaches through core (ADR-0001 D8).
  - The property and regression suites that use only pricing
    (`greeks_bounds`, `pricing_panic_freedom`, `put_call_parity` with its
    regression file, `identities`, `garch_regression`) move into the crate,
    unchanged; `make check-components`, the public-API snapshots and the
    forbidden-package check now cover pricing.

- **Math errors no longer carry a rendering variant** (#517).
  `CurveError::RenderError` and `SurfaceError::RenderError` are removed:
  nothing in the library constructed them, and rendering failures are the
  visualization layer's `GraphError` (`GraphError::Render`). Code that
  matched on them can drop the arm. The math crate docs now name the owner of
  every capability that takes a math type but lives in a higher layer
  (`BasicCurves`/`BasicSurfaces` in analytics, `Graph`/`Plottable` in
  visualization), and `make check-graph` fails when `optionstratlib-core` or
  `optionstratlib-math` resolves a forbidden package (`plotly`, `tokio`,
  `reqwest`, `csv`, `zip`, `tracing-subscriber`, …), with default features or
  with all of them. Core's list is ADR-0002's core-only fixture row; math has
  no row there, so its list is the pricing-only row plus the "no
  visualization, no I/O" rule.

- **Curves, surfaces and geometry are their own crate, `optionstratlib-math`**
  (#516). `curves`, `surfaces`, `geometrics` and their errors (`CurveError`,
  `CurvesResult`, `SurfaceError`, `InterpolationError`, `MetricsError`) move
  to `crates/optionstratlib-math`, which depends only on
  `optionstratlib-core` and general numeric crates (`rayon`, `statrs`,
  `itertools`, `num-traits`, `rand`). The facade re-exports the three modules
  and the errors, so `optionstratlib::curves::Curve` and the other existing
  paths still resolve; algorithms, tolerances and serialized forms are
  unchanged. What does change:
  - The compatibility re-exports in the math modules are removed:
    `curves::BasicCurves` and `surfaces::BasicSurfaces` (use
    `optionstratlib::analytics::{BasicCurves, BasicSurfaces}`), and
    `geometrics::{PlotBuilder, Plottable}` (use
    `optionstratlib::visualization::{PlotBuilder, Plottable}`). The prelude
    exports the same names from their owners.
  - The empty `curves::visualization` module, kept only so a 0.21 path
    resolved, is removed.
  - `optionstratlib-math` derives `utoipa::ToSchema` only under its `schema`
    feature; the facade enables it.
  - `optionstratlib_core::model` also re-exports `positive::is_positive`, which
    the math crate reaches through core (ADR-0001 D8).

- **The core domain model is its own crate, `optionstratlib-core`** (#514).
  `model`, `utils`, `constants` and the core errors (`DecimalError`,
  `OptionsError`, `PositionError`, `TradeError`, `OperationErrorKind`) move
  out of the facade into `crates/optionstratlib-core`, which depends on no
  other OptionStratLib crate. `Options`, `Position`, `Leg` and `Trade` keep a
  single definition, there. The facade re-exports the modules and the
  `nz!`, `f2d!`, `f2du!`, `d2f!`, `d2fu!` and `assert_decimal_eq!` macros,
  so `optionstratlib::model::Options`, `optionstratlib::error::DecimalError`
  and the other existing paths still resolve; serialized forms are
  unchanged. What does change:
  - Fully qualified type names (`std::any::type_name`, rustdoc) now read
    `optionstratlib_core::model::...`.
  - The checked `Decimal` helpers the upper layers call (`d_add`, `d_sub`,
    `d_mul`, `d_div`, `d_sum`, `d_sum_iter`, `d_product_iter`, `d_exp`,
    `d_ln`, `d_powd`, `d_sqrt`, `p_sqrt`, `finite_decimal`,
    `DIV_DEFAULT_SCALE`), `model::utils::sub_floor_zero` and the constants
    that were crate-private (`MIN_VOLATILITY`, `MAX_VOLATILITY`,
    `STRIKE_PRICE_LOWER_BOUND_MULTIPLIER`, `STRIKE_PRICE_UPPER_BOUND_MULTIPLIER`,
    `TOLERANCE`, `TRADING_DAYS`, `TRADING_HOURS`, `SECONDS_PER_HOUR`, `MINUTES_PER_HOUR`,
    `MILLISECONDS_PER_SECOND`, `MICROSECONDS_PER_SECOND`, `WEEKS_PER_YEAR`,
    `MONTHS_PER_YEAR`, `QUARTERS_PER_YEAR`) are now public, because the
    facade is a separate crate. `sub_floor_zero` is `#[doc(hidden)]`: it
    floors at zero and is not meant for callers outside OptionStratLib.
  - The pricing solver defaults move out of `constants` into the new public
    `pricing::constants` module, which ADR-0001 D2 assigns them to:
    `DEFAULT_BINOMIAL_STEPS`, `DEFAULT_MC_PATHS`, `DEFAULT_MC_STEPS` and
    `MAX_NEWTON_ITER` are now `optionstratlib::pricing::constants::*`.
  - `optionstratlib-core` derives `utoipa::ToSchema` only under its `schema`
    feature (ADR-0002). The facade enables it, so facade users keep every
    `ToSchema` impl; a direct core dependency gets none unless it asks.
  - Every published crate takes its version, edition, authors, license and
    repository from `[workspace.package]` (lockstep 0.22.0, ADR-0001 D1).

- **`model` no longer depends on any layer above it** (#498). Three moves
  remove the last production edges from the core domain into upper layers:
  - The Greek methods of `LegAble` (`delta`, `gamma`, `theta`, `vega`, `rho`)
    move to the new pricing-owned `greeks::LegGreeks` trait, implemented for
    `Leg`, `SpotPosition`, `FuturePosition` and `PerpetualPosition`. `LegAble`
    keeps what a leg *is*: side, quantity, fees and cost basis. Results are
    unchanged; callers import `LegGreeks`.
  - `Trade::pnl()` is removed; it was `self.into()`. Use `PnL::from(&trade)`,
    the `From` impl the P&L layer already owned.
  - `model::ProfitLossRange`, a re-export kept only so the old path resolved
    after the type moved to analytics (#594), is removed. Use
    `optionstratlib::analytics::ProfitLossRange`.

  Together with #499 this leaves `model` with no production edge to pricing,
  Greeks, P&L, analytics or any error those layers own: `make check-graph`
  tolerates two deferred edges, both in `strategies` (#505). It is the
  prerequisite for extracting `model` as `optionstratlib-core` (M2).

- **`Options` no longer carries pricing methods of its own** (#499). The seven
  inherent wrappers `calculate_price_black_scholes`, `calculate_price_binomial`,
  `calculate_price_binomial_tree`, `calculate_price_montecarlo`,
  `calculate_price_telegraph`, `time_value` and `calculate_implied_volatility`
  only forwarded to `pricing::OptionPricing`, and they made the core domain type
  depend on the pricing layer and on `VolatilityError`. They are removed; the
  same calls work with the trait in scope (`use
  optionstratlib::pricing::OptionPricing;`, or the prelude, which already
  re-exports it). Behavior and results are unchanged. This removes the
  `model -> pricing` and `model -> error/volatility` edges, two of the five
  that keep `model` from being extracted as `optionstratlib-core` (M2).

- **The probability kernels no longer price at a hidden 0.2 volatility**
  (#619). `calculate_single_point_probability` substituted a flat 0.2 when
  `volatility_adj` was `None`, and every strategy method forwarded `None`, so a
  strategy built at any other implied volatility was evaluated as if it were
  0.2. Every example's `probability_of_profit(None, None)` was affected.

  The kernels now take the volatility explicitly:
  `calculate_single_point_probability`, `calculate_price_probability` and
  `ProfitLossRange::calculate_probability` (with its
  `ProfitRangeProbability` trait method) take a `VolatilityAdjustment` instead
  of an `Option`. The analytics layer has no strategy to read a volatility
  from, so it no longer guesses one. `VolatilityAdjustment` is now `Copy`.

  On `ProbabilityAnalysis`, `volatility_adj: None` now means the strategy's
  own volatility, exposed as the new `reference_volatility()`: the implied
  volatility of the leg whose strike is closest to the underlying, with the
  lower strike winning a tie so the answer does not depend on leg order. An
  explicit adjustment still wins. **Results change for every `None` caller**,
  to the volatility the strategy was actually built with.

  One test asserted the opposite of the model and passed only because of the
  hidden default: `test_high_volatility_scenario` set a call butterfly's legs
  to 0.5 implied volatility and expected a higher expected value, but `None`
  priced it at 0.2. Measured with the volatility reaching the model, that
  short-volatility structure's expected value falls from 28.3 at its own 0.18
  to 0 at 0.5; the test now asserts that it falls.

- **utoipa 6 dependency line.** `utoipa` 5.5 -> 6.0, together with the
  crates whose types appear in this crate's public API and `ToSchema`
  derives: `positive` 0.6 -> 0.7, `expiration_date` 0.3 -> 0.4,
  `financial_types` 0.2 -> 0.3 and `option_type` 0.3 -> 0.4. The workspace
  resolves a single utoipa 6.0.0. Consumers must move to the same versions
  in the same step; the minimum supported Rust version follows utoipa 6
  (1.88).

### Fixed

- **`price_option_monte_carlo` discounts at the risk-free rate** (#651).
  The supplied-path Monte Carlo pricer discounted the mean payoff at
  `e^(-(r - q)T)`. The dividend yield belongs to the drift `r - q` of the
  risk-neutral terminal law the caller supplies, never to the discount
  factor (Hull, *Options, Futures and Other Derivatives*, risk-neutral
  valuation), so the price is now `e^(-rT) · mean(payoff)`. Prices change
  for every input with a non-zero dividend yield, by the factor `e^(-qT)`:
  with `r = 5 %, q = 2 %, T = 1` and payoffs averaging `7.5`, the price goes
  from `7.2783` to `7.1342`. Prices with `q = 0` are unchanged. The pinned unit value in
  `pricing::monte_carlo` moves from `4.85222766` to `4.75614712`
  (`5 e^(-0.05)`), and `tests/convergence.rs` gains
  `test_monte_carlo_supplied_paths_discount_at_risk_free_rate`.

- **Jump-diffusion walks jump with probability `λ·dt` per step** (#684). The
  jump trial compared a standard normal draw with `λ·dt`, so it fired with
  probability `Φ(λ·dt)`, about one half for every realistic intensity: a
  one-jump-a-year walk on daily steps jumped on half of its steps instead of
  0.4% of them. The trial now draws a genuine `U(0,1)` from the same per-path
  generator with the new `decimal_uniform_sample_with`, so
  `P(jump) = λ·dt`, `λ·dt >= 1` jumps on every step and `λ = 0` never
  jumps. This changes every jump-diffusion path, seeded or not; a seeded
  path stays reproducible bit for bit for its seed, only the stream differs.
  The draw order per step is unchanged: diffusion normal, jump trial, then
  the jump-size normal only when the trial fires. The jump-diffusion case of
  `deterministic_simulation_test.rs` is re-baselined; no other seeded
  fixture runs a jump-diffusion walk. Seeded tests check the jump count over
  500,000 steps at `λ·dt = 0.004` and over 100,000 steps at `λ·dt = 0.1`
  against the binomial mean within five standard errors, plus the
  `λ·dt = 1` and `λ = 0` limits.

- **The Heston and telegraph walk kernels report `Decimal` overflow instead
  of panicking** (#686). Three expressions in
  `crates/optionstratlib-simulation/src/simulation/traits.rs` still used the
  raw `Decimal` operators, which abort on overflow: Heston's `1 - rho^2`
  ahead of its square root, Heston's correlated draw
  `rho * z1 + sqrt(1 - rho^2) * z`, and the telegraph conversion
  `(|z| + 1) / 2`. They now go through `d_mul`, `d_add`, `d_sub` and
  `d_div`, so an overflow surfaces as a `SimulationError` like every other
  step of those kernels. The results are bit for bit unchanged: each checked
  helper wraps the same `rust_decimal` routine the operator calls, and the
  rounding `d_div` applies at scale 28 is a no-op on a quotient that already
  carries at most 28 decimal places. The draw order is unchanged, and the
  seeded regressions (`deterministic_simulation_test.rs`,
  `simulation_regression_test.rs`) pass unmodified.

- **Three simulation tests that never compiled now run** (#633).
  `tests/unit/simulation/model_and_randomwalk_tests.rs` was declared by no
  `mod.rs` from the commit that added it (341379aa), so its tests never
  compiled. They are revived against
  the current API: the `WalkType` display, `RandomWalk` and `Simulator`
  tests in `crates/optionstratlib-simulation/tests/model_and_randomwalk_test.rs`
  (replayed historical walks instead of an ad-hoc generator, no `unwrap` or
  `expect`), and the random walk's `Graph` data and config in the facade's
  `tests/unit/visualization/simulation_graph_test.rs`. One assertion was
  wrong from the start: it expected `Simulator`'s `Display` to begin with
  `"Simulator Title: SIM"`, a string that only ever existed in that test
  (commit 341379aa); `Display` prints the bare title, and the revived test
  asserts that (and the per-walk lines below it). The old generic test
  walker bounded `X` and `Y` by `Into<Positive>`; `WalkTypeAble` has required
  `TryInto<Positive>` since 18e13c91, so the revived tests use a `Clone`
  walker on `Positive` steps instead. `make check-test-modules` (`scripts/check_test_modules.py`,
  with self-tests, run by `make lint` and so by CI) now fails on any `.rs`
  file below a `tests/` or `benches/` subdirectory that no test root reaches
  through `mod` declarations.

- **`calculate_price_probability` and `expected_value` no longer floor an
  inverted CDF difference to zero** (#570). Both subtracted the probability
  below the lower bound from the one below the upper bound through
  `sub_floor_zero`, attributing any negative result to "a difference of one
  ulp" in the `Decimal -> f64 -> Decimal` round trip. That attribution was
  wrong. With a spot near `Positive::MAX` and a volatility of `1e-28`,
  `(MAX - 4) / MAX` rounds to `0.9999999999999999999999999999` at `Decimal`'s
  twenty-eight places and `Decimal::checked_ln` returns `+9e-28` for it where
  the true value is `-1e-28`, which puts the lower bound three standard
  deviations above the spot. Measured on that input,
  `calculate_price_probability` returned `(1, 0, 0.5)`: a
  `(below, in, above)` triple summing to **1.5**, silently.

  Both sites now follow the rule #569 established in
  `ProfitLossRange::calculate_probability`: equality is a zero-width range
  with probability zero, and an inversion is
  `ProbabilityCalculationErrorKind::InvalidProbability`. The three comments
  describing the subtraction now give the same mechanism instead of two
  contradictory ones. Ordinary inputs cannot reach it, so no caller that was
  getting a right answer starts getting an error: a probe over 21,168
  combinations found no inversions and 9,555 exact equalities, which stay
  `Ok(0)`.

  For `expected_value` the inversion is not reachable through the public API
  today, stopped by two independent limits. The display range is one:
  `get_best_range_to_show` scales the highest point by
  `STRIKE_PRICE_UPPER_BOUND_MULTIPLIER` (1.02), so a spot large enough for a
  consecutive price ratio to round to one overflows there before a single
  probability is computed; measured at `7.9e28`, `get_range_to_show` reports
  `mul_f64: overflow` while `calculate_profit_at` on the same strategy still
  returns `Ok(-24.18)`. The volatility is the other: the inversion needs a
  volatility around `1e-28`, and at that value the z-score leaves the finite
  range, so the kernel reports a conversion failure at every spot from `1e3` to
  `1e28` instead of producing two CDF values to subtract, while `1e-20` and
  above succeed everywhere in that span. The report is a guard there, and the
  tests pin both limits and that the guard does not misfire on the extreme
  inputs that are reachable.

- **The coverage job stopped reporting `Timed out waiting for test response`.**
  `cargo tarpaulin` is installed unpinned in CI. `--timeout` budgets one whole
  test binary's run under the LLVM engine, and it was `0`, which nothing
  enforced until 0.37.4 replaced the blocking wait on the child process with
  polling. From that release a zero budget expires immediately, so every run
  failed a few tests in, on source that had not changed and that 0.37.3 had
  covered successfully minutes earlier. The flag now carries an explicit
  1200s per binary, in the workflow and in both `make coverage` targets.
- **CI and both `make coverage` targets require `cargo-tarpaulin` >= 0.37.5.**
  CI runs the latest stable Rust, and releases before 0.37.5 cannot read Rust
  1.99 coverage data. The install now passes `--locked --version '>=0.37.5'`,
  which also replaces an older binary restored from the `~/.cargo/bin` cache.

### Changed

- **The statistical simulation tests are seeded** (#685). Every test that
  drives a built-in stochastic walk now sets `WalkParams::seed`, the OU and
  normal-sample tests draw from `deterministic_rng`, and the chain
  panic-freedom properties take the seed as a generated input, so a failure
  replays. Assertions and tolerances are unchanged. `seed: None` stays only
  where the test is about the unseeded path, with a comment saying so.

- **The synthetic chain and series generators live in
  `optionstratlib-market`, behind its own `synthetic` feature** (#537,
  ADR-0003). `generator_optionchain` and `generator_optionseries` move from
  the facade's transitional `synthetic` module into market's private
  `chains::generators` and `series::generators`, re-exported as
  `optionstratlib_market::chains::generator_optionchain` and
  `optionstratlib_market::series::generator_optionseries`. For facade users
  nothing breaks against 0.21.3: the facade paths are the 0.21 ones. Market declares
  `synthetic = ["dep:optionstratlib-simulation"]`, the only
  market-to-simulation edge; without it market resolves no simulation
  crate. The facade `synthetic` feature now forwards to it, so the facade
  paths are `optionstratlib::chains::generator_optionchain` and
  `optionstratlib::series::generator_optionseries` (and the prelude, as
  before); the `optionstratlib::synthetic` module is removed. A simulation
  failure still reaches the caller as `ChainError::Generator`, now through
  a `synthetic`-gated `From<SimulationError> for ChainError` in market (the
  enum is the same in every configuration, ADR-0002 section 4); the
  facade-private `GeneratorFailure` wrapper goes. The generic evaluation
  contract (`PathEvaluator`, `PathOutcome`, `PathStatistics`) already lives
  in `optionstratlib-simulation` (M1-07, #536). Checks:
  - `make check-graph` proves every simulation reference in the market
    crate sits behind `synthetic`, with self-tests, and checks market's
    forbidden packages under `synthetic` too;
  - `make check-components` builds, lints and tests market with `synthetic`
    alone;
  - two consumer fixtures, `fixtures/consumers/market-minimal` (no features:
    chains, series and a JSON round trip, no simulation or I/O in the graph)
    and `market-synthetic` (a replayed historical walk over a seed chain, and
    a short history arriving as `ChainError::Generator` with a
    `SimulationError` source), run in CI through
    `make check-consumer-market` and `make test-consumer-market`;
  - the `synthetic` feature-tree fixture now records the
    `optionstratlib-market -> optionstratlib-simulation` edge.

- **Shared pricing and Greek formulas live in one private module**
  (#523). `optionstratlib-pricing` gains a crate-private `kernels` module
  holding the formulas pricing models and Greeks share: `d1`, `d2`, `big_n`,
  the Black-Scholes and Black-76 d-value helpers (the two copies of
  `calculate_d1_d2_and_time` merged), and the `e^(-rT)` discount factor
  that 50 sites computed identically. Pricing no longer imports from
  `greeks`; the only edge between them is the numerical Greeks re-pricing
  through `pricing`, and `make check-graph` now enforces that internal
  direction. Public paths (`greeks::{d1, d2, big_n,
  calculate_d_values_black_76}`) are unchanged re-exports, and every price,
  Greek and error message is bit-identical (checked against a golden dump of
  every pricer and Greek). Formulas that only look alike stay with their
  model, each with a comment saying why. Eleven cross-check tests pin the
  pricer and Greek paths to the same d-values and parities.

- **Breaking: dependencies moved to the utoipa 6 line.** `utoipa` 5.5 -> 6.0,
  `positive` 0.6 -> 0.7, `expiration_date` 0.3 -> 0.4, `option_type`
  0.3 -> 0.4 and `financial_types` 0.2 -> 0.3. All five appear in the public
  API (`Positive`, `ExpirationDate`, `OptionType`, `Side`, `OptionStyle` and
  the `utoipa::ToSchema` impls on public types), so consumers must move to
  the same versions in the same step; crates still on utoipa 5 should stay on
  0.21. The sibling crates were already on utoipa 6 and need Rust 1.88 or
  newer; this crate declares no `rust-version` and builds on stable. No
  source change was required. `uuid` 1.26 -> 1.27; every other dependency was
  already at its latest stable minor.
- **The two forbidden edges M1-10 owned are removed, not deferred** (#507).
  `geometrics -> error/chains` came from `pub type ResultPoint<Point> =
  Result<Point, ChainError>`: the math layer's construction API named a market
  error, and the alias carried the edge to everyone who re-exported it.
  `ConstructionMethod` now carries the error type it reports
  (`ConstructionMethod<Point, Input, Error>`) and `GeometricObject::construct`
  binds it to `Self::Error`, so a parametric generator for a `Curve` reports a
  `CurveError` and one for a `Surface` a `SurfaceError`. `ResultPoint` is
  removed rather than retyped: it was an alias for `Result` and said nothing
  the new signature does not. The construction boundary no longer does
  `map_err(|e| ConstructionError(e.to_string()))`, so the generator's error is
  returned unchanged.
- **`CurveError` and `SurfaceError` gained a `Generator` variant** (#507),
  built through `CurveError::generator` / `SurfaceError::generator`. A
  parametric generator is written by the caller, so its failure belongs to
  whichever layer wrote it and the math layer cannot name that type without
  depending on the layer above. The cause travels as a boxed `source`: still
  reachable through `std::error::Error::source` and downcastable to the
  original type, which the curve and surface tests assert by downcasting back
  to `ChainError`.
- **`LegAble::pnl_at_price` and `Position::pnl_at_expiration` report
  `PositionError`** (#507), as do `unrealized_pnl`, `roe_percentage`,
  `margin_ratio` and `effective_leverage` on the future and perpetual legs.
  They reported `PricingError`, which made core depend on a pricing-owned
  error for failures that are all core-owned: `DecimalError`, `PositiveError`
  and `OptionsError`. `PositionError` gained an `Options` variant for the
  last of those and already carried the other two. Callers in the pricing and
  strategy layers are unaffected in behaviour: `PricingError` and
  `StrategyError` both already convert from `PositionError`.

### Added

- **`decimal_uniform_sample_with`, a `Decimal` uniform draw on `[0, 1)`**
  (#684), in `optionstratlib_core::model::decimal` next to
  `decimal_normal_sample_with`. It draws an integer `k` uniformly from
  `0..10^18` and returns `k / 10^18`, so the range is exactly
  `[0, 1 - 10^-18]`, no `f64` is involved, and `P(sample < p) = p` for any
  threshold with at most 18 decimal places. The simulation kernels use it
  for their Bernoulli trials.

- **The Plotly and static-export gate is verified, not just wired** (#544).
  `make check-graph` now fails when any workspace package other than
  `optionstratlib-visualization` (the facade and the examples included) declares `plotly` or
  `plotly_static`, when a lower-layer crate has a `plotly` or `static_export`
  feature or a feature that names the visualization crate, when the facade
  reaches the visualization crate other than through `visualization`,
  `plotly` and `static_export` or enables `plotly` or `static_export` by
  default, or when the visualization crate declares `plotly` non-optionally,
  with default features or with `static_export_default`, or its two features
  stop being exactly `dep:plotly` and `plotly` plus
  `plotly/static_export_default`; the self-tests cover each case. The facade
  joins the forbidden-package check: its default, `visualization`, `plotly`,
  `async` and `static_export` trees, and all features, may resolve the
  image-export, WebDriver, runtime and HTTP packages only where they ask for
  them. `make check-feature-trees` pins the `plotly` and `static_export`
  surfaces besides the headless ones and asserts, independently of the
  recorded fixtures, that Plotly, the export stack and the visualization
  crate appear only on the surfaces that ask for them (`--self-test`). Four
  consumer fixtures cover the facade surfaces: `facade-visualization`,
  `facade-plotly`, `facade-static-export` and `headless-full` (the facade
  defaults), each with `present` and `absent` package lists, a downstream
  `Graph` consumer and `compile_fail` doctests that the methods a surface
  lacks do not compile; they run under `make check-consumer-facade` and
  `make test-consumer-facade`. `make test-export` runs the PNG and SVG
  export tests (they need a chromedriver matching the installed Chrome) and
  passed on this revision; those tests now also assert the artifact (the
  file exists, is not empty and starts with the PNG signature or contains
  `<svg`). The `examples_metrics` and `examples_surfaces`
  crates no longer declare `plotly` themselves (nothing used it); they get
  Plotly and static export through the facade features.

- **Seeded, reproducible stochastic walks** (#539). With
  `WalkParams::seed = Some(seed)` every built-in stochastic walk (Brownian,
  geometric Brownian, log-returns, mean-reverting, jump-diffusion, GARCH,
  Heston, custom OU-volatility and telegraph), the public kernels
  `garch_walk` / `heston_walk` / `custom_walk` / `telegraph_walk`, and the
  drivers `walk_steps`, `walk_steps_par` and `generator_positive` draw the
  whole path from `deterministic_rng(seed)`: the same parameters and seed
  give the same path bit for bit on every run with a given `rand` /
  `rand_distr` version (`StdRng` and `StandardNormal` may change their
  streams across releases; a dependency bump that does is a reviewed change
  of the pinned fixtures). `Simulator::new` with a seeded
  `WalkParams` seeds walk `i` with the `i`-th `u64` of
  `deterministic_rng(seed)`, so its walks differ from one another, do not
  depend on how many walks follow, and the simulator is reproducible.
  Sequential/parallel contract: `walk_steps_par` draws the path once,
  serially, before fanning out `next_y`, and assembles in step order, so a
  seeded `walk_steps_par` equals `walk_steps` bit for bit for a pure
  `next_y`. Historical walks replay their prices and ignore the seed.
  `WalkParams`' `Display`, which the walk drivers log at debug level, now
  ends with `seed: …`, so a seeded run can be reproduced from its log. Core
  gains `decimal_normal_sample_with(&mut impl Rng)`, the
  standard-normal `Decimal` draw from a caller's generator;
  `decimal_normal_sample()` is now that helper applied to the thread RNG.
  `optionstratlib-simulation` depends on `rand` directly (it was already in
  its graph through core), recorded in the feature-tree fixtures. Fixed-seed
  regressions: `optionstratlib-simulation/tests/deterministic_simulation_test.rs`
  pins every stochastic path and volatility path, simulator terminal
  prices, exit reasons, holding periods, P&L and `PathStatistics` bit for
  bit, and the Monte-Carlo price over seeded paths exactly (its `f64`
  payoff step is a correctly rounded subtraction and multiply);
  `optionstratlib-backtest/tests/simulation_regression_test.rs`
  pins exit reasons and holding periods of `LongCall` / `ShortPut`
  simulations exactly and their Black-Scholes P&L within `1e-9`.

- **Facade consumer fixtures for `simulation` and `backtest`** (#541).
  `fixtures/consumers/facade-simulation` uses the facade with
  `default-features = false, features = ["simulation"]`: replayed walks
  through the prelude, a consumer `PathEvaluator` through
  `optionstratlib::simulation::evaluate_paths` over a rising and a falling
  replay (`PathStatistics`: mean 2, best 8, worst -4, win rate 50), and
  `same_item` checks that the facade paths are the simulation crate's items.
  Its `expect.toml` keeps market, analytics, strategies, backtest and
  visualization out of the graph, so `simulation` stays independent of them.
  `fixtures/consumers/facade-backtest` does the same with
  `features = ["backtest"]`: a long call backtested over replayed paths
  through the prelude (+14 and -6 per walk, the figures of the backtest
  crate's golden regression), with every lower capability `backtest`
  documents present and no plotting, I/O or async package. `make
  check-consumer-facade` and `make test-consumer-facade` now cover six
  single-capability facades. The `simulation` and `backtest` facade
  features themselves came with #536 and #538, and the `market,synthetic`
  routing with #537 (the `facade-market` fixture and the `synthetic`
  feature-tree fixture pin both sides).

- **Simulation-only and full-backtest consumer fixtures** (#540).
  `fixtures/consumers/simulation-only` depends on core and simulation alone:
  it replays historical walks, evaluates them with its own `PathEvaluator`
  through `evaluate_paths`, checks `PathStatistics`, and checks that repeated
  runs are identical; its graph excludes market, strategies, backtest,
  visualization, `prettytable-rs` and `indicatif`.
  `fixtures/consumers/full-backtest` declares core, simulation, strategies
  and backtest explicitly, with no facade, and backtests a long call over a
  rising and a falling replayed path, checking the report against the
  figures the backtest crate's golden regression pins (+14 and -6 per walk).
  Historical walks replay their prices, so both are deterministic. `make
  {test,tree}-consumer-simulation-only` and
  `{test,tree}-consumer-full-backtest` run them with no features and with all
  features, and the Components CI job runs them from a fresh target
  directory. Measured with the M0 method on commit 6ef94913 (rustc 1.99.0,
  Apple M5 Max; `cargo fetch`, then three clean `cargo check` runs with no
  features from an emptied target and build directory): simulation-only 68
  distinct packages (69 entries, `syn` twice), 5.79 / 5.82 / 5.93 s (median 5.82 s); full-backtest 80
  distinct packages (81 entries), 7.64 / 7.49 / 7.55 s (median 7.55 s); the same package
  counts with all features. The machine and toolchain differ from the M0
  baseline (BASELINE.md). Fixture lockfiles are not committed, so the
  counts are recorded, not asserted (ADR-0004 section 3).

- **`decimal_to_f64_correctly_rounded`** in
  `optionstratlib_core::model::decimal` (#656): converts a `Decimal` to the
  nearest `f64` by parsing its exact digits from a stack buffer, with no heap
  allocation. `decimal_to_f64` (through `Decimal::to_f64`) can land one ULP
  away from it at 15 or more decimal places, e.g. `2.999789999999902`; the
  new function returns the `f64` literal written with the same digits.
  Returns `DecimalError::ConversionError` rather than a non-finite value.

- **Facade consumer fixtures for `analytics` and `strategies`** (#535).
  `fixtures/consumers/facade-analytics` uses the facade with
  `default-features = false, features = ["analytics"]`: a short put's P&L
  at expiration through the prelude's `PnLCalculator`, the
  implied-volatility curve of a built chain, SPAN margin through
  `optionstratlib::risk`, and `same_item` checks that the facade paths are
  the analytics crate's items. Its `expect.toml` keeps
  `optionstratlib-strategies` out of the graph, so `analytics` provably never
  enables strategies. `fixtures/consumers/facade-strategies` does the same
  with `features = ["strategies"]`: a bull call spread's break-even, maximum
  loss, cost and fees (the strategies crate's own regression figures) and its
  probability of profit through the prelude. `make check-consumer-facade`
  and `make test-consumer-facade` now cover all four single-capability
  facades, and `make check-fixtures` asserts six fixture graphs. The
  `analytics` and `strategies` facade features themselves came with #529 and
  #531. The crate-level doc examples now compile under each capability on
  its own: each runs only when the capability it uses is on (`pricing`,
  `strategies`, or `strategies` plus `simulation` for the chart), and their
  `main` returns `Box<dyn std::error::Error>` instead of the unified
  `error::Error`, which exists only with `strategies` and `simulation`.
  `make lint` now also runs `cargo test -p optionstratlib` (library and doc
  tests) with no features and with each set in `FACADE_FEATURE_SETS`. The
  analytics and strategies facade prelude groups stay as they are, gated by
  their capability; curating the facade prelude is M7-03 (#551).

- **Analytics consumer fixture without strategies** (#533).
  `fixtures/consumers/analytics-only` is a real crate, excluded from the
  workspace, that depends on core, math, pricing, market and analytics only
  (path plus version), with no strategies crate and no facade. Its test
  computes a short put's P&L at expiration through `PnLCalculator` and its
  SPAN margin (both the floor-bound and the scenario-bound case), checks
  that the range and single-point probability kernels agree on their tails,
  and computes the risk-neutral density (non-negative, summing to one, mean
  near spot), the skew and the implied-volatility curve of a built
  `OptionChain`, all with canonical lower-layer types. `expect.toml` keeps
  strategies, simulation, backtest, visualization, CSV, ZIP, Tokio and Plotly
  out of its graph. `make {check,test,tree}-consumer-analytics-only` and
  `check-consumer-analytics-only-minimal` run it with all and with no
  analytics features, and the Components CI job runs them from a fresh
  target directory. Measured with the M0 method on commit b99f8a6b (rustc
  1.99.0, Apple M5 Max): `cargo fetch`, then three clean
  `cargo check --manifest-path fixtures/consumers/analytics-only/Cargo.toml`
  runs with no features, each from an emptied target directory: 7.15, 6.51
  and 6.32 s (median 6.5 s); 77 distinct packages (78 entries, `syn` twice)
  with no features and with all features. Fixture lockfiles are not
  committed, so the counts can move with registry updates (recorded, not
  asserted, as ADR-0004 section 3 sets).

- **One strategy capability, by measured decision** (#532). The
  `optionstratlib-strategies` docs now list the seven strategy families
  (single leg, vertical spreads, butterflies, condors, straddles and
  strangles, covered and protective, custom) and the shared surface every
  family uses, and record why there are no per-family features in 0.22: the
  crate adds no package to the analytics graph, and its own build (0.93 s
  check, 1.86 s debug, 3.12 s release) bounds what any grouping could save,
  against 5.85 s for a clean check of the layers below. Gating a family
  would also gate `StrategyType` variants (forbidden by ADR-0002 section 4)
  or make `StrategyRequest` fail depending on features. `make
  measure-strategies` (`scripts/measure_strategies.py`) reproduces the
  numbers, and `tests/strategy_families.rs` fails to compile if a new
  strategy is not assigned a family.

- **First consumer fixture: core plus pricing** (#527).
  `fixtures/consumers/pricing-only` is a real crate, excluded from the
  workspace, that depends on `optionstratlib-core` and
  `optionstratlib-pricing` only (path plus version). Its test builds a
  contract, prices it in closed form (Hull's 4.76/0.81), reads its Greeks
  and payoff, and recovers its implied volatility, all with the canonical
  types. `expect.toml` lists what its graph must and must not resolve:
  no facade, market, analytics, strategies, simulation, backtest,
  visualization, CSV, ZIP or Tokio. `make check-fixtures` asserts every
  fixture and prints its package count. `make
  {check,test,tree}-consumer-core-pricing` and
  `check-consumer-core-pricing-minimal` run this one, and the Components CI
  job runs them from a fresh target directory. Measured: 67 packages and a
  5.7 s clean check, against 131 packages and 13.7 s for the 0.21.3 facade
  default (BASELINE.md, different machine).

- **Standalone pricing regression suites** (#526) in `optionstratlib-pricing`,
  run without market, simulation, strategies or the facade:
  - `pricing_identities`: parity for Black-Scholes, Black-76 and
    Garman-Kohlhagen, side symmetry, Greek identities, exotic reductions and
    the zero-time and zero-vol limits over a parameter grid.
  - `analytic_references`: published values from Hull and Haug, with the
    tolerance their printed precision allows.
  - `convergence`: binomial to Black-Scholes, and numerical against
    closed-form Greeks.
  - `implied_volatility`: IV round trips.

  55 tests pass. Ten pre-existing numerical defects they found are filed
  with ready-to-merge tests instead of being merged as ignored tests:
  barrier (#646), fixed-strike lookback (#647), American edge cases (#648),
  gap put (#649), quanto and Kirk spread (#650), Monte Carlo discounting
  (#651) and IV solvers with no solution (#652).

- **`make check-float-boundary` guards the public `f64` surface of the
  component crates** (#522). It reads the `public-api/optionstratlib-*.txt`
  snapshots and fails on any `f64` outside the error types' own items
  (diagnostics carrying the offending input or a non-finite intermediate)
  and the reviewed `public-api/float-boundary-allowlist.txt`,
  which lists the pre-existing exceptions with a reason each. `make
  public-api-check` runs it. The audit of the extracted pricing crate found
  no public `f64` outside its error diagnostics. Filed as follow-ups: core's
  `f64` payoff kernel (#637), the pricing entry points that sample the
  thread RNG (#638), and the exotic pricers that turn a failed numeric step
  into zero (#639).
  `make scan-banned` also rejects `println!`, `eprintln!`, `print!`,
  `eprint!`, `dbg!` and `tracing_subscriber` in production code.

- **The facade documents and pins its core and math exports** (#520). A
  "Workspace Crates" section in the crate docs (and the README) lists which
  facade paths each component crate backs: `model`, `utils`, `constants`,
  the core errors, the root types and the decimal macros from
  `optionstratlib-core`; `curves`, `surfaces`, `geometrics` and the math
  errors from `optionstratlib-math`. Every one is an explicit module or item
  re-export, never a glob over a component. The prelude now takes `Positive`
  and the `positive` macros through `optionstratlib_core` (ADR-0001 D8); the
  types are unchanged. `tests/unit/model/component_paths_test.rs` passes
  values between facade, prelude and component paths with no conversion.

- **`make check-components` and a `Components` CI job verify each extracted
  crate on its own** (#519): `optionstratlib-core` and `optionstratlib-math`
  are tested with default, no and all features, linted with Clippy, built as
  docs with broken links and missing docs denied, and packaged, with the
  archive checked for `Cargo.toml`, `README.md`, `LICENSE` and `src/lib.rs`.
  The facade is not in those builds. The property suites that only use core
  or math (`model_panic_freedom`, `curves_panic_freedom`, `point_contract`)
  move into those crates with their cases and tolerances unchanged; the
  `PnLCalculator` half of the model suite stays in the facade. Two facade
  unit-test files move too, one of which (`decimal_ops_test`) had never been
  declared in a `mod.rs` and now runs for the first time.

- **`optionstratlib-core` and `optionstratlib-math` have no prelude, by
  decision** (#518). Each crate's docs record the measurement behind it: 168
  of 178 example files import through the facade prelude; explicit imports
  name 8 distinct core items in the examples, and 31 core and 20 math items
  in tests and benches, with no small common core. The docs also show the
  canonical imports, which compile fixtures in both crates
  (`tests/common_imports.rs`) now pin. The module roots are the
  curated entry points; the facade prelude is M7-03's.

- **`optionstratlib-core` re-exports every foundational type** (#515).
  `optionstratlib_core::model` now also re-exports `Positive` and
  `PositiveError`, and the crate root re-exports the `pos_or_panic!`,
  `spos!` and `assert_pos_relative_eq!` macros, next to the
  `expiration_date`, `financial_types` and `option_type` re-exports it already
  had. They are the standalone crates' own types, not wrappers: the core
  crate docs carry a table of each type's defining crate and every path that
  reaches it, and compile-time fixtures pass values between the defining
  crate, core and facade paths with no conversion. `make check-graph` fails
  when two workspace packages ask for different versions of a foundational
  crate, when the resolved graph holds two versions of one, or when a
  component other than core (and, for now, the facade) depends on one
  directly.

- **A `## Module Boundaries` section in the crate docs** (#507) with the layer
  DAG, the per-file partition of `src/error` and `src/utils`, the single
  feature-gated edge, and the two commands that enforce them. `AGENTS.md` and
  `CLAUDE.md` carry the same graph, and the retired accepted-breaks section is
  gone from both.
- **A self-test refusing a deferred edge with no owning issue** (#507), so the
  tolerated list cannot grow an entry that nobody has to remove.

### Changed

- **The market-to-simulation edge is now proven to be behind `synthetic`, not
  just declared to be** (#512). `ChainError::Simulation` and its
  `From<SimulationError>` conversion were ungated, so a consumer building with
  `default-features = false` still named a simulation type through the market
  error enum. Both are now `#[cfg(feature = "synthetic")]`, which is the last
  simulation reference in the minimal market surface. `make check-graph` gained
  a gate check: every production `crate::simulation` reference under
  `src/chains`, `src/series` and in the market-owned `src/error/chains.rs` must
  sit under the feature attribute, carried either by the item or by the `mod`
  declaration that brings the file in. Attributes are attached to the item that
  follows them and each gated item's whole extent is marked, so a gated sibling
  cannot lend its gate to the next declaration or enum variant, and a reference
  deep inside a gated function or `impl` is still recognised as gated. Listing a
  file in `SYNTHETIC_FILES` no longer launders an ungated edge, and eleven
  self-test cases cover the rule.
- **`make check-feature-trees` pins the dependency graph of both market
  surfaces** (#512), from `tests/fixtures/feature-trees/{minimal,synthetic}.txt`.
  Each fixture holds the whole graph as a sorted `parent -> child` edge list
  plus the features enabled on each package, so a dependency added, removed,
  re-parented or promoted from transitive to direct shows up, as does a feature
  the flag turns on for a package both surfaces already share. The difference
  between the two is derived and printed; today it is `optionstratlib
  [synthetic]` against `optionstratlib []`, the feature itself and no crate. The graph
  is resolved with `cargo tree --target all` so it is the same on every host,
  and nodes carry no version because `Cargo.lock` is not committed; #616 tracks
  that decision. The check runs in the lint workflow, and
  `make feature-trees-update` records an intended change.

### Removed

- **`chains::generator_positive` is gone** (#512). It was a deprecated
  re-export of `simulation::generator_positive`, a generic walk generator that
  never depended on option chains, and it was the market layer's only
  re-export of a simulation function it does not own. Use
  `optionstratlib::simulation::generator_positive`, which the prelude already
  exports.

### Removed

- **`setup_logger` and `setup_logger_with_level` are gone, and the library no
  longer depends on `tracing-subscriber`** (#506, #545). Installing a global
  subscriber is an application decision, not a library one
  (`rules/global_rules.md`, "Logging & Observability"), and a library that
  installs one silently wins the race against the binary that wanted its own.
  `optionstratlib::utils::logger`, the two functions and the
  `prelude::setup_logger` re-export are removed rather than deprecated, and
  `tracing-subscriber` is dropped from the crate's dependencies. Call
  `tracing_subscriber::fmt().with_max_level(..).init()` from your binary; the
  example binaries take theirs from the unpublished `osl-example-support`
  workspace member. `tracing` itself and every instrumented span are
  unchanged.

### Changed

- **`src/utils` carries a per-file owner instead of a blanket `core`** (#506).
  It was the one catch-all helper module left, so the ownership was implicit
  and nothing stopped a new helper from landing there. `others.rs` is split
  into `numeric.rs` (`approx_equal`, `calculate_log_returns`) and `rng.rs`
  (`deterministic_rng`, `get_random_element`, `random_decimal`,
  `DETERMINISTIC_RNG_DEFAULT_SEED`), each file names one concern, and
  `scripts/check_module_boundaries.py` gained `UTILS_FILE_LAYER`, which
  resolves `src/utils/<file>.rs` to its owning layer the way `ERROR_FILE_LAYER`
  already did for `src/error`. A new file in `src/utils` that is not listed
  there fails the graph check, and a helper that grows an edge above its
  owner's layer fails it too. Three self-test cases cover the new resolution.
  Public paths change: `utils::others::approx_equal` is
  `utils::numeric::approx_equal`, `utils::others::calculate_log_returns` is
  `utils::numeric::calculate_log_returns` (also re-exported from the prelude
  and from `utils`), and the `rng` helpers keep their `utils::` re-exports.
- **`prepare_file_path` moved to `visualization`** (#506). Preparing a path on
  disk exists to write a rendered chart and `visualization::plotly` is its only
  production caller, so it sits with its owner as
  `optionstratlib::visualization::prepare_file_path` rather than in the shared
  helper module. Behaviour is unchanged.

### Fixed

- **The `d_sqrt` cycle test no longer measures wall-clock time** (#604). It
  asserted that a thousand calls on the oscillating input finish in under
  50 ms, which is a statement about an instrumented build: under
  `cargo tarpaulin` every call is slower and the `code_coverage_report` job
  failed while every other job passed. The property it exists for is a
  count, so the Newton loop now reports its iterations through the
  crate-private `sqrt_with_iterations`, which `d_sqrt` wraps, and the test
  asserts the period-2 cycle resolves in fewer than ten iterations against a
  converging control. `d_sqrt`'s signature, results and errors are
  unchanged.
### Migration notes for 0.22

- **Compatibility with 0.21.3 is not a requirement of 0.22.0** (#606). The
  machinery built to preserve it is retired: the cumulative comparison
  against the published crate, the `v0.21.3` reference, the register of
  individually authorised breaks and its approval flow. What replaces it is
  a report: `scripts/report_api_changes.py` lists, per feature surface, the
  public items a pull request removes or reshapes incompatibly against its
  own base, so a reviewer judges the change instead of authorising it item by
  item. Additions are not `cargo-semver-checks` findings and reach the
  reviewer through the public-api snapshot, which `make public-api-check`
  refuses until it is regenerated.
  The job still fails on a tool, build or parser problem, because an
  unreadable report must never be read as "no change". The `synthetic`
  requirement previously recorded as AB-01 is superseded by this policy and
  stays documented as the migration note below. History, tags and past
  releases are untouched.

- **The manifest declares 0.22.0** (#602). For a 0.x crate the breaking bump
  is the minor digit, so this is what authorises the incompatible changes of
  the multi-crate migration. Nothing is tagged, released or published by this
  change; that is the rest of Milestone 8, once every issue of the migration
  is closed.
- **The `synthetic` feature is required for the generator functions**
  (#512, landed as #578; surfaced by the accepted-breaks gate of #592).
  `chains::generator_positive`, `chains::generator_optionchain`,
  `series::generator_optionseries` and their prelude paths existed in 0.21.3
  with any feature set; they now exist only when `synthetic` is enabled. The
  feature is on by default, so a build that takes the default features is
  unaffected; a build with `default-features = false` must list `synthetic`
  explicitly.

### Added

- **The boundary checker resolves error types to their owning file** (#590,
  follow-up of #507). A reference spelled `crate::error::Name` used to be
  read as a reference to the facade-level `error` module, so a lower layer
  naming a higher layer's error type in a signature was never reported.
  `scripts/check_module_boundaries.py` now maps every public type defined
  under `src/error/` to its file and resolves names through each file's `use`
  bindings and the crate's re-export graph: bare, grouped, nested, `as`
  aliases, file-qualified paths, `self::`/`super::` paths (including a
  binding that is private in the parent module), the uniform bare-relative
  form used by the crate's `mod.rs` files, module aliases
  (`use crate::error as err`), `pub type` aliases over an error type (the
  edge reaches every consumer of the alias, and the right-hand side is
  resolved through the defining file's bindings, so an alias over a foreign
  `Error` creates none), globs,
  and paths in expression position. Names bound from outside the crate
  (`use std::io::Error`) raise no edge, a glob name shadowed by a local
  definition or another import is ignored, and a type defined by two error
  files is ambiguous, so every candidate is taken rather than the first one
  silently. A `// facade-compat` `pub use` still raises no edge itself, but
  the type it re-exports is resolved for every consumer of that path, so the
  marker cannot launder a higher layer's type into a lower one. Twenty
  further reverse edges become visible, none of them new: the
  `PricingError`/`GreeksError` channels of `LegAble` and `Position`,
  `ChainError` in `model::utils` and the `ResultPoint` alias, `MetricsError`
  in curves, surfaces and geometrics, `SimulationError` in
  `volatility::utils`, `OhlcvError` in `utils::csv`, the unified `Error` in
  `utils::others`, and the enum variants inside `src/error/` that hold a
  higher layer's error. All are recorded in `DEFERRED`, file by file, with
  the issue that owns each resolution (31 entries), so `make check-graph`
  still passes while the debt is printed on every run. New `--inventory`
  mode prints the deferred edges, the `facade-compat` lines (with source
  layer and compat target) and the ambiguous error names as tables. The
  self-test grows to 40 cases, several of which assert which file carries
  the edge, and the module docstring states what the resolver does not see.
- **Accepted-breaks register and three-baseline semver gate** (#592,
  multi-crate roadmap M1-17). The 0.22 migration needs incompatible changes
  while the manifests stay at 0.21.3 and nothing is published, so
  `cargo semver-checks` reports every removal against the published crate.
  `public-api/accepted-breaks.toml` is now the only place where such a report
  may be expected instead of failing, and `scripts/check_accepted_breaks.py`
  compares the two. Three comparisons run per feature surface: **C1** against
  the published 0.21.3 (what a consumer of the release sees), **C2** against
  the newest commit carrying an `Accepted-Breaks:` trailer among the
  ancestors of the state *before* the change under test (so an integrating
  push is never compared with itself), and **C3** against the pull request's
  base, taken from the first parent of the merge commit CI checks out, which
  is the only comparison that sees the removal of an item added after 0.21.3.
  All six surfaces (`none`, `default`, `plotly`, `static_export`, `async`,
  `all`) run on every pull request, because an item can leave the default
  surface and stay behind a feature, which `--all-features` cannot see. The
  parser is fail-closed: a non-zero, non-100 exit, a missing summary, a block
  count that disagrees with it, a truncation marker, an unknown lint or an
  unpinned tool version, or an item line with no item, is an error, never
  "zero breaks"; sixteen fixtures under `tests/fixtures/semver-reports/`
  (six real reports, eight derived defects, one duplicate-item case and the
  reproducible four-step sequence script with its recorded output) are parsed
  by `--self-test`, which `make check-breaks` runs. The register's structure
  is enforced in code (an approved entry needs an approval reference pointing
  at the register issue, a decision and a migration note, and may not store a
  commit SHA), `--verify-approvals` reads each approval comment and checks
  its author is the register's owner, and a pull request that moves one of
  its own entries to `approved` is refused: authorisation belongs to a
  separate change. An authorised break is expected by C1 for ever and by C2
  or C3 only while it is inside that comparison's delta, so an ordinary pull
  request stays green without a label once a break has landed; the sequence
  fixture runs the register comparison itself and records its 24 verdicts. The register ships with no approved entry, the
  workflow is informational and the existing `semver` job is unchanged. Its
  first run already found one real incompatibility on `main`: the `synthetic`
  gate (#512) removes `chains::generator_positive`,
  `chains::generator_optionchain`, `series::generator_optionseries` and their
  prelude paths from every surface that does not enable the feature, which a
  consumer building with `default-features = false` had in 0.21.3. The
  finding is recorded as a `proposed` register entry (AB-01) and waits for
  the owner's decision on #592; until then the job reports without blocking.
- **`make check-graph` enforces the module boundaries** (#507, multi-crate
  roadmap M1-10). `scripts/check_module_boundaries.py` scans production code
  for `crate::<module>` references (including multi-line `use crate::{...}`
  groups, qualified groups such as `use crate::error::{graph::GraphError}`
  and `crate::error::<file>` paths), maps every module and every
  error file to its target crate (ADR-0001 D2 and D6) and fails on any edge
  against the approved graph. `pub use` lines marked `// facade-compat:
  <layer>` (the compatibility re-exports that become facade code at
  extraction) are exempt, and only those: a marked plain import is scanned
  like any other; the eleven known reverse edges whose removal is a breaking change
  are listed in the script per file with the issue that removes them (the
  same module pair in any other file fails) and the run prints how many
  marked lines each layer carries; a self-test proves the scanner catches what it
  must. The `lint` workflow runs it on every push. The last two imports
  through `crate::prelude` inside the library (`pricing::telegraph`,
  `strategies::delta_neutral`) now use canonical paths.
- **`optionstratlib::analytics` module** (#513, multi-crate roadmap M1-16):
  the strategy-neutral home of the price-probability kernels.
  `VolatilityAdjustment`, `PriceTrend`, `calculate_single_point_probability`
  and `calculate_price_probability` moved from `strategies::probabilities`
  to `analytics::probability`; `strategies::probabilities::{...}` re-exports
  them, so no import changes. `ProbabilityAnalysis` and
  `StrategyProbabilityAnalysis` stay strategy-owned and consume the kernels
  downward. No formula or tolerance changed.
- **`synthetic` feature, on by default** (#512, multi-crate roadmap M1-15).
  It gates the simulation-backed generators `chains::generator_optionchain`
  and `series::generator_optionseries` (and the deprecated
  `chains::generator_positive` alias), the only place where the market layer
  depends on the simulation engine. With the default features nothing
  changes; `default-features = false` now gives a market surface with no
  simulation-backed generation. `make test` additionally builds that surface
  (`cargo build --no-default-features`).
- **`OptionChain` metric and RND implementations moved to the analytics
  layer** (#509). The twenty-eight `impl <MetricTrait> for OptionChain` blocks and
  `impl RNDAnalysis for OptionChain` lived in `chains::chain`, so the market
  module imported `metrics` and owned the risk-neutral-density surface: a
  Market-to-Analytics edge in the direction ADR-0001 forbids. The metric impls
  now sit in the private `metrics::chain` module next to their traits, and
  `RNDAnalysis`, `RNDParameters`, `RNDResult` and `RNDStatistics` together
  with the chain impl live in the new top-level `analytics::rnd` module.
  `chains` no longer references `metrics`; it reads nothing from `analytics`
  except the compatibility re-export marked `// facade-compat: analytics`.
  The public surface is unchanged: `chains::{RNDAnalysis, RNDParameters,
  RNDResult}` keep resolving, `RNDStatistics` gains a nameable path under
  `analytics`, and every metric is still reached by importing its trait from
  `metrics`. No formula changed; the moved tests assert the same values.
  `ChainError::EmptyDensities` and `ChainError::EmptySkewData` stay in
  `ChainError` (ADR-0001 D5): analytics constructing a market error is a
  downward reference. `OptionData::get_option` and the
  `OptionChain::expiration_date` field are widened from `pub(super)` /
  private to `pub(crate)` for the moved impls and their tests.

- **Option projections and graph adapters moved out of the math layer**
  (#502). `curves::basic` and `surfaces::basic` priced `Options` and read
  Greeks (a Math-to-Pricing edge), and `curve.rs`, `surface.rs` and the three
  `visualization/plotters.rs` files under `curves`, `surfaces` and
  `geometrics` implemented `Graph` and `Plottable` (a Math-to-Visualization
  edge). `BasicCurves` and `BasicSurfaces` now live in
  `analytics::projections`, together with `impl BasicCurves for OptionChain`,
  `impl BasicSurfaces for OptionChain` and the inherent `OptionChain`
  wrappers (`gamma_curve`, `vanna_surface`, `theta_time_surface`, ...) that
  call them; the wrappers keep their `OptionChain::` paths because an inherent
  `impl` may sit in any module of the crate. `impl Graph for Curve`,
  `impl Graph for Vec<Curve>`, `impl Graph for Surface`, the `Plottable`
  impls, `Plottable` and `PlotBuilder` now live under `visualization`
  (`visualization::{PlotBuilder, Plottable}` are new public paths). Every
  0.21 path resolves as before through re-exports marked
  `// facade-compat`: `curves::BasicCurves`, `surfaces::BasicSurfaces`,
  `geometrics::{PlotBuilder, Plottable}`; `curves::visualization` stays as an
  empty public module. `curves`, `surfaces` and `geometrics` no longer
  reference `greeks`, `chains`, `metrics` or `visualization` except through
  those marked lines. `CurveError::{Greeks, MetricsError, Graph}`,
  `SurfaceError::Greeks` and the `SurfaceError` graph variants are left in
  place: removing a variant is a breaking change and is batched behind the
  0.22.0 bump (ADR-0001 D6). No interpolation, projection or rendering
  behaviour changed.
- **Capability behaviour moved off the core types into extension traits**
  (#499, M1-02). The model-based valuations of `Options`
  (`calculate_price_binomial`, `calculate_price_binomial_tree`,
  `calculate_price_black_scholes`, `calculate_price_montecarlo`,
  `calculate_price_telegraph`, `time_value`, `calculate_implied_volatility`)
  now have their real bodies in the pricing-owned extension trait
  `pricing::OptionPricing` (`src/pricing/option_pricing.rs`), implemented for
  `Options` and re-exported through the prelude. `use
  optionstratlib::pricing::OptionPricing;` is the canonical 0.22 form; the
  inherent methods of the same names stay as one-line forwarding wrappers so
  every 0.21 call site that never imported a trait keeps compiling; the
  wrappers are a live `model -> pricing` edge (an inherent method cannot be
  facade code), listed by the boundary checker (#507) as deferred and
  removed in the batch behind the 0.22.0 bump. No `Position` extension trait was needed: all
  of its P&L helpers (`total_cost`, `premium_received`,
  `net_premium_received`, `net_cost`, `fees`, `break_even`, `unrealized_pnl`,
  `pnl_at_expiration`, `max_profit`, `max_loss`) need only core data and stay
  inherent. Whole trait-impl blocks moved to the layer that owns the trait,
  bodies unchanged: `impl Greeks for Options/Position` to
  `src/greeks/model_impls.rs`; `impl PnLCalculator for Options/Position` and
  `impl TransactionAble for Position` to `src/pnl/model_impls.rs` (valuation
  now goes through `OptionPricing`); `impl BasicAble for Options/Position` to
  `src/strategies/model_impls.rs`; `impl Graph for Options/Position` to
  `src/visualization/model_impls.rs`; `impl TryFrom<&OptionData> for Options`
  and the crate-internal `update_from_option_data` pair (now the
  `pub(crate)` trait `chains::model_impls::UpdateFromOptionData`) to
  `src/chains/model_impls.rs`. Every public path, signature and test assertion
  is unchanged; the tests of the moved impls moved with them. Compile fixtures
  in `tests/unit/model/capability_traits_test.rs` cover the inherent call
  without any trait import, the trait form from `pricing`, the prelude form
  and the moved trait impls.
- **Upper-layer dependencies removed from the core model** (#498, M1-01).
  Every `model -> {chains, greeks, pnl, pricing, series, strategies,
  visualization, geometrics}` production reference that could go without a
  public change is gone: `From<OptionChain>`/`From<&OptionChain> for
  Positive` moved to `src/chains/model_impls.rs` and
  `From<OptionSeries>`/`From<&OptionSeries> for Positive` to
  `src/series/model.rs` (`positive_ext.rs` keeps only `impl ToRound for
  Positive`); `impl Display`/`impl Debug for Strategy` moved from
  `src/model/format.rs` to `src/strategies/base.rs` next to the struct;
  `impl HasX for Decimal` moved from `src/model/decimal.rs` to
  `src/geometrics/interpolation/traits.rs` next to the trait;
  `ProfitLossRange::calculate_probability` has its body in the new
  analytics-owned extension trait
  `analytics::profit_range::ProfitRangeProbability`
  (`src/analytics/profit_range.rs`, built on `analytics::probability`;
  `strategies::probabilities` re-exports it) and the inherent method is a
  forwarding wrapper. Nothing public moved path, changed signature or
  changed a result; the tests moved with their bodies.
  What stays, and why (every remaining hit of the boundary scan
  `rg -n 'crate::(chains|greeks|pnl|pricing|series|strategies|visualization|analytics|geometrics|curves|surfaces|metrics)' src/model`
  is inside `#[cfg(test)]`, a doc link, or one of the deferred edges below,
  each annotated `// deferred edge` and listed by the boundary checker):
  - `src/model/option.rs` `use crate::pricing::OptionPricing;` (pricing): the seven inherent pricing wrappers from #499 forward to the
    trait; removing them is the 0.22 break recorded in
    `doc/API-BASELINE.md` 3.3.
  - `src/model/leg/leg_enum.rs` `use crate::greeks::Greeks;` inside the
    five `Option` arms of `impl LegAble for Leg` (pricing): a live
    core-to-pricing edge. `LegAble` is
    a core trait on a core type, so the impl cannot move under the orphan
    rule; its Greek methods (which already return the pricing-owned
    `GreeksError`) move to a pricing-owned extension trait in the batch
    behind the 0.22.0 bump. The boundary checker (#507) lists it as a
    deferred edge so M1 closes with it documented, not hidden.
  - `src/model/trade.rs` `use crate::pnl::PnL;` (analytics):
    `Trade::pnl() -> PnL` is public inherent API returning an
    analytics-owned type; `PnL::from(&trade)` is the 0.22 form.
  - `src/model/profit_range.rs` `ProfitRangeProbability`, `PriceTrend`,
    `VolatilityAdjustment` (analytics): the inherent
    `ProfitLossRange::calculate_probability` wrapper keeps its 0.21
    signature, which names the two analytics-owned parameter types.
  - `src/model/option.rs:7`, `src/model/position.rs:14`,
    `src/model/types.rs:15` and the `types.rs` test modules: `Payoff`,
    `PayoffInfo`, `standard_payoff`, `Profit`; owned by #500 (PR #572), left
    untouched here.
  Error-type references (`PricingError`, `ProbabilityError`, `GreeksError`
  in `model` signatures) are `error/` ownership and belong to M1-14 (#511);
  `ProfitLossRange::new` returning `ProbabilityError` is the deferred
  breaking item ADR-0001 D6 assigns to the 0.22.0 batch.
- **Generic simulation contracts** (#504, multi-crate roadmap M1-07).
  `simulation::PathEvaluator` (one method, `evaluate_path`, with an
  associated `Outcome`), `simulation::PathOutcome` (per-path P&L, holding
  period, exit reason, outcome flags and premium marks, built from core types
  only), `simulation::PathStatistics::from_outcomes` (mean, median, sample
  standard deviation, best, worst, win rate, average holding period; the same
  arithmetic the strategy simulations use) and `simulation::evaluate_paths`
  (drives an evaluator over every walk of a `Simulator`). A simulation-only
  evaluator can now generate and summarise paths without naming a strategy.
- **Backtesting adapters** (`backtesting::adapters`): `From<PathOutcome> for
  SimulationResult`, `From<&SimulationResult> for PathOutcome`,
  `SimulationStatsResult::from_results` and
  `SimulationStatsResult::from_outcomes`, so the strategy-bound result shapes
  are derived from the generic ones in one place.
- `SimulationStats::update_outcome` folds a `PathOutcome` into the
  accumulator; `SimulationStats::update` now delegates to it.
- **Single-leg strategy simulation orchestration lives in backtesting**
  (#505, multi-crate roadmap M1-08). `backtesting::strategy_simulation`
  provides `SingleLegSimulation` (the side of the leg and the fee adjustment
  applied to every mark, the only things that differed between the four
  bodies), `SingleLegPathEvaluator` (a `PathEvaluator` producing one
  `SimulationResult` per walk) and `simulate_single_leg` (the loop, its
  progress bar and the aggregate through `SimulationStatsResult::from_results`).
- **`MonteCarloPricer` contract and the generic pricing engine** (#508,
  multi-crate roadmap M1-11, ADR-0001 D3). `pricing::MonteCarloPricer` is
  the one method pricing needs from a simulator (`price_monte_carlo(&self,
  &Options) -> PricingResult<Positive>`, `Send + Sync`, also implemented for
  `&M`); `pricing::NoMonteCarlo` is the zero-sized pricer that reports
  `PricingError::SimulationError`; `pricing::GenericPricingEngine<M =
  NoMonteCarlo>` has the same four arms as `PricingEngine` with `MonteCarlo
  { simulator: M }`; `pricing::price_option_with` dispatches it with static
  dispatch; `pricing::ClosedFormEngine` is `GenericPricingEngine<NoMonteCarlo>`.
  `Simulator<Positive, Positive>` implements `MonteCarloPricer` by delegating
  to `get_mc_option_price`, and `From<PricingEngine>` converts the concrete
  engine into `GenericPricingEngine<Simulator<Positive, Positive>>`.

### Changed

- **Errors no longer carry a higher layer's error** (#511, multi-crate
  roadmap M1-14). Eleven variants existed only to wrap the error of a layer
  above: `OptionsError::Greeks`, `CurveError::{Greeks, Graph}`,
  `SurfaceError::{Greeks, Graph}`, `VolatilityError::Chain`,
  `SimulationError::{Strategy, Chain, GraphError}` and
  `StrategyError::Simulation`. They are removed with their conversions, and
  the layer that composes both now owns an error that keeps each cause
  **typed**, never a formatted message: `SimulationError::Volatility` holds
  the pricing failure, the new analytics-owned `ProjectionError` holds the
  `GreeksError` behind a named Greek, the new backtest-owned `BacktestError`
  holds the strategy and simulation failures, and `ChainError` keeps holding
  the Greek failure of a strike aggregation. `PathEvaluator` gains an
  associated `Error` type so an evaluator reports its own layer's error.
  Where a cause is reported: the analytics projections report a Greek
  failure as `CurveError::MetricsError` / `SurfaceError::AnalysisError` naming
  the Greek, `chains::options::deltas` reports `OptionsError::greeks_error`,
  backtesting reports a strategy fee failure as
  `SimulationError::InvalidParameters`, and the IV solver reports
  `VolatilityError::NumericalFailure`. `simulation::generator_positive` now
  reports `SimulationError` instead of the market `ChainError` it never
  belonged to. Ten reverse edges disappear from the boundary report.
  Example binaries that mixed a math error with a rendering error in their
  `main` now return `Box<dyn std::error::Error>`, which is what mixing two
  layers' errors in one entry point actually means.


- **Four misplaced helpers return to their owning layer** (#599, multi-crate
  roadmap M1). Each edge existed only because of where a file sat.
  `model::utils::calculate_optimal_price_range` is a chain helper reporting
  `ChainError` and moves to `chains::utils`; the private `utils::csv` module
  (`OhlcvCandle`, `read_ohlcv_from_zip`, `read_ohlcv_from_zip_async`,
  `OhlcvError`) is market-data I/O and moves to `chains::csv`;
  `volatility::utils::generate_ou_process` is a mean-reverting path
  generator reporting `SimulationError` and moves to `simulation::ou`. Each
  is reachable at its owner's path only, and the prelude keeps exporting
  `OhlcvCandle` and `read_ohlcv_from_zip`, from there. `MetricsError` is
  reclassified as math-owned with no code move: its only crate references
  are `CurveError` and `SurfaceError`, both math, and the metric traits that
  report it live in `curves`, `surfaces` and `geometrics`; the `metrics`
  module itself stays in analytics. Four reverse edges gone, leaving 20.

- **The pricing dispatcher becomes the generic engine, and the combination
  helper joins strategies** (#598, multi-crate roadmap M1). `PricingEngine`
  stored a concrete `Simulator`, so pricing named simulation, and
  `price_option` / `Priceable` existed to dispatch over it. The
  component-level form of #508, `GenericPricingEngine<M>` with the
  `MonteCarloPricer` contract, already did that work without the dependency,
  so the concrete enum, `price_option` and the `From<PricingEngine>`
  conversion are **removed**: one engine type, parameterised by its Monte
  Carlo pricer. `Priceable` moves to `pricing` and is generic over that
  pricer, so `option.price(&engine)` keeps working for every arm, including
  `GenericPricingEngine::MonteCarlo { simulator }`. Callers replace
  `price_option(&o, &PricingEngine::ClosedFormBS)` with
  `price_option_with(&o, &ClosedFormEngine::ClosedFormBS)`, and
  `PricingEngine::MonteCarlo { simulator }` with
  `GenericPricingEngine::MonteCarlo { simulator }`.
  `utils::others::process_n_times_iter`, whose only production consumer is
  `strategies::custom`, moves to `strategies::combinations` and reports
  `StrategyError` instead of the crate-level unified `Error`. Two reverse
  edges are gone (`pricing -> simulation`, `utils -> error/unified`) and no
  compatibility alias is left behind.

- **`Simulate` and `SimulationStats` move to backtesting** (#595, multi-crate
  roadmap M1, decision D2). Both are backtest concepts that happened to live
  under `src/simulation/`: `Simulate::simulate` returns the backtest-owned
  `SimulationStatsResult` and every implementation of the trait already lives
  in `backtesting::strategy_simulation`, while `SimulationStats` stores
  `Vec<SimulationResult>`. The trait joins its implementations and the struct
  becomes `src/backtesting/stats.rs`; `backtesting::{Simulate, SimulationStats}` is now
  the only defining path; the simulation layer no longer re-exports them, and
  the prelude takes both from their owner. Code that imported
  `optionstratlib::simulation::Simulate` imports
  `optionstratlib::backtesting::Simulate`, or the prelude. The simulation layer
  no longer names a backtest type: the `simulation -> backtesting` deferred
  edge is gone, leaving 28. Bodies, bounds and tests moved unchanged.

- **`ProfitLossRange` moves to the analytics layer** (#594, multi-crate
  roadmap M1, decision D2). The type sat in `src/model/profit_range.rs`, in
  the core layer, while every part of it belonged to analytics: its
  `calculate_probability` forwards to
  `analytics::profit_range::ProfitRangeProbability`, its signature names
  `analytics::probability::{PriceTrend, VolatilityAdjustment}` and its
  constructor reports `ProbabilityError`. It now lives beside that trait in
  `src/analytics/profit_range.rs`; `model::ProfitLossRange` and the prelude
  path are downward re-exports, so no 0.21 path changes. The two reverse
  edges the boundary checker tolerated for this file (`model -> analytics`,
  `model -> error/probability`) are gone, leaving 29. Bodies and tests moved
  unchanged.


- **Every error file names its target crate and wraps only lower layers**
  (#511, multi-crate roadmap M1-14). The `From` conversions whose source
  error belongs to a higher layer moved next to that source
  (`From<StrategyError> for PositionError/ProbabilityError/SimulationError`,
  `From<PricingError> for OptionsError`, `From<MetricsError>` and
  `From<GraphError> for CurveError`, `From<GraphError> for SurfaceError`,
  `From<ChainError> for SimulationError/VolatilityError`); no conversion was
  removed or changed. `error::simulation` imports `GraphError` from its
  owning file instead of through the prelude. The ownership table is in the
  `error` module docs; the variants that still reference a higher layer are
  listed there and are removed after the 0.22.0 version bump.
- **Payoff contracts are owned by the core model** (#500, multi-crate
  roadmap M1-03). `Payoff`, `PayoffInfo`, the implementation for every
  `OptionType` variant and the exotic payoff helpers now live in
  `optionstratlib::model::payoff`; `optionstratlib::pricing::payoff` and the
  prelude re-export them, so `use optionstratlib::pricing::{Payoff,
  PayoffInfo}` keeps compiling. `impl Profit for Options` and `impl Profit for
  Position` moved from `model` to `pricing::payoff`, beside the `Profit` trait
  they implement. No signature or numerical result changed; the only new
  public path is `optionstratlib::model::payoff`.
- **`FindOptimalSide` is owned by the market layer** (#501, multi-crate
  roadmap M1-04). The strike-selection enum moved from `strategies::utils` to
  `chains::utils` and is re-exported from `chains`; `strategies::utils::FindOptimalSide`,
  `strategies::FindOptimalSide` and the prelude path are re-exports of the
  same type, so no import changes. `chains` no longer imports anything from
  `strategies`: `OptionData::get_option_for_iv` writes the implied volatility
  field directly instead of going through the strategy `BasicAble` setter.
- **`DeltaAdjustment` is owned by the analytics layer** (#503, multi-crate
  roadmap M1-06). The adjustment enum and `DeltaAdjustmentSameSize` moved
  from `strategies::delta_neutral::model` to the new `pnl::adjustment`
  module and are re-exported from `pnl`; the `strategies::delta_neutral::DeltaAdjustment`
  and `strategies::DeltaAdjustment` paths are re-exports of the same type.
  `pnl` no longer imports anything from `strategies`.
- **The `OptionChain` ATM-IV adapter lives with the chain** (#510,
  multi-crate roadmap M1-13). `impl AtmIvProvider for OptionChain` moved from
  `volatility::traits` to `chains::chain`; the `AtmIvProvider` and
  `VolatilitySmile` traits stay generic in `volatility`, which no longer
  imports option chains. Behaviour and error mapping are unchanged.
- **`DELTA_THRESHOLD` is owned by the Greeks layer** (#506). The constant
  moved from `strategies::delta_neutral` to `greeks`, where
  `calculate_delta_neutral_sizes` uses it; `strategies::delta_neutral::DELTA_THRESHOLD`
  and `strategies::DELTA_THRESHOLD` are re-exports of the same constant.
  `greeks` no longer imports anything from `strategies`.
- **Single-leg strategy simulation lives in `backtesting`** (#505). The
  `Simulate` implementations for `LongCall`, `LongPut`, `ShortCall` and
  `ShortPut` moved from the strategy files to
  `backtesting::strategy_simulation` as one generic body;
  `strategy.simulate(&simulator, exit)` is unchanged. A new golden test
  (`tests/unit/backtesting/single_leg_simulation_golden_test.rs`) runs the
  four strategies over five deterministic historical paths and seven exit
  policies (140 runs, two walks each), serialises the results with
  `PnL.date_time` removed (it is stamped with `Utc::now()`), and compares
  them with a golden file generated on the code before this change; every
  other field matches byte for byte. The `indicatif` progress bar moved with
  the loop; M6-05 removes it from the library. The `Graph` implementations
  for every concrete strategy and the `impl_graph_for_payoff_strategy!`
  macro moved from `strategies::graph` to `visualization::strategies`; the
  macro keeps its crate-root path and the `strategies::graph` module stays
  (empty) until the 0.22.0 breaking batch. Two `strategies` references to
  upper layers remain and are listed by the boundary checker (#507) as
  deferred edges: `strategies::simulation_impls` (the `BasicAble` impls for
  the simulation containers) and the `Strategable: Graph` supertrait bound
  in `strategies::base`; both go with the batch.
- **`price_option` dispatches through the generic engine** (#508).
  `PricingEngine` and `price_option` are unchanged in shape; `price_option`
  now views the engine as `GenericPricingEngine<&Simulator<..>>` and calls
  `price_option_with`, so the two dispatchers share every arm and cannot
  drift. Prices are identical (closed-form arms are the same functions; the
  Monte Carlo arm is the same `get_mc_option_price` call, failures still
  reported as `PricingError::SimulationError`). Numerical Greeks are
  untouched. The `simulation` import in `pricing::unified` is the one
  remaining reverse edge, marked `// deferred edge` (listed by the boundary checker); per
  ADR-0001 D3 the 0.22.0 batch makes `GenericPricingEngine` the only engine
  and the facade aliases `PricingEngine` to
  `GenericPricingEngine<Simulator<Positive, Positive>>`.
- **`simulation` no longer imports `strategies` or `visualization`** (#504).
  `impl BasicAble for Simulator` / `RandomWalk` moved to
  `strategies::simulation_impls` and `impl Graph for Simulator` / `RandomWalk`
  moved to `visualization::simulation` (a trait impl lives with the trait when
  the type's layer must not depend on it). Both impls behave exactly as
  before; no import path changes. The `BasicAble` impls are scheduled for
  removal in the 0.22.0 breaking batch (ADR-0001 D2): the inherent
  `get_title` accessors already cover their only use. The two remaining
  reverse edges, `Simulate::simulate` returning the backtest-owned
  `SimulationStatsResult` and `SimulationStats` storing `SimulationResult`,
  are annotated `// deferred edge`, listed by the boundary checker (#507)
  and move with the batch.

### Deprecated

- **`utils::logger::setup_logger` and `setup_logger_with_level`** (#506,
  multi-crate roadmap M1-09). A library must not install a global `tracing`
  subscriber; install one from your binary with
  `tracing_subscriber::fmt().with_max_level(..).init()`. The functions are
  deprecated on `main` after 0.21.3 and removed in 0.22.0 (M6-04, in the batch
  that follows the version bump), so no 0.22 release ships them; the
  deprecation is the signal for anyone building from `main` in between. The
  example binaries now take their logger from the non-published
  `osl-example-support` package under `examples/support`.

### Fixed

- **Every `Decimal` square root is total** (#588). Upstream
  `rust_decimal::MathematicalOps::sqrt` aborts the process with
  `geo mean circuit breaker` when its Newton iteration oscillates at the 28th
  decimal instead of converging; `dec!(4.0000000000000000000000000003).sqrt()`
  reproduces it, and `garch_volatility(&[1, 1, 1, 1], omega = 1,
  alpha = 1e-28, beta = 1)` reaches that variance on its third step, which is
  how the unseeded `test_garch_volatility_never_panics` property test found
  it. `model::decimal::d_sqrt` now carries its own Newton iteration with the
  upstream seed and update step plus a bounded loop that resolves the
  oscillation by returning the candidate whose square is closest to the
  input; the new crate-private `p_sqrt` routes every former
  `Positive::checked_sqrt` call site (28) through it, and the remaining
  `Decimal::sqrt` call sites (39, across pricing, simulation, volatility,
  metrics, curves, surfaces and strategies) call `d_sqrt` with their previous
  fallback or error mapping unchanged. `make scan-banned` now flags any
  `.sqrt()` / `.checked_sqrt()` in production code; the `f64::sqrt` sites
  carry an allow marker. No input on which upstream converged changes value:
  `d_sqrt` yields the bit-identical result there, and the reproducer plus a
  deterministic GARCH regression test pin the previously aborting case.

## [0.21.3] - 2026-09-19

### Fixed

- **A range whose upper-bound probability comes out below its lower-bound
  probability is reported, not aborted on** (#569).
  `ProfitLossRange::calculate_probability` subtracted the two with the raw
  `Positive` operator; on a spot near `Positive::MAX` with a volatility of
  `1e-28`, `Decimal::checked_ln` returns `+9e-28` for `(MAX - 4) / MAX` where
  the true value is `-1e-28`, the distribution function evaluates higher below
  the lower bound (`0.9987`) than below the upper one (`0.5`), and
  `Positive::sub` aborted the process. It now returns
  `ProbabilityError::CalculationError(InvalidProbability)` carrying the negative
  difference and both probabilities. Flooring the difference to zero was
  rejected: a probability nobody computed is worse than an error. A zero-width
  range still reports probability zero. Ordinary inputs cannot reach the error;
  the `ln` error is at the 28th decimal and only surfaces below
  `vol * sqrt(T) ~ 1e-11`.

## [0.21.2] - 2026-09-18

### Changed

- Dependencies updated to latest stable versions (`rust_decimal` 1.42 -> 1.43;
  `indicatif` 0.17 -> 0.18 in `examples_simulation`).

### Removed

- **The crate no longer builds a `cdylib`** (#496). `Cargo.toml` declared
  `crate-type = ["cdylib", "rlib"]`, but the library exposes no C ABI: there is
  not one `extern "C"`, `#[no_mangle]`, `#[unsafe(no_mangle)]` or
  `#[export_name]` in `src/`, so the dynamic library it produced had no callable
  entry point. It was build and release surface without a foreign-function
  interface behind it.

  The compatibility impact is nil for any consumer using the crate as a Rust
  dependency: the `rlib` is unchanged and the public API snapshot does not move.
  It is only observable to something loading `liboptionstratlib.dylib` /
  `.so` / `.dll` by filename, which would have found no symbols to call. Nothing
  in this repository consumed the artifact — not the `Makefile`, not the
  workflows, not `Docker/`.

  To reinstate a dynamic library, the crate needs an actual `extern "C"` surface
  first; restoring the target alone would rebuild the same empty artifact.

## [0.21.1] - 2026-08-30

### Fixed

- **A worthless option was priced as absent, which truncated the strike
  ladder** (#487). `OptionData::calculate_prices` mapped a non-positive
  Black-Scholes price to `None`: Black-Scholes on `Decimal` undershoots by an
  epsilon for an option worth nothing, measured at `-2.992e-25` for a call 300
  points out of the money at seven and a half hours to expiry, and
  `Positive::new_decimal` rejected it. A contract that was merely worthless
  became one that does not quote, and since `build_chain` reads
  `some_price_is_none()` as "this wing does not quote", two strikes missing
  only their out-of-the-money side stopped the ladder: at spot 5100 with
  `strike_interval` 25 at 0.3125 days, `chain_size` 20 returned 23 strikes
  instead of 41, with the 4825 put and the 5375 call absent while their other
  sides carried real prices. A successful but non-positive price now reads as
  zero, so the tick floor from #439 quotes it as a market would; a genuine
  pricing failure still produces no price.

## [0.21.0] - 2026-08-30

### Fixed

- **Seven entry points in `src/volatility/` aborted on inputs the type system
  accepts** (#442). A `catch_unwind` probe over the module's public API across
  3305 extreme-input cases reported 287 aborts before the sweep and none
  after. `historical_volatility` handed a zero `window_size` to
  `slice::windows`, which panics with `window size must be non-zero`;
  `garch_volatility` seeded its recursion with `returns[0]` and aborted on an
  empty slice with `index out of bounds: the len is 0 but the index is 0`;
  `simulate_heston_volatility` ran its Euler step on raw `Decimal` operators
  and aborted with `Multiplication overflowed`; `annualized_volatility`,
  `de_annualized_volatility`, `volatility_for_dt` and `adjust_volatility` used
  the panicking `Positive` operators for the square-root-of-time rescaling and
  aborted with `Positive arithmetic overflow in mul` / `in div`, and with
  `Positive invariant broken in div: result would be non-positive` for a
  `TimeFrame::Custom(Positive::ZERO)` divisor; and `implied_volatility`
  computed its grid size as `100 * max_iterations`, which aborts a debug build
  with `attempt to multiply with overflow` at `i64::MAX` and wraps silently in
  release.

  Every one of these already returned `Result`, so no signature changed. The
  degenerate timeframe is now `VolatilityError::InvalidTime` with the same
  message shape `adjust_volatility` already used for its target frame, so the
  two entry points agree on where the domain ends. Well-formed inputs keep
  their values: the checked `Positive` helpers are the same code path the
  operators call before panicking. `tests/property/volatility_panic_freedom_test.rs`
  drives the sweep over the sample shapes, the timeframes and the option
  geometries.

- **`Options::payoff`, `payoff_at_price` and `intrinsic_value` reported a
  payoff of zero when it was too large to represent** (#442). All three
  evaluated the payoff in `f64`, scaled it by the quantity, and converted with
  `Decimal::from_f64(..).unwrap_or_default()`. Both factors reach
  `Positive::MAX` (`≈ 7.92e28`), so the product routinely leaves the `Decimal`
  range — and a long call struck at zero on an underlying at `Positive::MAX`
  leaves it at quantity one, because the nearest `f64` to `Decimal::MAX` rounds
  above it. The three now return `OptionsError::PayoffError` naming the value
  and the entry point. An out-of-the-money leg still returns `Ok(0)`, which is
  the answer rather than a fallback, and no existing test changed.

- **`DecimalStats::mean` and `DecimalStats::std_dev` aborted on a sample they
  could not sum** (#442). `mean` folded with `iter().sum()`, which aborts with
  `Addition overflowed`; `std_dev` squared each centred deviation with
  `.powd(Decimal::TWO)`, which aborts with `Pow overflowed`. `vec![Decimal::MAX;
  2]` reached both. The two trait methods now return
  `Result<Decimal, DecimalError>` and fold through the checked helpers in
  `src/model/decimal.rs`; the sample standard deviation is unchanged digit for
  digit for a representable sample.

  This is a breaking change to a public trait. To migrate, add `?` where the
  value feeds a fallible function or `.expect("…")` at a boundary that cannot
  propagate; an external implementor changes the two signatures and returns
  `Ok(..)`. No further version bump: 0.21.0 is already the breaking version
  for this unreleased cycle.

- **`decimal_normal_sample` constructed a distribution that could be
  rejected** (#442). It built `Normal::new(0.0, 1.0)` on every call and had an
  `unreachable!` in the `Err` arm. It samples `rand_distr::StandardNormal`
  instead, a unit struct with no constructor and nothing to reject.
  `Normal::sample` is `mean + std_dev * z` over the same `StandardNormal`, so
  at `(0.0, 1.0)` the two are the same value as well as the same distribution.

- **Every multi-leg strategy aborted the process when its break-even vector
  was shorter than the point it read** (#463). `get_profit_area`,
  `get_profit_ratio`, `get_profit_ranges`, `get_loss_ranges` and
  `get_best_range_to_show` indexed `break_even_points` directly across the
  fourteen multi-leg strategies, 75 unguarded accesses, and an empty vector
  turned each into `index out of bounds: the len is 0 but the index is 0`.
  The vector is a `pub` field and every strategy derives `Deserialize`, so a
  JSON document carrying `"break_even_points": []` reached all five. Two ways
  in needed no extreme input at all: `LongButterflySpread::get_strategy` and
  `ShortButterflySpread::get_strategy` never called
  `update_break_even_points`, so a butterfly assembled from a leg set always
  carried an empty vector; and a butterfly whose wing never crosses zero
  profit legitimately has one break-even point or none, which made
  `the len is 1 but the index is 1` reachable from an ordinary constructor.

  The two constructors now populate the vector like every other constructor
  in the crate. The readers report the shortfall, `StrategyError` on the
  `Strategies` methods and `ProbabilityError::RangeError` on the probability
  ranges, and name which of the two points is missing. Every value a
  populated vector produced is unchanged: the widths taken between two
  break-even points go through `price_gap`, which is the same subtraction for
  an ordered pair, and a zero-width region instead of a panic for a crossed
  one.

  The arithmetic around those reads moved to the checked helpers in the same
  files, since a `Result` that reports a missing point and then aborts on the
  next line is not panic-free. `update_break_even_points` divides the premium
  by a quantity that can be zero; `get_max_loss` on the bear call spread
  subtracted strikes in an order nothing enforces; the strangles' default
  strikes multiply the spot by 1.1 and 0.9, which overflows at
  `Positive::MAX`; and the fourteen `PnLCalculator` impls summed their legs
  with `impl Add for PnL`, which returns `Self` and so has nowhere to report
  an overflowing four-leg total. They now use the checked `PnL::try_add` that
  #460 added for exactly this. A probe over the public API of these files
  went from 43,857 panics in 254,070 calls to none, and the property suite
  gained four cases covering the fourteen strategies, both butterfly leg-set
  constructors and the emptied vector.
- **Six chain tests wrote their artifacts into the working tree and asserted
  their own cleanup succeeded.** `cargo` runs test binaries concurrently
  against one working directory, so `assert!(fs::remove_file(..).is_ok())`
  asserts that nothing else touched the file, which is not a property of the
  test: `test_load_from_json` and `test_deserializer_field_handling` failed
  together on a full-suite run and passed individually. Each writes into a
  directory of its own under the system temp directory now, and the round trip
  is asserted rather than the deletion. The directory is unique per process
  rather than per test, since a path derived from a test's own name is shared
  by every process running that test. `test_load_from_json` was also writing
  into `tests/`, a source directory.

- **`prepare_file_path` reported failure for a file that was already gone.**
  It tested `Path::exists` and then removed, so anything deleting the file in
  between made `remove_file` fail with `NotFound` — for a postcondition that
  already held. Every `write_html` / `write_png` caller inherited it. Two
  tests sharing a working directory were enough to lose the race, and it
  aborted a whole `cargo tarpaulin` run on `main` with `Failed to remove
  existing file: multiple_curves_test.html`, taking a coverage report down
  over a file nobody was reading. The removal is now attempted
  unconditionally and `NotFound` is accepted, which also makes the function
  idempotent for concurrent writers downstream.
- The four `plotters` tests that came in `_bis` pairs wrote to the same two
  paths as their originals, so they raced each other by construction. Each
  pair now writes its own file.
- **The arithmetic Asian is accurate at short maturities, where it used to
  return confident wrong prices** (#462). The general branch of the
  Turnbull-Wakeman second moment recovered `σ²_asian·T` from the difference of
  two terms that grow as `2S²/(a·c·T²)` while the answer shrinks with `T`; at
  fourteen minutes to expiry the terms are around `2.8e15` and the signal is
  `3.7e-8`. It overpriced by a factor of **612** at eighty-six seconds and
  collapsed to approximately zero below `1e-5` days. The moment is now twice
  the first divided difference of `φ(w) = (e^w − 1)/w`, which makes all three
  removable singularities ordinary points and never subtracts two terms larger
  than the result. Relative error across the nine maturities of the issue's
  table is now at most `6.9e-12`, against `1e-9` asserted.

  Prices are **bit-for-bit unchanged at 7, 30, 90, 182.5 and 365 days**, call
  and put. One ordinary maturity moves: **at 1 day the price changes by a
  relative `2.5e-10`, and it moves toward the reference** — measured against
  50-digit `mpmath` quadrature, the old value sat `2.5e-10` away from it and
  the new one sits `8.5e-14` away. A one-day option moving by `2.5e-10`
  relative is orders of magnitude below a tick, so no quoted price changes;
  the direction and size are stated here so the claim can be checked rather
  than taken.

- **`Curve::merge` and `Surface::merge` return the same digits on every run,
  on every machine** (#453). The `Multiply` arm of both types reduced the
  interpolated values with a rayon `reduce`, and `Curve`'s `Divide` arm folded
  its reciprocals with the same reducer. `Decimal` multiplication rounds once
  a product needs more than the 28 decimal places it stores, so it is not
  associative: over a sweep of 205,379 reciprocal triples, 628 of them, 0.31%,
  regroup to a different last digit.

  Rayon takes the grouping from its length splitter, whose threshold is the
  length of the input divided by eight times the number of threads in the
  ambient pool. So the result moved two ways. Run to run, from work stealing:
  merging forty curves returned **1996 distinct results in 2000 runs of one
  binary over one input** on an eight-thread pool, 809 in 2000 at four threads
  and 52 in 2000 at two. And machine to machine, from the pool size: on one
  thread and on sixteen it returned one result across 2000 runs each, and
  those two results disagreed with each other. The second is the worse of the
  two, because two services on differently sized machines then disagree on one
  input while each looks perfectly stable where it runs.

  Both arms now fold left to right, `Multiply` through the new
  `d_product_iter` placed beside the `d_sum_iter` the `Add` arm already used.

  **Both `Multiply` and `Divide` move in their last digits, and they are
  separate changes.** `Multiply` moves on both types. Three constant curves at
  `0.010989010989010989010989011`, `0.010752688172043010752688172` and
  `0.0094339622641509433962264151` merged to
  `0.0000011147302687168785768907` before, the reducer having bracketed them
  as `a * (b * c)`, and merge to `0.0000011147302687168785768908` now, which
  is `(a * b) * c`: one unit in the last of the 28 decimal places, `9e-23`
  relative to a value of that size. `Divide` moves on `Curve` only. #452 made
  `Surface`'s `Divide` a sequential chain of divisions and it is untouched
  here; `Curve`'s was still a parallel product of reciprocals, so it took its
  bracketing from the same splitter and had to change with `Multiply`. Three
  constant curves at 97, 91 and 93 merged to
  `0.0114616566229469455275906915` before and merge to
  `0.0114616566229469455275906888` now, a move of 27 units in the last
  decimal place, `2.4e-25` relative. The `Divide` move is the larger of the
  two because each reciprocal is separately rounded to 28 decimal places
  before it is multiplied in.

  Both are more than twenty orders of magnitude below a tick, so no quoted
  price, greek or premium changes; the values are given so the claim can be
  checked rather than taken. A downstream consumer holding a golden file at
  full `Decimal` precision will see the difference, which is the reason this
  entry exists. Every merge test in the crate compares within `1e-3` or
  `1e-4`, so none of them moved.

  **The parallelism it removes was worth nothing.** Over five alternating
  Criterion runs — 500 samples a side on the curve group, 50 on the surface
  group, whose `sample_size(10)` keeps a 51 x 51 grid affordable — the
  sequential fold is 2% to 12%
  faster at the least-contended end of the distribution and indistinguishable
  from the reducer at the median, against a noise floor on the measuring
  machine of roughly 30%. What it removes is a rayon dispatch per grid point
  to multiply between two and ten numbers; measured on its own that dispatch
  costs 17 µs against 22 ns for the fold at two operands, and 28 µs against
  794 ns at ten. `benches/geometrics/merge.rs` is new and carries all three
  measurements.

  `Max` and `Min` keep their parallel reducers: they select a value instead of
  accumulating one, so no rounding enters and the extremum does not depend on
  the grouping.

### Changed

- **`make doc` builds documentation.** It ran `cargo clippy -- -W
  missing-docs`, which builds none, so no intra-doc link was ever resolved by
  it. It is now `cargo doc --all-features --no-deps`. The target starts green:
  two `private_intra_doc_links` warnings stand (`RNDStatistics::new` in
  `src/chains/rnd.rs`, `lower_break_even` in `src/strategies/base.rs`) and
  warnings do not fail it. `AGENTS.md` records this and four other gaps
  between what the gates check and what they appear to check.

- **`make scan-banned` guards the panicking maths and the panicking macros,
  not just `unwrap`/`expect`.** It now also rejects `panic!`,
  `unreachable!`, `todo!`, `unimplemented!`, `.exp()`, `.ln()`, `.powd(` and
  `.sqrt().unwrap()`. The `f64` methods share three of those names and do not
  abort, so those call sites carry a `// scan-banned: allow` marker saying so;
  grep cannot tell the receiver types apart. Its comment filter no longer
  skips every line starting with `*`, which had been swallowing the
  continuation lines of multi-line products — five `.exp()` calls in
  `src/pricing/compound.rs` were invisible to it.
- **Twenty public methods whose return type had no error channel are now
  fallible, which is a breaking change** (#471). Each aborted the process on
  arithmetic over `pub` fields. #460 closed everything reachable through the
  public constructors, and what remained was reachable only by writing a
  field directly, by deserializing a document that carries one, or through a
  trait whose signature forbade failure — so none of them could be fixed
  without changing the signature, which is why they were reported rather than
  fixed at the time. The values returned are unchanged for every input the
  panicking forms accepted. To migrate, add `?` where the value feeds a
  fallible function, or handle the `Result` at a boundary that cannot
  propagate.

  **`LegAble` gains an error channel on three methods.** `pnl_at_price`
  returns `Result<Decimal, PricingError>` instead of `Decimal`, and
  `total_cost` and `fees` return `Result<Positive, PositionError>` instead of
  `Positive`. `pnl_at_price` computes `(price - cost_basis) * quantity` and
  aborted with `Subtraction overflowed`; `total_cost` multiplies size by cost
  basis before adding two fees, and aborted with `Positive arithmetic overflow
  in add`. The four implementors — `SpotPosition`, `FuturePosition`,
  `PerpetualPosition` and the `Leg` enum — are updated, and every one of the
  three now uses the checked helpers. `Leg::pnl_at_price` and
  `Leg::total_cost` propagate the option leg's failure instead of swallowing
  it as `unwrap_or(Decimal::ZERO)` / `unwrap_or(Positive::ZERO)`; both
  fallbacks were values a legitimate leg can also return, so neither could be
  told apart from an answer. `Position::total_cost` and `Position::fees` have
  returned `Result` since #470, so the leg trait was the last infallible
  wrapper over the same sums.

  **Five `PerpetualPosition` and `FuturePosition` methods follow.**
  `FuturePosition::unrealized_pnl` and `PerpetualPosition::unrealized_pnl`
  return `Result<Decimal, PricingError>`, and with them
  `PerpetualPosition::roe_percentage`, `margin_ratio` and
  `effective_leverage`. `unrealized_pnl` is the entire body of
  `pnl_at_price` on those two types, so leaving it infallible would have
  moved the abort one frame down rather than removed it. The three ratios
  keep their existing degenerate answers — `Decimal::ZERO` for a zero margin
  or a zero notional, `Decimal::MAX` for wiped-out equity — rather than
  turning those into errors; only the overflow is new.

  **`PnL::try_add` is now public, and the operator impls are documented as
  superseded.** `impl Add for PnL` and `impl Sum for PnL` add `initial_costs:
  Positive + Positive` with the raw operator. `Add::add` and `Sum::sum` are
  fixed by `std` to return `Self`, so no fallible form of either exists and no
  signature change makes them safe. #460 routed every accumulation inside the
  library through a `pub(crate) try_add`; it is `pub` now so callers have the
  same route. The four operator impls are kept, because removing them would
  break every `a + b` and `.sum()` at a call site with nothing to overflow.
  They carry a `# Deprecated in favour of PnL::try_add` doc section rather
  than a `#[deprecated]` attribute: Rust rejects that attribute on a trait
  `impl` block and on a trait method inside one (`error: #[deprecated]
  attribute cannot be used on trait impl blocks`), so the compiler cannot
  emit this warning at all.

  **Three `Collar` methods.** `net_premium` returns `Result<Decimal,
  PricingError>` instead of `Decimal`, and `is_zero_cost` and `is_credit`
  return `Result<bool, PricingError>` instead of `bool`. `net_premium`
  multiplies each leg's `premium` by its `quantity`, both `pub` fields, and
  aborted on overflow; the two predicates read it. They report the same
  failure rather than collapsing it into `false`, which would have read as a
  definite classification.

  **`CoveredCall::effective_cost_basis` and
  `ProtectivePut::effective_cost_basis`** return `Result<Positive,
  PositiveError>` instead of `Positive`. Both divide the premium by
  `spot_leg.quantity`. Both constructors reject a zero share count, but a
  strategy deserialized from JSON or mutated in place can carry one, and
  `Positive` has no value meaning "undefined per-share figure".

  **Four `PortfolioGreeks` methods.** `delta_gap` and `gamma_gap` return
  `Result<Decimal, GreeksError>`, `combined` returns
  `Result<PortfolioGreeks, GreeksError>`, and `add` returns `Result<(),
  GreeksError>` instead of nothing. All five Greek fields are `pub` and are
  written directly by callers aggregating from their own sources, so
  `from_positions` guarantees nothing about them. `add` now writes through
  `combined`, so a failure on the fourth of five sums leaves the receiver
  exactly as it was rather than committing the first three.

  No error enum and no error variant was added, so nothing that matches
  exhaustively on a public error type needs a new arm. The property suite's
  `test_spot_leg_strategies_never_panic` drops the bounded `spot_leg_money`
  generator that existed only for these signatures and runs against the
  unbounded `extreme_money`, so `Positive::MAX` is back in range for the
  three spot-leg strategies.

  Two siblings with the same defect are deliberately out of scope and remain
  infallible: `AdjustmentTarget::delta_gap` / `gamma_gap` / `vega_gap` /
  `is_satisfied`, which subtract the same `pub` Greeks a `PortfolioGreeks`
  carries.

- **`Curve::bilinear_interpolate` now answers across the whole interior
  domain, and its exact-match branch fails on a repeated abscissa** (#451).
  The method read a four-sample window starting at the bracket index, which
  `find_bracket_points` only guarantees to `i + 1`, so every `x` in the last
  two segments left the window past the end and returned
  `InterpolationError::Bilinear`. The cell is now built from the segment
  bracketing `x` and the segment two positions on, the far one clamped to the
  curve's last segment where it would run off the end. On the final segment
  the two edges coincide and the answer is the linear interpolant there; on
  the second-to-last the far edge is the immediately following segment.

  The clamp reaches only those last two segments, so every abscissa the
  method already answered keeps its answer digit for digit, and no existing
  test changed. Values were validated against a reference implementation of
  the rule in exact rational arithmetic, cross-checked against `mpmath` at 60
  digits, over three curves including a non-uniform one.

  The near edge is deliberately not clamped, unlike `cubic_interpolate` at its
  own boundary: the window start is the denominator of the fraction along the
  cell, so moving it back off the segment holding `x` pushes that fraction
  past `1`, turns two of the four weights negative and stops the answer being
  a convex combination of the cell's corners. On `(0,0), (1,1), (2,4), (3,9)`
  that variant returns `9.5` at `x = 2.5`, above every ordinate on the curve.

  The exact-match branch now goes through `Curve::exact_point_at` like the
  three other interpolators, so several ordinates at the queried abscissa
  return `InterpolationError::DegenerateInterval` instead of the lowest of
  them. This is a behaviour change on a curve that breaks the
  one-point-per-abscissa rule, and it closes the exception the #466 entry
  below left open. No signature changed and the public API is unchanged.

- **The crate moves to 0.21.0, which is the breaking bump for a `0.x`
  version.** `cargo-semver-checks` compares against the published 0.20.0 and
  rejected the rename below under `inherent_method_missing`, since a version
  unchanged from its baseline is read as a minor bump and a minor bump may not
  remove a public method. Every earlier change in this cycle was a return type
  moving to `Result`, which no lint covers, so this is the first one the check
  could see.

- **`Surface::get_curve` is renamed to `Surface::project_onto` and returns
  `Vec<Point2D>` instead of `Curve`, which is a breaking change** (#466).
  One change with one reason: the method is a projection, and both its name
  and its type said it was a curve. Projecting a surface onto one axis is
  multi-valued by construction: every row of a grid contributes a different
  height above the same projected abscissa, and two rows that agree on the
  two surviving coordinates project onto the very same point. A `Curve` is a
  function of its abscissa and stores its points in a `BTreeSet`, so it
  silently dropped the collisions. The vector keeps every point:
  `surface.project_onto(axis).len() == surface.points.len()`, always.
  Contents are sorted ascending by the projected `(x, y)` pair, which is the
  order the `BTreeSet` gave; only its deduplication is gone.

  The method has no production callers and its `Axis` parameter type is not
  re-exported, so it is uncallable from outside the crate. To migrate inside
  it, aggregate the ordinates sharing an abscissa to one value, then build the
  `Curve`; `Curve::from_vector(surface.project_onto(axis))` reproduces the old
  behaviour, losses included. The note on `Surface::get_curve` under the #450
  entry below describes the method by its former name.

- **Interpolating a curve at a repeated abscissa now fails instead of
  returning the lowest ordinate stacked there** (#466). `Curve` is documented
  as a function of its abscissa, but nothing enforces it (`points` is a `pub`
  field, so no constructor could). `linear_interpolate`, `cubic_interpolate`
  and `spline_interpolate` return `InterpolationError::DegenerateInterval`
  when several points share the requested `x`, rather than picking one of
  them: the value there is not defined and no choice among them is less
  arbitrary than another. A curve with one point per abscissa is unaffected,
  and `AxisOperations::get_values` still reads every ordinate at an abscissa.
  The exact-match branch of `Curve::bilinear_interpolate` was left out of that
  pass and is covered by the #451 entry above, which brings it into line.

  The one-point-per-abscissa rule, and what each consumer does when it is
  broken, are now stated on `Curve::new`, `Curve::from_vector`,
  `Curve::merge`, `get_point`, `contains_point`, `get_values`,
  `merge_axis_interpolate`, the four interpolation traits and
  `find_bracket_points`, with the matching one-height-per-xy-coordinate rule
  on `Surface::new`.
- **`model::utils::mean_and_std` is now fallible, which is a breaking change**
  (#470). It returns `Result<(Positive, Positive), PositiveError>` instead of
  `(Positive, Positive)`. It summed with the raw `Positive` operator, which
  panics on overflow, and divided by `vec.len() as f64`, which divides by zero
  on an empty sample; every `get_profit_ranges` implementation averages its leg
  volatilities through it. The returned figures are unchanged for every sample
  the panicking form accepted — the squared deviations are still taken in
  `f64` — but a deviation that overflows is now reported instead of collapsing
  to zero through `unwrap_or(Positive::ZERO)`. To migrate, add `?` where the
  value feeds a fallible function.

  `PositionError` gains a `PositiveError` variant in the same change, so
  `Position::total_cost` and `Position::fees` can carry the cause of a
  `Positive` overflow instead of flattening it into a message. Both already
  returned `Result`, so their signatures are unchanged. Matching exhaustively
  on `PositionError` needs a new arm.

  `model::resolve_expiration_date` and `model::reject_unrepresentable_expiration`
  are new. `ExpirationDate::get_date()` aborts with
  `` `DateTime + TimeDelta` overflowed `` for a day count no calendar can
  represent, and every strategy reaches it through
  `calculate_pnl_at_expiration`; these resolve or reject it instead. The guard
  #441 added inside `OptionChain::build_chain` is replaced by a call to the
  shared one. The instant returned is identical for every input the panicking
  form accepted.

- **Two `ProtectivePut` methods are now fallible, which is a breaking change**
  (#460). `total_fees` returns `Result<Positive, PositiveError>` instead of
  `Positive`, and `protection_level` returns
  `Result<Decimal, StrategyError>` instead of `Decimal`. Both aborted the
  process on inputs a caller can supply: the fee sum used the raw `Positive +
  Positive`, which panics for a fee at `Positive::MAX`, and `protection_level`
  divided by the spot cost basis with the raw operator, which a zero
  underlying price turns into a panic. Adding `try_` twins instead would have
  left the panicking methods in place, which is the opposite of what this
  sweep is for.

  To migrate, add `?` where the value feeds a fallible function, or
  `.expect("…")` at a boundary that cannot propagate. `ProtectivePut::new` now
  rejects a zero underlying price as well, so a position built through the
  constructor never reaches the error arm of `protection_level`; a
  deserialized one can.

- **A zero-volatility American or Bermuda option is no longer priced as a
  European, so its price goes up** (#449). `price_binomial` short-circuited
  every contract with `volatility == 0` to the discounted forward payoff,
  `e^{-rT}·payoff(S·e^{rT})`, which is the European value and silently drops
  the early-exercise premium. The same branch is taken when the lattice
  collapses (`u == d`, `σ√dt` below the representable scale), so both paths
  change. An American is now worth the better of exercising immediately and
  holding to expiry; a Bermuda the best of its schedule and expiry, with a
  schedule that is empty, or entirely beyond expiry, still pricing as a
  European. A European is unchanged.

  A zero-volatility American put at `S = 90`, `K = 100`, `r = 5%`, `T = 1`:

  | | before | after |
  |---|---|---|
  | price | `5.122942450071404` | `10` |

  `10` is `K − S`, the value of exercising on the spot. The old number was
  `e^{-0.05}(100 − 90·e^{0.05})`, roughly half. The matching American call at
  `S = 110`, `K = 100` is unchanged at `14.877057549928595`, since a call on
  a non-dividend-paying forward is never exercised early while `r > 0`; flip
  the rate to `r = −5%` and it moves from `4.872890362397602` to `10`.
- **`Point2D` and `Point3D` compare, order and hash on all their coordinates**
  (#450). Both types broke the `Ord` contract, which requires `a == b` exactly
  when `a.cmp(&b)` is `Ordering::Equal`: `Point2D` compared equal on `x` alone
  while ordering on `(x, y)`, and `Point3D` compared equal on `(x, y)` while
  ordering on `(x, y, z)`. Two points could be equal *and* strictly ordered.
  `PartialEq` now reads every coordinate on both types, and `Point3D` gains a
  `Hash` impl (it had none). Ordering is unchanged.
- **A surface no longer loses half its grid when its axes are merged.**
  `Surface` indexes itself by `Point2D` through `AxisOperations<Point3D,
  Point2D>`, and `merge_indexes` deduplicates those indices in a `HashSet`.
  With `Point2D` hashing on `x` alone, every column of the grid collapsed onto
  a single cell: merging a 2x2 surface with itself produced **two** indices
  instead of four, and `merge_axis_interpolate` then worked from the truncated
  axis. It now keeps all of them. Anything asserting the narrow result will
  see more indices, and more interpolated points, than before.
- **A `BTreeSet` of points no longer depends on how it was built.**
  `BTreeSet::from_iter` (and therefore `collect`) sorts and then deduplicates
  adjacent elements with `PartialEq`, while `insert` deduplicates with `Ord`.
  While those two disagreed, the same points produced different sets by
  different routes: four `Point3D` stacked on one xy-coordinate collected to a
  set of **one** and inserted to a set of **four**. Collecting now keeps every
  distinct point, so `Surface::get_curve` returns the whole projection — an
  `n`-by-`m` grid yields up to `n * m` points where it used to yield `n`,
  having silently dropped all but the greatest ordinate per abscissa. Curves
  collected from an option chain are unaffected, their abscissae being unique
  strikes.
- `Surface::bilinear_interpolate` now reports `"Invalid quadrilateral"` for
  points stacked on one xy-coordinate. That check existed but was unreachable:
  the collapse above reduced such a set to a single point first, so the
  `"Need at least four points"` guard answered instead.
- Membership probes against `Point2D` / `Point3D` are now exact. Code doing
  `points.contains(&probe)`, `points.iter().any(|p| p == &probe)` or
  `HashSet`/`BTreeSet` lookups used to match any point sharing the probe's
  leading coordinates; it now matches only the point itself. Nothing in the
  crate relied on the loose behaviour except `merge_indexes` above, but a
  downstream caller might.
- **`OptionData::apply_spread` widens a thin quote instead of withdrawing it**
  (#439). A contract whose mid sat below one full spread used to lose `bid`,
  `ask` **and** `middle`. Both sides are now quoted around the mid — `mid ±
  half_spread` — and floored at one tick (`10^-decimal_places`); a supplied mid
  is kept, held inside the widened quote rather than cleared. Only a quote with
  no mid and no two-sided book is still erased.
- **Chains keep their cheap strikes, so row counts change.**
  `OptionChain::build_chain` stops generating strikes once both wings come back
  unpriced, so the erasure above also truncated the chain: a build with
  `chain_size = n` returned `n + 1` rows and now returns the full `2n + 1`.
  Downstream assertions on row counts, and anything iterating a chain, will see
  the wings that used to disappear as they decayed.
- `apply_spread` no longer changes the previous quote for a mid at or above the
  tick. Below the tick it does deviate deliberately: a bid that used to round
  to `0.00` is now floored at one tick, since a zero bid is not a market.
- A `decimal_places` beyond `Decimal`'s maximum scale used to panic through
  `Positive::round_to`; `apply_spread` now leaves the quotes untouched and logs.

### Fixed

- **Aggregating greeks aborted the process on two legs whose theta had
  vanished, and returned a silently-wrong total when one such leg met an
  ordinary one** (#469). `Greeks::greeks` and the twelve per-greek aggregators
  summed their legs with the raw `Decimal` operator, which panics on overflow.
  For eleven of the twelve that needs an extreme book; for `alpha` it does not.
  `alpha` returns `Decimal::MAX` as a documented sentinel when a leg's theta
  rounds to zero while its gamma does not — an at-the-money contract at a
  quantity around `1e-27` is enough — so two such legs aborted on
  `Decimal::MAX + Decimal::MAX`.

  The quieter half is the one worth knowing about. A single sentinel leg beside
  an ordinary one did *not* abort: `Decimal::MAX + x` for `|x| < 0.5` rescales
  the addend down to zero and rounds straight back to `Decimal::MAX`, so the
  addition succeeds, the running total stands still, and the ordinary leg's
  alpha is dropped without a trace. `checked_add` cannot detect that, because
  nothing overflowed. Both halves are
  now a `GreeksError` naming the leg that carried the sentinel, raised by an
  explicit guard ahead of the arithmetic rather than by the arithmetic itself.
  The remaining twenty-two accumulations moved to checked addition, so an
  ordinary sum that leaves the representable range is reported instead of
  aborting; every sum that stayed inside it is unchanged, `Decimal::checked_add`
  and `Add::add` being the same operation but for the overflow arm.

  **A lone sentinel leg still returns the sentinel**, as does a sentinel beside
  legs whose own alpha is zero, because no contribution is lost in those sums.
  `OptionData::greeks_snapshot` therefore behaves exactly as before: it computes
  the twelve greeks for one option and maps an `alpha` of `Decimal::MAX` to
  `None` on the wire. Chain snapshots have not started failing. The sentinel
  itself is unchanged, and so is the single-option `greeks::alpha`.
- **Every arithmetic Asian option with `r = q` was mispriced** (#454). The
  Turnbull-Wakeman second moment short-circuited its removable `b = 0`
  singularity to `M2 = S² e^{σ² T}`, which is `E[S_T²]`: the second moment of
  the *terminal* price, not of its average. The `b → 0` limit of the double
  integral the function actually implements is
  `M2 = (2 S² / (σ² T²)) [ (e^{σ² T} − 1) / σ² − T ]`. The stand-in handed the
  moment matching `σ_adj = σ` where the average carries `σ_adj ≈ σ / √3`, so
  the price came out far too high and the branch was discontinuous: a one-year
  ATM call on `S = K = 100`, `σ = 20%`, `r = q = 4%` returned
  **7.6532330880**, against `4.4308752837` and `4.4308753262` for a carry a
  billionth either side. It now returns **4.4308753052**, which agrees with
  quadrature on the defining integral to `3e-10` and with both neighbours to
  `2.2e-8`. A forward-priced or fully-carried underlying is an ordinary
  contract, so every pinned `r = q` Asian value moves.
- **A zero-volatility Asian option priced off the terminal forward instead of
  the average** (#447). Both kernels short-circuited `σ = 0` to
  `(S e^{bT} − K)⁺ e^{−rT}`, but a deterministic path still has to be
  averaged: the geometric mean of `S e^{b t}` over the window is `S e^{bT/2}`
  and the arithmetic mean is `S (e^{bT} − 1) / (bT)`, which are also the
  `σ → 0` limits of the Kemna-Vorst and Turnbull-Wakeman formulas the branches
  stand in for. A one-year ATM call on `S = K = 100`, `r = 5%`, `q = 0`
  returned **4.8770575499** from both kernels and now returns
  **2.4080487528** (geometric) and **2.4182085485** (arithmetic). The old
  value was only correct at `b = 0`, where every average collapses to `S`.
- **The simple chooser was priced with the wrong formula (#448).**
  `chooser_black_scholes` discounted its two `y` legs at the choice date `t`
  instead of at expiry `T`, and built `y1` from `b*t` where Rubinstein (1991)
  has `b*T + σ²*t/2`, so every simple chooser came out too expensive. On
  Haug's worked example (S = K = 50, t = 0.25, T = 0.5, r = b = 0.08,
  σ = 0.25; reference **6.1071**) the function returned **6.524120** and now
  returns **6.107077**: 6.8% high before, within 1e-14 of the closed form now.
  The module doc already stated the corrected formula; the code did not
  implement it.
- With no diffusion left to run to the choice date (`σ√t` collapsing to zero)
  the surviving branch is now picked on the sign of `ln(S/K) + b*T`, the
  forward moneyness at expiry, rather than on `ln(S/K)`. A chooser whose spot
  sits below the strike but whose forward sits above it prices as the call,
  not as the put. A zero numerator, which that limit used to report as an
  undefined `0 / 0`, is exactly forward parity — where call and put are worth
  the same — and now returns that common value instead of an error (#448).
- **Reachable panic in the lower break-even of eight strategies.**
  `strike - credit` was computed with the raw `Positive - Decimal` operator,
  which panics when the credit (or, on a long structure, the debit) exceeds the
  strike — reachable from the optimizers as soon as a chain keeps its cheap
  wings. `ShortStraddle`, `ShortStrangle`, `IronCondor`, `IronButterfly`,
  `LongStraddle`, `LongStrangle`, `BearPutSpread` and `ShortButterflySpread` now
  share `lower_break_even`, which floors at zero. A lower break-even of `0.00`
  means the strategy has none (#439).

## [0.20.0] - 2026-08-28

Two things land together. The option chain now carries the full twelve-greek set
per strike, and every greek in the crate returns the sensitivity of the
**position** rather than of one long contract. Both are **breaking**: the first
for anyone constructing `OptionData` with an exhaustive struct literal, the
second for anyone who was compensating for the old unsigned behaviour. See
*Migration*.

### Added

- `OptionData::greeks_call` and `OptionData::greeks_put`, each an
  `Option<GreeksSnapshot>` carrying all twelve greeks for that option style, so
  consumers no longer convert through `TryFrom<&OptionData> for Options` and
  recompute on every read. Both are computed for **one long contract**: a
  consumer holding a short negates all twelve, since every value is a derivative
  of the long position value.
- `OptionData::calculate_greeks`, which populates both snapshots and refreshes
  the `delta_call`, `delta_put` and `gamma` mirror fields together.
- `OptionChain::update_greek_snapshots`, the full-set counterpart to
  `update_greeks`.
- `OptionChainBuildParams::with_greek_snapshots`, opting a chain build into the
  snapshots. Off by default.
- `impl From<Greek> for GreeksSnapshot`.
- `alpha` is now re-exported from `crate::greeks`, alongside the other eleven.

### Fixed

- **Breaking behaviour change.** Every Black-Scholes/Merton greek in
  `greeks::equations` now respects `Side`. Only `delta` did before, so any
  strategy holding a short leg reported gamma, theta, vega, rho and the
  higher-order greeks with the sign of the equivalent long position, next to a
  delta that was signed correctly. A short-premium position looked as though it
  was losing to time decay when it was collecting it. `Options::quantity` is
  `Positive` and can never carry direction, so `Side` is the only carrier of it
  (#428).
- **Breaking behaviour change.** The Black-76 and Garman-Kohlhagen greek
  families now use the same position-sign convention. Previously only their
  deltas respected `Side`; a short futures or FX option therefore reported
  gamma, vega, theta and rho with the sign of the equivalent long. All greeks in
  both families are now signed by `Side` exactly once and scale linearly with
  `quantity` (#436).
- `PortfolioGreeks::from_positions` re-applied `quantity * sign` to greeks that
  already carried both, which squared the position size and cancelled the sign.
  `DeltaAdjustment` applied the same second sign to delta.
- `rho_d` and `vanna` returned an error at expiry where their ten siblings
  returned zero, so `Greeks::greeks()` lost the entire set for any expired
  option. Both now return zero.
- The non-European fallbacks in `delta` and `gamma` returned a per-contract long
  value, dropping both side and quantity, because the numerical engine prices
  through an absolute value.

### Changed

- `Greeks::greeks()` computes the shared Black-Scholes intermediates once per
  option rather than once per greek, which makes it about 6.5x faster and cuts
  the cost of a chain build with greek snapshots by about 4.7x. Every value is
  unchanged (#431).
- `GreeksSnapshot` no longer sets `#[serde(deny_unknown_fields)]`. It is a wire
  type now, and adding a thirteenth greek must not break deserialization for
  consumers built against an older version.
- `OptionData::set_volatility` and `set_extra_params` drop the stored snapshots
  rather than leave them stale against changed pricing inputs, and
  `OptionChain::update_expiration_date` does the same.

### Migration

`OptionData` gained two public fields. Any exhaustive struct literal needs
`..Default::default()` or the two new fields; `OptionData::new` is unchanged and
needs no edit. Serialized chains are unaffected in both directions: the new
fields are skipped when absent, and a payload written before this release
deserializes with both set to `None`.

On the sign convention, any consumer that was compensating by negating the
unsigned greeks itself must stop, in all three families. A long and a short of
the same contract now net to exactly zero. `alpha` is the one exception and is
unchanged, being the ratio `gamma / theta`, which a short negates in both terms.
`OptionData::greeks_call` and `greeks_put` are also unaffected, being built
through `get_option(Side::Long, style)` and so per-long-contract by
construction.

## [0.19.1] - 2026-08-28

Correctness and housekeeping follow-up to the cost-of-carry fix in 0.19.0. No
API changes.

### Fixed

- `vomma` and `veta` applied the position quantity twice, because `vega`
  already carries it. Both are first derivatives of vega and are linear in
  position size, so a 10-lot position reported 10x the true value and the error
  propagated into `PortfolioGreeks`.
- `d1`'s documentation still described its third argument as the risk-free rate
  after it was renamed to `carry_rate`, stating two different meanings for the
  same argument in one doc block. `d1` and `d2` are public and in the prelude,
  so a caller following it disagreed with `option.delta()` by about 10%.
- Four stale `charm` expectations meant two tests over the identical portfolio
  asserted different numbers, both within 2% of their tolerance.
- `test_dividend_high_q_carry_regression` pinned eleven greeks at `1e-28`, far
  tighter than the f64 normal CDF behind them supports. Relaxed to `1e-12`.

### Changed

- The `format_check` CI job ran `make fmt`, which formats rather than checks and
  exited 0 whatever the input, so no pull request was ever format-checked. It
  now runs `make fmt-check`.

## [0.19.0] - 2026-08-17

Dependency refresh: every dependency moved to its latest stable minor, and the
three in-house crates jumped a breaking release each. It is **breaking** for
consumers — see *Migration*.

### Added

- **`simulation::expanding_window_vols` is public** (also re-exported from the
  prelude): the per-step, look-ahead-free volatility estimator that
  `walk_steps` / `walk_steps_par` use for `WalkType::Historical`, whose
  volatility is not a model parameter and has to be estimated from the series.
  A consumer that generates a historical path itself — with
  `WalkTypeAble::generate_with_vol`, pricing its own chains — can now use the
  same estimate instead of falling back to a constant volatility or keeping a
  second copy of the mathematics. The contract is documented and tested:
  one estimate per price, element `i` a function of `prices[..=i]` only
  (extending the series leaves earlier estimates untouched), the first two
  indices backfilled with the first computable estimate, `Ok(None)` below three
  prices, and the last estimate equal to whole-series `constant_volatility`.
  (#423, downstream OptionChain-Simulator#63)

### Changed — breaking

- `positive` `0.5` -> `0.6`, `expiration_date` `0.2` -> `0.3`,
  `option_type` `0.1` -> `0.3`. All three appear throughout this crate's public
  API, so consumers must move in the same step.
- **JSON wire format**: `positive` 0.6 serialises `Positive` as the exact
  decimal in a *string* (`"42.5"` instead of `42.5`), so every serialised type
  carrying a `Positive` changes shape: `OptionData`, `OptionChainBuildParams`,
  `OptionSeries`, `PnL`/`PnLMetricsDocument`, `Step`/`Xstep`/`Ystep`,
  `StrategyRequest`, and the rest. Deserialisation still accepts the old
  numeric form, so stored documents keep loading; anything asserting on the
  serialised text has to be updated.
- **`utils::others::calculate_log_returns` returns `Vec<Decimal>`**, not
  `Vec<Positive>`. A log return is signed — `ln(105/110) < 0` — and the old
  signature could not represent it. `Positive::ln` itself now returns
  `Decimal` for the same reason.
- **Exotic payloads are `Positive`**, following `option_type` 0.3:
  `OptionType::Barrier { barrier_level, rebate }`,
  `Bermuda { exercise_dates }`, `Chooser { choice_date }`,
  `Cliquet { reset_dates }`, `Spread`/`Exchange { second_asset }`,
  `Quanto { exchange_rate }`, `Power { exponent }`. A negative or non-finite
  value is now unrepresentable rather than an error at pricing time, so
  `power_black_scholes` no longer has a negative-exponent rejection path and
  `barrier_black_scholes` no longer returns `PricingError::NonFinite` for its
  barrier level or rebate.
- `OptionType` and every sub-enum (`AsianAveragingType`, `BarrierType`,
  `BinaryType`, `LookbackType`, `RainbowType`) are `#[non_exhaustive]`
  upstream. Matches in this crate gained fallback arms: payoffs degrade to the
  plain intrinsic value, and the Black-Scholes kernels return
  `PricingError::UnsupportedOptionType` / `PricingError::other` rather than
  failing to compile against a future variant.

### Changed

- `rust_decimal` `1.41` -> `1.42`, `itertools` `0.14` -> `0.15`,
  `zip` `6.0` -> `8.6`, `uuid` `1.23` -> `1.24`, `utoipa` `5.4` -> `5.5`,
  `tokio` `1.52` -> `1.53`; dev-only `mockall` `0.14` -> `0.15`,
  `tempfile` `3.23` -> `3.27`, `proptest` `1.5` -> `1.11`. Every other
  dependency was already at its latest stable minor.
- `Positive::INFINITY` is deprecated upstream in favour of `Positive::MAX`
  (the value was always `Decimal::MAX`, never an infinity). The unlimited-upside
  strategies (`long_call`, `short_call`, the straddles and strangles,
  `call_butterfly`) and `ProfitRange`/`Position` now use `MAX`, so an unbounded
  max-profit renders as `79228162514264337593543950335` instead of `inf`.
- Deprecated conversions replaced: `Positive::to_i64`/`to_u64`/`to_usize` gave
  way to their `*_checked` forms, so an out-of-range day count, plot bound or
  chain-size counter saturates instead of panicking.
- `Positive::sub_or_zero` is deprecated upstream; the zero floor is now taken
  once, in `model::utils::sub_floor_zero`, on top of the checked
  `sub_or_none`. Behaviour is unchanged for the bid/ask spread, the skew scan
  and the OU drift term.
- Comparisons of the form `decimal > Positive::ZERO.into()` became
  `decimal > Positive::ZERO`: `positive` 0.6 adds
  `PartialOrd<Positive> for Decimal`, which made the inferred `.into()`
  ambiguous.

### Migration

```rust
// log returns are signed
- let r: Vec<Positive> = calculate_log_returns(&prices)?;
+ let r: Vec<Decimal>  = calculate_log_returns(&prices)?;

// exotic payloads carry `Positive`
- OptionType::Barrier { barrier_type, barrier_level: 95.0, rebate: None }
+ OptionType::Barrier { barrier_type, barrier_level: pos_or_panic!(95.0), rebate: None }

// unbounded profit
- Ok(Positive::INFINITY)
+ Ok(Positive::MAX)

// serialised prices are strings
- {"strike_price": 100.0}
+ {"strike_price": "100"}
```

### Fixed

- **The Security Audit workflow is green again** (#422). It had been red on
  `main` since 2026-08-05 on five advisories. The dependency refresh above
  resolves three of them outright — `crossbeam-epoch` (RUSTSEC-2026-0204),
  `quinn-proto` (RUSTSEC-2026-0185) and `rustls-webpki` (RUSTSEC-2026-0104) now
  resolve to patched releases. The remaining two are unreachable and carry
  documented waivers in `.cargo/audit.toml`, each with rationale, owner and a
  2027-02-15 review date:
  - RUSTSEC-2026-0235 (`rkyv` 0.7.46) — lockfile-only, an optional dependency
    of `rust_decimal` that this crate never enables.
  - RUSTSEC-2025-0119 (`number_prefix` 0.4.0, unmaintained) — likewise
    lockfile-only, via `indicatif`.
  - RUSTSEC-2025-0134 (`rustls-pemfile` 1.0.4, unmaintained) — a *build*
    dependency of `plotly_static` (through `webdriver-downloader` and
    `reqwest` 0.11), reachable only with `static_export`; it downloads a
    webdriver on the build machine and is never linked into the library.
  With those waived, the workflow now runs with `denyWarnings: true`, so a new
  unmaintained or unsound dependency fails the gate instead of passing as a
  warning.

### Housekeeping

- `.cargo/audit.toml` added, mirroring the `positive` crate's policy file: one
  documented waiver per advisory (rationale, owner, review date) and
  `informational_warnings` on, so unmaintained/unsound/notice advisories are
  reported rather than dropped. See *Fixed* above for the entries.
- The version strings in the crate-level docs (and therefore in the generated
  `README.md`) say `0.19.0`; they had been left at `0.18.0` through the 0.18.1
  release.
- `cargo test` no longer spawns a browser. Five visualization tests needed a
  WebDriver whose major version matches the installed browser (PNG/SVG export
  through `plotly_static`), one really did hand a chart to the default browser
  despite a comment claiming otherwise (`OutputType::Browser` calls
  `Plot::show()`), and four doc examples wrote a PNG when executed. The tests
  are now `#[ignore]`d with a reason and the doc examples are `no_run`, so they
  still compile and are still runnable on demand: `make test-visual`, a new
  target that runs exactly the ignored set. The `make test` recipe also passes
  `--features static_export,plotly`; the comma was missing, so `plotly` was
  being parsed as a test-name filter rather than a feature.

## [0.18.1] - 2026-08-07

### Changed

- `statrs` bumped `0.18` -> `0.19` (pulling `nalgebra 0.35` / `simba 0.10.2`), which
  drops the unmaintained `paste` crate (RUSTSEC-2024-0436) from the dependency graph
  entirely — `cargo tree -i paste --all-features` now prints nothing. The `statrs`
  surface this crate uses (`distribution::{Normal, ContinuousCDF}`) is unchanged
  between the two lines, so no code changes. Requested by the Layer V fleet, where
  this chain was the only path bringing `paste` into every downstream tree (#420,
  Layer-V/common-rs#266). (#420)

## [0.18.0] - 2026-07-12

Overhaul of the option-chain walk generators (issues #404-#411, PRs
#412-#419): one shared, error-generic walk driver; unified generator
contracts; per-step stochastic volatility propagated into rebuilt
chains; smile-preserving rebuilds; rayon-parallel per-step builds
(25-step chain walk: 12.46 ms -> 0.88 ms, ~14x); and deterministic
multi-step behavioral test coverage.

### Performance

**Parallel per-step chain builds** (#411): new
`simulation::walk_steps_par` — a parallel variant of `walk_steps` with an
identical contract that fans the per-step y-value construction out to the
rayon thread pool (the walk itself, per-step volatilities and x-step
sequence stay serial and deterministic). `generator_optionchain` and
`generator_optionseries` now use it; output is identical to the serial
driver for the same inputs. Criterion (Apple Silicon):
`generator_optionchain` 10 steps 2.59 ms → 0.47 ms (−82%), 25 steps
7.01 ms → 0.88 ms (−87%).

### Testing

**Behavioral test coverage for the walk generators** (#410): a
deterministic ramp walker (`simulation::walk_test_support`, test-only)
replaces RNG-driven size-1 smoke tests. New multi-step tests pin: exact
price propagation, y-index increments, per-step time-to-expiry decay,
rebuilt-chain expiration tracking, ATM IV tracking of the walk
volatility, Historical walks replaying the provided prices with the
expanding-window estimate, truncation exactly at expiration, series
aging (and the walk stopping once every series expiration has passed),
empty-walker outputs and `size = 0` across all three generators.

### Fixed

**IV smile preserved across chain rebuilds** (#409):

- `OptionChain::to_build_params` now fits `skew_slope` / `smile_curve`
  from the chain's own per-strike IVs by least squares (the exact inverse
  of the parametric model `build_chain` uses), instead of resetting them
  to the `SKEW_SLOPE` / `SKEW_SMILE_CURVE` constants. Round-tripping a
  real market chain previously flattened a ~7 vol-point smile to under
  0.1 vol-points; the smile shape now survives rebuilds (and therefore
  survives the whole simulated walk in `generator_optionchain`). The
  constants remain as fallback when the fit is underdetermined.
- `adjust_volatility` now caps adjusted per-strike IVs at 200% instead of
  100%, so legitimate high-vol wings survive; the cap logs at `debug!`
  when it engages.

### Added

**Per-step volatility paths** (#408):

- `simulation::WalkPath` — walker output carrying `prices` plus optional
  per-step ANNUALIZED `vols`.
- `WalkTypeAble::generate_with_vol()` plus `garch_with_vol` /
  `heston_with_vol` / `custom_with_vol` / `telegraph_with_vol` provided
  methods. The stochastic-volatility models already simulated a vol path
  internally and discarded it; it is now returned. The built-in dynamics
  live in public kernels (`garch_walk`, `heston_walk`, `custom_walk`,
  `telegraph_walk`) shared by both method families; the price-path
  methods remain standalone override points (overriding one method never
  changes the other's default). Implementors overriding a price-path
  method must also override the `*_with_vol` sibling (wrapping their own
  dynamics or composing the public kernel) for the walk generators —
  which consume `generate_with_vol` — to see their dynamics.

### Changed

**Chains/series rebuilt with per-step volatility** (#408):

- Under `Garch` / `Heston` / `Custom` / `Telegraph` walks,
  `generator_optionchain` and `generator_optionseries` now stamp each
  rebuilt chain with the simulated volatility prevailing at that step
  instead of freezing the walk's initial volatility for the whole walk.
- `Historical` walks now use an expanding-window volatility estimate that
  only uses prices up to each step (the previous full-sample estimate had
  look-ahead bias). The estimate at the final step matches the old
  full-sample value.
- Per-step IVs are capped at 100% before stamping a chain (`build_chain`
  rejects IV > 1; simulated vol paths can spike above it).

**Single generic walk driver** (#407):

- `simulation::walk_steps` — the shared dispatch/advance/build loop behind
  all step generators; custom generators can now be written as a closure
  over it instead of forking a 100-line function.
- `WalkType::volatility()` — accessor for the variant's volatility
  parameter (`None` for `Historical`).
- `WalkTypeAble::generate()` — provided method dispatching to the walk
  method matching `params.walk_type`; adding a new `WalkType` variant now
  requires touching only the enum and the trait, not every generator.

### Changed

**`generator_positive` relocated** (#407): it never depended on option
chains, so it moved from `chains::` to `simulation::`. The old path
`chains::generator_positive` remains as a deprecated re-export; the
prelude now re-exports the new location (no deprecation warnings for
prelude users).

**Unified walk-generator contracts** (#406) — the three walk generators
(`chains::generator_optionchain`, `chains::generator_positive`,
`series::generator_optionseries`) now share one documented contract.
Behavior changes observable from the public API:

- `WalkType::Historical` with fewer prices than `WalkParams::size` now
  returns `ChainError::Simulation(InsufficientHistoricalData)` from ALL
  three generators. Previously `generator_optionchain` and
  `generator_optionseries` silently returned a 1-step walk that was
  indistinguishable from a legitimate size-1 walk.
- `generator_positive` no longer panics when a custom walker returns an
  empty vector; it returns the initial step only, like the other two.
- Walks longer than `WalkParams::size` are now truncated at runtime by
  all three generators (previously chains generators only checked this
  with a `debug_assert!`, a no-op in release builds).
- A step-advance failure other than reaching expiration is now
  propagated as an error instead of silently truncating the walk.
- `generator_optionseries` now ages the series along the walk: each
  step's series expirations are reduced by the elapsed walk time and
  expired entries are dropped (previously rebuilt series kept their
  original expirations for the whole walk). The walk ends early once
  every expiration has passed.
- The undocumented `0.20` volatility fallback in
  `generator_optionseries` was removed (dead code under the unified
  contract).

## [0.17.2] - 2026-04-26

Release adding two new closed-form pricing models:
- **Black-76** (Black 1976) for European options on futures and forwards.
- **Garman–Kohlhagen** (1983) for European FX options.

`0.17.0` and `0.17.1` were preparatory iterations of this work
(`0.17.0` was never published; `0.17.1` shipped to crates.io with a
partial subset). `0.17.2` is the first version that ships both models
together. `PricingEngine` is `#[non_exhaustive]` (semver-major from the
0.16.x line) and the two new variants are appended at the tail of the
enum so existing discriminants are preserved.

### Added

**Black-76 model** (Black 1976):
- `pricing::black_76`: closed-form `black_76(option) -> Result<Decimal, PricingError>`
  for European options on futures / forwards. Reuses the existing `d1`
  / `d2` / `big_n` helpers; `Decimal` end-to-end via `d_mul` / `d_sub`;
  `tracing::instrument` on the entry point. Only `OptionType::European`
  is supported — American, Bermuda and exotics return
  `PricingError::UnsupportedOptionType`.
- `pricing::Black76` trait with default `calculate_price_black_76`
  (mirrors `BlackScholes`).
- `pricing::PricingEngine::ClosedFormBlack76` variant + dispatch from
  `price_option`.
- `greeks::utils::calculate_d_values_black_76` `pub(crate)` helper.
- `examples/examples_pricing/src/bin/black_76.rs`: runnable demo
  (Hull canonical example, ITM commodity-futures call, unified-API
  dispatch, short-side sign convention).

**Garman–Kohlhagen model** (Garman & Kohlhagen 1983):
- `pricing::garman_kohlhagen`: closed-form
  `garman_kohlhagen(option) -> Result<Decimal, PricingError>` for
  European options on FX spot rates. Structurally identical to BSM
  with `q = r_f`; the implementation delegates to `black_scholes`
  after type validation, guaranteeing bit-exact equivalence (verified
  to `1e-9` in the tests).
- `pricing::GarmanKohlhagen` trait with default
  `calculate_price_garman_kohlhagen` (mirrors the `BlackScholes`
  trait pattern).
- `pricing::PricingEngine::ClosedFormGK` variant + dispatch from
  `price_option`.
- `examples/examples_pricing/src/bin/garman_kohlhagen.rs`: runnable
  demo (Hull canonical USD/GBP, ITM EUR/USD with FX parity check,
  unified-API dispatch, symmetric-rate degenerate case).

**Infrastructure updates**:
- `examples/examples_pricing/`: new workspace member with binaries for
  both models.
- `lib.rs` mermaid: `Forward-Priced` subgraph routing
  `black_76 -> {Future, Forward}`; new `FX / Currency` subgraph routing
  `garman_kohlhagen -> FX Spot`.

### Changed

- `pricing::PricingEngine` is now `#[non_exhaustive]` so future engine
  variants do not require a new major bump.
- `pricing::mod.rs` Core Models / Model Selection Guidelines /
  Performance Considerations now include both Black-76 and
  Garman–Kohlhagen with explicit field mapping documentation.
- `financial_types` bumped to `0.2.2` (adds `UnderlyingAssetType::Future`
  and `UnderlyingAssetType::Forward`).
- `PricingError` and `GreeksError` pass-through in closed-form dispatch
  (BS, Black-76, GK) for full error-variant fidelity.

## [0.16.5] - 2026-04-20

Documentation-only release. Refresh the crate-level rustdoc and
mermaid diagrams so they describe the 0.16.x quality discipline
(checked arithmetic, `NonFinite` guards, `NonZeroUsize` step counts,
`deny(indexing_slicing)` / `deny(missing_docs)`, structured tracing,
deterministic RNG, pricing-identity regression tests) and the
post-migration example layout.

### Changed

- `src/lib.rs`: new "Quality & Discipline (0.16.x)" section with the
  full list of crate-wide invariants; new **Arithmetic-Error Cascade**
  mermaid diagram (`d_add` / `d_sum_iter` / `finite_decimal` →
  `DecimalError::Overflow` / `PricingError::NonFinite` / …); new
  **Observability** diagram showing the five instrumented public hot
  paths.
- Testing section updated to the current count (3760 unit + 205
  doctest) and mentions the seeded-RNG helper and the pricing-identity
  regression tests.
- Examples section lists every sub-crate under `examples/` and the
  correct `--manifest-path=` invocation (with a note about the
  demo-friendly hourly grid on simulation-heavy examples).
- `README.tpl` passthrough regenerates `README.md` with the updated
  module docs.

[Unreleased]: https://github.com/joaquinbejar/OptionStratLib/compare/v0.21.2...HEAD
[0.21.2]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.21.2
[0.21.1]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.21.1
[0.21.0]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.21.0
[0.20.0]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.20.0
[0.19.1]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.19.1
[0.19.0]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.19.0
[0.18.1]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.18.1
[0.18.0]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.18.0
[0.17.2]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.17.2
[0.16.5]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.5

## [0.16.4] - 2026-04-20

### Changed

- Bump workspace dependencies: `rust_decimal` 1.40 → 1.41,
  `rayon` 1.11 → 1.12, `uuid` 1.19 → 1.23, `tokio` 1.43 → 1.52.

### Fixed

- Repair three doctests broken by the `NonZeroUsize` migration
  in 0.16.0: `pricing` module-level examples for `telegraph` and
  `monte_carlo_option_pricing` now wrap literal step / simulation
  counts with `nz!(..)`; the `utils::deterministic_rng` doctest
  uses `rand::RngExt` for `random::<u64>()`.

[0.16.4]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.4

## [0.16.3] - 2026-04-20

Hot-fix targeting the runnable-example audit.

### Fixed

- Simulation-heavy demo binaries
  (`long_call_strategy_simulation`, `short_put_strategy_simulation`,
  `position_simulator`, `strategy_simulator`, `random_walk_chain`)
  now use an hourly grid over the week instead of a minute-level
  grid (10 080 steps × 100 simulations, 43 200 for the chain
  walker). The code paths are exercised identically; the demos
  just run in a few seconds in debug mode rather than the minutes
  the example runner timed out on. (#385, #386)
- `examples_volatility::test` brute-force scan cut from
  1 000 000 to 10 000 iterations — the example is a demo, not a
  local benchmark. (#386)

[0.16.3]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.3

## [0.16.2] - 2026-04-19

Hot-fix for two panic / I/O bugs caught while running every example
binary under `examples/`.

### Fixed

- Strategy P&L / break-even arithmetic crossed the `Positive`
  boundary without a guard and panicked mid-optimizer-scan
  (`Positive invariant broken in add_decimal / sub`) in:
  - `CallButterfly::update_break_even_points`,
  - `CallButterfly::get_profit_area`,
  - `LongButterflySpread::update_break_even_points`,
  - `BullPutSpread::get_max_loss`.
  All four sites now lower to `Decimal`, then rewrap via
  `Positive::new_decimal(..)` — invalid candidates are dropped
  cleanly or surfaced as typed `StrategyError` instead of
  panicking. Unblocks `strategy_call_butterfly_best_{area,ratio}`,
  `strategy_long_butterfly_spread_best_{area,ratio}`,
  `strategy_call_butterfly_delta`, and
  `strategy_bull_put_spread_extended_delta` examples. (#387)
- `examples_chain::async_chain_ops` was passing a filename where a
  directory was expected and failing with `ENOENT`; it now writes
  under `std::env::temp_dir()/optionstratlib-async-chain-ops` and
  creates the directory up front. (#388)
- `examples_chain::creator` pointed at a Germany-40 JSON file that
  was never committed; now reads the one that ships in
  `examples/Chains/`. (#388)

[0.16.2]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.2

## [0.16.1] - 2026-04-19

Hot-fix for CI flakiness introduced by sub-day `ExpirationDate`
arithmetic in test fixtures, plus a doc-link warning.

### Fixed

- Chain test fixtures (`create_test_option_chain`) now use
  `get_x_days_formatted(30)` instead of `get_tomorrow_formatted()`.
  `Actual365Fixed::day_count` in `expiration_date 0.2.0` truncates
  to integer days, so tomorrow's fixed 18:30 UTC expiry evaluated
  after that time collapsed to `t = 0` and broke every
  Black-Scholes-driven axis on the chain curve/surface tests
  (`test_curve_multiple_axes`, `test_curve_price_short_put`,
  `test_surface_different_greeks`, `test_vanna_surface`). 30 days
  puts every test well above the integer-truncation boundary.
- `constants.rs`: `MAX_NEWTON_ITER` no longer links to the private
  `MAX_ITERATIONS_IV` — the doc just names the crate-private
  counterpart in prose, so `cargo doc` emits zero warnings again.

[0.16.1]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.1

## [0.16.0] - 2026-04-19

Breaking release. Focus: panic-free core, arithmetic discipline,
typed errors everywhere, and a crate-wide discipline pass over
attributes, docs, and test hygiene.

### Added

- Checked `Decimal` helpers `d_add` / `d_sub` / `d_mul` / `d_div`
  plus `d_sum` and the iterator-based `d_sum_iter` in
  `src/model/decimal.rs`. Every monetary-path kernel now routes
  through them instead of raw `+ - * /`, surfacing `DecimalError::Overflow`
  with an operation tag. (#335, #336, #337, #338, #372)
- Domain-specific `NonFinite { context, value }` variants on
  `PricingError`, `GreeksError`, `VolatilityError`, and
  `SimulationError` plus the crate-private `finite_decimal(f64)`
  guard used at every `f64 → Decimal` boundary. (#336, #337, #338)
- Public `tracing::instrument` on hot paths: `pricing::black_scholes`,
  `pricing::monte_carlo_option_pricing`, `pricing::price_binomial`,
  `volatility::utils::implied_volatility`, and
  `strategies::base::Optimizable::{get_best_ratio, get_best_area}`. (#342)
- `utils::deterministic_rng(seed)` plus
  `DETERMINISTIC_RNG_DEFAULT_SEED` — canonical entry point for
  reproducible Monte-Carlo / simulation tests. (#344)
- Deterministic regression tests under
  `tests/unit/pricing/identities_test.rs` covering put-call parity,
  CRR binomial convergence to Black-Scholes, and Greek
  sanity identities (`Γ_c == Γ_p`, `V_c == V_p`,
  `Δ_c − Δ_p ≈ e^{-qT}`). (#345)
- `CHANGELOG.md` following Keep a Changelog 1.1.0. (#346)

### Changed

- Breaking: step / simulation counts on `price_binomial`,
  `monte_carlo_option_pricing`, and related kernels are now
  `NonZeroUsize` so zero is structurally invalid at the type
  level. (#337)
- Breaking: many public surfaces now return
  `Result<T, concrete_error>` instead of panicking; `unsafe`
  blocks have been removed from the core in favour of typed
  guards. (#333, #334, #335, #338)
- Canonical `#[derive]` ordering
  (`Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
  Default, …, Serialize, Deserialize, ToSchema`), `#[repr(u8)]`
  on small stable enums, `#[serde(deny_unknown_fields)]` on
  input DTOs, and `#[serde(rename_all = "snake_case")]` on
  public-facing enums unless an existing wire contract
  forbids it (e.g. `BasicAxisTypes` keeps Pascal case). (#340)
- `#[inline]` applied on hot-path helpers and public entry
  points, `#[inline(never)]` on multi-arg builders, and
  `#[cold] #[inline(never)]` on every error constructor across
  `src/error/*`. (#339)
- `CustomStrategy::calculate_profit_at` no longer allocates a
  `Vec<Decimal>` per invocation; aggregates via `try_fold` + `d_add`. (#372)

### Fixed

- Doc-coverage floor: crate-level
  `#![deny(missing_docs, rustdoc::broken_intra_doc_links)]`
  with every previously-bare `pub` item now documented, and
  broken intra-doc links (e.g. `DecimalError::Overflow` →
  `crate::error::DecimalError::Overflow`) repaired. (#343)
- Unchecked `[]` indexing in production code migrated to
  `.get(..).ok_or_else(..)` on the highest-risk paths
  (`OptionChain` file-name / CSV readers, binomial-root lookup
  in `Option::binomial_price`) and
  `#![deny(clippy::indexing_slicing)]` enforced crate-wide
  with scoped, documented escapes on the remaining modules
  as follow-up work. (#341)

### Internal

- `#[must_use]` applied across the pure / builder public
  surface to catch discarded results at compile time.

[0.16.0]: https://github.com/joaquinbejar/OptionStratLib/releases/tag/v0.16.0
