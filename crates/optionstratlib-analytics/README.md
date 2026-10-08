# optionstratlib-analytics

Strategy-neutral analytics of [OptionStratLib](https://github.com/joaquinbejar/OptionStratLib):
profit and loss, SPAN margin, price-probability kernels, risk-neutral
densities, option-chain metrics and the projections of option data onto
curves and surfaces. The crate depends on `optionstratlib-core`,
`optionstratlib-math`, `optionstratlib-pricing` and `optionstratlib-market`,
and on no strategy, simulation, backtesting or plotting code.

| Module      | Contents                                                                                         |
|-------------|--------------------------------------------------------------------------------------------------|
| `analytics` | probability kernels, `ProfitLossRange`, `RNDAnalysis`, `BasicCurves`, `BasicSurfaces`, `OptionChainProjections` |
| `pnl`       | `PnL`, `PnLCalculator`, `Transaction`, `DeltaAdjustment`, P&L metrics documents                   |
| `risk`      | `SPANMargin`, `RiskMetricsSimulation`, `RiskCategory`                                            |
| `metrics`   | price, risk, composite, temporal, stress and liquidity metrics over an `OptionChain`              |
| `error`     | `ProbabilityError`, `ProjectionError`, `TransactionError`                                        |

The probability analysis of a concrete strategy builds on these kernels and
lives with the strategies; the `optionstratlib` facade provides it.

## Place in the workspace

- **Depends on** `optionstratlib-core`, `optionstratlib-math`,
  `optionstratlib-pricing` and `optionstratlib-market`.
- **Must not depend on** `optionstratlib-simulation`, `-strategies`,
  `-backtest` and `-visualization`; `make check-graph` enforces the layering
  (ADR-0001 D9).
- **In the facade:** `optionstratlib::{analytics, pnl, risk, metrics}` and the
  analytics errors in `optionstratlib::error`, under the facade feature
  `analytics`. The facade paths are the same types as the paths here; the
  [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

Moving from 0.21? The [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
maps each redesign to its replacement workflow; 0.22 keeps no
compatibility with 0.21.

## Minimal example

```rust
use optionstratlib_analytics::error::ProbabilityError;
use optionstratlib_analytics::analytics::{VolatilityAdjustment, calculate_single_point_probability};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use rust_decimal_macros::dec;

fn main() -> Result<(), ProbabilityError> {
    // Probability of finishing below and above 105 in 30 days from 100.
    let (below, above) = calculate_single_point_probability(
        &Positive::HUNDRED,
        &pos_or_panic!(105.0),
        VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.2),
            std_dev_adjustment: Positive::ZERO,
        },
        None,
        &ExpirationDate::Days(pos_or_panic!(30.0)),
        Some(dec!(0.05)),
    )?;
    assert!((below.to_dec() + above.to_dec() - dec!(1)).abs() < dec!(1e-9));
    Ok(())
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-analytics`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/analytics); `make tree-example-direct-analytics`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                                         |
|----------|---------|--------------------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on analytics types and enables `schema` below        |

## License

MIT
