# Changelog

All notable changes to **OptionStratLib** are documented in this file.

The format is based on [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

Upgrading from 0.21: start with the [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md), which
groups the changes below by the workflow that replaces them. 0.22 keeps no
compatibility with 0.21. The [0.22.0 release
notes](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/release/0.22/RELEASE-NOTES.md)
summarize the release.

### Changed — breaking

- **Walkers and `Simulator::new` generators must be `Send + Sync`**
  (#860). `Simulator::new` now builds its walks on the rayon pool (see
  *Changed*), so the trait object in `WalkParams::walker` and the generator
  cross threads. `WalkTypeAble<X, Y>` gains `Send + Sync` supertraits, and
  `Simulator::new` requires `F: Send + Sync`, `E: Send`, and `X`, `Y:
  Send + Sync`. Migration: a walker or generator holding `Rc`, `Cell` or
  `RefCell` state moves to `Arc`, an atomic or a `Mutex`; every walker and
  generator in the workspace already met the bounds. A failing build still
  returns the error of its lowest-indexed failing walk. From
  `PARALLEL_MIN_WALKS` walks up it no longer stops at that walk: it
  finishes the round of 1024 walks that holds it first.

- **The core payoff of every option family is its contract's terminal
  payoff** (#844). `OptionType::payoff`, behind `Options::payoff`,
  `payoff_at_price`, `intrinsic_value` and the P&L built on them, disagreed
  with the value each family's pricing kernel returns at `T = 0`. Each
  family is now valued for a long position by one `Decimal` kernel and
  signed once by the side, and `optionstratlib-pricing`'s
  `terminal_payoff_test` asserts, for every family, call and put, long and
  short, at spots 95, 100 and 105 against a strike of 100, that
  `Options::payoff` equals the kernel's `T = 0` value exactly. API:
  `PayoffInfo` gains `exotic_params: Option<ExoticParams>` (`None` by
  default), which `Options::payoff` fills with the option's own, so the
  two-asset rainbow and the cliquet see their parameters. Values that
  change (per unit, `S` the spot, `K` the strike):
  - Binary, Chooser and Power are signed by the side: a short position pays
    what the long receives (a short in-the-money cash-or-nothing call is
    `-1`, it was `+1`). So are an Asian with fixings and a floating-strike
    lookback, which were unsigned as well.
  - Floating-strike lookback: the path extremes include the expiry spot, so
    with none observed the payoff is `0`; it was `S` (call) or `-S` (put).
  - Asian with no fixings: the averaging window is the expiry instant, whose
    average is the spot, so the payoff is the vanilla intrinsic value on
    `S`, as `asian_black_scholes` prices a contract at `T = 0`; it was `0`.
    With fixings the payoff is unchanged for a long position.
  - Compound: the vanilla payoff of the compound strike on the underlying
    option's long payoff `U` (`max(U - K, 0)` call, `max(K - U, 0)` put, the
    underlying taking the compound's style); it was `U`, so a call on a call
    with `S = 105`, `K = 100` was 5 and is 0.
  - Spread: `max(S - S2 - K, 0)` call, `max(K - (S - S2), 0)` put; it was the
    vanilla payoff on `S` (the put of #844, `S = 105`, `S2 = 98`, `K = 100`,
    was 5 and is 93).
  - Exchange: `max(S - S2, 0)` for either style, no strike; it was the
    vanilla payoff on `S`.
  - Rainbow (two assets): the vanilla payoff on `max(S, S2)` (best of) or
    `min(S, S2)` (worst of), `S2` from `exotic_params`; it was the vanilla
    payoff on `S`. A rainbow on other than two assets, or without
    `rainbow_second_asset_price`, is now a `PayoffError`, as the pricer
    rejects it.
  - Cliquet: no reset fixings reach the payoff, so nothing has accrued: `0`,
    clamped by `exotic_params`' global cap and floor; it was the vanilla
    payoff on `S`.
  - Every family's vanilla part is the exact `Decimal` difference instead
    of an `f64` difference taken back to `Decimal`, which was off in the
    16th significant digit for about a third of two-decimal prices
    (`50.07 - 50` was `0.0700000000000003`; 1 709 of 4 999 in-the-money
    calls on a grid of two-decimal spots), and a call struck at 0 on
    `Positive::MAX` is `Decimal::MAX` instead of an error. Quanto's product
    with the exchange rate is a `Decimal` product. The two seeded
    Monte-Carlo regressions that sum `payoff_at_price` over paths move and
    are re-baselined: the telegraph price by `8.8e-17` and
    `Simulator::get_mc_option_price` by `8.0e-16`. The payoff is normalized
    (no trailing zeros), the form `Decimal::from_f64` produced, so an exact
    payoff keeps its serialized and displayed representation; the chart
    goldens are unchanged. Asian averages of fixings and the power `S^n`
    stay in `f64`, as in the pricers.
  - Unchanged: European, American, Bermuda, fixed-strike lookback, barrier
    (signed since #826) and quanto values, beyond the exact `Decimal`
    difference above.

  The pricer's spread kernel prices a strike below `1e-4` with Margrabe's
  `max(S1 - S2, 0)` for either style, so a spread put struck at zero still
  differs from its payoff `max(S2 - S1, 0)` at `T = 0`; the test avoids that
  strike and the kernel is unchanged here.

  Migration: a `PayoffInfo { .. }` literal names `exotic_params` (or ends
  with `..Default::default()`); handle `PayoffError` from a rainbow payoff
  without its second asset; re-read P&L and payoffs of the families above.

- **`RiskMetrics::beta` is `coefficient_of_variation`, computed the same
  way for curves and surfaces, and a failed square root in the curve and
  surface metrics is an error** (#824). By owner decision the field is
  renamed rather than removed: it promised a sensitivity to market
  returns, which no curve or surface has the data for, and held two
  different things, `volatility / mean` on a `Curve` and a placeholder `0`
  on a `Surface`.
  - `coefficient_of_variation` is the population standard deviation (the
    `std_dev` of `compute_basic_metrics`) over the mean, signed as the
    mean, on both types. A zero mean, where it is undefined, is the new
    `MetricsError::ZeroMean { metric: "coefficient of variation" }`, so
    `compute_risk_metrics`, `compute_curve_metrics` and
    `compute_surface_metrics` return that error for data with a zero mean
    where they returned `beta = 0`. Empty data still reports every field
    as zero.
  - Values that change. A `Curve`'s value was `volatility / mean`, whose
    `volatility` divides the sum of squared deviations `S` by `sqrt(n)`, so
    the old value was the new one times `sqrt(S)`: for `y = 1..5`,
    `1.4907119849998597976061157791` becomes
    `0.4714045207910316829338962414` (`sqrt(2) / 3`), and for `y = 1..9`,
    `4` becomes `0.5163977794943222513572353866`. A `Surface`'s value was
    always `0` and is now the same quantity: `z = 1..9` gives
    `0.5163977794943222513572353866`, digit for digit the curve's. A
    constant curve or surface stays at `0`. `volatility` itself is
    unchanged on both types, including the curve's `S / sqrt(n)` grouping.
  - `d_sqrt(..).unwrap_or(..)` in `compute_basic_metrics`,
    `compute_shape_metrics` and `compute_risk_metrics` of both types
    reported a failed square root as a zero standard deviation or
    volatility (or, in the shape metrics, a unit one). Each is now
    `MetricsError::BasicError`, `ShapeError` or `RiskError`. A square root
    of a variance fails only when its iteration overflows, so ordinary
    data is unaffected.
  - `test_verify_curve_metrics_failure` asserted `is_ok()` under a
    `TODO`; it now asserts the `Ok(false)` the mismatched targets give, and
    `Ok(true)` for the curve's own metrics.

  Migration: rename `beta` to `coefficient_of_variation` in field reads and
  `RiskMetrics { .. }` literals; handle `MetricsError::ZeroMean` (a
  `match` on `MetricsError` needs the arm) where data can have a zero
  mean; and do not read the new value as a market beta.


- **The risk-neutral density analysis has an analytics-owned error**
  (#829). `RNDAnalysis::calculate_rnd`, `RNDAnalysis::calculate_skew`,
  `RNDResult::new` and `RNDStatistics::new` return
  `optionstratlib_analytics::error::RNDError` (re-exported as
  `optionstratlib::error::RNDError`, and wrapped by the facade's
  `error::Error::Rnd`) instead of the market crate's `ChainError`.
  `ChainError::EmptyDensities` and `ChainError::EmptySkewData` move to
  `RNDError::EmptyDensities` and `RNDError::EmptySkewData`, with the same
  messages. The other failures are `RNDError::InvalidParameters` (naming
  `derivative_tolerance`, `strike_interval`, `expiration_date` or
  `underlying_price`), `RNDError::Decimal` and `RNDError::Positive`, which
  used to arrive as `ChainError::ChainBuildError`,
  `ChainError::OptionDataError` and `ChainError::PositiveError`; a chain
  failure (no ATM implied volatility) is carried unchanged as
  `RNDError::Chain`. Migration: match `RNDError` where you matched
  `ChainError` on these calls; `RNDError: From<ChainError>`, so a function
  that mixes chain and RND calls can return `RNDError`.
- **The metric surfaces no longer make up a volatility or a zero-day
  value** (#822).
  - `vanna_volga_surface` returns `SurfaceError::OperationError`
    (`InvalidParameters` naming `implied_volatility`) when no option in the
    chain has an implied volatility. It used to measure the cost from a
    made-up ATM volatility of 0.20, the pattern #619 removed from the
    probability kernels.
  - `smile_dynamics_surface` at zero days (or a day count whose
    `sqrt(days / 30)` underflows to zero) returns the same error naming
    `days_to_expiry` for a strike whose skew is not zero: the adjustment
    `skew / sqrt(days / 30)` diverges there. It used to return the
    strike's unadjusted IV (`max(iv, 0.01)`). A zero skew still returns the
    ATM volatility, which is the `T -> 0` limit; a continuity test pins it.
  - `volume_profile_surface` at zero days returns the same error naming
    `days` for a strike with a non-zero volume, where `volume *
    sqrt(30 / days)` diverges; it used to return the unadjusted volume. A
    zero volume still returns zero, its limit.
  - The 0.01 floor on the smile-dynamics volatility stays, documented as a
    domain floor (`SMILE_VOL_FLOOR`): a volatility must be positive.
  Every other input returns the same value as before. Migration: pass a
  chain with at least one implied volatility to `vanna_volga_surface`, and
  day counts above zero to the two time surfaces.
- **`OptionChain::load_from_csv` rejects a file name it cannot read the
  chain's metadata from** (#827). The CSV carries only the quotes; the
  symbol, expiration date and underlying price come from the
  `symbol-day-month-year-price.csv` name `save_to_csv` writes. When the name
  did not parse, the loader returned the chain anyway with symbol and
  expiration `"unknown"` and an underlying price of zero. It now returns
  `ChainError::ChainBuildError` (`InvalidParameters`) naming `file_name`,
  `symbol`, `expiration_date` or `underlying_price`; `load_from_csv_async`
  runs the same loader and returns the same errors. `set_from_title` also
  rejects an empty symbol and a date that is not `YYYY-MM-DD`, `DD-MM-YYYY`
  or `DD-mon-YYYY`, and changes nothing unless the whole name parses. The
  price is parsed as a `Decimal` instead of an `f64`: a price an `f64`
  round-trips loads as before, and a longer one keeps all its digits.
  Migration: name the file in the `save_to_csv` form, for example
  `SPX-2030-01-15-5781.88.csv`, before loading it.
- **`Expirable::expiration_timestamp` and `Expirable::is_expired` return
  `Result`** (#810). `expiration_timestamp` returns `Result<i64,
  PositionError>` and `is_expired` returns `Result<bool, PositionError>`.
  `FuturePosition` read an expiration date that resolves to no calendar
  instant (for example `ExpirationDate::Days(Positive::MAX)`) as timestamp
  `0`, the Unix epoch, and as not expired; both now return
  `PositionError::DecimalError(DecimalError::ExpirationDate(_))`. Values for
  dates that resolve are unchanged. `Trade::is_expired` reads the trade
  status, has no fallback and keeps returning `bool`.
  - Migration: add `?` (or match the `Err`) at each call, and implement both
    methods with the `Result` return in a custom `Expirable`.

- **Core helpers that aborted on extreme inputs now return `Result`**
  (#788, core, math and pricing). Each of these panicked inside a
  `Decimal` or `Positive` operator, an integer division or a `chrono`
  addition. A probe reproduced every case, for example
  `PerpetualPosition::default().liquidation_price(..)` with `Division by
  zero` and an out-of-the-money Asian put payoff with `Positive invariant
  broken in sub_f64`. Values for inputs that worked before are unchanged.
  - **Leg traits.** These methods now return
    `Result<_, PositionError>`:
    - `LegAble::notional_value`;
    - `Marginable::{initial_margin, maintenance_margin, leverage,
      is_liquidation_risk}`, and `Marginable::liquidation_price` returns
      `Result<Option<Positive>, PositionError>` (the `Option` of #805,
      below, inside the error channel);
    - `Fundable::{funding_payment, annualized_funding}`.

    `annualized_funding` rejects a zero `funding_interval_hours` with
    `PositionError::ValidationError`. A zero quantity in
    `liquidation_price` is a `PositionError::DecimalError`.
    `Expirable::time_to_expiration_years` divides with the checked `d_div`.
  - **Leg structs.** `SpotPosition::{initial_value, market_value,
    percentage_return, break_even_price}` and
    `FuturePosition::{notional_value_at_entry, notional_value_at_price,
    tick_value, total_margin_required, implied_leverage}` return
    `Result<_, PositionError>`, and so does
    `PerpetualPosition::{notional_value_at_entry,
    notional_value_at_price}`.
  - **Balances.** `Balance::{get_unrealized_pnl, is_profitable}` and
    `Portfolio::{get_total_unrealized_pnl, has_profitable_positions}`
    return `Result<_, DecimalError>`; the other balance values became
    fallible in #805.
  - **Time helpers.**
    - `utils::time::convert_time_frame` returns
      `Result<Positive, PositiveError>`. It reports a source
      `TimeFrame::Custom(Positive::ZERO)` and an out-of-range result.
    - `get_x_days_formatted` and `get_tomorrow_formatted` return
      `Result<String, ExpirationDateError>`, with
      `ExpirationDateError::ArithmeticOverflow` for a day offset beyond the
      calendar, as `get_x_days_formatted_pos` does since #805.
  - **Price grid.** `model::utils::generate_price_points` returns
    `Result<Vec<Decimal>, DecimalError>` and rejects fewer than two points.

  Migration: add `?` (or handle the `Err`) at each call.
  `optionstratlib::error::Error` has no `From<ExpirationDateError>`. In a
  function returning it, map the date helpers through
  `DecimalError::from`. `Xstep::next` / `previous` already return
  `SimulationError` and propagate the new `PositiveError`.

- **`Optimizable::find_optimal`, `get_best_ratio` and `get_best_area`
  return `Result<(), StrategyError>`** (#793). They returned `()`, so a
  search that changed nothing could not be told from one that worked:
  failures were only logged. A search that finds a candidate picks the
  same legs as before.
  - No candidate (an empty filtered chain, or every combination discarded
    because it cannot be built, scored or, for `CustomStrategy`, given
    break-evens) returns the new `StrategyError::NoValidCandidate {
    strategy: StrategyType }` and leaves the strategy exactly as it was.
    `ShortStrangle` applies the chain's expiration before searching; a
    failed search now restores the previous one. `CustomStrategy` no longer
    reapplies its original legs and recomputes their break-evens when
    nothing is eligible: it restores the pre-search state.
  - `CustomStrategy` reports a position it cannot update from the chain
    (`update_from_option_data`, for example a strike without the quote the
    leg needs) and the failed break-even recomputation of the best
    positions, both with the strategy left as it was. A missing quote was
    ignored and the candidate scored with the leg's previous premium; the
    search now stops with that error.
  - A strategy without a search (`LongCall`, `LongPut`, `ShortCall`,
    `ShortPut`, and the trait default used by `CoveredCall`,
    `ProtectivePut` and `Collar`) returns
    `StrategyError::OperationError(OperationErrorKind::NotSupported { .. })`
    with operation `"find_optimal"` instead of a warning.
  - Candidates that cannot be built or scored are still skipped (logged at
    `DEBUG`) and do not stop the search.
  - Migration: add `?` to `strategy.get_best_area(..)`,
    `strategy.get_best_ratio(..)` and `strategy.find_optimal(..)`, or match
    `Err(StrategyError::NoValidCandidate { .. })` where an empty search is
    expected and the seed should be kept. An `Optimizable` implementation
    returns `Ok(())` after applying its best candidate and
    `Err(StrategyError::no_valid_candidate(..))` when there is none. A
    `match` on `StrategyError` needs an arm for `NoValidCandidate`.

- **Core time, futures and balance helpers report a failed step instead of
  inventing a value** (#805). Values for every input that worked before are
  unchanged, except that the `Balance` products are now exact `Decimal`.
  - `utils::time::get_x_days_formatted_pos` returns
    `Result<String, ExpirationDateError>`. A day count past `i64` or past
    the calendar was replaced by today's date; it is now
    `ExpirationDateError::ArithmeticOverflow`.
  - `Marginable::liquidation_price` returns `Option<Positive>`. A long whose
    margin buffer is wider than its entry price has no reachable
    liquidation price and returns `None`; it returned `Positive::ZERO`,
    which `is_liquidation_risk` compared against the spot, so a long was
    reported at risk at a price of zero it survives. A negative short
    threshold stays `Some(Positive::ZERO)`, a documented floor: every
    non-negative price crosses it. `is_liquidation_risk` keeps its
    signature and returns `false` for `None`.
  - `Expirable::days_to_expiration` returns `Result<Positive,
    PositionError>` and `Expirable::time_to_expiration_years` returns
    `Result<Decimal, PositionError>`. `FuturePosition` turned a `get_years`
    error into zero days, which reads as expired, and multiplied the years
    by 365 with a panicking operator; both are now errors. The future rho
    reports the failure as `GreeksError::Pricing`.
  - `Balance::get_total_value`, `Balance::get_cost_basis` and
    `Portfolio::get_total_value` return `Result<Positive, PositiveError>`,
    and `Balance::get_percentage_return` returns `Result<Decimal,
    DecimalError>`. They round-tripped through `f64` with `unwrap_or(0)`;
    they now stay in `Decimal` and report an overflow as an error.
  - Migration: add `?` (or match the `Err`) at each call; match `None` from
    `liquidation_price` where a long may have no liquidation level, and
    implement `days_to_expiration` with the `Result` return in a custom
    `Expirable`.

- **The strategy width, break-even and aggregation helpers and the
  adjustment-target gaps return a `Result`** (#788). Each one used a
  panicking `Positive` / `Decimal` operator on values the caller controls
  (the strikes are `pub` fields, the targets and Greeks are `pub`
  `Decimal`s), so crossed strikes, a credit larger than the strike or a
  total past `Positive::MAX` aborted the process. Values for every input
  that worked before are unchanged.
  - `SpreadStrategy::spread_width`, `ButterflyStrategy::wing_width`,
    `CondorStrategy::{inner_width, outer_width, put_spread_width,
    call_spread_width}` and `StrangleStrategy::strangle_width` return
    `Result<Positive, StrategyError>` (`StrategyError::PositiveError` for
    crossed strikes).
  - `credit_spread_break_even`, `debit_spread_break_even`,
    `aggregate_fees` and `aggregate_premiums` return
    `Result<Positive, StrategyError>`.
  - `AdjustmentTarget::delta_gap` returns `Result<Decimal, GreeksError>`,
    and `gamma_gap` / `vega_gap` return
    `Result<Option<Decimal>, GreeksError>`, like the `PortfolioGreeks`
    gaps.
  - Migration: add `?` (or match the `Err`) at each call.

- **`BasicAble::one_option` and the getters that read through it return a
  `Result`** (#788). `one_option` / `one_option_mut` return
  `Result<&Options, StrategyError>` / `Result<&mut Options, StrategyError>`,
  and `get_symbol`, `get_strike`, `get_type`, `get_underlying_price`,
  `get_risk_free_rate` and `get_dividend_yield` return their former type
  wrapped in `Result<_, StrategyError>`. The trait defaults of `one_option`
  / `one_option_mut` panicked, so any `impl BasicAble for X {}` aborted on
  `get_underlying_price`; they now return
  `StrategyError::OperationError(NotSupported { .. })`, like the other
  defaults of the trait. A `CustomStrategy` with no legs (its `positions`
  field is `pub` and it derives `Deserialize`) returns
  `StrategyError::EmptyCollection` from `one_option`, `one_option_mut` and
  `get_type` instead of `index out of bounds`; `get_symbol` and
  `get_underlying_price` answer from its own `symbol` and
  `underlying_price`, and `get_strike`, `get_risk_free_rate` and
  `get_dividend_yield` return an empty map. Every shipped strategy, `Options`
  and `Position` return `Ok` with the same values as before.
  `Optimizable::is_valid_optimal_option` admits no strike on the `Upper` or
  `Lower` side of a strategy without a spot. Migration: add `?` after each
  of these calls, return `Ok(..)` from an override, and override
  `one_option` / `one_option_mut` in a custom strategy that holds options.

- **Two step and count helpers report overflow instead of aborting** (#788).
  `Ystep::next` returns `Result<Ystep<T>, SimulationError>`: at index
  `i32::MAX` it returns `SimulationError::StepError` where `index + 1`
  aborted in debug builds and wrapped to `i32::MIN` in release builds, the
  contract `Step::next` and `Xstep::next` already had.
  `RandomPositionsParams::total_positions` returns `Result<usize,
  ChainError>`: quantities that sum past `usize::MAX` return
  `ChainError::ChainBuildError` naming `total_positions` instead of
  aborting. Migration: add `?` (or handle the error) at both call sites.
- **`PnL` no longer implements `Add` or `Sum`** (#788). `impl Add for PnL`,
  `impl Add for &PnL`, `impl Sum for PnL` and `impl Sum<&PnL> for PnL`
  added the `Positive` costs with the raw operator and aborted on an
  overflowing total; `std` fixes both traits to return `Self`, so no
  signature could report it. Migration: replace `a + b` with
  `a.try_add(&b)?` and `items.iter().sum::<PnL>()` with
  `items.iter().try_fold(PnL::default(), |acc, item| acc.try_add(item))?`;
  both return `Result<PnL, PricingError>`.
- **Package contents and metadata are verified for 0.22.0** (M8-03, #559),
  with the evidence in `docs/release/0.22/packages.md`.
  - **Rust 1.89 is the declared minimum.** The workspace sets
    `rust-version = "1.89"` and all ten packages inherit it. The documented
    1.88 was not enough: `uuid` 1.27 and `statrs` 0.19.1 (with `nalgebra`
    0.35 and `wide` 1.7) need 1.89, so a 1.88 build of the published crates
    fails to resolve. The README, the crate docs and the release notes now
    say 1.89. Migration: build
    with Rust 1.89 or newer.
  - **The facade archive ships only the library.** Its `include` list is
    anchored at the package root and keeps `src/`, the two `docs/*.md` and
    the 0.22 release notes its rustdoc includes or its README links,
    `Cargo.toml`, `README.md` and `LICENSE`. The Makefile,
    `rust-toolchain.toml`, two nested READMEs, the integration tests and the
    benches are no longer published (43 files to 13); they stay in the
    repository and CI. The component `include` lists are anchored the same
    way.
  - **Descriptions place each crate.** Every component description ends with
    the facade feature that re-exports it, and the facade's names its nine
    component crates.
  - **New checks.** `make check-packages` (`scripts/check_packages.py`, with a
    self-test) asserts each archive's contents (source, README and LICENSE
    present; no local, build or planning artifact), the lockstep metadata,
    crates.io-valid categories and keywords, versioned path dependencies,
    additive features that match `docs/ownership.md`, and tracked README
    links. `make check-package-archives` (`scripts/check_package_archives.sh`)
    builds, documents with warnings denied and doc-tests the ten crates from
    their unpacked archives, then checks them with the declared rust-version.
    Both run in the Components workflow and are release gates
    (`packages`, `package-archives` in `scripts/release_gates.py`, which now
    also counts `ignoring benchmark` package notices apart). The packaging is
    shared with the packaged-example and packaged-consumer checks through
    `scripts/package_archives.sh`.
  - **The 53 `ignoring test` / `ignoring benchmark` package notices are
    accepted**: shipping `tests/` and `benches/` would publish tests that
    read repository data outside the package, and cargo has no switch to
    silence the notice short of removing the targets from the repository's
    test runs. `docs/release/0.22/packages.md` lists them per package.

- **`AdjustmentError` moved to `optionstratlib_strategies::error`, as a
  `thiserror` error** (#556). The delta-neutral optimiser's error was the
  one public error outside a crate's `error` module, and the one with a
  hand-written `Display`. Same variants, same messages, same `From`
  conversions (now `#[cold]`). Migration:
  `optionstratlib_strategies::strategies::AdjustmentError` and
  `strategies::delta_neutral::AdjustmentError` (facade:
  `optionstratlib::strategies::AdjustmentError`) become
  `optionstratlib_strategies::error::AdjustmentError`, exported by the
  facade as `optionstratlib::error::AdjustmentError`. The aggregate
  `optionstratlib::error::Error` gains an `Adjustment` variant with a
  `From<AdjustmentError>`.
- **The facade no longer depends on `positive`** (#556, ADR-0001 D8).
  `error::Error::Positive` wraps `optionstratlib_core::model::PositiveError`,
  the same type through core, so the variant is unchanged; `positive` stays
  a dev-dependency of the facade's tests and benches. The feature-tree pins
  lose the `optionstratlib -> positive` edge on every surface.

- **The facade prelude is minimal, explicit and documented** (#551).
  `optionstratlib::prelude` now holds the domain vocabulary, the extension
  traits whose methods callers use on library types, and the entry type of
  each capability: 86 named items, each re-exported from its canonical path
  (#550; the flat parent path where the defining crate has one) and gated
  by the feature of its capability. The module docs list every item, its
  reason and its feature, and say when to import from a module instead.
  The five globs (`pricing::*`, `greeks::*`, `volatility::*`, `metrics::*`,
  `backtesting::*`), which admitted every new public item of those modules
  and their submodule paths (`prelude::black_scholes_model`,
  `prelude::composite::vanna_volga`, …), are gone, so are the aggregate
  `Error` (which shadowed `std::error::Error` for glob importers), every
  error type, every free function, the chart models, the `io`, `synthetic`
  and `plotly` items, and the standard-library and `num-traits`
  re-exports. `Decimal`, `dec!`, `Utc` and the `tracing` macros stay
  (ADR-0001 D7), as do `Positive` and its macros (D8). No prelude item now
  depends on `io`, `synthetic`, `plotly`, `static_export` or `async`.
  - Kept through the globs as named items, because they are extension
    traits: `OptionPricing`, `Greeks`, `Profit` (pricing) and
    `VolatilitySmile` (`OptionChain::smile`).
  - Removed, with the path to import instead:

  | Removed from `optionstratlib::prelude` | Import from |
  | --- | --- |
  | `Error` | `optionstratlib::error::Error` |
  | `ChainError, CurveError, DecimalError, GraphError, GreeksError, InterpolationError, MetricsError, OhlcvError, OperationErrorKind, OptionsError, PositionError, PricingError, ProbabilityError, StrategyError, SurfaceError, TransactionError, VolatilityError` | `optionstratlib::error::<Type>` |
  | `Action` | `optionstratlib::model::types::Action` |
  | `Trade` | `optionstratlib::model::Trade` |
  | `BasicAxisTypes` | `optionstratlib::model::BasicAxisTypes` |
  | `Payoff, PayoffInfo` | `optionstratlib::model::payoff::{Payoff, PayoffInfo}` |
  | `ToRound` | `optionstratlib::model::utils::ToRound` |
  | `calculate_log_returns` | `optionstratlib::utils::calculate_log_returns` |
  | `convert_time_frame, get_tomorrow_formatted, get_x_days_formatted` | `optionstratlib::utils::time::<fn>` |
  | `Curvable, StatisticalCurve` | `optionstratlib::curves::{Curvable, StatisticalCurve}` |
  | `Surfacable` | `optionstratlib::surfaces::Surfacable` |
  | `adjust_volatility, constant_volatility` | `optionstratlib::volatility::<fn>` |
  | `OptionChainParams` | `optionstratlib::chains::utils::OptionChainParams` |
  | `StrategyLegs` | `optionstratlib::chains::StrategyLegs` |
  | `OhlcvCandle, read_ohlcv_from_zip` (`io`) | `optionstratlib::chains::csv::{OhlcvCandle, read_ohlcv_from_zip}` |
  | `generator_optionchain` (`synthetic`) | `optionstratlib::chains::generator_optionchain` |
  | `generator_optionseries` (`synthetic`) | `optionstratlib::series::generator_optionseries` |
  | `StrategyType` | `optionstratlib::strategies::base::StrategyType` |
  | `AdjustmentAction, AdjustmentConfig, AdjustmentError, AdjustmentOptimizer, AdjustmentPlan, AdjustmentTarget, PortfolioGreeks` | `optionstratlib::strategies::<Type>` |
  | `WalkPath, WalkTypeAbleClone, check_exit_policy, expanding_window_vols, generator_positive, walk_steps, walk_steps_par` | `optionstratlib::simulation::<item>` |
  | `SimulationStats` | `optionstratlib::backtesting::SimulationStats` |
  | `GraphData, Series2D, Surface3D, TraceMode` | `optionstratlib::visualization::<Type>` |
  | `make_surface` (`plotly`) | `optionstratlib::visualization::make_surface` |
  | `Path` | `std::path::Path` |
  | `ToPrimitive` | `rust_decimal::prelude::ToPrimitive` |
  | everything else the five globs exported (free functions such as `black_scholes`, `delta`, `implied_volatility`, `historical_volatility`; types such as `Greek`, `GreeksSnapshot`, `BinomialPricingParams`, `GenericPricingEngine`, `Priceable`, `DELTA_THRESHOLD`; the option-chain metric traits; the backtest results and metrics) and their module paths | the same name under `optionstratlib::pricing`, `optionstratlib::greeks`, `optionstratlib::volatility`, `optionstratlib::metrics` or `optionstratlib::backtesting` |

  - New executable checks: a facade test target, `tests/prelude/main.rs`,
    with no required features, so it runs with the defaults, with
    `--all-features`, with `--no-default-features` and with each
    capability alone (`make lint` runs the facade tests once per
    `FACADE_FEATURE_SETS` entry). For every prelude item it imports the
    item from the prelude and from its defining module and fails to compile
    (`E0659`) if the two differ; it also prices Hull's call through the
    prelude's `OptionPricing`. `compile_fail` doctests in the prelude docs
    cover removed paths.
  - Every example, test and consumer fixture that relied on a removed item
    now imports it from its canonical module.

- **Canonical 0.22 error paths: the duplicate error file modules are gone**
  (#550). Each error type has one canonical path, flat in its crate's
  `error` module and in the facade's `optionstratlib::error`. The file
  modules that held nothing beyond a type already exported flat were public
  only because 0.21 exposed them, and are now private. The modules that hold
  detail enums (the `...Kind` types) stay public and are canonical for
  them: `error::position` (core), `error::greeks` (pricing),
  `error::chains` (market), `error::probability` (analytics) and
  `error::strategies` (strategies). The kinds are not flattened because
  their names collide once the facade gathers every crate's errors
  (`StrategyErrorKind` in core `position` and market `chains`,
  `PriceErrorKind` in analytics `probability` and strategies `strategies`).
  No type, variant or behaviour changes.
  Removed paths and their migration:

  | Removed path | Use instead |
  | --- | --- |
  | `optionstratlib_core::error::decimal::{DecimalError, DecimalResult}` | `optionstratlib_core::error::{DecimalError, DecimalResult}` |
  | `optionstratlib_core::error::trade::TradeError` | `optionstratlib_core::error::TradeError` |
  | `optionstratlib_math::error::curves::{CurveError, CurvesResult}` | `optionstratlib_math::error::{CurveError, CurvesResult}` |
  | `optionstratlib_pricing::error::pricing::{PricingError, PricingResult}` | `optionstratlib_pricing::error::{PricingError, PricingResult}` |
  | `optionstratlib_simulation::error::simulation::{SimulationError, SimulationResult}` | `optionstratlib_simulation::error::{SimulationError, SimulationResult}` |
  | `optionstratlib::error::decimal::*` | `optionstratlib::error::{DecimalError, DecimalResult}` |
  | `optionstratlib::error::trade::*` | `optionstratlib::error::TradeError` |
  | `optionstratlib::error::curves::*` | `optionstratlib::error::{CurveError, CurvesResult}` |
  | `optionstratlib::error::pricing::*` | `optionstratlib::error::{PricingError, PricingResult}` |
  | `optionstratlib::error::simulation::*` | `optionstratlib::error::{SimulationError, SimulationResult}` |
  | `optionstratlib::error::unified::Error` | `optionstratlib::error::Error` (or `optionstratlib::prelude::Error`) |

  Every `...Result` alias now has a flat path: `optionstratlib_pricing::error`
  exports `GreeksResult` flat (it was reachable only as
  `error::greeks::GreeksResult`, which still resolves), and the facade's
  `error` module adds `GreeksResult`, `ProbabilityResult` and
  `StrategyResult`.

  Canonical-path policy for the rest of the API: an item defined in a
  submodule and re-exported by its parent is canonical at the parent
  (`optionstratlib::pricing::black_scholes`, not
  `optionstratlib::pricing::black_scholes_model::black_scholes`). The
  defining submodules (65 of them, such as `pricing::black_scholes_model`,
  `strategies::long_call` and `backtesting::metrics`) stay public as
  documentation anchors; nothing there is removed. The crate docs of
  `optionstratlib` state the policy under "Canonical paths".

  The facade module docs and each changed component's `error` module docs
  list the canonical paths, with a `compile_fail` doctest per removed path
  next to a compiling one for its replacement. A new facade test,
  `tests/unit/canonical_paths_test.rs`, compares every canonical
  facade path with its defining crate by `TypeId`: the root re-exports, one
  type per facade module, and every flat error re-export, kind module and
  the aggregate `Error`. Workspace call sites that used the long paths
  (math curves and surfaces, pricing kernels, telegraph, pricing utils and
  the Greeks error) now use the flat ones.

- **Options carry a contract multiplier** (#733). `Options` gains
  `contract_size: Positive`, the units of the underlying one contract
  covers (100 for a standard US equity option). It defaults to 1 and is
  `#[serde(default)]`, so payloads written before the field existed
  deserialize unchanged, and with a contract size of 1 every result is
  identical to before. Payoff (`payoff`, `payoff_at_price`,
  `intrinsic_value`), premium and cost (`Position::total_cost`,
  `premium_received`, `net_premium_received`, `net_cost`), P&L
  (`pnl_at_expiration`, `unrealized_pnl`, `PnLCalculator` for `Options` and
  `Position`), every Greek (Black-Scholes, Black-76, Garman–Kohlhagen and
  the numerical fallback), SPAN margin and the strategies' break-evens,
  max profit/loss and premiums scale by `quantity × contract_size`.
  `Position` fees stay per contract (`fee × quantity`); `Position::premium`
  is quoted per unit of the underlying, and `Position::trade()` carries the
  multiplier into the trade (`Trade::contract_size`, #760). Prices
  from the pricing models stay per unit. New API:
  `Options::with_contract_size(contract_size)` and
  `Options::position_size()` (`quantity × contract_size`, checked).
  `Options::new` keeps its signature and builds a one-unit contract.
  `Position::diff_position_pnl` rejects positions whose contract sizes
  differ. `Display`/`Debug` for `Options` print the contract size only when
  it is not 1.

  Migration: a struct literal `Options { .. }` must name the new field; add
  `contract_size: Positive::ONE` to keep the previous behaviour, or build
  with `Options::new(..)` and chain `.with_contract_size(..)`. To size a
  leg in market contracts, set `quantity` to the contract count and
  `contract_size` to the multiplier, and quote the premium per unit of the
  underlying; per-contract fees are unchanged.

- **Trades, transactions and strategies carry the contract size** (#760),
  closing the gaps #733 left.
  - `Trade` and `pnl::Transaction` gain their own `contract_size: Positive`
    (`#[serde(default)]` = 1), so a record stays self-contained once the
    position it came from is gone. Their premium is quoted per unit of the
    underlying and their fees stay per contract: `Trade::cost`, `income`,
    `net` (and `PnL::from(Trade)`) and `Transaction::pnl` use
    `premium × contract_size`. `Position::trade()` copies the option's
    contract size and now records the per-unit `Position::premium` in
    `Trade::premium` (with #733 alone it recorded `premium ×
    contract_size`); the trade's cost, income and net are the same as
    before. New API: `Trade::with_contract_size`,
    `Transaction::with_contract_size` and `Transaction::contract_size()`.
    `Trade::new` and `Transaction::new` keep their signatures and record a
    one-unit contract.
  - Strategies: `BasicAble` gains `get_contract_size()` (the size the
    option legs share, an error when they differ) and
    `set_contract_size(contract_size)` (sets every option leg and
    recomputes the break-evens; zero is rejected). Both have
    `NotSupported` defaults, and every built-in strategy and
    `CustomStrategy` overrides them. The option quantities and fees stay
    per contract, so the exposure scales with the multiplier. `CoveredCall`,
    `Collar` and `ProtectivePut` instead re-express their option legs in
    contracts of the new size, keeping the units covered and the fee per
    share, so their payoff and fees are unchanged.
    `Optimizable::create_strategy`, and so `find_optimal`, `get_best_area`
    and `get_best_ratio`, keep the contract size of the strategy they
    rebuild from; a strategy whose legs carry different sizes cannot be
    rebuilt and returns an error. The delta-neutral optimizer sizes a new
    leg like the positions it adjusts (1 when they differ).
  - `ProtectivePut` break-evens are the zeros of the expiry P&L for any put
    size: an under-hedged put (`quantity × contract_size` below the shares)
    has one break-even, above or below the strike, and an over-hedged put
    can have one below the strike as well. An exact hedge keeps the
    previous single break-even.

  With a contract size of 1 every result is unchanged. Migration: a struct
  literal `Trade { .. }` must name the new field; add
  `contract_size: Positive::ONE`, or build with `Trade::new(..)` and chain
  `.with_contract_size(..)`. Code that read the per-contract premium from a
  `Trade` built by `Position::trade()` multiplies `trade.premium` by
  `trade.contract_size`. `Transaction` has private fields and no literal to
  migrate.

- **`Trade` monetary helpers are checked, and the covered strategies
  handle partial cover** (#765), closing the follow-ups #760 left.
  - `Trade::cost` and `Trade::income` return `Result<Positive, TradeError>`
    and `Trade::net` returns `Result<Decimal, TradeError>`. They multiply
    public `Positive` fields (`premium × contract_size × quantity`,
    `fee × quantity`), and the operator arithmetic they used could abort on
    overflow; the products and the sum are now checked. `TradeError` gains
    an `ArithmeticOverflow { operation, reason }` variant and its
    `TradeError::arithmetic_overflow` constructor. `From<Trade>` and
    `From<&Trade>` for `PnL` become `TryFrom<Trade>` and `TryFrom<&Trade>`
    with `Error = TradeError`. `Transaction::pnl` was already checked and is
    unchanged.
  - `ProtectivePut::max_loss_potential` (and so `get_max_loss`) is the
    deepest point of the expiry P&L for any put size: a put covering fewer
    units than the shares (`quantity × contract_size` below the spot
    quantity) reports `N C - U K + U p + F`, reached at zero; an exact or
    over-hedged put keeps the loss floored at the strike.
  - `Collar` break-evens are the zeros of the expiry P&L for any put and
    call size: below the put strike and above the call strike as well as
    between them. Its max profit and max loss follow the same P&L: an
    under-covered call reports an unbounded profit (`Positive::MAX`), an
    over-covered call an unbounded loss (`Positive::MAX`), and otherwise the
    extreme among zero and the two strikes.
  - The profit and loss zones (`ProbabilityAnalysis::get_profit_ranges`,
    `get_loss_ranges`) of `ProtectivePut` and `Collar` with a mismatched leg
    are the pieces between the break-evens, signed by the P&L inside them.
    An under-hedged in-the-money put, whose break-even sits below the
    strike, used to fail with an inverted range.

  With a contract size of 1 and legs that cover the shares exactly every
  result is unchanged. Migration: add `?` (or handle the error) to calls of
  `trade.cost()`, `trade.income()` and `trade.net()`; replace
  `PnL::from(trade)` / `trade.into()` with `PnL::try_from(trade)?` /
  `trade.try_into()?`. A `match` on `TradeError` needs an arm for
  `ArithmeticOverflow`.

- **`CoveredCall` handles partial cover, covered strategies recompute their
  break-evens on edits, and the `Trade` timestamp is checked** (#771),
  closing the follow-ups #765 left.
  - `Trade::new` returns `Result<Trade, TradeError>` and
    `Trade::set_timestamp` returns `Result<(), TradeError>`. When the exact
    nanosecond timestamp leaves the `i64` range they still fall back to the
    whole seconds in nanoseconds, but that product is now checked: a date
    before 1677 or after 2262 returns `TradeError::ArithmeticOverflow`
    instead of overflowing `i64`, and `set_timestamp` leaves the timestamp
    unchanged.
  - `CoveredCall` break-evens are the zeros of the expiry P&L for any call
    size: below the strike and, when the call covers more units than the
    shares, above it too. An under-covered call (`quantity × contract_size`
    below the spot quantity) reports an unbounded max profit
    (`Positive::MAX`), an over-covered call an unbounded max loss
    (`Positive::MAX`), and otherwise the extreme of the P&L at zero and at
    the strike. Its profit and loss zones with a mismatched call are the
    pieces between the break-evens, as for `ProtectivePut` and `Collar`.
  - `Collar::add_position` / `modify_position` and
    `CoveredCall::add_position` / `modify_position` recompute the
    break-evens, which used to stay stale after an edit.
    `ProtectivePut::add_position` / `modify_position` already recomputed
    them but discarded a failure, leaving them cleared. All three now
    return the failure as a `PositionError` and leave the strategy as it
    was before the edit.

  With a contract size of 1 and a call that covers the shares exactly
  every result is unchanged. Migration: add `?` (or handle the error) to
  calls of `Trade::new(..)` and `trade.set_timestamp(..)`. Code that called
  `update_break_even_points` after `add_position` / `modify_position` on a
  `Collar` or `CoveredCall` can drop the call.

- **Stochastic pricing entry points take the generator from the caller**
  (#638). No public function of `optionstratlib-pricing` draws from the
  thread-local RNG implicitly any more, so a seeded generator reproduces
  every result. Each entry point gains a trailing `rng` argument; there are
  no unseeded convenience wrappers. Migration: append a generator, either
  `&mut optionstratlib_core::utils::deterministic_rng(seed)` (re-exported as
  `optionstratlib::utils::deterministic_rng`) for reproducible results or
  `&mut rand::rng()` for the previous fresh-draw behaviour:
  - `pricing::monte_carlo_option_pricing(option, steps, simulations)` →
    `monte_carlo_option_pricing(option, steps, simulations, rng)`.
  - `pricing::telegraph(option, no_steps, lambda_up, lambda_down)` →
    `telegraph(option, no_steps, lambda_up, lambda_down, rng)`; the same
    generator feeds the returns simulated to estimate a missing rate.
  - `pricing::TelegraphProcess::new(lambda_up, lambda_down)` →
    `new(lambda_up, lambda_down, rng)`, and `next_state(dt)` →
    `next_state(dt, rng)`.
  - `pricing::simulate_returns(mean, std_dev, length, time_step)` →
    `simulate_returns(mean, std_dev, length, time_step, rng)`.
  - `volatility::simulate_heston_volatility(kappa, theta, xi, v0, dt, steps)`
    → `simulate_heston_volatility(kappa, theta, xi, v0, dt, steps, rng)`.
  - `OptionPricing::calculate_price_telegraph(no_steps)` →
    `calculate_price_telegraph(no_steps, rng)`, where `rng` is
    `&mut dyn rand::Rng` so the trait stays dyn-compatible.

  The free functions are generic over `R: rand::Rng + ?Sized`, as
  `decimal_normal_sample_with` (#539) is. For a given generator state each
  function makes the same draws, in the same order, that it made from the
  thread RNG, so the distribution of every result is unchanged. In core,
  `utils::get_random_element_with(set, rng)` is the new seeded form of
  `get_random_element`, which remains as the documented thread-RNG
  wrapper, and `utils::random_decimal` now accepts an unsized generator
  (`R: Rng + ?Sized`), which no existing call site notices.

- **The telegraph pricer is a Monte-Carlo expectation driven by a normal
  shock** (#743). `pricing::telegraph` used to return the discounted payoff
  of a single path whose per-step shock was `sqrt(dt) * U`, `U` uniform on
  `[0, 1)`, with the drift multiplied by that shock as well. Each step now
  applies the log-Euler update `S *= exp((r - sigma^2 / 2) * dt + sigma *
  state * sqrt(dt) * Z)` with `Z` standard normal, every path draws its own
  initial regime, and the price is the discounted payoff averaged over
  `no_paths` paths. The signature gains that count:
  `telegraph(option, no_steps, lambda_up, lambda_down, rng)` →
  `telegraph(option, no_steps, no_paths, lambda_up, lambda_down, rng)`,
  with `no_paths: NonZeroUsize`, in the position `monte_carlo_option_pricing`
  gives its `simulations`. The new `pricing::TELEGRAPH_PATHS` (10 000) is the
  count `OptionPricing::calculate_price_telegraph` uses; that method's
  signature is unchanged. Migration: insert a path count after `no_steps`,
  `optionstratlib::pricing::TELEGRAPH_PATHS` for the trait's default.
  Results change for every input and seed: prices now converge to the
  risk-neutral expectation (the Black-Scholes price for a European option
  without dividend yield) instead of being one biased draw, and a call
  costs `no_paths` times as many steps as before.

- **The telegraph regime switches the volatility** (#755). The telegraph
  regime used to enter only as the sign of the diffusion term; since the
  normal shock is symmetric, the price was Black-Scholes whatever
  `lambda_up` / `lambda_down` were. The regime now selects one of two
  volatility levels: each step applies `S *= exp((r - q - sigma_s^2 / 2) *
  dt + sigma_s * sqrt(dt) * Z)` with `sigma_s` = `sigma_plus` in the +1 regime and
  `sigma_minus` in the -1 regime. Every step carries the risk-neutral drift
  of its regime, so discounted prices stay martingales and put-call parity
  holds; a European price lies between the Black-Scholes prices at the two
  levels and rises with the time the rates keep the path in the
  higher-volatility regime. New API: `pricing::RegimeVolatility`
  (`new(sigma_plus, sigma_minus)`, `constant(sigma)`, both rejecting a zero
  level, and the `sigma_plus()` / `sigma_minus()` accessors) and
  `PricingError::InvalidParameter { parameter, value, reason }`, also
  returned when a supplied transition rate is negative. Signatures:
  - `telegraph(option, no_steps, no_paths, lambda_up, lambda_down, rng)` →
    `telegraph(option, no_steps, no_paths, lambda_up, lambda_down,
    volatility, rng)`, with `volatility: RegimeVolatility`; the option's
    `implied_volatility` no longer diffuses the price (it still feeds the
    estimate of a missing rate).
  - `OptionPricing::calculate_price_telegraph(no_steps, rng)` →
    `calculate_price_telegraph(no_steps, volatility, rng)`. One implied
    volatility does not identify two levels, so the caller states them
    rather than the trait inventing a split.

  Migration: pass `RegimeVolatility::constant(option.implied_volatility)?`
  to keep the previous law (it converges to the same Black-Scholes price),
  or `RegimeVolatility::new(sigma_plus, sigma_minus)?` for two regimes. A
  negative `lambda_up` / `lambda_down`, which used to be read as a rate that
  never or always flips, is now an error. Seeded prices change even at equal
  levels, because the shock is no longer signed by the regime.

- **`OptionChain::strike_price_range_vec` works in `Positive`** (#642). The
  signature changes from `strike_price_range_vec(&self, step: f64) ->
  Option<Vec<f64>>` to `strike_price_range_vec(&self, step: Positive) ->
  Option<Vec<Positive>>`, so strikes no longer cross the public boundary as
  `f64` and a decimal step such as `0.3` accumulates without binary rounding
  drift. A zero step still returns `None`, and a strike that overflows while
  stepping now returns `None` instead of panicking. Migration: pass the step
  as `Positive` (`pos_or_panic!(5.0)`, `Positive::new_decimal(dec!(0.5))?`)
  and read the strikes as `Positive`; call `.to_f64()` on an element only
  where a non-monetary `f64` is genuinely needed, e.g. a plot axis.

- **`OptionSeries` reads its expiry keys back as absolute dates** (#643).
  Each `chains` key is still written as `YYYY-MM-DD`, but deserialization
  now reads it as `ExpirationDate::DateTime` at 18:30 UTC on that date (the
  time the `expiration_date` crate gives a date-only expiry) instead of
  `ExpirationDate::from_string_to_days`, which turned it into a day count
  from the moment of reading and moved every key one day earlier. Write
  then read now returns the same `YYYY-MM-DD` keys whenever it runs, and
  reading no longer overwrites the thread-local `ExpirationDate` reference
  datetime. Keys must be `YYYY-MM-DD`: a day count (`"30"`) or another
  date format (`"20300115"`, `"15-01-2030"`) is rejected with `Invalid date
  format`. Migration: match on `ExpirationDate::DateTime` (or call
  `get_days()` / `get_date()`, which work for both variants) where code
  expected a deserialized key to be `ExpirationDate::Days`, and rewrite any
  hand-written key in another format as `YYYY-MM-DD`. `OptionChain` is not
  affected: it stores and serializes `expiration_date` as the string it was
  given and parses it only when used, so its round trip is already the
  identity.

- **The payoff kernel speaks `Decimal` at its public boundary** (#637).
  `Payoff::payoff` returns `OptionsResult<Decimal>` instead of `f64`, and
  `PayoffInfo::spot_prices` / `spot_min` / `spot_max` are
  `Option<Vec<Positive>>` / `Option<Positive>` / `Option<Positive>` instead
  of `f64`. The evaluation stays in a private `f64` kernel; its result is
  converted once with the checked `finite_decimal`, so every payoff the
  former signature returned comes back as `Decimal::from_f64` of the same
  `f64`, and a kernel value with no `Decimal` representation (non-finite,
  or beyond the `Decimal` range) is `Err(OptionsError::PayoffError)`
  instead of a raw `NaN` / `±∞`. Migration:
  - Callers: replace `option_type.payoff(&info)` (an `f64`) with
    `option_type.payoff(&info)?` (a `Decimal`); drop the `f2d!` /
    `finite_decimal` step that used to follow it.
  - `PayoffInfo` literals: wrap Asian fixings and lookback / barrier
    extrema in `Positive` (`Some(vec![Positive::new(98.0)?, …])`,
    `spot_min: Some(Positive::new(80.0)?)`).
  - Implementors: return `OptionsResult<Decimal>`; report an
    unrepresentable value as `OptionsError::PayoffError`.
  - `Options::payoff`, `payoff_at_price` and `intrinsic_value` keep their
    signatures but scale the payoff by the quantity with a checked `Decimal`
    multiplication instead of an `f64` one, so a fractional quantity no
    longer picks up `f64` rounding, and a zero payoff on a short position
    is `0` rather than `-0` (equal as numbers; only the rendering changes,
    as the visualization golden file shows).
  - The binomial pricer (`price_binomial`, `generate_binomial_tree`) and
    the compound pricer report an unrepresentable payoff as
    `PricingError::Options(OptionsError::PayoffError)` instead of
    `PricingError::Decimal` / `PricingError::NonFinite`.

- **`ProbabilityAnalysis::expected_value` is signed** (#623). It returns
  `Result<Decimal, ProbabilityError>` instead of `Result<Positive, _>`, and
  `StrategyProbabilityAnalysis::expected_value` is a `Decimal`. A strategy
  whose probability-weighted payoff is negative used to report exactly zero,
  so "loses money on average" could not be told from "breaks even": the
  `BullCallLadder` test fixture (the former `CallButterfly`) read 0 at every
  volatility from 0.3 up; at 0.5 it now reads `-52.98…`. The zero-volatility
  shortcut returns the profit at the current price with its sign instead of
  flooring it. Two related changes in the same function:
  - The sum is computed in `Decimal` with checked arithmetic instead of in
    `f64`; an overflow is a `ProbabilityCalculationErrorKind::ExpectedValueError`.
    Results move in the last digits.
  - The extra `1 / (1 + |drift|)` scaling applied after the sum, only when
    the sum was positive, is removed. The trend's drift (weighted by its
    confidence) already shapes the distribution the probabilities come
    from, so the sum is the expectation under that trend; the scaling
    counted the drift twice, ignored the confidence (a zero-confidence trend
    still divided the value by `1 + |drift|`), and made the result
    discontinuous at zero. Expected values computed with a trend are larger
    in magnitude by that factor.

  Migration: compare against `Decimal::ZERO` instead of `Positive::ZERO`, and
  treat a negative value as a valid answer. Code that needs the old floor can
  write `ev.max(Decimal::ZERO)`.

- **`SimulationStats` sums `PnL::total_pnl` and its summary prints from
  `statistics()`** (#691, owner decisions on the two behaviours #677 kept).
  - Decision 1: `SimulationStats::update` folds the backtest adapter's
    projection unchanged, so a run's P&L is `PnL::total_pnl` (realized plus
    unrealized), as in `SimulationStatsResult`. The two used to differ for a
    result whose PnL reports a non-zero unrealized leg (or a realized `None`
    with an unrealized `Some`); such a run now counts with its unrealized
    leg, in the statistics, the summary and the individual-results table of
    `SimulationReport for SimulationStats`.
  - Decision 2: the duplicated running aggregates are gone. `SimulationStats`
    keeps only its close and exit-reason counters plus the stored outcomes
    and results; mean, median, standard deviation, best, worst and average
    holding period come from `SimulationStats::statistics()`
    (`PathStatistics`). The `SimulationReport` summary of a `SimulationStats`
    prints from it: the average holding period is the `PathStatistics` mean
    rounded half-up to two places (313 steps over 6 runs prints `52.17`; a
    tie such as `2.125` prints `2.13`, where the former `f64` half-to-even
    printed `2.12`), the average P&L is `PathStatistics::average_pnl`, and the
    maximum profit and loss are `best_pnl` / `worst_pnl`, which count a run
    without a P&L as zero (an empty or all-`None` set prints `$0.00` instead
    of the `Decimal::MIN` / `Decimal::MAX` sentinels). When the statistics
    cannot be computed (P&L sum or variance outside the `Decimal` range)
    `render_summary` / `print_summary` return the new
    `GraphError::Backtest(Box<BacktestError>)`.
  - The outcome rows read `Take-Profit Closes` and `Stop-Loss Closes`
    instead of `Profitable Closes (50% reduction)` and
    `Loss Closes (100% increase)`, wording left from the short-put origin of
    the type; the same wording is gone from the field docs.

  Migration: `SimulationStats::max_profit()`, `max_loss()` and
  `avg_holding_period()` are removed; read `best_pnl`, `worst_pnl` and
  `average_holding_period` from `statistics()?`. `total_pnl()` returns
  `Result<Decimal, BacktestError>`, summed over the stored outcomes when
  called. `update` and `update_outcome` no longer fail on a P&L total
  overflow (there is no running total); the overflow is reported by
  `total_pnl()` and `statistics()`, and the accumulator only rejects a run
  whose counter overflows. Every outcome and every result is stored, so
  memory grows linearly with the number of runs.

- **Terminal presentation lives in `optionstratlib-visualization` only**
  (M6-05, #546). No crate below visualization resolves `prettytable-rs`,
  `indicatif` or `pretty-simple-display` any more, and no computational API
  writes to stdout. Migration:
  - `OptionChain::show()` is removed. Print the bordered, coloured tables
    with `ChainReport::print_table()` (returns `Result<(), GraphError>`) or
    get them as plain text with `ChainReport::render_table()`, from
    `optionstratlib::visualization::terminal` (crate path
    `optionstratlib_visualization::visualization::terminal`).
  - `impl Display for OptionChain` writes a dependency-free plain-text table
    instead of a box-drawn one: a `Symbol: … Underlying Price: …
    Expiration Date: …` line, then the same 13 columns, left-aligned and
    separated by two spaces. The columns and the cell formatting are now
    data: `chains::OPTION_CHAIN_TABLE_HEADERS`,
    `chains::OPTION_CHAIN_TABLE_COLUMNS` and `OptionData::table_cells()`. A
    gamma whose ×100 display value leaves the `Decimal` range is an empty
    cell instead of an abort.
  - `SimulationStatsResult::print_summary` / `print_individual_results` and
    `SimulationStats::print_summary` / `print_individual_results` are
    removed. Import `optionstratlib::visualization::terminal::SimulationReport`
    and call the same names, which now return `Result<(), GraphError>`, or
    the `render_summary` / `render_individual_results` forms that return the
    tables as a `String`. The section titles are part of the report (written
    to stdout or the string) instead of separate `tracing` events. No
    printed figure changes: counts, percentages (the same `f64` expression,
    `NaN%` for zero runs included), money, the `SimulationStats` sentinel
    best and worst values before a P&L arrives, and the average holding
    period (rounded half-to-even from the exact `f64`, which is what its
    `f64` `{:.2}` printed; a unit test sweeps running averages and ties)
    all read as before; money is still `Decimal`'s `{:.2}`, which truncates
    to two decimal places. Display-only differences: exit reasons are listed
    in the order of their text instead of hash order; the trailing `=====`
    banner line `SimulationStats::print_summary` logged after its tables is
    dropped; headers are blue in both types (the `SimulationStats` headers
    are a title row now), and the `SimulationStatsResult` P&L cells keep
    their colour by sign while the `SimulationStats` summary keeps its fixed
    colours (profitable closes and maximum profit green, loss closes and
    maximum loss red); the final P&L of the individual results is now
    coloured by sign for both types. The total and
    average P&L of the summary use checked `Decimal` arithmetic with the
    same results, and an overflow is the new `GraphError::Decimal` instead
    of an abort. Each type still shows its own P&L: the total for
    `SimulationStatsResult`, the realized leg for `SimulationStats`.
  - `SimulationStats` gains read accessors (`total_simulations`,
    `profitable_closes`, `loss_closes`, `expired_trades`, `total_pnl`,
    `max_profit`, `max_loss`, `avg_holding_period`, `exit_reasons`,
    `results`) so renderers outside the crate read what it accumulated.
    `avg_holding_period` returns `Option<Decimal>` (steps), converted from
    the internal running `f64` with `Decimal::from_f64_retain`.
  - The `DebugPretty` / `DisplaySimple` / `DebugSimple` derives are replaced
    by `optionstratlib_core::{impl_json_debug_pretty, impl_json_display,
    impl_json_debug}` over `optionstratlib_core::utils::json_format::{write_json,
    write_json_pretty}`. The output of every `Debug` and `Display` they
    implement is unchanged, byte for byte, including the
    `Error serializing to JSON: …` fallback.
  `optionstratlib-visualization` now depends on `optionstratlib-backtest`
  (allowed by the layer DAG; the facade `visualization` feature already
  implied `backtest`) and on `prettytable-rs`. `make check-graph` forbids
  the three packages in every crate below visualization, with self-tests,
  and every consumer fixture that does not resolve visualization lists them
  as absent (the visualization-resolving fixtures list
  `pretty-simple-display`); the terminal reports are
  pinned by snapshot tests (`cargo test -p optionstratlib-visualization
  terminal`). With the derives gone, `serde_json` leaves the
  normal dependencies of `optionstratlib-pricing` (unused) and of
  `optionstratlib-math`, `optionstratlib-strategies` and
  `optionstratlib-backtest` (test-only, now a dev-dependency).

- **Garman–Kohlhagen takes a signed foreign rate,
  `ExoticParams::foreign_rate: Option<Decimal>`** (#720). The FX pricer and
  its Greeks read the foreign rate `r_f` from `Options::dividend_yield`,
  a `Positive`, so a negative foreign rate (CHF, JPY or EUR in parts of
  2015–2022) could not be priced. `r_f` is now `exotic_params.foreign_rate`
  when that is set, which may be negative, and falls back to
  `dividend_yield` otherwise, so every option built without the field keeps
  its price and Greeks bit for bit. When both are set, `foreign_rate` wins
  and Garman–Kohlhagen ignores `dividend_yield`. `Options::dividend_yield`
  stays `Positive`, and no other model reads the new field (Quanto keeps
  `quanto_foreign_rate`). `garman_kohlhagen` and `delta_gk`, `gamma_gk`,
  `vega_gk`, `theta_gk`, `rho_domestic_gk` and `rho_foreign_gk` resolve the
  rate in one place, so prices and Greeks always use the same `r_f`. The
  European Black–Scholes–Merton kernel takes the yield as a signed
  parameter internally; `black_scholes` passes `dividend_yield` to it, so
  its results are unchanged. Breaking: `ExoticParams` gains a public field,
  so a struct literal that lists every field (rather than ending in
  `..ExoticParams::default()`) or an exhaustive destructuring pattern no
  longer compiles. Migration: add `foreign_rate: None` to such literals, or
  end them with `..ExoticParams::default()`. Serialized `ExoticParams` gain
  a `foreign_rate` key (`null` when unset); input without it still
  deserializes, as `None`. New tests price `S = 1.00`, `K = 0.98`,
  `r_d = 2%`, `r_f = -0.75%`, `sigma = 10%`, `T = 0.5` against the closed
  form evaluated independently (call 0.047738205471098, put
  0.014230002497975, both within `5.1e-16`), check FX put-call parity
  `C - P = S e^(-r_f T) - K e^(-r_d T)` at `r_f` in -1%, 0 and 3% within
  `1e-12`, the fallback and the precedence bit for bit, delta parity
  `Delta_call - Delta_put = e^(-r_f T)` at the same rates, and delta and
  the foreign rho against central differences of the price at `r_f = -1%`.
  The Garman–Kohlhagen example prices the negative-rate case.

- **The option legs of `CoveredCall`, `Collar` and `ProtectivePut` are
  sized in shares, with option fees per share** (#731). An option here has
  no contract multiplier (its payoff is `intrinsic × quantity`), and the
  three types disagreed on how to size it against their shares.
  `CoveredCall` and `Collar` used `quantity / 100`, so one option unit
  covered one share in a hundred: their payoffs were neither capped nor
  floored, and disagreed with their own `max_profit_potential` and
  `max_loss_potential` (502.40 against +4957 at 150 on 100 shares).
  `ProtectivePut` already used `quantity` but divided its put fees by 100.
  The owner chose shares; a real contract multiplier is its own issue.
  - `CoveredCall::new` and `Collar::new` size their option legs in
    `quantity` (shares), and `ProtectivePut::new` no longer divides the
    put fees by 100. Every option fee argument is per share and scales
    with the leg's quantity, as `Position::fees` does.
  - The fee totals follow: `Collar`'s private fee sum and the new
    `CoveredCall::total_fees` count each option leg's fees times its
    quantity, so `max_profit_potential`, `max_loss_potential` and the
    break-evens charge what the payoff charges. The covered call's
    break-even now includes the call's fees (it counted the share fees
    only).
  - The same 100-share hedge gives identical put legs in `ProtectivePut`
    and `Collar`, and each payoff is capped or floored at its own maximum.
  - Migration: pass option fees per share, not per contract, and expect
    option legs whose `quantity` is the share count. A `CoveredCall` or
    `Collar` that relied on a hundredth of a contract per share should now
    pass the number of shares the option actually covers.
  - Values that move, before → after (hand-checked):
    - 100 shares at 100, call 105 @ 2.40, put 95 @ 2.10, share fees
      0.50 + 0.50, option fees 0.50 + 0.50 per share:
      - covered call: break-even 99.99 → 98.61, max profit 500.40 → 639,
        max loss 9999.60 → 9861, payoff at 150 4955.40 → 639 (capped),
        expiry P&L at 103 300.40 → 439, mark-to-market at 103 (10 days,
        vol 0.20) 300.09 → 309.14;
      - collar: break-even 100.03 → 101.71, max profit 497.30 → 329, max
        loss 502.70 → 671, payoff at 50 -4957.70 → -671 and at 150 4952.30
        → 329, expiry P&L at 103 297.30 → 129, mark-to-market 299.60 →
        260.31;
      - protective put: break-even 102.12 → 103.11, max loss 712 → 811,
        payoff at 150 4788 → 4689, expiry P&L at 103 88 → -11; the
        mark-to-market (251.17) does not move, the put leg's quantity being
        unchanged.
    - `protective_put_test` (put fees 0.65 + 0.65 per share now cost 130,
      were 1.30): break-even 153.53 → 154.82, max loss 853.3 → 982,
      profit at 100 -853.3 → -982 and at 200 4646.7 → 4518.
    - `test_covered_call_per_share_premium_rounds_to_nearest_even` sets
      the call to one unit against three shares so the quotient still
      repeats; its pinned value does not move.
    - Golden chart data regenerated on purpose: `strategy_covered_call`,
      `strategy_collar` and `strategy_protective_put` (one share, fees
      0.50). The covered call now caps at 5.40, the collar floors at -7.70
      and caps at 2.30, and the protective put floors at -9.10 with its
      break-even at 104.10 (was 103.11). Every other entry is unchanged.

- **Butterflies are textbook 1/2/1, `CallButterfly` is removed, and the
  1x1x1 call ladder it was is `BullCallLadder`** (#706). This closes the
  three butterfly exceptions #696 left.
  - `LongButterflySpread::get_strategy` and
    `ShortButterflySpread::get_strategy` enforce `validate()`, which requires
    the body to carry twice the wing quantity, the structure `new` already
    built. A request with one contract on every leg (net long or net short
    one call above the top strike) now returns
    `StrategyError::InvalidStrategy` instead of a `WARN`. Migration: give the
    middle leg twice the quantity of each wing.
  - `CallButterfly`, the `call_butterfly` module,
    `CALL_BUTTERFLY_DESCRIPTION` and `StrategyType::CallButterfly` are
    removed, with the prelude export and the `Graph` implementation. The
    type never matched its name: it was long the lower strike and short the
    middle and the upper ones, a ladder, while a textbook call butterfly is
    long the outer strikes and short twice the middle one. Migration:
    `CallButterfly` removed; the textbook long call butterfly is
    `LongButterflySpread`; the former `CallButterfly` 1x1x1 ladder is
    `BullCallLadder`. No alias is kept, so code that names the old type
    fails to compile instead of silently building a different strategy.
  - Serialized names: `StrategyType` serializes by variant name, so a
    document with `"CallButterfly"` (a `kind` field, a `StrategyRequest`)
    no longer deserializes and `"CallButterfly".parse::<StrategyType>()`
    fails. Write `"BullCallLadder"` for the ladder or
    `"LongButterflySpread"` for the butterfly.
  - `BullCallLadder` (`StrategyType::BullCallLadder`, `bull_call_ladder`
    module, `BULL_CALL_LADDER_DESCRIPTION`), in a new "Ladders" family,
    keeps the fields, `new` and `validate` of the old type, and:
    - `new` and `get_strategy` enforce `validate()`. `get_strategy` used to
      take short low / long middle / short high, which its own `validate`
      rejected; it now takes long low / short middle / short high, like
      `new`.
    - `add_position` used to tell the two short calls apart by comparing
      with the long strike, as for a butterfly, so the second short call
      overwrote the first. It now keeps them ordered by strike.
    - It no longer implements `ButterflyStrategy`: a ladder has no body.
  - `StrategyType` is exhaustive, so a `match` on it needs the new arm and
    loses the old one.
  - Golden chart data: `strategy_call_butterfly` is removed,
    `strategy_bull_call_ladder` is added, and
    `strategy_long_butterfly_spread` and `strategy_short_butterfly_spread`
    are regenerated on purpose with a 1/2/1 body; the short butterfly's body
    premium also drops from 4.50 to 2.50 so its credit covers its fees.
    Every other entry is unchanged. On the golden legs (fees 0.50 open and
    0.50 close per contract): the long butterfly 90/100/110 breaks even at
    97.70 and 102.30 with a 2.30 maximum profit and 7.70 maximum loss; the
    short butterfly at 93.70 and 106.30 with 3.70 and 6.30; the bull call
    ladder 95/100/105 at 98.60 and 106.40 with a 1.40 maximum profit and an
    unlimited loss.
  - Pinned values: in the ladder's `create_test_increasing_adjustments`
    (both delta modules) the fixture had its two short strikes in the wrong
    fields, which its own `validate` rejects. Ordered, the first short leg
    the adjustment reaches is the 5800 call, so the buy-back quantity moves
    from 0.2835618144021385 to 0.1338190182607754 and its strike from 5850
    to 5800. The net-delta-zero check after the adjustment holds either way.
  - The butterfly P&L tests pin expiry P&L on hand-computed 1/2/1
    fixtures. The mark-to-market tests, dropped here because
    `Position::calculate_pnl` ignored the quantity, are restored by #725.
  - The `strategy_call_butterfly*` examples demonstrated the ladder and are
    `strategy_bull_call_ladder*` now, with valid legs.

- **The put spread builders take textbook legs, and every strategy
  constructor and builder returns `StrategyError::InvalidStrategy` instead
  of a strategy that fails its own `validate()`, except the three butterfly
  builders deferred to #706** (#696).
  - `BullPutSpread::get_strategy` now requires the long put at the lower
    strike and the short put at the higher one (a credit spread), and
    `BearPutSpread::get_strategy` the reverse (a debit spread). Each used to
    require the other's legs, so it built the other strategy under its own
    name, and a correctly built spread passed by position was rejected. A
    `StrategyRequest` for either type now charts and prices the textbook
    payoff: on the pinned 95/105 golden request the bull put break-even
    moves from 98 to 102 and the bear put one from 102 to 98, and the
    `test_strategy_{bull,bear}_put_spread` Greeks flip sign. Migration: send
    long 95 / short 105 for a bull put spread and short 95 / long 105 for a
    bear put spread; the inverted legs are now an `OperationError`.
  - These now return the new `StrategyError::InvalidStrategy { strategy,
    reason }` (helper: `StrategyError::invalid_strategy`) when the strategy
    they assembled fails `validate()`. Most of them used to call
    `validate()` and drop the answer; the rest did not call it at all.
    - `new` on `BullPutSpread`, `BearPutSpread`, `BullCallSpread`,
      `BearCallSpread`, `LongButterflySpread`, `ShortButterflySpread`,
      `IronCondor`, `IronButterfly`, `LongStraddle`, `ShortStraddle`,
      `LongStrangle`, `ShortStrangle`, `PoorMansCoveredCall`, `Collar`,
      `CoveredCall`, `ProtectivePut`, `LongCall`, `ShortPut` and
      `CustomStrategy` (and the crate-private `new` of `LongPut` and
      `ShortCall`).
    - `StrategyConstructor::get_strategy` on `BullPutSpread`,
      `BearPutSpread`, `BullCallSpread`, `BearCallSpread`, `IronCondor`,
      `IronButterfly`, `LongStraddle`, `ShortStraddle`, `LongStrangle`,
      `ShortStrangle`, `PoorMansCoveredCall` and `CustomStrategy` (through
      its `new`). The single legs have no builder: theirs returns
      `OperationError` as before.
  - Three exceptions, deferred to #706, which settles the butterfly
    convention: `LongButterflySpread::get_strategy`,
    `ShortButterflySpread::get_strategy` and `CallButterfly` (`new` and
    `get_strategy`). The two spread builders accept the same quantity on
    every leg, while `new` and `validate` require a doubled body (1/2/1);
    `CallButterfly::get_strategy` takes short low / long middle / short
    high, while its `new` and `validate` use long low / short middle / short
    high, and the textbook call butterfly is long low / two short middle /
    long high. Enforcing `validate` would reject inputs these builders
    accept today and move the `call_butterfly` golden entry, so the two
    builders log a `WARN` naming #706 instead, and `CallButterfly::new`
    does not validate yet.
  - Migration. These inputs used to build and are now rejected:
    - by the strategy's own checks: inverted or equal strikes, including
      both strikes left at zero so that they default to the spot (a
      vertical, a strangle, a collar with the put at or above the call, an
      iron condor or iron butterfly whose wings are not outside the body);
    - by `Position::validate`: a short leg with a zero premium;
    - by `Options::validate`: a zero quantity, a zero underlying price, an
      empty underlying symbol and a zero strike. A negative risk-free rate
      is not among them: #709 allows it.

    A seed for `get_best_area` / `get_best_ratio` must therefore be a valid
    strategy: give it ordered strikes and a non-zero short premium instead
    of zeros (the optimizer replaces both). Match
    `StrategyError::InvalidStrategy` where a caller relied on getting an
    invalid strategy back. `StrategyError` is exhaustive, so a `match` on it
    needs the new arm.
  - `CoveredCall::max_profit_potential` and `max_loss_potential` multiplied
    the call premium by its contract count with the unchecked operator. A
    leg set that `new` rejects but the public fields admit overflowed it
    and aborted; both report the overflow now. The panic-freedom properties
    found it once they started driving the rejected cases through the
    public fields instead of skipping them.
  - The `strategy_bull_put_spread` and `strategy_bear_put_spread` entries of
    the visualization golden file are regenerated on purpose; every other
    entry is unchanged.

- **`uncertain_volatility_bounds` returns signed `Decimal` bounds, and a
  short position's are the negated long bounds** (#715). It returned
  `(Positive, Positive)` and built each bound with
  `Positive::new_decimal(..).unwrap_or(ZERO)`, so a short option, whose
  Black–Scholes price is negative, came back as `(0, 0)`. It now returns
  `Result<(Decimal, Decimal), VolatilityError>` on the `short == -long`
  convention of #646 and #648: both bounds are priced long, a long position
  gets `(price at min_volatility, price at max_volatility)` exactly as
  before, and a short one `(-long_upper, -long_lower)`, negative and still
  ordered. A short ATM call (`S = K = 100`, 30 days, `r = 5%`, vols 0.1 and
  0.3) now bounds at the short Black–Scholes prices at 0.3 and 0.1 instead
  of `(0, 0)`. Migration: a caller of a long option takes `.to_dec()` off
  its comparisons or wraps the bounds with `Positive::new_decimal(..)?`; a
  caller of a short option gets the signed values it previously could not
  see.

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
  unrealized leg, an expiry has a zero one. They differ only for a result
  whose PnL reports a non-zero unrealized leg (a caller-built result, or a
  downstream `SingleLegSimulation` whose `calculate_pnl_at_expiration`
  returns `realized: None, unrealized: Some(x)`). Unit tests pin both
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
  to its nearest `f64` with `decimal_to_f64` (correctly rounded since #670) and
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
    round trip returns the same expiry keys; see the #643 entry above.
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
  in the same step. The minimum supported Rust version is 1.89, the
  `rust-version` every crate declares since #559.

### Fixed

- **A compound option at zero volatility returns its deterministic value**
  (#867). Before expiry, `compound_black_scholes` at `σ = 0` valued the
  underlying through `black_scholes`, which rejects a zero volatility, so it
  returned an error.
  - **The value now returned.** With the forward `F = S e^((r - q) T2)`,
    the underlying is worth `V = e^(-r (T2 - T1)) max(±(F - K2), 0)` at
    `T1`, and the compound pays `e^(-r T1) max(±(V - K1), 0)`. That is the
    `σ → 0` limit of the Geske price. It covers all four call/put
    combinations, with the underlying taking the compound's style as at
    every other volatility.
  - **Examples.** The kernel's test contract is `S = 100`,
    `K1 = K2 = 5`, `T1 = 91.25` days. Its call on a call was an error and
    is now about 90.18, and its put on a put about 4.94.
  - **Unchanged.** Values at expiry, every `σ > 0` value, and
    `black_scholes()` dispatch: its `d1` / `d2` gate still rejects
    `σ = 0` for `T > 0`.
  - **Tests.**
    - All four combinations equal the hand-computed deterministic value
      at `σ = 0`.
    - The closed form at `σ = 1e-4` is within `1e-4` of it.
    - The kernel prices both styles at `σ = 0`, and their expiry values
      are unchanged.

- **A spread put struck near zero prices as the put, not the Margrabe
  call** (#852). For a strike below `1e-4`, `spread_black_scholes`
  switched to Margrabe's exchange formula `max(S1 - S2, 0)` for both
  styles, so the put was priced as the call and, at expiry, differed from
  its payoff `max(S2 - S1, 0)`.
  - **The fix.** The put now uses Margrabe on the swapped assets, the
    option to exchange `S1` for `S2`. The call, every strike from `1e-4`
    up (Kirk), and every T > 0 call are unchanged.
  - **Values** with `S2 = 100`, `σ1 = 20 %`, `σ2 = 25 %`, `ρ = 0.5`,
    `r = 5 %`, `q = 0`, `K = 0`:
    - put at `S1 = 105`, 90 days: 7.5706 → 2.5706;
    - put at `S1 = 95`, 90 days: 2.3646 → 7.3646;
    - put at expiry: 5 → 0 for `S1 = 105`, and 0 → 5 for `S1 = 95`.
  - **Tests.**
    - Both styles equal `OptionType::payoff` at expiry.
    - The exchange put-call parity `C - P = S1 e^(-q1 T) - S2 e^(-q2 T)`
      holds at `K = 0` and `K = 5e-5` with unequal dividend yields, and is
      within `K` of the full parity.
    - The put is continuous across the `1e-4` threshold.

- **Compound options are priced by the Geske closed form they claim, and
  the price is continuous at expiry** (#845).
  - **The bug.** A put with `S = 105`, `K = 100` was priced near 62.3 at
    every maturity from 90 days down to one minute, but 100 at expiry.
    Checked against an independent quadrature of
    `e^(-r T1) E[max(±(V(S_T1) - K1), 0)]`, the closed form was wrong in
    three ways:
    - it valued every underlying as a call, so a put was a put on a call
      while its value at expiry is a put on a put;
    - it approximated the critical spot as the forward scaled by
      `1 ± 0.4 σ√T1` instead of solving `V(I) = K1`;
    - its bivariate normal CDF integrated with five mis-weighted points
      (`M(0, 0; 0.5) = 0.2633` against the exact 1/3) and returned `NaN`
      for large arguments.
  - **The fix.**
    - The underlying takes the compound's style, as the expiry and
      zero-volatility branches already did: a call is a call on a call, a
      put a put on a put.
    - The critical spot is solved by bisection. A put underlying worth
      less than `K1` everywhere makes the compound put a forward on it and
      the compound call worthless.
    - The CDF is Genz's algorithm, accurate to about `1e-15`, and handles
      `ρ = -1` exactly.
  - **Values** (`σ = 25 %`, `r = 5 %`, `q = 0`), before → after; each
    "after" value matches the quadrature to `1e-6`:

    | `S`, `K`, `T1` | Put | Call |
    | --- | --- | --- |
    | 105, 100, 90 days | 62.3209 → 94.7987 | 0 → 0.000001 |
    | 100, 5, 91.25 days | 0 → 4.9379 | 61.9417 → 90.1856 |
    | 100, 10, 182.5 days | 0 → 9.7531 | 57.1621 → 80.7346 |
    | 100, 105, 182.5 days | 66.7444 → 92.5262 | 0 → 0.000191 |

    The values at expiry are unchanged.
  - **Errors.** A `d1` / `d2` rejection surfaces as `PricingError::Greeks`
    instead of a `MethodError` string.
  - **Tests.**
    - The four Geske contracts with distinct strikes and `T2 ≠ 2 T1`, and
      the no-critical-spot branch, are checked against the quadrature.
    - Compound put-call parity is checked.
    - The bivariate CDF is checked against a one-dimensional quadrature
      for both signs of `ρ` and all three Gauss-Legendre rules.
    - The price one minute out matches the price at expiry to `1e-4`.

- **`pricing::monte_carlo_option_pricing` prices puts and short positions**
  (#864). Every path was paid `max(S_T - K, 0)` and the result was never
  signed, so a put was priced as the call (10.45 for the at-the-money put
  whose Black-Scholes price is 5.57) and a short position as the long one.
  The payoff now follows `option_style` (`max(K - S_T, 0)` for a put) and
  the side is applied once, to the discounted mean, as `black_scholes`
  applies it: a short position returns the negated long price. Long calls
  are unchanged, bit for bit, including the pinned seeded price. New seeded
  tests check the put and the call against Black-Scholes, put-call parity
  with a dividend yield, and short as the exact negation of long.
  Migration: a caller that priced a put or a short position through this
  function was getting the long call; the value it now returns is the
  instrument's.

- **`make check-spanish` fails on Spanish comments, not on a clean tree.**
  The recipe ran `rg ... || exit 1`, but `rg` exits 1 when it finds no
  match, so the target failed on every clean tree and passed whenever a
  match was found. It now fails on a match (status 0) and on an `rg` error
  (status 2), and passes on status 1. No issue; found by the #789 run.

- **`pricing::black_scholes()` prices every exotic option at expiry**
  (#843). The dispatcher computed `d1` / `d2` before it looked at the
  option type. Both divide by `σ√T`, so at `T = 0` every exotic failed with
  `PricingError::Greeks(InputError(InvalidTime))`, although each exotic
  kernel defines its value at expiry.
  - **The change.** At `T = 0` an exotic is now dispatched straight to its
    kernel, and so are `price_option_with(ClosedFormBS)` and
    `OptionPricing::calculate_price_black_scholes`, which call
    `black_scholes()`. Each family returns its payoff at the spot, per unit
    and signed by the side. For example, a quanto call at `S = 105`,
    `K = 100` and a rate of 1.5 is 7.5, and a power call with exponent 2 is
    10925.
  - **What is unchanged.**
    - For `T > 0` the dispatch is unchanged: `d1` / `d2` are still computed
      first and the kernel's price is returned as before.
    - American and Bermuda options keep the errors they returned before.
  - **European options at `T = 0`.** A European option at expiry returns
    its intrinsic value, per unit and signed by the side, instead of
    `InvalidTime`, so every type with a closed form has a value at
    expiry. For example, a call at `S = 105`, `K = 100` is 5, and the short
    call is −5.
  - **Tests.** `tests/expiry_dispatch_test.rs` checks every exotic family:
    the price at expiry equals the contract payoff and the kernel, for both
    styles and sides. For the families whose price is continuous there, the
    price one minute out matches it to three places. It also checks that
    for `T > 0` the dispatcher returns exactly the kernel's price. It
    checks the European intrinsic at `T = 0` for call and put, long and
    short.

- **A barrier option's payoff at expiry pays its rebate, and its price at
  `T = 0` agrees with the Reiner-Rubinstein price as `T → 0`** (#826).
  - **The unhit knock-in rebate.** An `UpAndIn` / `DownAndIn` whose barrier
    was never hit paid zero at expiry. The closed form includes Haug's `E`,
    the rebate paid at expiry in exactly that case, so the price tended to
    the rebate and the payoff did not. `OptionType::payoff` now pays the
    rebate. For example, a down-and-in call with `S = 100`, `H = 95` and
    rebate 3 had a payoff of 0 and now has 3.
  - **The sign of the rebate.** A hit `UpAndOut` / `DownAndOut` paid its
    rebate with a positive sign for a short position. It now carries the
    side's sign, like the vanilla payoff: a short pays it, `-3` instead of
    `3`.
  - **The price at `T = 0`.** `barrier_black_scholes` returned
    `Options::payoff`, which scales by `quantity × contract_size`. It now
    returns the per-unit payoff, like the closed form. For example, a
    down-and-out call with `K = 90`, `S = 100`, quantity 5 and contract size
    100 was priced 5000 and is now 10. A payoff with no `Decimal`
    representation is reported as `PricingError::Options(PayoffError)`, no
    longer as an untyped `PricingError::MethodError`.
  - **How the barrier state is read at expiry.** The barrier is hit when
    `spot_max` reaches an up barrier or `spot_min` a down one. Without
    them it is judged from the final spot. `Options::payoff` and the `T = 0`
    price carry no path extremes, so they read only the underlying price.
    This is now documented on `PayoffInfo`, `Options::payoff` and
    `barrier_black_scholes`.
  - **Every other path is unchanged.** Barriers without a rebate (or with a
    zero one), long knock-outs and the closed form at `T > 0` return the same
    values as before. Callers of the payoff see the new values too:
    `Options::payoff`, `payoff_at_price` and the Monte-Carlo and telegraph
    terminal payoffs for a barrier with a rebate.
  - **Tests.** A test checks the closed form for four unhit knock-ins at
    10 days, 1 day and 1 hour against an independent `f64`
    Reiner-Rubinstein `C + E`; their price one minute out is the rebate to
    four places. Another sweeps all eight contracts, unhit, at the barrier
    and through it, for both sides. Away from the money, the price one
    minute out matches the price at expiry to three places.

- **A `Curve`'s risk `volatility` is the population standard deviation**
  (#840). `Curve::compute_risk_metrics` divided the sum of squared
  deviations `S` by `sqrt(n)`, a variance-like figure `sqrt(S)` times the
  standard deviation, where `Surface` took `sqrt(S / n)`. By owner decision
  both now take `sqrt(S / n)` from one crate-private helper,
  `population_std_dev`, so a curve and a surface over the same values
  report the same risk metrics digit for digit (a test pins it). The
  curve's volatility now equals the `std_dev` of its basic metrics, and the
  parametric VaR (`mean - 1.645 * volatility`), the expected shortfall
  below it and the Sharpe ratio (`mean / volatility`) move with it. The
  coefficient of variation already used the population standard deviation
  (#824) and does not move, nor does any `Surface` value: the helper runs
  the operations `Surface` ran, in the same order. Values that move, each
  checked against a 50-digit reference to the last of `Decimal`'s 28
  places (old -> new):

  | Curve values | volatility | VaR | Sharpe |
  | --- | --- | --- | --- |
  | `1..5` | `4.4721359549995793928183473373` -> `1.4142135623730950488016887242` | `-4.3566636459743081011861813699` -> `0.6736186898962586447212220487` | `0.6708203932499369089227521006` -> `2.1213203435596425732025330863` |
  | `0, 2, 4, 6, 8` | `17.888543819998317571273389349` -> `2.8284271247461900976033774484` | `-25.426654583897232404744725479` -> `-0.6527626202074827105575559026` | `0.2236067977499789696409173669` -> `1.4142135623730950488016887242` |
  | `1..9` | `20` -> `2.5819888974716112567861769332` | `-27.900` -> `0.7526282636591994825867389449` | `0.25` -> `1.9364916731037084425896326999` |
  | `x^2 mod 7`, `x = 0..20` | `9.165151389911680013176094387` -> `1.4142135623730950488016887242` | `-13.076674036404713621674675267` -> `-0.3263813101037413552787779513` | `0.2182178902359923812660974854` -> `1.4142135623730950488016887242` |
  | `1 x 9, 50` | `683.33657958578508985164090602` -> `14.7` | `-1118.1886734186164728059492904` -> `-18.2815` | `0.0086341053241674478498743449` -> `0.4013605442176870748299319728` |
  | `1, 10 x 9` | `23.0530041426274853302719339389` -> `2.7` | `-28.8221918146222133682973313295` -> `4.6585` | `0.3947424788413203281370360940` -> `3.3703703703703703703703703704` |

  The old volatility put the VaR below every sample of these curves, so
  the expected shortfall always read 0; with `1, 10 x 9` the VaR now sits
  above the dip and the shortfall is `1` (`0` before). A constant curve
  still reports a zero volatility and Sharpe ratio and a VaR at its level.

- **`OptionSeries` keeps every expired expiry** (#825). Its `chains` map is
  keyed by `ExpirationDate`, whose ordering in `expiration_date` 0.4.1
  clamped every past date to zero days, so two expired expiries compared
  equal and the second chain replaced the first. `expiration_date` 0.4.2
  orders, compares and hashes an expiration by the instant it resolves to,
  and the workspace requires it. Two past expiries now survive an
  `OptionSeries` JSON round trip as two chains in date order, and
  `OptionChain`'s ordering sorts expired chains by date instead of by
  symbol. `OptionBasicType`, a strategies `HashMap` key, gets an `Eq` that
  agrees with its `Hash`. The OptionStratLib API is unchanged.

- **`greeks::theta` and `greeks::vega` price exotic options with their own
  pricer** (#817). For every non-European `OptionType` they returned the
  European Black-Scholes closed form, although their docs said they fell
  back to the numerical Greeks. They now dispatch exactly as `delta` and
  `gamma` do: theta is `numerical_theta` (per day) and vega is
  `numerical_vega` divided by 100, signed by `Side` and scaled by
  `quantity × contract_size`. `numerical_vega` is per unit of volatility
  while `vega` is per vol point, so the division keeps one unit for every
  option type. European values are unchanged, digit for digit, and every
  type still reports `0` at expiry. Values change for **Barrier, Asian,
  Lookback, Binary, Chooser, Compound, Cliquet, Rainbow, Spread, Quanto,
  Exchange and Power** options; for example, a 90-day at-the-money
  up-and-out call (barrier 130, `sigma = 0.25`) moves from theta
  `-0.03251` / vega `0.19564` to `-0.00129` / `-0.01470`, and an arithmetic
  Asian call to `-0.01830` / `0.11327`. Each family agrees with a
  bumped-price reference of its own pricer within 1% (absolute floors
  `1e-4` per day, `5e-4` per vol point). American and Bermuda options, which
  have no closed form, now return `GreeksError::Pricing` from `theta` and
  `vega`, as they already did from `delta` and `gamma`, instead of the
  European value.
- **`numerical_vega` below one vol point** (#817). At `sigma < 0.01` the
  lower bump `sigma - 0.01` went negative and was folded back through
  `abs`, so the central difference spanned less than its stated `0.02` and
  came out about half the true vega near zero volatility. At
  `sigma <= 0.01` it is now the one-sided `(P(sigma + 0.01) - P(sigma)) /
  0.01`.
- **`numerical_theta` is implemented** (#796). It was a public stub:
  `Err(GreeksError::CalculationError)` for any expiry of at least 0.01
  years and `0` below. It is now a central difference in time, by owner
  decision with a one-day bump either way, like the other numerical Greeks'
  symmetric bump on their input: `(P(T - 1 day) - P(T + 1 day)) / 2`, per
  day, the unit of the closed-form `theta`, for one long unit contract as
  the other numerical Greeks. Within a day of expiry it is the one-sided
  `P(T) - P(T + 1 day)`, and `0` at expiry. Against the closed form on
  long European calls and puts (strikes 80/100/120, 30 days to a year,
  with and without a dividend yield) the error is at most `6.6e-6` per day,
  asserted at `2e-5`; put-call parity holds to `5.1e-11`, asserted at
  `1.5e-10`. The truncation `h^2/6 * V'''` grows as `T^(-5/2)` at the money,
  so a one-day step is coarse in the last week (`2.5e-4` per day at 7
  days, about 7 % at 1.5 days); the docs say so. `numerical_theta` joins
  the pricing greeks bench.

- **The geometric Asian payoff averages its fixings as a log-sum** (#806).
  `Payoff::payoff` for `OptionType::Asian { averaging_type: Geometric }`
  multiplied the fixings in `f64` and took the `n`-th root, so 100 fixings
  at `1e4` (`1e400`) overflowed and the payoff was an error, and small
  fixings underflowed. By owner decision the mean is `exp(mean(ln x_i))`,
  centred on the first fixing (`m * exp(mean(ln(x_i / m)))`, the same
  quantity) because the uncentred log-sum loses digits: against a 60-digit
  reference on 2 001 fixing sets (3 to 252 fixings, levels `1e-2` to `1e4`)
  the centred form is within 3 ulps (median 0), the uncentred one within 56
  (median 2). A zero fixing still gives a zero mean. Last-ulp changes,
  accepted by the owner, measured on 2 000 seeded fixing sets: 278 whose
  product overflowed or underflowed now have a finite mean; of the other
  1 722, 1 189 move in `f64`, all by at most 6 ulps but one (a 252-fixing set
  whose product went subnormal at `2e-323`, where the old mean was wrong in
  its fourth digit, `0.052412` for `0.052419`); at the `Decimal` boundary
  465 payoffs change, in their last digit (for example `9.80685118896913`
  to `9.80685118896914`). The pinned `(90, 100, 110)` call at strike 95
  moves from `4.66554934125961` to `4.66554934125965`; the exact payoff is
  `4.66554934125963638...`. Only payoffs evaluated on caller-supplied
  fixings change: the closed-form Kemna-Vorst price of
  `pricing::asian` reads no fixings and is unchanged.

- **The binomial pricer documents its cost, with no step limit** (#807).
  By owner decision `no_steps` stays unbounded and the cost is the
  caller's: `price_binomial` and `generate_binomial_tree` gain a `# Cost`
  section with the time (`N²` node evaluations, an American or Bermuda
  several times a European) and memory (`16 (N + 1)` bytes for the
  pricer, two `(N + 1)²` lattices, `32 (N + 1)²` bytes, for the tree) and
  Criterion timings (`pricing/binomial`: a European 2.2 ms at 200 steps and
  47 ms at 1 000, an American put 15 ms and 0.7 s, the tree 15 ms at 200).
  `price_binomial` now reserves its `N + 1` terminal nodes fallibly before
  pricing any of them, as the tree already reserved its lattices (#788): a
  step count whose nodes cannot be allocated (`usize::MAX`, `2^60`) is
  `PricingError::InvalidParameter` at once, where the pricer used to start
  pricing leaves and grow its vector one node at a time. Prices are
  unchanged. The stale `# Returns` of `price_binomial`, which said `f64`,
  now says `Decimal`.

- **No arithmetic operator in core, math or pricing can abort the caller**
  (#788).
  - **Lints.** The three crates deny `clippy::arithmetic_side_effects` and
    the truncating, wrapping and sign-dropping cast lints in production
    code. Every `+ - * /` on `Decimal`, `Positive` and the integers goes
    through `d_*` / `checked_*`. `clippy.toml` exempts only the unary minus
    on `Decimal`, which cannot overflow.
  - **Panics removed without a signature change.**
    - `Options` and `SpotPosition` `Display` printed a percentage or a fee
      total that aborted on overflow. They now print the fraction or the
      two fees.
    - `generate_binomial_tree` overflowed `no_steps + 1`. It now reserves
      its lattice fallibly and returns `PricingError::InvalidParameter`.
    - `simulate_returns` aborted with `capacity overflow` on a huge
      `length`. It now returns `DecimalError`.
    - The future and perpetual `LegGreeks` (`delta`, `rho`, `theta`)
      overflowed on extreme legs. They now return `GreeksError`.
    - `calculate_delta_neutral_sizes` reports `NegativePositionSize` when
      the first size rounds above the total.
    - The peak detection behind `Curve` shape metrics is checked.
  - **Scoped `#![allow(clippy::indexing_slicing)]` exceptions (#341).**
    The seven in `curves/utils.rs`, `volatility/utils.rs`, `telegraph.rs`,
    `cliquet.rs`, `binomial_model.rs`, `pricing/utils.rs` and `compound.rs`
    are gone.
  - **Test-only fixtures.** The unused `surfaces::utils` fixtures are
    `#[cfg(test)]`.
  - **`make scan-banned`.** It now also rejects the following in these
    three crates, with reviewed `scan-banned: allow` exceptions for
    modular RNG seeds:
    - `saturating_*` / `wrapping_*`;
    - `pos_or_panic!` / `spos!`;
    - `Duration::days(` and its siblings;
    - the aborting `Positive` helpers;
    - `Decimal` trig;
    - `assert!`;
    - `sum` / `product` over `Decimal` and `Positive`.

- **Unlimited amounts render as `Unlimited`, and handled data conditions
  no longer log as errors or warnings** (#801). Returned values do not
  change; only logging and display do.
  - `optionstratlib_core::model::DisplayMoney` (with `unlimited_label` and
    the `UNLIMITED` label) renders an amount as `$` plus the number, or as
    `Unlimited` / `-Unlimited` for the `Decimal::MAX` (= `Positive::MAX`) /
    `Decimal::MIN` sentinels. The `Strategy` and `ProtectivePut` `Display`
    impls (max profit, max loss, break-even) and the backtest / simulation
    terminal reports use it, so a short strangle's max loss shows
    `Unlimited` instead of `$79228162514264337593543950335.00`. The examples
    print max profit and max loss through it.
  - `OptionData::validate` logs a zero strike and a row without call or put
    prices at `DEBUG` instead of `ERROR`: the chain filters those rows out,
    for example the unpriced strikes of `Germany-40-…json`.
  - Every `find_optimal` search (the verticals, straddles, strangles,
    butterflies, condor, ladder, `PoorMansCoveredCall` and `CustomStrategy`)
    logs a skipped candidate (an invalid combination, an unscorable metric,
    a custom candidate whose break-evens cannot be recomputed) at `DEBUG`
    instead of `WARN`.

- **`Options::graph_data` returns a bounded series for any strike**
  (#797). The single-contract payoff chart (also drawn for a `Position`)
  stepped one unit of the underlying at a time across its price range, so
  a strike of 1e15, whose range is about 6e14 units wide, looped for hours.
  A range of up to 10,000 units is still charted one unit at a time, so
  every chart that returned before returns the same series (the golden
  `options_*` and `position_long_put` entries are unchanged). A wider range
  is sampled at 10,000 evenly spaced prices. No signature changes.

- **An inverted `BearCallSpread` / `BullPutSpread` is reported as a
  structural error, not as a loss of zero** (#803). `get_max_loss` used
  `ProfitLossError::MaxLossError` both for "the worst case still gains" and
  for legs that do not form the strategy (the long call below the short
  call, the short put below the long put; reachable through the `pub` legs
  or `Deserialize`). Since #788 the profit area and ratio read
  `MaxProfitError` / `MaxLossError` as zero profit or zero loss, so an
  inverted spread got a profit ratio of `Decimal::MAX` instead of an error.
  The two inverted cases now return `StrategyError::InvalidStrategy`, which
  `get_profit_area` / `get_profit_ratio` propagate. `MaxProfitError` and
  `MaxLossError` now carry only sign reports. The four "Net premium
  received is negative" reports are sign reports (the best case still
  loses) and keep their kind. Values for valid strategies are unchanged.
  Migration: match `StrategyError::InvalidStrategy` instead of
  `MaxLossError` to detect an inverted vertical.

- **`Curve::merge` and `Surface::merge` keep their grid inside the common
  range** (#795). Both resample on `min + step * i` with
  `step = span / steps` rounded at 28 places, so on a span with a 27- or
  28-place mantissa `step * steps` could land past the maximum and the merge
  failed on its last grid point (`interpolation target
  10.000000000000000000000000020 is outside the supported range` for a curve
  sampled at `x = (10 / 511) * i`). Each grid coordinate is now clamped to
  the range's maximum. A merge that succeeded before had no point past it, so
  its result is bit-identical: on 964 merges of curves and surfaces sampled
  at `(end / n) * i` (`n` up to 119, four ends), the 684 that succeeded
  before return the same points, and 272 of the 280 that failed now succeed
  (the other 8 are two-point curves that cubic interpolation rejects).
  Regression tests cover the `10 / 511` curve and a surface ending on
  `10.000000000000000000000000018`, and the math benches' fixture goes back
  to the natural `step * i` abscissas it had avoided.

- **`CustomStrategy::find_optimal` no longer leaves stale break-evens
  when they cannot be recomputed** (#791). The search only logged a
  failed recomputation, so it scored candidates against stale
  break-evens and could finish with the best legs and the break-evens of
  other legs. A candidate whose break-evens cannot be recomputed is now
  skipped with a warning and the search continues; when the best legs
  cannot be applied at the end, the strategy is left exactly as it was
  before the search, with an error logged; since #793 `find_optimal`
  also returns the error. A run where every recomputation succeeds
  returns the same result as before.

- **Strategies, backtesting and visualization no longer abort on extreme
  or degenerate input** (#788). Every site below panicked; each now
  returns a typed error or, where the API has no error channel, the
  answer the input defines. Values for every input that worked before
  are unchanged.
  - `Strategies::get_volume` (the trait default, `LongStrangle`,
    `ShortStrangle`) sums the quantities with `checked_add`; legs at
    `Positive::MAX` aborted with `Positive arithmetic overflow in add`.
  - `LongButterflySpread::new` / `ShortButterflySpread::new` double the
    wing quantity with `checked_mul_dec` and report the overflow; their
    `validate` treats a doubled wing past `Positive::MAX` as not matching
    the body.
  - `calculate_profit_ratio` divides and scales with `d_div` / `d_mul`
    (`Division overflowed` before); `ShortButterflySpread::get_profit_area`
    adds the two wing profits with `d_add`;
    `AdjustmentTarget::is_satisfied` reads a deviation past `Decimal` as
    not met.
  - `CustomStrategy` with no legs no longer indexes `positions[0]`; see the
    `BasicAble` entry under "Changed — breaking".
  - Numbers that were made up are reported or justified. `get_profit_area`
    and `get_profit_ratio` of `LongCall`, `LongPut`, `ShortCall`,
    `ShortPut`, the four verticals, the two butterflies, `IronCondor`,
    `IronButterfly` and `BullCallLadder` turned a `NaN`, infinite or
    out-of-range `f64` into `0` through `Decimal::from_f64(..)
    .unwrap_or(Decimal::ZERO)`; they now return
    `StrategyError::NumericConversion` (the profit ratio of a long call or
    put, whose maximum profit is unlimited, is one such case). The
    `Positive::new_decimal(x).unwrap_or(Positive::ZERO)` fallbacks (premium
    refreshes, max-profit and max-loss figures, delta adjustment
    quantities, the probability analysis `risk_reward_ratio`,
    `BullCallLadder`'s profit-area bases) now propagate the error: for an
    `abs()` or an already checked sign the value is unchanged, and a
    negative ratio or a crossed base is an error instead of zero. The max
    profit / max loss floors of `Collar`, `CoveredCall` and `ProtectivePut`
    (`x.max(Decimal::ZERO)`) stay: a worst case that still gains loses
    nothing, and the comment at each site says so. The single-contract
    chart leaves out a price whose payoff cannot be computed instead of
    drawing it at zero.
  - The profit area and profit ratio of every strategy no longer read a
    failed `get_max_profit` / `get_max_loss` as zero (44
    `unwrap_or(Positive::ZERO)` calls and four `Err(_) => 0` arms). A
    strategy reporting the sign of its own extreme (`MaxProfitError`: the
    best case still loses; `MaxLossError`: the worst case still gains)
    keeps its existing meaning, zero profit or zero loss (shown as an
    unbounded ratio where the strategy already did so); every other error
    (pricing, overflow, empty break-evens) is now returned. No test
    expectation changed. The payoff chart omits its current-price marker
    and label when the payoff there cannot be computed, instead of
    labelling it `0.00`.
  - `ProtectivePut`'s `Display` skips the break-even line when the `pub`
    `break_even_points` is empty.
  - The payoff chart (`impl_graph_for_payoff_strategy!`) pads its vertical
    extent with checked arithmetic; a range where no price could be scored,
    or an extent past `Decimal`, charts as empty like the macro's other
    failures (`Subtraction overflowed` before). The single-contract chart's
    fallback range around a strike near `Positive::MAX` does the same.
  - `Graph::show` writes the page and starts the platform opener itself and
    returns `GraphError::Io` when either fails; `plotly`'s `Plot::show`
    aborted when the temp file could not be written or no opener exists
    (`xdg-open` on a headless server).
  - The single-leg backtest rejects a walk step with a negative index
    (`Xstep::previous` from the first step) with a `SimulationError`;
    `as usize` wrapped it into a holding period near `usize::MAX`.
  - Every scoped `#![allow(clippy::indexing_slicing)]` in
    `optionstratlib-strategies` (23 files) and `optionstratlib-visualization`
    (3 files) is gone: the positional reads use slice patterns, `.get()` or
    iterator zips.
  - `make scan-banned` bans `pos_or_panic!` in production code, recognizes
    `#[cfg(all(test, ...))]` as a test gate and no longer counts braces
    inside one-line string literals, which ended or extended a skipped test
    body at the wrong line.

- **`CustomStrategy` no longer discards a failed break-even recomputation
  after an edit** (#784). `add_position`, `modify_position` and
  `replace_position` recomputed the break-evens but ignored a failure, so
  an edit whose break-evens could not be computed reported success with
  the previous break-evens. They now go through the shared helper of
  #780: a failed recomputation restores the strategy (legs and
  break-evens) and returns the error as a `PositionError`. `CustomStrategy`
  keeps its stricter contract for an edit that leaves it invalid: it is
  still rejected with the same `PositionError`, and the strategy is now
  also restored instead of keeping the rejected leg. `new` and every
  successful edit return the same results as before. No signature
  changes.

- **Every example and bench builds and runs, and a smoke pass keeps it so**
  (#787). All 185 example binaries (`examples_*` and
  `osl-example-direct-*`) were run to completion with a chromedriver that
  matches Chrome, so every PNG/SVG export ran for real, and the 14 Criterion
  targets (558 benchmarks) ran once in test mode; the evidence is
  `docs/release/0.22/examples-benches.md`. Three real defects came out of it.
  `examples_chain`'s `default` features omitted the package feature `async`
  that `async_chain_ops` and `async_ohlcv` require (#774 made them
  `required-features`), so `cargo run -p examples_chain --bin async_ohlcv`
  failed to find the binary. `async_ohlcv` read `../../examples/Data/...`,
  a path that only resolves from two directories down, and logged the error
  instead of returning it, so it exited 0 having read nothing; it now reads
  `examples/Data/cl-1m-sample.zip` and propagates the error. Two examples
  found no strategy and printed a strategy with zero strikes: `creator` searched
  a snapshot taken at the expiration instant (every delta is 0 or 1) for a
  delta between 0.15 and 0.3, and `option_chain_raw_delta` searched the
  S&P 500 chain for a price range copied from the DAX example (21,600 to
  21,700); they now use a range their data can satisfy. `make smoke-examples`
  builds and runs every example binary (`SMOKE_EXPORT=1` makes a failed image
  export a failure instead of `needs-webdriver`), `make smoke-benches`
  compiles every bench and runs each once, and the new Examples and benches
  workflow runs both on every pull request (examples without a WebDriver); the
  weekly Static Export workflow runs every example again with a matching
  Chrome and chromedriver, where an export failure is a bug.

- **Simulation, market and analytics no longer abort on extreme input**
  (#788). Each site below was a raw `Decimal`, `Positive` or `usize`
  operator, or an unchecked allocation; each now returns a typed error, and
  every input that worked before returns the same value.
  - Chain metrics: `volatility_skew` on a zero spot, `premium_concentration`
    on a book whose premia are all zero, `dollar_gamma_curve`,
    `delta_gamma_curve` and `price_shock_curve` when the spot square or the
    shock leaves the `Decimal` range, `iv_surface`, `smile_dynamics_surface`
    and `volume_profile_surface` on day counts that overflow their scaling,
    and `bid_ask_spread_curve` on a quote whose sides sum past the range.
    These return `CurveError::MetricsError` or `SurfaceError::AnalysisError`.
  - The grid surfaces (`vanna_volga_surface`, `delta_gamma_surface`,
    `volatility_sensitivity_surface`, `price_shock_surface`,
    `time_decay_surface`, `theta_surface`, `charm_surface`, `color_surface`)
    return `SurfaceError::OperationError` naming `price_range` or
    `vol_range` when the upper bound is below the lower one. With zero steps
    the bounds are still not compared. `bid_ask_spread_curve` rejects a
    crossed quote (ask below bid) the same way.
  - `calculate_optimal_price_range` returns `ChainError::ChainBuildError`
    naming `implied_volatility` when the four-sigma band reaches below zero
    (`4·σ·√T > 1`), and naming `underlying_price` when the spot and the
    strike are both zero. Overflowing bounds return an error too.
    `OptionChain::to_build_params` checks the sum of the bid-ask spreads,
    and `OptionChain::get_random_positions` reserves its positions with
    `try_reserve_exact`.
  - Every walk kernel reserves its path with `try_reserve_exact`. A walk
    size of `usize::MAX`, or one whose points cannot be allocated, returns
    `SimulationError::InvalidParameters`; `Vec::with_capacity` overflowed
    or aborted the process there. `Simulator::new` no longer reserves an
    unallocatable `size` up front.
  - `Position::diff_position_pnl` checks the realized and unrealized
    differences, and the percentages in `OptionDataPriceParams`'s
    `Display` render as `n/a` when they leave the range.
  - The metric surfaces no longer stand in a value for a failed step
    (#639): `iv_surface`, `smile_dynamics_surface` and
    `volume_profile_surface` propagate a failed square root instead of
    reading it as 1, and the grid surfaces propagate a grid point that is
    not a valid `Positive` instead of pricing at a spot of 1 or a
    volatility of 0.01. Neither fallback fires on a finite input, so every
    value returned before is unchanged.
  - The `Display` of `PnLMetricsStep` and `Ystep` and the quote cells of
    the chain table round the `Decimal` (`round_dp`) instead of calling
    `Positive::round_to`, which routes through `unwrap_or_panic`; the
    printed digits are unchanged.
  - The file-wide `#![allow(clippy::indexing_slicing)]` in
    `simulation/randomwalk.rs`, `simulation/simulator.rs`,
    `simulation/traits.rs` and `chains/csv.rs` is gone (#341). `traits.rs`
    had no indexing left; `csv.rs` binds its seven fields with a slice
    pattern. The allow now sits only on the `Index` / `IndexMut` impls of
    `RandomWalk` and `Simulator`, which panic out of bounds by the `std`
    contract; `get_step` and `get_random_walk` are the checked forms.
  - `make scan-banned` also rejects `.round_to_nice_number()` and the
    `*_unchecked` forms in production code.

- **`OptionDataPriceParams`'s `Display` rounds its numbers instead of
  truncating their text** (#788). The `{:.3}` / `{:.4}` / `{:.2}` precisions
  were applied to already rendered strings, so a 5% rate printed as `5.%`,
  a missing one as `No%`, and 30 days as `0.08` years. The spot now prints
  to three places, the years to four and the rate and dividend yield as
  percentages to two, each rounded half to even (`100.000`, `0.0822`,
  `5.00%`); a missing field prints `None`.

- **Every strategy refreshes its break-evens on `add_position` /
  `modify_position`** (#780). #771 did this for `Collar`, `CoveredCall`
  and `ProtectivePut`; the other strategies kept the break-evens of their
  previous legs after an edit. `BullCallSpread`, `BearCallSpread`,
  `BullPutSpread`, `BearPutSpread`, `IronCondor`, `IronButterfly`,
  `LongButterflySpread`, `ShortButterflySpread`, `BullCallLadder`,
  `LongStraddle`, `ShortStraddle`, `LongStrangle`, `ShortStrangle`,
  `PoorMansCoveredCall`, `LongCall`, `LongPut`, `ShortCall` and `ShortPut`
  now recompute them in `add_position` and `modify_position` (and
  `ShortStrangle::replace_position`), through one shared helper with the
  #771 contract:
  - a strategy that validates after the edit gets fresh break-evens; if the
    recomputation fails, the leg and the break-evens are restored and the
    error is returned as a `PositionError` (`ShortStrangle` used to keep
    the new leg in that case);
  - a strategy that does not validate after the edit (one assembled leg by
    leg from `Default` with legs still unset, or an edit that leaves the
    legs inconsistent, which `add_position` has always accepted) reports
    no break-evens instead of stale ones, and the edit stands.

  The constructors fill their legs through a private path, so `new` and
  `get_strategy` results are unchanged (except the single-leg constructors,
  below). No signature changes. Tests: for one strategy of each family
  (vertical spread, iron condor, butterfly, ladder, straddle, strangle,
  PMCC, single leg) an edit leaves the break-evens of the same legs built
  by `new`, and an edit whose premium total overflows is rejected with the
  strategy unchanged.

- **`LongCall::new`, `LongPut::new`, `ShortCall::new` and `ShortPut::new`
  now report their break-even** (#780). They returned an empty
  break-even list, while every multi-leg constructor computes it; with
  `add_position` now refreshing break-evens, the same leg would have had a
  break-even when added to a `Default` strategy and none when built by
  `new`. Behaviour change: **a single-leg strategy built by `new` now has
  one break-even** (the strike shifted by the premium net of fees per
  contract), and `new` returns a `StrategyError` when
  that computation fails. No signature changes. The visualization golden
  `graph_data.json` is regenerated for the four single-leg charts only,
  which had pinned the missing (or, for the two built as `Default` +
  `add_position`, never refreshed) break-even; every other chart is
  unchanged. Tests: each single-leg strategy built by `new` matches the
  same leg added to a `Default` one (chart data for `LongCall` and
  `ShortPut`, whose `new` is public; strategy state for `LongPut` and
  `ShortCall`).

- **Path-based pricers include the dividend yield in the drift** (#756).
  `pricing::telegraph` simulated the log price with drift
  `r - sigma^2/2` and `pricing::monte_carlo_option_pricing` grew it by
  `1 + r dt + sigma dW`; both ignored `option.dividend_yield`, so on a
  dividend-paying underlying they priced as if `q = 0`, overpricing calls
  and underpricing puts against Black-Scholes. The drifts are now
  `r - q - sigma^2/2` and `(r - q) dt`. No signature changes, but **results
  change for every option with `dividend_yield > 0`** (including
  `OptionPricing::calculate_price_telegraph`, which calls the telegraph
  kernel); options with `q = 0` price exactly as before on the same seed.
  The other path-based code (`price_option_monte_carlo`, which takes
  caller-supplied terminal prices, and the Heston variance simulation) has
  no price drift and is unchanged. Tests: with `q = 3%` and a fixed seed,
  the telegraph kernel with switching disabled (call and put, 40 000
  paths) and `monte_carlo_option_pricing` (20 000 paths) land within
  Monte-Carlo tolerance of Black-Scholes with the same `q` (call 8.6525),
  where the old drift sat near the `q = 0` price 10.45.

- **Heston volatility simulation draws a normal Wiener increment** (#742).
  `volatility::simulate_heston_volatility` drew `dW` as
  `uniform[0, 1) * sqrt(dt)`, a strictly positive shock with mean
  `sqrt(dt) / 2`, so the `xi * sqrt(v) * dW` term pushed the variance up on
  every step and the path drifted far above `theta` instead of
  mean-reverting to it. `dW` is now a standard normal from the caller's
  `rng` scaled by `sqrt(dt)`, i.e. `N(0, dt)`. No signature changes, but
  **simulated Heston paths change**: the same seed yields a different path,
  and results pinned against the old draw must be re-baselined. Tests: a
  seeded 100 000-draw check that the increment has zero mean and variance
  `dt`; a 200-path seeded check that the long-run mean variance
  (`kappa = 2`, `theta = 0.04`, `xi = 0.3`, Feller satisfied) lands within
  0.004 of `theta` (0.0406), where the uniform draw gave 0.64.

- **Visualization crate debt carried over from the monolith** (#690).
  `impl_graph_for_payoff_strategy!` now names every item it expands to
  (`Graph`, `GraphData`, `Series2D`, `Positive`, `Decimal`, the strategy
  traits, `tracing`) through `$crate` paths, so callers no longer need to
  import them; the hidden `optionstratlib_visualization::__private` module
  carries the re-exports and is not public API. `write_png` and `write_svg`
  document that they block the calling thread, and lose a stale `# Safety`
  section and `LC_ALL` note that described code that does not exist. The
  `GraphError` conversions from `CurveError` and `SurfaceError` are `#[cold]`,
  an orphan comment block in `error/graph.rs` is gone, and the module docs
  name the real `GraphSurface` variant and no longer suggest extending
  `GraphData` from a downstream crate. Chart data is unchanged.

- **`CoveredCall`, `ProtectivePut` and `Collar` report a failing option
  leg in `calculate_profit_at`** (#731). Each replaced a leg whose
  `Position::pnl_at_expiration` failed (a cost or an income that leaves the
  `Positive` range, a payoff out of range) with `unwrap_or(Decimal::ZERO)`,
  so the payoff, the charts and the expiry P&L built on it silently dropped
  the leg. The leg's `PositionError` now propagates as the `PricingError`
  the signature already returns; no signature changes. Tests: a leg with a
  premium at the top of the `Positive` range on two contracts makes
  `calculate_profit_at` return an error for each strategy and each leg of
  the collar, where it used to return the share leg's P&L alone.

- **`CoveredCall`, `ProtectivePut` and `Collar` mark to market in
  `calculate_pnl`** (#728). They returned `calculate_pnl_at_expiration`, the
  payoff at expiry, as the "unrealized" P&L and ignored the
  `expiration_date` and `implied_volatility` arguments. Each option leg is
  now marked through `Position::calculate_pnl` with the given date and
  volatility, the share leg is valued at the current price, and the legs
  are summed with `PnL::try_add`, as every other strategy does:
  - `unrealized` is the change in the book's value since entry: the option
    legs' `quantity * (BS(now) - BS(entry))` plus the shares'
    `(price - cost basis) * quantity`;
  - `realized` is the entry cash flow, income less costs. `initial_costs`
    now also counts the option legs' fees (it held the share cost and the
    put premium only); `initial_income` is the short call's premium as
    before.

  The values that move are those of `calculate_pnl` on the three types:
  `unrealized` goes from the expiry payoff to the change in value, and
  `realized` from `None` to the entry cash flow. No test or golden value
  pinned them. `calculate_pnl_at_expiration` is unchanged. Tests: for each
  type the P&L is the sum of its legs', it moves with the date and the
  volatility, and with every leg entered at its Black-Scholes value and no
  fees it converges to the expiry P&L as the time to expiry goes to zero
  (the gap is 0.0146 at one day for the covered call and the collar and
  zero from 0.01 days).

- **Mark-to-market P&L counts every contract** (#725).
  `Position::calculate_pnl` (`crates/optionstratlib-analytics/src/pnl/model_impls.rs`)
  reported the unrealized P&L as `BS(now) - BS(entry)` for one contract and
  ignored `option.quantity`, so every leg sized above one lot, a
  butterfly's doubled body included, and every strategy summing those legs
  (`calculate_pnl` of the single-leg strategies, verticals, straddles,
  strangles, condors, butterflies, `PoorMansCoveredCall` and
  `CustomStrategy`, and
  `diff_position_pnl`) reported the wrong value. It is now
  `quantity * (BS(now) - BS(entry))` through `d_sub` / `d_mul`; the side
  is applied once, by `black_scholes`, which negates a short. Results
  change for every leg with `quantity != 1`, by the factor `quantity`;
  one-lot legs are unchanged.
  - `Options::calculate_pnl_at_expiration` signed a short option's premium
    twice (`initial_price * quantity` with the already negative short
    price), so `Positive` rejected the income and the call failed for every
    short option. It now records `-price * quantity` as income, like
    `Options::calculate_pnl`, through a shared checked helper.
  - Paths checked and already scaled: `Position::calculate_pnl` realized
    (`premium_received - total_cost`, both per-quantity), and
    `Position::calculate_pnl_at_expiration` (intrinsic value, cost and
    income all carry the quantity); `Options::calculate_pnl` (unrealized and
    premium already scaled, now through checked arithmetic) and
    `Options::calculate_pnl_at_expiration` realized (`payoff_at_price`
    scales). `CoveredCall`, `ProtectivePut` and `Collar` build their P&L
    from `calculate_profit_at`, which sums quantity-scaled leg profits.
  - No pinned value in the workspace moved: every test, example and
    doctest that pins a P&L uses one-lot legs. The legs above one lot that
    the suite reaches are in the panic-freedom properties, which assert no
    value. New tests: an N-lot `Position` (N = 2, 3, 10, 2.5) reports N
    times the one-lot unrealized and realized P&L, cost and income, for
    long and short calls and puts; a short leg is the negated long one; the
    expiry P&L scales the same way; a 1/2/1 call butterfly's legs add up to
    the change in value of the book priced by `black_scholes`, and as a
    `CustomStrategy` its P&L equals the sum of its legs and scales with
    the lot count; a `BullCallSpread` of 3 reports three times the
    one-lot P&L; `Options` scales and a short `Options` reports its expiry
    income.
  - The `LongButterflySpread` and `ShortButterflySpread` mark-to-market
    tests that #706 dropped are restored on its 1/2/1 fixtures (spot 100,
    30 days, `sigma = 0.2`, `r = 5 %`, `q = 1 %`, entry book value 1.629777
    per lot), pinned to hand-computed changes in the book's Black-Scholes
    value (`erfc`-based, outside the library) for one lot and three times
    that for three lots: -1.380131 at spot 90 (20 days, `sigma = 0.2`),
    +1.513339 at 100 (20 days, `sigma = 0.1`) and -1.308170 at 110 for the
    long butterfly, the negations for the short, within `1e-8` per lot. They
    also equal the change the library's `black_scholes` gives with the body
    counted twice, and the doubled body reports twice a one-contract body.

- **Pricing kernels report a failed numeric step instead of substituting a
  value** (#639). Every `unwrap_or(0)`-style fallback on a failed step in
  `optionstratlib-pricing` now propagates a typed `PricingError`;
  legitimate domain floors and parameter defaults stay, each with a comment.
  - A failed normal CDF (`big_n(..).unwrap_or(0)`) is an error in the
    Asian, binary, chooser, cliquet, compound and floating-strike lookback
    kernels. `big_n` fails only when a `Decimal` cannot reach `f64`, which
    no `Decimal` input does today, so no price changes.
  - `price_option_monte_carlo` returns `PricingError::Options` when a path's
    payoff is out of the `Decimal` range; that path used to count as a zero
    payoff.
  - The compound bivariate normal reports an unconvertible argument
    (`PricingError::NonFinite`) instead of reading it as `0`, and its
    internal `f64` normal CDF is now the total `erfc(-x/√2)/2` (the
    expression `big_n` evaluates), with no `Decimal` round trip to fall
    back to `0` or `0.5`.
  - A compound at its own expiry values the underlying at its payoff. Its
    Black-Scholes valuation is undefined at `T = 0`, and that failure was
    read as a worthless underlying: every compound call priced `0` and every
    compound put its full strike `K1`. At `S = 100, K = 5` a call-on-call
    goes from `0` to `90`; a put-on-put stays `5` because its underlying put
    expires worthless.
  - The chooser reports a choice date that is not a valid year fraction
    (`PricingError::Positive`) instead of choosing today.
  - The arithmetic Asian floors its moment-matched variance at zero (Jensen:
    `M2 ≥ M1²`, so a negative value is round-off) instead of falling back to
    the input volatility when the square root failed.
  - Barone-Adesi-Whaley's convergence tolerance is a `Decimal` constant
    (`1e-6`), removing an `f64` conversion and its fallback.
  - Kept, now commented: the barrier's unset rebate (zero), the exchange and
    spread second dividend yield (zero), the rainbow second dividend yield
    (the first asset's) and correlation (`0.5`), and the cliquet local cap and
    floor (`10 %`, `0 %`); these are parameter defaults, documented on each
    pricer. The cliquet reset-date sort uses `f64::total_cmp` (NaN is
    rejected before it). The telegraph process keeps its documented limits
    inside the infallible `next_state` and its diagnostic fields.

- **The fixed-strike lookback follows Conze-Viswanathan** (#647).
  `lookback_black_scholes` priced a fixed-strike lookback as a vanilla plus
  an ad hoc premium `S σ√T (N(λ) - ½) / 2`. It now uses the closed form of
  Conze and Viswanathan (1991), Haug, *The Complete Guide to Option Pricing
  Formulas*, §4.15.2, on a new contract (`S_max = S_min = S`), with the
  `b → 0` limit of its `σ²/(2b)` term below a carry of `1e-8`.
  - Calls on Haug's grid (`S = 100, T = 0.5, r = b = 10 %`), rows
    `K = 95, 100, 105`, columns `σ = 10, 20, 30 %`: from `10.8209, 12.6609,
    15.1705`; `6.8087, 9.4397, 12.3250`; `3.8380, 6.8563, 9.9210` to
    `13.2687, 18.9263, 24.9858`; `8.5126, 14.1702, 20.2296`; `4.3908,
    9.8905, 15.8512`.
  - Puts used the same ad hoc premium and move onto the same formula:
    `0.6899, 4.4448, 8.9213`; `3.3917, 8.3177, 13.1579`; `8.1478, 13.0739,
    17.9140` on that grid.
  - At `σ = 0` the extremum of the deterministic path `S e^(bτ)` is taken
    over the whole life, so with a negative carry the call keeps the
    payoff on the starting spot instead of the lower forward (and the put,
    symmetrically, with a positive carry). Other zero-volatility prices are
    unchanged.

- **`volatility/utils.rs` drops its impossible zero fallbacks** (#715).
  `constant_volatility`, `ewma_volatility`, `simulate_heston_volatility`
  and the `implied_volatility` grid built `Positive` values from a count, a
  square root, a variance floored at zero and a grid point in `(0, 1)` with
  `.unwrap_or(Positive::ZERO)`. None of them can be negative, so each now
  uses `?` with a comment saying why it cannot fire; a failure, if one ever
  occurred, would surface as `VolatilityError::PositiveError` instead of a
  silent zero. The grid's `min_by` compared `Decimal` differences with
  `partial_cmp(..).unwrap_or(Equal)` and now uses `Ord::cmp`, through a
  fallible reduction that keeps the first of equally close candidates as
  `min_by` did. No returned value changes. The `uncertain_volatility_bounds`
  part of #715 is under Changed — breaking.

- **The `default` API-change report covers the real default surface**
  (#688). `scripts/report_api_changes.py` still defined it as the 0.21
  feature set, `synthetic` alone, which under `--only-explicit-features`
  left out `analytics`, `strategies`, `backtest`, `visualization` and `io`,
  so API changes there were reported only under `all`. `SURFACES["default"]`
  now spells out the facade default (`pricing`, `market`, `analytics`,
  `strategies`, `simulation`, `backtest`, `visualization`, `synthetic`,
  `io`), and the self-test (`make check-api-report`, and the CI step before
  every report) fails, naming both lists, when it differs from the root
  `Cargo.toml` `[features] default`. It also fails when a surface names a
  feature the facade does not declare. The `plotly`, `static_export` and
  `async` surfaces stay each feature alone, as `default-features = false`
  builds them; their comment now states what each resolves. On Python
  before 3.11, which has no `tomllib`, a narrow `[features]` parser reads
  the manifest, and the self-test checks it against `tomllib` where both
  exist.

- **`generate_binomial_tree` exercises at the Bermuda root, and its root is
  `price_binomial` exactly** (#716). The Bermuda branch still exempted the
  root, although `price_binomial` exercises there when a date lies within
  `dt / 2` of `t = 0`. A put on `S = 50`, `K = 100`, `sigma = 0.2`,
  `r = 5%`, `T = 1` with dates `[0, 0.5]` rooted at 45.122942450071399,
  47.530991202833290, 45.122942450071418, 47.530991202833271 and
  47.530994940751114 on 1, 2, 3, 10 and 50 steps, and with the single date
  `0.001` between 45.1229424500714 and 45.1246968721929; it now roots at 50
  (the short tree -50), as `price_binomial` gives. On a coarse lattice a date
  away from `t = 0` snaps to the root by the same rule (`0.25` on one step,
  `dt / 2 = 0.5`). The tree also compares American and Bermuda nodes with
  their intrinsic value in `Decimal`, as `price_binomial` does, instead of
  through an `f64` round trip, and builds each spot with `lattice_spot`'s
  factor order (`S · u^ups · d^downs`), so the two walk the same numbers.
  Asset nodes move by at most `1e-26`, interior American nodes by at most
  `3.0e-13` and interior Bermuda nodes by at most `5.5e-14`; European
  option nodes do not move. The 16-digit root #708 introduced is full
  precision again (`S = 90`, `K = 100` American put at 1 step:
  10.57571371000381 -> 10.575713710003807657116643569). Over 80 American
  and 320 Bermuda cases (calls and puts, long and short, `S` in 50, 90,
  100, 130, 1, 2, 3, 10 and 50 steps, four schedules) the tree root now
  equals `price_binomial` digit for digit; the American test that allowed
  `1e-12` asserts equality, and new tests pin the Bermuda root to
  intrinsic, the `dt / 2` rule and the equality.

- **`generate_binomial_tree` exercises at the American root** (#708). The
  root node took the continuation value without comparing it with the
  intrinsic value, unlike every other American node and unlike
  `price_binomial`. A deep in-the-money American put therefore rooted below
  its intrinsic value: `S = 50`, `K = 100`, `sigma = 0.2`, `r = 5%`, `T = 1`
  gave 45.122942450071399 at 1 step, 47.530991202833290 at 2, 49.501247919268228
  at 10 and 49.900049983337489 at 50, where `price_binomial` gives 50. The
  tree root, and with it `Options::calculate_price_binomial_tree`, now
  returns 50 (the short tree -50). Where holding beats exercising, the root
  keeps the continuation value but now goes through the same `f64`
  comparison as every other node, so it carries 16 significant digits:
  `S = 90`, `K = 100` put at 1 step, 10.575713710003807657116643569 ->
  10.57571371000381. New tests pin the deep in-the-money root to intrinsic
  for 1, 2, 10 and 50 steps, check that the European continuation is below
  it, and hold the tree root to `price_binomial` within `1e-12` for American
  calls and puts, long and short, out of, at and in the money, at 1, 2, 3,
  10 and 50 steps (largest gap `5.4e-14`). No existing pinned value moved.

- **A gap put prices as a put** (#649). `binary_black_scholes` formed a gap
  option as `asset-or-nothing - K · cash-or-nothing` for both styles, which
  for a put is the negated put. The put is now
  `K · cash-or-nothing put - asset-or-nothing put`
  (`K e^(-rT) N(-d2) - S e^(-qT) N(-d1)`, Haug, *The Complete Guide to
  Option Pricing Formulas*, §4.19.3 with the trigger equal to the strike),
  so with the library's single strike it equals the vanilla put. Every gap
  put changes sign: `S = 100, K = 105, T = 0.5, σ = 25 %, r = 5 %,
  q = 1 %` goes from `-8.6574` to `8.6574` long (and from `8.6574` to
  `-8.6574` short). Gap calls are unchanged. The side is no longer stripped
  and reapplied around the legs, which already carry it.

- **The quanto pricer reads the foreign rate, and Kirk's spread
  approximation the second dividend yield** (#650).
  - `quanto_black_scholes` never read `ExoticParams::quanto_foreign_rate`
    and grew the forward at `r_d - q - ρ σ_S σ_E`. It now uses
    `r_f - q - ρ σ_S σ_E`, discounted at `r_d` (Haug, *The Complete Guide to
    Option Pricing Formulas*, §5.16.1): the put at `S = 100, K = 95,
    T = 0.5, r_d = 10 %, r_f = 3 %, q = 5 %, σ = 20 %, ρ = 0, E_p = 1` goes
    from `2.4648` to `3.5165`. Prices change whenever `quanto_foreign_rate`
    differs from the domestic rate; when it is `None` the pricer keeps
    `r_f = r_d`, so those results are unchanged.
  - `spread_black_scholes` (Kirk, `K ≠ 0`) took the adjusted strike as
    `(S2 + K) e^(-rT)` and ignored `spread_second_asset_dividend`, which is
    right only when `q2 = r`. It now works on present values,
    `S2 e^(-q2 T) + K e^(-rT)`, for the adjusted strike and for the Kirk
    weight `F2 / (F2 + K)` (Haug §5.4.2 on forwards), so it meets the
    Margrabe branch as `K → 0` and satisfies spread put-call parity exactly.
    Prices change whenever `q2 ≠ r`: `S1 = 100, S2 = 95, K = 1e-3,
    T = 0.5, r = 5 %, q1 = 5 %, q2 = 0` moves by about `-1.24` onto
    Margrabe. Futures-style inputs (`q1 = q2 = r`), Haug's Kirk example
    (`2.1670`) included, are unchanged. A present value of `S2 + K` flushed
    to zero by its discount factors now prices at its limit (the call worth
    `S1 e^(-q1 T)`) instead of failing the logarithm.
  - Re-baselined unit tests in `pricing::spread`: the zero-volatility
    branch tests move from `r = 25 % / -10 %` to `r = 0` with `S1 = 130`
    and `q1 ∈ {0, 10 %}` (the Kirk weight is exact only at `r = 0` now);
    the underflow test sets `q2 = 100` alongside `r = 100`; and the spread
    put-call parity bound tightens from `2.0` to `1e-9`, since the old gap
    of `S2 (1 - e^(-rT)) ≈ 1.23` is gone.

- **The barrier pricer follows Reiner-Rubinstein for all eight contracts,
  pays each rebate on the right leg and honours `Side`** (#646).
  `barrier_black_scholes` now composes every contract from Haug's
  §4.17.1 terms `A`–`F` (*The Complete Guide to Option Pricing Formulas*).
  - Up-barrier calls and every barrier put used the wrong terms. Without a
    rebate, at `S = 100, T = 0.5, r = 8 %, q = 4 %, σ = 25 %`, the
    up-and-out call (`K = 100, H = 105`) goes from `-2.2564` to `0.0127`,
    and the down-and-out put (`K = 90, H = 95`) from `9.3890`, above the
    vanilla put `2.2845`, to `0.0000`. Knock-outs now stay within
    `[0, vanilla]`.
  - The rebate legs were swapped: a knock-in received the rebate paid at
    the hit (`F`) and a knock-out the one paid at expiry (`E`). With a rebate
    of 3 the down-and-out call at `K = 90, 100, 110, H = 95` goes from
    `7.4188, 5.1867, 3.2701` to `9.0246, 6.7924, 4.8759` and the down-and-in
    call from `9.3684, 5.6167, 3.6633` to `7.7627, 4.0109, 2.0576`. Every
    rebate-3 entry of Haug's Table 4-13 at `σ = 25 %` and `30 %` is now
    reproduced to four decimals.
  - A short barrier is the negated long price; the pricer used to ignore
    `option.side` (`S = 100, K = 105, H = 90, T = 0.5, σ = 25 %, r = 5 %,
    q = 1 %`: the short down-and-out call goes from `+5.1218` to
    `-5.1218`).
  - A spot already at or beyond the barrier prices the knock-in as the
    vanilla and the knock-out as its rebate paid now, instead of evaluating
    the formulas outside their domain.
  - Down-barrier calls without a rebate are unchanged.

- **A negative risk-free rate is a valid input** (#709). `Options::validate`
  rejected `risk_free_rate < 0`, so an option, a `Position` holding it and
  every strategy validated through its positions refused rates that EUR, CHF
  and JPY markets quoted for years, although the pricers handle them. The
  check is removed; the empty-symbol, zero-quantity, zero-strike,
  zero-underlying and negative-volatility checks stay. The rest of the
  workspace was swept for the same rule and has none: every rate is a
  `Decimal` (`Options::risk_free_rate`, `BinomialPricingParams::int_rate`,
  `RNDParameters::risk_free_rate`, the chain, series and option-data
  `risk_free_rate`, the probability kernels' `risk_free_rate`), and no
  constructor, builder, chain or simulation parameter compares one with
  zero; `Position::validate`, the strategies' `validate` and
  `StrategyRequest::get_strategy` reach the rate only through
  `Options::validate`. New tests: an option at r = -1 % (and 0, -0.75 %,
  -25 %) validates while the other checks still reject; a `Position` at
  -1 % validates; a `BullCallSpread` at -1 % builds and validates through
  `new`, `get_strategy` and `StrategyRequest`, with the same expiry profile
  as at 5 %; Black-Scholes at r = -0.5 %, -1 %, -2 % and -5 % matches the
  closed form and put-call parity. One limit remains by type, not by a
  check: `garman_kohlhagen` maps the foreign rate onto
  `Options::dividend_yield`, a `Positive`, so a negative foreign rate still
  cannot be expressed (documented there since it was added).
  - A probe of the vanilla pricers at r = -1 % and -5 % found one that
    failed: `barone_adesi_whaley` for a put returned
    `PricingError::Decimal(Overflow)` from the critical-price power
    (`(S / S*)^q1` with `S / S* = 10000`). At `r < 0` with `q >= 0` early
    exercise of a put is never optimal, since holding is worth at least
    `K e^(-rT) - S e^(-qT) >= K - S`, so it now returns the European put,
    the mirror of the existing `q = 0` call rule (at `S = K = 100`,
    `sigma = 0.2`, `T = 0.5`: 5.905478 at -1 %, 7.063118 at -5 %). `r = 0`
    keeps the quadratic approximation. A new test holds it to Black-Scholes
    within `1e-9` for three negative rates and four `(S, q)` points, and to
    the 500-step binomial American put within `0.01`, which at `q = 0`
    equals the binomial European put exactly.

- **The American pricers honour early exercise and `Side`** (#648).
  - `barone_adesi_whaley` at `σ = 0` returned the European value
    `max(K e^(-rT) - S e^(-qT), 0)`, below the intrinsic value of an
    in-the-money put: `S = 80, K = 100, r = 10 %, T = 1` priced `10.4837`
    instead of `20`. It now returns the exact deterministic optimum, the best
    of exercising now, at expiry, or at the interior stationary point
    `τ* = ln(rK / (qS)) / (r - q)` when that lies inside `(0, T)` (a put with
    `r < q`, a call with `r > q`; `S = K = 100, T = 50, r = 1 %, q = 5 %`
    gives `53.4992` where the expiry-only value was `52.4446`). Every
    `barone_adesi_whaley` result is also floored at the intrinsic value
    (Hull: an American option is worth at least immediate exercise), which
    changes the `q = 0` call at a negative rate: `S = 150, K = 100,
    σ = 10 %, r = -2 %, T = 1` goes from the European `47.9800` to `50`.
  - `price_binomial` and `generate_binomial_tree` applied the early-exercise
    `max(continuation, intrinsic)` to side-signed values, so a short
    American picked the writer's smaller liability: `S = 100, K = 105,
    σ = 25 %, r = 5 %, T = 0.5`, 200 steps, priced the short call at `0`
    instead of `-5.9823`. The lattice is now valued from the holder's side
    and the side applied once, so a short price (and every node of a short
    tree) is the negated long one. European prices are unchanged.
  - `OptionPricing::calculate_price_binomial_tree` negated the root of the
    already side-signed tree a second time, so every short option priced at
    the long price. It now returns the root as is; the unit test that
    asserted a positive short price asserts the negated long price instead.

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

- **Telegraph walks switch regime with probability `1 - e^(-λ·dt)` per
  step** (#683). The switch trial turned a standard normal draw into
  `(|z| + 1) / 2`, which is never below one half and is unbounded above, so
  whenever `1 - e^(-λ·dt) <= 0.5` (every usual rate on daily or finer steps)
  the regime never switched, and above that the frequency was wrong. The
  trial now draws a genuine `U(0,1)` from the same per-path generator with
  `decimal_uniform_sample_with` (#684), so `P(switch) = 1 - e^(-λ·dt)`; a
  zero rate never switches and a rate whose `e^(-λ·dt)` flushes to zero
  switches on every step. This changes every telegraph path, seeded or not;
  a seeded path stays reproducible bit for bit for its seed. The draw order
  is unchanged: the sign of one normal picks the initial regime, then each
  step draws the switch trial and the price normal. The telegraph case of
  `deterministic_simulation_test.rs` is re-baselined (its first four steps
  keep their values) and keeps `λ = 250`, now so that its seven-step path
  switches in both directions rather than to switch at all. A seeded test
  runs 200,000 daily steps at the realistic rates `λ_down = 12` and
  `λ_up = 4` a year, tallies the trials and switches from each regime off
  the volatility path, and checks each count against its binomial mean
  within five standard errors; the zero-rate and flushed-rate limits are
  tested too.

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

- **The price-probability kernel uses the lognormal threshold, so the
  risk-neutral case is `N(-d2)`** (#664). `calculate_single_point_probability`
  in `crates/optionstratlib-analytics/src/analytics/probability.rs` computed
  `z = (ln(K / S) - mu T) / (sigma sqrt(T))`, which leaves out the Ito
  convexity term: the log price drifts at `mu - sigma^2 / 2`, not `mu`. It now
  computes `z = (ln(K / S) - (mu - sigma^2 / 2) T) / (sigma sqrt(T))`, so with
  no trend `P(S_T < K) = N(-d2)` (Hull, ch. 15). The drift input is the
  arithmetic drift: `risk_free_rate`, plus `drift_rate * confidence` when a
  `PriceTrend` is given, corrected by `-sigma^2 / 2` like the rate; the
  `PriceTrend` and kernel docs state it, and the strategies probability docs
  give the model as `ln(S_T / S_0) ~ N((mu - sigma^2 / 2) T, sigma^2 T)`.
  - Which results change: every probability the kernel returns for
    `sigma > 0`, and everything built on it: `calculate_price_probability`,
    `ProfitLossRange::calculate_probability`, and the strategy
    `probability_of_profit`, `probability_of_loss`,
    `calculate_extreme_probabilities`, `expected_value` and
    `analyze_probabilities`. The probability below a target rises, by
    `N(-d2) - N(-d2 - sigma sqrt(T) / 2)`: S = 100, K = 105, sigma = 0.2,
    T = 30/365, r = 0.05 moved from 0.78208 to 0.79043.
  - New regressions against closed-form `N(-d2)` for six `(S, K, sigma, T,
    r)` including Hull's worked example (S = 42, K = 40, r = 10%,
    sigma = 20%, T = 0.5, `N(-d2) = 0.26505`), the tail partition
    `P(S_T < K) + P(S_T >= K) = 1`, the range kernel as the difference of two
    `N(-d2)` values, and a trend of `mu` matching a rate of `mu`.
  - Re-baselined pins, each with a comment naming #664: in
    `probability.rs`, `tests_price_trend` (trend -0.37 / 0.65: below 110
    0.895412344777716 -> 0.903902952229174, above 0.104587655222284 ->
    0.096097047770826, range triple 0.2950713010903496 / 0.6003410436873664
    -> 0.3119431542059403 / 0.5919597980232337, no trend 0.82862940208696 ->
    0.840628037745865; trend 2.999789999999902 / 0.1234567890123456789012345678:
    below 0.682928086717717 -> 0.699924018145762, above 0.317071913282283 ->
    0.300075981854238, range 0.0936575883560122 / 0.5892704983617048 ->
    0.10199178784555 / 0.597932230300212) and `test_target_equals_current`
    (at the money, zero drift, sigma 0.8, T = 1: 0.5 / 0.5 -> 0.655421741610324
    / 0.344578258389676, `N(0.4)`); in the strategies
    `probabilities/core.rs`, the `BullCallSpread` expected value with a
    15-place drift (0.002802439458791824 -> 0.003104587227155649). No
    tolerance changed.

- **The implied-volatility solvers report targets that have no implied
  volatility** (#652). A Black-Scholes price is strictly increasing in `σ`
  inside the no-arbitrage band `max(S e^(-qT) - K e^(-rT), 0)` to
  `S e^(-qT)` for a call, `max(K e^(-rT) - S e^(-qT), 0)` to `K e^(-rT)` for
  a put (Hull, bounds on option prices). A target outside that band, widened
  by `IV_TOLERANCE = 1e-5`, used to return an edge of the solver's search
  instead of an error; both solvers now check the band for European options.
  - `OptionPricing::calculate_implied_volatility` (bisection on
    `σ ∈ [0, 5]`) returns `VolatilityError::InvalidPrice` for such a target,
    where it returned `0.0000763` (target `15` for a call with intrinsic
    `20`) or `4.99992` (target `130` above a spot of `120`). Its
    `NoConvergence` variant is now reachable: a target inside the band whose
    implied volatility exceeds `500 %` returns it instead of `4.9999`
    (`S = K = 100, T = 1, r = q = 0`, target `99.5` above the `σ = 5` price
    `98.758`).
  - `volatility::implied_volatility` and `calculate_iv` (grid search over
    `σ = i / 1000`) return `VolatilityError::IvNotFound` for such a target,
    where they returned `0.046` and `0.999`, and now treat the top grid
    point like the bottom one: a best match on either edge means the root
    lies at or beyond it, so a target whose implied volatility exceeds the
    grid (`σ = 1.5`) returns `IvNotFound` instead of `0.999`.
  - Migration: a caller that read an edge value as "no solution" now
    receives the error; targets inside the band and the bracket invert to
    the same volatilities as before.

- **`decimal_to_f64` returns the nearest `f64`, and is the only `Decimal` to
  `f64` conversion in core** (#670). It went through `Decimal::to_f64`, which
  divides the mantissa by a power of ten in floating point and can land a few
  ULPs from the nearest `f64` once the value has 15 or more decimal places:
  `2.999789999999902` came back as `2.999789999999903`, three ULPs above
  `2.999789999999902_f64`. It now divides the mantissa by `5^scale` in `u128`
  integers, rounds the quotient to 53 bits once (ties to even) from its
  dropped bits and the exact remainder, and scales by an exact power of two,
  so it returns the `f64` the literal with the same digits parses to. Zero of
  either sign converts to `+0.0`. The error is built by a `#[cold]`
  constructor through `ok_or_else`, so the success path no longer formats an
  error string and does not allocate.
  - `decimal_to_f64_correctly_rounded`, added earlier in this cycle (#656)
    and never released, is folded into it and removed. Migration: call
    `decimal_to_f64`, which now has the same contract. The `PriceTrend`
    conversion in the analytics kernels and the trend adjustment of
    `ProbabilityAnalysis::expected_value` call it, with unchanged results.
  - Which results change: those of `decimal_to_f64` and of the `d2f!` /
    `d2fu!` macros, by a few ULPs on values with 15 or more decimal places.
    In the workspace the only other caller is `generate_binomial_tree`, which
    converts each American or Bermuda node value before comparing it with the
    intrinsic value of early exercise. A differential run over 9,720 trees (both
    exercise styles, calls and puts, long and short, 3 to 64 steps, 14.8
    million node values) changed 551,266 node values in 3,329 trees, by at
    most `2e-12` absolute and `5.6e-14` relative. `price_binomial` does not
    go through it and is unchanged. No pinned test value moved, and no
    tolerance changed.
  - New tests: a seeded sweep over every scale from 0 to 28 with mantissas
    up to `2^53` and over the full 96 bits, exact ties at `2^53 + 1` and
    `2^53 + 3` (as integers and scaled by `5^s / 10^s`), the values around
    `2^53`, zero of either sign and scale, and the #670 case
    (`2.999789999999902` against its nearest `f64` and against the former
    `Decimal::to_f64` result).

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

- **`Simulator::new` builds its walks in parallel** (#860). From
  `PARALLEL_MIN_WALKS` (3) walks up, the walks are built on the rayon pool,
  in rounds of 1024, and collected in index order; below it they are built
  serially, which the crossover measurement found faster there (30-step
  walks: one or two walks tie or lose on the pool, three already win). The
  seed of every walk is still drawn in order from the master seed before
  its walk is built, so a seeded simulator is bit-identical to the serial
  build: `tests/parallel_simulator_test.rs` compares every step of every
  path with a walk-by-walk serial reference for five seeds, on both sides of
  the threshold, across rounds and for geometric Brownian and Heston walks.
  Criterion `simulation/simulator/new/*` on an Apple M5 Max (18 cores,
  macOS 27.0.1, rustc 1.99.0), median before and after: 100 paths x 30
  steps 4.25 ms to 0.53 ms, 1000 x 30 37.5 ms to 2.95 ms, 100 x 252
  37.6 ms to 2.75 ms. The host was shared (load average about 30 during
  the after run), so the medians carry its load.

- **Errors and defaults are built lazily** (#857). `f64_to_decimal`,
  `random_decimal`, `IronButterfly`'s position lookup, the chain built from
  option data and `get_today_or_tomorrow_formatted` built their error or
  default eagerly with `ok_or(..)` / `unwrap_or(..)`, so the success path
  paid for it too: `f64_to_decimal` formatted the value and allocated three
  `String`s on every successful conversion. They use `ok_or_else` /
  `unwrap_or_else` now, with the same values, and every library crate denies
  `clippy::or_fun_call` so the pattern does not come back. Criterion on the
  bench host of `docs/release/0.22/benchmarks.md`, before and after:
  `f64_to_decimal` 149 ns to 80 ns (-46%), and the normal CDF `big_n`, which
  returns through it, 200 ns to 128 ns (-36%).

- **Curve and surface interpolation no longer scan every point per read**
  (#858, M1). `Curve` brackets `x` with two `BTreeSet::range` lookups
  instead of collecting the points into a `Vec` and scanning it, and reads
  the bilinear cell and the cubic window off the tree instead of by index;
  `find_bracket_points` is overridden for `Curve` with the same indices.
  `Surface` finds the 3, 4 or 9 nearest points for the linear, bilinear and
  cubic interpolators by walking outwards from `x` in the tree and stopping
  a side once the abscissa offset alone exceeds the k-th best distance,
  instead of keying and sorting every point. The exact-match lookups
  (`contains_point`, `get_point`, `get_values`, the interpolators'
  exact-sample branch and the merge resampling) are range reads on both
  types. Results are bit-identical: the same bracket, window and
  neighbours are selected, ties and errors included (a surface whose
  distances could overflow takes the old sort, so the error names the same
  point); unit tests compare each against the scan it replaced, and a
  differential run of 63,823 interpolation, lookup, derivative and merge
  results over curves with stacked abscissas and tie-heavy surface grids
  matched the previous commit exactly.
  Criterion medians, before and after, on one Apple M5 Max under other
  load (`cargo bench -p optionstratlib-math --bench math`): curve
  `linear/2048` 39.4 µs to 0.50 µs, `cubic/2048` 53.5 µs to 1.07 µs,
  `linear_sweep_100/128` 404 µs to 51 µs, `curve_derivative_at/512` 8.0 µs
  to 0.22 µs, `curve_merge_with_add/512` 2.26 ms to 0.25 ms; surface
  `linear/32x32` 151 µs to 21 µs, `cubic/32x32` 143 µs to 38 µs,
  `surface_merge_with_add/16x16` 19.2 ms to 9.4 ms. Spline interpolation
  still rebuilds its system per read (M2).

- **The prelude's `dec!` documents that it needs `rust_decimal`** (#777).
  `optionstratlib::prelude` re-exports `rust_decimal_macros::dec`, which
  expands to `::rust_decimal` paths, so a consumer that writes `dec!` fails
  to compile until `rust_decimal` is one of its own dependencies; every
  consumer fixture already listed it, which hid the gap. By owner decision
  the macro stays, with its compile-time validation. The prelude module docs
  and the `dec` re-export, the migration guide's import section, and the
  facade docs (and so `README.md`) say so, and the facade manifests there
  list `rust_decimal`. `make check-release-notes` pins it: a release-notes
  manifest whose program writes `dec!` without `rust_decimal` is refused,
  and the new `--self-test` builds a facade-only manifest
  (`default-features = false`) that writes `dec!`, asserting that it fails
  on the missing `rust_decimal` and compiles once the dependency is added.
  No code changes.

- **The 0.22 build and dependency baseline comparison** (#563, M8-07).
  `docs/release/0.22/baseline-comparison.md` re-runs the Milestone 0
  commands (`doc/BASELINE.md`) for 0.21.3 (`80efdc02`) and 0.22
  (`d274e8f7`) on one host (x86_64 Linux, 16 threads) and one toolchain
  (Rust 1.99.0), with same-day fresh lockfiles, `cargo clean` before every
  clean sample, three samples per cell and six for the default and
  all-features facade. Focused profiles (core, pricing, market, simulation,
  analytics, a headless facade) are compared with the 0.21.3 default, the
  smallest 0.21.3 surface: they resolve 41 to 69 package pairs against 131
  and check from clean in 3.8 to 11.3 s against 17.1 s. The facade default
  resolves 124 against 131 and checks in 15.7 s; all features 264 against
  284, checking in 34.5 s against 41.7 s. Samples that overlapped another
  agent's benchmark run on the host were discarded and re-taken. Each focused graph is verified to
  exclude the capabilities it does not use. The document records medians,
  ranges, raw samples, artifact sizes, the `plotly_static` build-script stub
  used for both revisions, and the confounders (14 % more production code,
  a larger default feature set, host and toolchain differing from M0); the
  raw logs, scripts and package lists are under
  `docs/release/0.22/baseline-comparison/`. The release notes' "Measured
  baseline" section carries the headline values without attributing them to
  the split alone.

- **Draft 0.22.0 release notes** (#562). `docs/release/0.22/RELEASE-NOTES.md`
  lists the ten packages at 0.22.0 in dependency order with their crates.io
  and docs.rs links, the layer graph, the API policy, the facade defaults
  and every feature, and the source, feature, serialization, toolchain and
  operational changes of the release, each with its issue, plus the fixes
  that change numerical results. Its minimal manifests are checked twice:
  `make check-release-notes` (`scripts/check_release_notes.py`) builds and
  runs each one as a standalone crate with exactly the `[dependencies]`
  shown, patched to the checkout, and fails on a manifest without a
  program; the facade runs the programs as doctests
  (`ReleaseNotesDoctests`, under `cfg(doctest)` with `visualization`,
  absent from every build and the public API) and ships the file in its
  package. The measured-baseline section is a marked placeholder until
  #563 validates the measurements. Nothing is tagged, released or
  published.

- **The workspace examples declare only the capabilities they use** (#774).
  Every `examples/examples_*` package depended on the facade with its default
  features, which enable every capability, and most added `plotly` and
  `static_export`. Each now sets `default-features = false` and lists what
  its binaries need: `pricing` for `examples_pricing` and `examples_exotics`;
  `visualization` for `examples_volatility` and `examples_strategies_delta`
  (their `error::Error` is the unified error, which needs it);
  `static_export` for `examples_curves`, `examples_metrics`,
  `examples_strategies`, `examples_strategies_best`, `examples_surfaces` and
  `examples_visualization` (their `PlotBuilder::save`, `write_png` and
  `write_svg` calls); `static_export` and `synthetic` for
  `examples_simulation`. `examples_chain` needs the async wrappers or image
  export in seven of its ten binaries, so those declare `required-features`
  (`async` or `static_export`, both package features) and `default` enables
  `static_export`: `cargo run -p examples_chain --bin <name>` works as before
  and `--no-default-features` builds the three that need neither.
  `examples_pricing` takes the facade from the workspace table like the
  others. The leftover `examples/Local` package, referenced only by a
  `.gitignore` line, is removed. `make check-graph` now fails when a workspace
  package that is not a component depends on the facade with its defaults, with
  no explicit features, or by path (`example_facade_violations`, with
  self-tests), so this cannot regress.

- **Root and crate-level documentation for the 0.22 workspace** (#554).
  The facade docs (and so the generated `README.md`) explain when to depend
  on the facade and when on component crates, with a crate-selection table
  giving each of the nine component crates its responsibility, its own
  features (from the manifests), its facade feature and its facade paths,
  and link `docs/ownership.md` and `examples/direct`. Every component's
  crate root and `README.md` now share a "Place in the workspace" section
  (what the crate depends on, which crates it must not depend on, enforced
  by `make check-graph`, and how the facade re-exports it) and, except
  visualization (whose crate docs already chart a strategy), a minimal
  example that compiles as a doctest against the crate's default features.
  Each crate root also includes its `README.md` under `cfg(doctest)`
  (`ReadmeDoctests`, absent from every build and from the public API), so
  `cargo test --doc` compiles the README's Rust blocks too; the facade does
  the same for the generated `README.md` when `visualization` is on, since
  `cargo-readme` drops the hidden feature gates of those examples.
  Links that leave the crate are absolute, so they work on crates.io and
  docs.rs (the README's license badge was relative). The prerequisites now
  state the minimum Rust version (1.89 since #559). Each README and crate doc carries one
  `<!-- #553 ... -->` marker where #553 links the migration guide. No
  ignore rule changes: planning material stays under the locally excluded
  `doc/`, user-facing documents live in the tracked `docs/` (#556).

- **The 0.22 architecture and adoption guide** (#553, closes the #554
  markers). `docs/migration-0.22.md` covers the crate layers and their
  dependency direction, facade versus direct component use with the feature
  of every crate, minimal consumers (`examples/direct`, the consumer
  fixtures), the import rules (canonical paths, the explicit prelude, the
  flat error paths) and, for each API redesign of 0.22, the workflow that
  replaces the 0.21 one: seeded stochastic pricing and walks, binding
  strategy validation, the textbook put spreads and 1/2/1 butterflies,
  `CallButterfly`'s removal, the contract multiplier and covered-strategy
  sizing, the analytics inputs and the `-sigma^2 / 2` probability term,
  market `io`/`synthetic`, payoffs, negative and foreign rates, backtest
  reports and charts. A table lists the serialized-data changes (a
  `"CallButterfly"` strategy type no longer deserializes; put-spread and
  butterfly requests in the 0.21 shape are rejected; covered-strategy
  quantities mean shares; `contract_size` and `foreign_rate` keys;
  `OptionSeries` keys) and a list of the fixes that
  change results. The facade compiles and runs the guide's Rust examples
  as doctests (`MigrationGuideDoctests`, under `cfg(doctest)` with
  `visualization`, absent from every build and the public API), and ships
  `docs/*.md` in its package. Every crate root, crate `README.md`, the
  facade docs and `README.tpl` link the guide where #554 left a marker.

- **`optionstratlib-core` builds without `utoipa` unless `schema` is on**
  (#628). `expiration_date` 0.4.1 forwards `positive/utoipa` only from its own
  `utoipa` feature, so the workspace now requires `expiration_date` 0.4.1 and
  `cargo tree -p optionstratlib-core -e normal` no longer lists `utoipa`
  (`--features schema` still does). `utoipa` joins the forbidden packages of
  every component crate in `scripts/check_module_boundaries.py` (allowed only
  through each crate's `schema` feature set) and the absent list of the
  component consumer fixtures.

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
  0.21. The sibling crates were already on utoipa 6; the minimum Rust
  version is 1.89, declared as `rust-version` since #559. No
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

- **Every component crate has Criterion benches** (#789). Fourteen bench
  targets now cover the public hot paths of the workspace: `core` (checked
  `Decimal` helpers, `Positive`, payoffs, construction), `math` (curve and
  surface construction, every interpolation method by size, metrics,
  transformations), `pricing` (`pricing`: Black-Scholes, Black-76,
  Garman-Kohlhagen, Barone-Adesi-Whaley, binomial by step count,
  Monte-Carlo and telegraph by path and step count, every exotic; `greeks`:
  every Greek, the snapshot, the numerical, Black-76 and Garman-Kohlhagen
  Greeks, the chain sweep; `volatility`: the IV solvers and the estimators),
  `market` (`chains`, plus `chains_io` behind `io` and `synthetic` behind
  `synthetic`), `analytics`, `strategies` (construction, break-evens, P&L,
  probability, delta neutrality, the optimisers on the SP500 fixture and on
  growing synthetic chains), `simulation` (every walk model, the simulator),
  `backtest` and `visualization` (chart data and terminal tables, no
  rendering). Each bench checks that its fixture succeeds before timing it,
  so an error branch is never measured as the path. `criterion` moves to
  `[workspace.dependencies]`; no dependency is added. `make bench-build`
  compiles every bench target and runs in the Lint workflow, and
  `make bench-workspace` runs them all. The first full run, on a dedicated
  Linux host, is recorded in `docs/release/0.22/benchmarks.md`: machine,
  toolchain, command, a summary per crate, the bottlenecks it found (filed
  as follow-up issues per crate) and the full Criterion output. The facade's
  `curve_merge_multiply` and `surface_merge_multiply` now assert that the
  merge succeeds before timing it; they timed `is_ok()`, so a failing merge
  was measured as its error branch.

- **The 0.22 release gates are one command, and their evidence is tracked**
  (#557). `make release-gates` (`scripts/release_gates.py`) runs every gate of
  ADR-0004 section 6 and the per-crate and facade matrices: formatting,
  Clippy, the tests of each crate with default, no and all features and of the
  facade capability matrix, rustdoc with warnings denied with and without
  features, the release build, `scan-banned`, the public API snapshots, the
  architecture and feature-tree checks, the consumer fixtures and the 0.22
  consumers against the workspace and the package archives, the direct
  examples, `check-components` and the independent `plotly` and
  `static_export` surfaces. `docs/release/0.22/gates.md` records the commit,
  the toolchains, every command, its result, time and warnings, and
  `docs/release/0.22/api-classification.md` sorts the 1,163 module-level items
  `cargo-semver-checks` reports against 0.21.3 (`scripts/classify_api_changes.py`)
  into 734 that still resolve at their path, 404 reachable at a canonical
  facade path and 25 removed, each removal linked to its issue; it also
  records the 3 variants the unified `Error` gained. The run found
  and fixed one failing check: the API report's default surface was missing
  `schema` since it became a default feature (#549), so `make check-api-report`
  and the API CHANGES workflow's parser self-test failed.

- **Executable 0.22 consumers and `make test-022-consumers`** (#552). Every
  directory under `fixtures/consumers/` is a downstream crate with the
  manifest a 0.22 user writes, built outside the workspace with a target dir
  of its own. New and extended consumers:
  - `facade-plotly/tests/strategy_workflow.rs`: the complete
    strategy-construction-to-chart workflow. A `StrategyRequest` is analysed
    through `get_strategy()` as a `Box<dyn Strategable>` (break-even points,
    profit bounds, payoff, fees across a `Vec` of trait objects) and charted
    by dispatching on its `strategy_type` to a function generic over
    `S: StrategyConstructor + Graph`, since `Graph` is no longer a supertrait
    of the strategy contract (#658). The tests prove the chart is the
    analysis: every point of every payoff segment, the current-price marker
    and the break-even labels equal `calculate_profit_at` and
    `get_break_even_points` on the trait object, the Plotly figure and the
    HTML page carry the same traces, and a request without a builder or with
    the wrong legs fails on both sides with `StrategyError`. The dispatch
    names every `StrategyType`, the seven without a builder included, so a
    strategy that gains a builder does not compile there until it is
    chartable.
  - `headless-full/tests/workflow.rs`: on the facade defaults, Greeks
    through a generic bound and through `dyn Greeks` over an option and a
    strategy, the `synthetic` chain generator through
    `chains::generator_optionchain` with each generated chain rendered as a
    terminal table, and a backtest read as data and rendered with
    `visualization::terminal::SimulationReport`.
  - `facade-async`: the facade with `async` alone, which resolves `tokio`,
    `csv` and `zip` and no simulation, strategy, backtest or chart crate. A
    chain round-trips through JSON and CSV with the `*_async` wrappers, two
    loads run concurrently, the async OHLCV reader agrees with the
    synchronous one, and missing files arrive as `ChainError::FileError` and
    `OhlcvError::IoError`.
  - `market-io`: `optionstratlib-market` with `io` alone (ADR-0002
    `osl-fixture-market-io`), with JSON and CSV round-trips, OHLCV reads with
    and without date bounds and the documented error paths. Both I/O
    fixtures read a nine-candle `tests/data/ohlcv-sample.zip` cut from
    `examples/Data/cl-1m-sample.zip`, so a copy built outside the repository
    reads the same data.

  `make test-022-consumers` asserts every fixture's graph
  (`check-fixtures`), then lints (`-D warnings`) and tests each fixture
  under its defaults and, when it declares features of its own, with none
  and with all (`scripts/test_consumers.py`, with a self-test), and finally
  tests a copy of every fixture outside the repository against the packaged
  crates (`make check-022-consumers-packaged`). That reuses the
  `[patch.crates-io]` mechanism of the direct-component examples (#555):
  `scripts/check_packaged_examples.sh` now also packages the facade, copies a
  scenario's `tests/`, runs a scenario only when it is a binary, and reads
  its scenarios from `OSL_PACKAGED_SOURCE` (default `examples/direct`, whose
  build dir is now `target/packaged-direct`). A new fixture directory is
  picked up without a Makefile change. The per-fixture
  targets `check-consumer-core-pricing`, `check-consumer-core-pricing-minimal`,
  `test-consumer-core-pricing`, `check-consumer-analytics-only`,
  `check-consumer-analytics-only-minimal`, `test-consumer-analytics-only`,
  `test-consumer-simulation-only`, `test-consumer-full-backtest`,
  `check-consumer-market`, `test-consumer-market`, `check-consumer-facade`
  and `test-consumer-facade` are retired in its favour, and the
  `tree-consumer-*` targets become `make tree-consumer FIXTURE=<scenario>`;
  the Components workflow runs `OSL_REUSE_PACKAGES=1 make test-022-consumers`
  after `check-components`.

- **A published, checked ownership map of the public API** (#556).
  `docs/ownership.md` records, for every concept group (domain model,
  utilities, constants, curves, surfaces, geometry, pricing models, Greeks,
  volatility, simulation, option chains and series, analytics, P&L, risk,
  chain metrics, strategies, backtesting, visualization), every
  feature-gated capability inside a module (`io`, `async`, `synthetic`,
  `plotly`, `static_export`, `schema`), every concrete error (21 component
  errors and the facade aggregate, each with one owner and its kind module),
  every facade root item and macro, and the four externally owned
  foundational crates, the defining package and module, the direct import,
  the facade path, the feature and the 0.22 status, with the canonical-path
  policy of #550/#752. `scripts/check_ownership_map.py`, run by
  `make check-graph` (and so by CI on every pull request), parses the map
  and fails when a path is missing from the public-API snapshots, when a
  facade path has no compiled identity check in
  `tests/unit/canonical_paths_test.rs`, when a feature differs from the
  `#[cfg]` gate in `src/lib.rs` / `src/error/mod.rs` or from the forwards
  in the manifests, when a facade module, root item, concrete error or
  foundational crate has no row, when a public `...Error` enum lives
  outside its crate's `error` module, when a feature-gated item is not
  declared under its component feature (`#[cfg(feature = ..)]` on the item,
  its `pub use` or its `pub mod`), when a root macro row names a different
  macro or src/lib.rs re-exports it from another crate, when a public error
  module is no row's kind module, or when a crate other than core, the
  facade included, depends on a foundational crate. The facade crate docs,
  the README and its contribution steps link to the map.

- **Direct-component examples** (#555). `examples/direct/<scenario>/` holds
  eight runnable programs that depend on the component crates directly and
  on no facade feature: `math` (a volatility-smile `Curve` interpolated
  linearly, cubically and by spline), `pricing` (core, pricing), `market`,
  `analytics`, `strategies`, `simulation`, `backtest` (simulation,
  strategies, backtest) and `visualization` (strategies, visualization with
  `plotly`). Each is a workspace member named `osl-example-direct-<scenario>`
  (ADR-0004 section 8) with a self-contained `Cargo.toml` (explicit versions,
  nothing inherited from the workspace, the OptionStratLib crates by path
  plus version, the facade equivalent in a comment), a `src/main.rs` that
  installs `tracing-subscriber` and logs its workflow, tests for its key
  figures, and an `expect.toml` of the packages its resolved graph must and
  must not contain. `make test-direct-component-examples` lints, tests and
  runs all of them, `make tree-example-direct-<scenario>` prints and asserts
  one graph, `make check-fixtures` (now also discovering `examples/direct`)
  asserts them with the consumer fixtures, and `make
  check-direct-examples-packaged` copies each example out of the repository
  and builds, tests and runs the copy against the packaged component crates
  through `[patch.crates-io]`, which is what a consumer gets once they are
  published (`scripts/check_packaged_examples.sh`); the Components workflow
  runs them. The README in `examples/direct` maps each scenario to its facade
  equivalent and to the fixture that proves the same graph, and states the
  trade-off. The component READMEs and crate docs, and the facade docs, link
  to them. Moving from a facade import to a direct one is
  `optionstratlib::model::X` to `optionstratlib_core::model::X` (and likewise
  per layer): they are the same types.

- **The facade's feature model is reconciled and pinned** (#549).
  The facade declared twelve dependencies it never imports (`approx`,
  `statrs`, `rand`, `rand_distr`, `num-traits`, `serde`, `serde_json`,
  `rayon`, `utoipa`, `expiration_date`, `financial_types`, `option_type`) as
  normal dependencies, so `--no-default-features` resolved `statrs`, `rayon`
  and `utoipa` before any capability was on; they are removed (the few its
  tests and doc examples use are dev-dependencies). `async` no longer enables
  the unused optional `tokio`, `async-trait`, `reqwest` and `futures` of the
  facade and forwards only `optionstratlib-market/async` as ADR-0002 section
  2 says, so `async` alone resolves `tokio` and no `reqwest`, `futures` or
  `async-trait` (`static_export` still resolves `reqwest` and `async-trait`
  through `plotly_static`).
  - **`schema` and `parallel`**, the two features ADR-0002 section 2 lists and
    main lacked. `schema` (now in `default`, so the default surface is
    unchanged) forwards `optionstratlib-core/schema` and the weak
    `optionstratlib-{math,pricing,simulation,market,analytics,strategies,backtest}?/schema`
    of the components another feature enabled; the component dependencies
    are no longer declared with `features = ["schema"]`, so a build with
    `default-features = false` derives no `utoipa::ToSchema` and resolves no
    `utoipa` (add `schema`, or keep the default, to get the derives back).
    `parallel` is reserved and empty.
  - **`expiration_date` 0.4.1** is the minimum (#628): it brings `utoipa`
    only behind its own `utoipa` feature, so no build without `schema`
    resolves the package. Because the facade no longer enables `utoipa`
    itself, the default graph also loses the `utoipa/axum_extras` feature the
    facade used to switch on through unification. A consumer that relied on
    it (the `axum` integration of `utoipa`) must enable `axum_extras` on its
    own `utoipa` dependency.
  - **Checks.** `make check-graph` pins every row of the facade's feature
    table and the default exactly, and forbids `utoipa` in each component's
    default tree. `make check-feature-trees` pins 15 surfaces (the facade with
    no capability, `schema`, `pricing`, `simulation`, `market`, `io`, `async`,
    `synthetic`, `analytics`, `strategies`, `backtest`, `visualization`,
    `plotly`, `static_export` and the default, i.e. the headless full domain),
    asserts for each the exact set of `optionstratlib-*` components and the
    backends it may resolve, that `utoipa` resolves only where `schema` is on,
    and that a component has an edge to it only there. The new
    `facade-schema-off` fixture shows by `compile_fail` doctests that one
    type of each component implements no `ToSchema` without the feature and
    that its normal graph resolves no `utoipa`; `headless-full` shows the
    same types implement it by default and lists `utoipa` as present. The
    other component and facade fixtures built without `schema` list `utoipa`
    as absent. The `FACADE_FEATURE_SETS` loop of `make lint` and `build.yml`
    also cover `schema`, `io`, `async`, `synthetic`, `math` and the
    all-features build, and the `lib.rs` docs carry the feature routing
    table, the 0.22 default and the combinations that do not exist.

- **The facade's visualization routing is asserted, not just wired** (#548).
  `make check-graph` fails unless the facade's `visualization`, `plotly` and
  `static_export` features are exactly what ADR-0002 section 2 documents
  (`visualization` = the crate plus `backtest`; `plotly` = `visualization`
  plus the crate's `plotly`; `static_export` = `plotly`, `async` and the
  crate's `static_export`), or when any other feature (a lower capability,
  `synthetic`, `io`, `async`) enables the visualization crate or one of the
  three; the self-tests cover each case, including `backtest` implying
  `visualization`, which is the reverse of the documented direction.
  `make check-feature-trees` pins the no-capability facade (`none`) and
  asserts it resolves no visualization, Plotly or export package. The four
  facade consumer fixtures (`facade-visualization`, `headless-full`,
  `facade-plotly`, `facade-static-export`) now prove by `TypeId` and by
  trait bounds in both directions that the facade paths, the prelude names
  and the unified `Error::Graph` are the items `optionstratlib-visualization`
  defines, and the Plotly fixtures do the same for `make_scatter` and
  `make_surface`. The strategy-builder-to-chart workflow is the doctest in
  the crate docs of `optionstratlib-visualization` (#543).

- **Each visualization compilation surface is verified on its own** (#547).
  `make check-visualization-surface SURFACE=neutral|plotly|static_export`
  checks, lints, tests (doctests included) and documents with warnings denied
  both `optionstratlib-visualization` and the facade feature that routes to it
  (`--no-default-features`, `plotly`, `static_export,plotly`), runs the
  consumer fixtures of the surface and the dependency assertions
  (`check-graph`, `check-feature-trees`, the fixtures' `present`/`absent`
  lists); `make check-visualization` runs all three. The new
  `visualization.yml` workflow runs one job per surface on every push and
  pull request, so the backend-neutral `Graph` is compiled and tested apart
  from the all-features run and the headless surfaces provably exclude Plotly
  and the export stack. The scheduled `static_export.yml` (#698) keeps running
  the `#[ignore]`d PNG/SVG tests; the Makefile now records its platform
  requirements and retention policy.

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

- **Depending on a component instead of the facade** (#555). Each capability
  has a runnable example under `examples/direct/<scenario>/` with a minimal
  manifest and the facade dependency that replaces it
  (`examples/direct/README.md`): the imports change from
  `optionstratlib::<module>::X` to `optionstratlib_<crate>::<module>::X` and
  name the same types.

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
