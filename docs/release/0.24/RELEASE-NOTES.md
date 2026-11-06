# OptionStratLib 0.24.0 release notes

OptionStratLib 0.24.0 makes the wall clock optional and adds calibrated
volatility surfaces, SVI slices, static arbitrage checks, settlement obligations
and a quadratic rough Heston simulation. **It is a breaking release**: the list
below says who has to change something.

- [Migration guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.24.md):
  what to change and why.
- [CHANGELOG](https://github.com/joaquinbejar/OptionStratLib/blob/main/CHANGELOG.md):
  every change with its issue.

## Packages

Ten packages, all at **0.24.0** (`docs/versioning-policy.md`: a minor release
resets the family), in dependency order: `optionstratlib-core`, `-math`, `-pricing`,
`-simulation`, `-market`, `-analytics`, `-strategies`, `-backtest`, `-visualization`
and `optionstratlib`.

## Who has to change something

- **You set `default-features = false`**: add the new `clock` feature to keep
  reading the time. Without it, a time the library needs comes from an argument or
  the thread's reference datetime, or the call fails.
- **You call `iv_surface`**: it is now
  `HorizonVolatilitySurface::horizon_volatility_surface`, and the values do not
  change. `ImpliedVolatilitySurface` now names the surface calibrated from quotes.
- **You call `delta_exposure` or `gamma_exposure`**: a strike with no Greek is now
  an error in every build, not zero. Call `try_update_greeks` first.
- **You match `TradeError`, `PositionError`, `ChainError`, `StrategyError`,
  `TransactionError` or `optionstratlib::error::Error` exhaustively**: they have new
  variants.
- **You order a mix of `Days` and `DateTime` expirations**: under `expiration_date`
  0.6 every `Days` sorts first. Use `ExpirationDate::cmp_at` to order in time. If you
  name `expiration_date` or `option_type` directly, move them to 0.6.

## Minimal manifests

Each manifest below is followed by a complete program; `make
check-release-notes` builds and runs every pair as its own crate outside the
repository, against this checkout, and the facade's doctests compile the
same programs. A program that writes `dec!` depends on `rust_decimal` itself.

A quadratic rough Heston price through the facade. Every parameter is illustrative,
and none is calibrated to any underlying:

```toml
[dependencies]
optionstratlib = { version = "0.24.0", default-features = false, features = ["clock", "simulation"] }
rust_decimal = "1.43"
```

```rust
use optionstratlib::prelude::*;
use optionstratlib::simulation::{
    QuadraticRoughHestonGrid, QuadraticRoughHestonKernel, QuadraticRoughHestonParams,
    price_vanilla_quadratic_rough_heston,
};
use std::num::NonZeroUsize;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let kernel = QuadraticRoughHestonKernel::try_new(dec!(0.6), 5, dec!(5))?;
    let params = QuadraticRoughHestonParams::try_new(
        dec!(0.5),
        dec!(0.1),
        dec!(0.01),
        dec!(1.5),
        dec!(1),
        kernel,
        vec![dec!(-0.05); 5],
        Positive::HUNDRED,
    )?;
    let grid = QuadraticRoughHestonGrid::try_new(dec!(0.25), 50)?;
    let paths = NonZeroUsize::try_from(2_000_usize)?;
    let estimate = price_vanilla_quadratic_rough_heston(
        &params,
        &grid,
        OptionStyle::Call,
        Positive::HUNDRED,
        paths,
        958,
    )?;
    assert!(estimate.price() > Decimal::ZERO);
    Ok(())
}
```

A butterfly check on the pricing crate alone, on the slice of Gatheral and Jacquier
(arXiv:1204.0646) known to violate it:

```toml
[dependencies]
optionstratlib-pricing = "0.24.0"
optionstratlib-core = "0.24.0"
rust_decimal = "1.43"
rust_decimal_macros = "1.40"
```

```rust
use optionstratlib_core::model::Positive;
use optionstratlib_pricing::volatility::{SviParameters, SviSlice, check_butterfly};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let slice = SviSlice::try_new(
        SviParameters::try_new(
            dec!(-0.0410),
            dec!(0.1331),
            dec!(0.3586),
            dec!(0.3060),
            dec!(0.4153),
        )?,
        Positive::HUNDRED,
        Positive::ONE,
    )?;
    let grid: Vec<Decimal> = (-15_i64..=15).map(|k| Decimal::new(k, 1)).collect();
    let report = check_butterfly(&slice, &grid)?;
    assert!(!report.passed());
    Ok(())
}
```

## Changes

- **Optional clock** (#944): a `clock` feature, on by default, in every crate that
  reads the time; pure `*_at` forms and fallible `try_*` forms; `ClockError`. Items
  with no error channel do not exist without it. `clock` is not behaviourally additive:
  if any crate in the final graph enables it, a `DateTime` measures from the wall clock.
- **Calibrated surface** (#947): `ImpliedVolatilitySurface::calibrate` builds one SVI
  slice per expiry from bid and ask quotes at an as-of time. Every quote is clean or
  rejected with a reason, and a later quote cannot change the surface.
- **SVI slices** (#945): `SviSlice` and `calibrate_svi`.
- **Static arbitrage checks** (#946): `check_butterfly`, `check_calendar` and
  `check_static_arbitrage`, on a grid.
- **Settlement obligations** (#949): `settlement_obligation` for expiry, exercise and
  assignment.
- **Quadratic rough Heston** (#958): the factor kernel, a seeded simulation by the scheme
  (A.1) of Rosenbaum and Zhang, and Monte Carlo vanilla prices with their standard error.
- **`HorizonVolatilitySurface`** (#948): the rename above.

## Limits worth knowing

- **Passing the static arbitrage checks means only those checks, on that grid.** A
  surface with findings is still built, so read its report.
- **A surface's fit limit is on a weighted RMSE.** `SurfaceExpiry::unweighted_rmse` and
  `quotes_outside_bid_ask` report a fit that the weights hide, and do not reject it.
- **A small Monte Carlo standard error does not mean a good rough Heston price.** With
  strong feedback, runs blow up without an error, so compare seeds and step counts.
- Every quote is priced as European Black-Scholes, with flat rates across expiries.

## Results that change

- `delta_exposure` and `gamma_exposure` return an error on a strike with no Greek.
- Mixed `Days` and `DateTime` no longer compare by instant under `Ord`.
- `OptionChain::get_random_positions` opens all its positions at one instant per call.

Nothing else changes in a default build's prices, Greeks or P&L.
