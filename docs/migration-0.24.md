# Moving to 0.24: an optional clock, and new volatility and settlement APIs

0.24 makes the wall clock optional, renames a surface whose name overstated what it
computes, and adds calibrated volatility surfaces, SVI slices, static arbitrage checks
and settlement obligations. The full list is in `CHANGELOG.md`. All ten crates move to
`0.24.0`.

## If you set `default-features = false`

`clock` is a new default feature, so a build that names its capabilities loses it.
Add it to keep reading the time:

```toml
optionstratlib = { version = "0.24.0", default-features = false, features = ["clock", "pricing"] }
```

A plain `optionstratlib = "0.24.0"` keeps the clock, and so does a direct dependency on
a component with its default features. `optionstratlib-math` reads no clock and has no
such feature.

## Building without the clock

Drop `clock` when a build must be deterministic, as a simulator or a replay engine
must. No published crate then enables `chrono/clock`, and a time the library needs comes
from an argument or from the thread's reference datetime, or the call fails.

| What you call | Without `clock` |
|---|---|
| `Trade::new`, a strategy's `new`, and other constructors that return a `Result` | Typed error. Use `Trade::new_at`, `Position::blank_at`, or a strategy's `new_at` |
| `Position::days_held`, `days_to_expiration`, `OptionChain::get_random_positions` | Typed error. Use `days_held_at`, `days_to_expiration_at`, `get_random_positions_at` |
| `OptionSeries::odte` | Absent. Use `odte_at` |
| `OptionChain::update_greeks`, `update_greek_snapshots` | Absent. Use `try_update_greeks`, `try_update_greek_snapshots` |
| `is_delta_neutral`, `meets_greek_targets` | Absent. Use `delta` and `portfolio_greeks` |
| `Default` for `Position`, the strategies, `TradeRecord` and `PnLMetrics` | Absent. Build the value with its explicit constructor |
| `long` and `short` on the spot, future and perpetual legs | Absent |
| `create_sample_position`, `get_today_formatted`, `get_today_or_tomorrow_formatted` | Absent |
| `From<PathOutcome> for SimulationResult` | Absent. Use `SimulationResult::from_outcome_at` |

A `Days` expiration prices to the same decimal in every build. It needs a time only where
it must become a calendar date: `build_chain`, `build_series` and the P&L of a bare
`Options`. A `DateTime` measures from the thread's reference datetime, which the library's
parallel sections carry to their workers. A thread of your own has to carry it itself,
with `optionstratlib_core::utils::time::ReferenceCarrier`.

**`clock` is not behaviourally additive.** If any crate in your final dependency graph
enables it, a `DateTime` measures from the wall clock, whatever your own manifest says. A
replay engine should check this at startup:

```rust,ignore
assert!(optionstratlib_core::utils::time::now().is_err());
```

The facade's `clock` does not forward to `optionstratlib-visualization`. That crate has no
production item gated on it, so nothing is lost.

## Errors that gained variants

`TradeError`, `PositionError`, `ChainError`, `StrategyError` and `TransactionError` have a
new `Clock(ClockError)` variant. The aggregate `optionstratlib::error::Error` gains `Clock`,
`Settlement`, `Svi`, `StaticArbitrage` and `ImpliedVolatilitySurface`. These enums are
exhaustive, so a `match` over one needs the new arms or a wildcard.

A time to expiry that cannot be measured does not arrive as `ClockError`.
`expiration_date` cannot name that type, so it arrives as
`ExpirationDateError::InvalidDateTime` holding the same message, inside the error of the
crate you called: `OptionsError::ExpirationDate`, `GreeksError::ExpirationDate`, and so on.

## Exposures fail on a strike with no Greek

`OptionChain::delta_exposure` and `gamma_exposure` now return an error when a strike has no
Greek, in every build. With a clock they used to add it as zero, so a chain whose Greeks
were never computed reported zero exposure.

```rust,ignore
chain.try_update_greeks()?;
let exposure = chain.delta_exposure()?;
```

`update_greeks` leaves a Greek empty when it cannot be calculated for a strike, and logs
it, so handle the error wherever that can happen.

## `iv_surface` is now `horizon_volatility_surface`

| 0.23 | 0.24 |
|---|---|
| `ImpliedVolatilitySurface` (trait) | `HorizonVolatilitySurface` |
| `iv_surface` | `horizon_volatility_surface` |

The numbers do not change. They were always `sigma * sqrt(days / 365)`, the volatility over
that horizon, so 20% at 30 days comes back as about 5.73%. For annualized implied volatility
by strike, read `ImpliedVolatilityCurve::iv_curve`.

The name `ImpliedVolatilitySurface` now belongs to a different thing: the struct calibrated
from quotes, described below. Code that imported the trait under the old name fails to
compile rather than changing meaning.

## `expiration_date` and `option_type` 0.6

- Every `Days` now sorts before every `DateTime` under `Ord`, where a mix used to compare by
  instant. This affects `OptionSeries::chains` and whatever walks it, and `OptionChain`'s
  `Ord` when `expiration_date` strings mix the two. One variant orders as before. To order a
  mix in time, use `ExpirationDate::cmp_at(&other, base)`.
- `ExpirationDate::from_string` and `get_days` no longer write the thread's reference
  datetime. Call `set_reference_datetime` where you relied on it.
- `get_years`, `get_days`, `get_date`, `get_date_string` and `from_string_to_days` exist only
  with the `clock` feature of `expiration_date`.

If you name either crate directly, move it to 0.6 too: two versions of `ExpirationDate` are
different types.

## Other behaviour changes

- `OptionChain::get_random_positions` opens all its positions at one instant per call.
- `Strategy::new` opens all the legs of one call at one instant.
- `SimulationStatsResult::from_outcomes` stamps all its results with one instant.

## New APIs

| Need | API | Example in `examples_pricing` |
|---|---|---|
| Expiry, exercise and settlement obligations | `settlement_obligation` | `settlement_obligations` |
| A raw SVI slice fitted to observations | `calibrate_svi` | `svi_calibration` |
| Butterfly and calendar checks on a grid | `check_butterfly`, `check_calendar`, `check_static_arbitrage` | `static_arbitrage` |
| A surface calibrated from bid and ask quotes | `ImpliedVolatilitySurface::calibrate` | `calibrated_surface` |

None of them reads a clock. Passing the static arbitrage checks means only those checks, on
that grid. A surface with findings is still built, so read its report.
