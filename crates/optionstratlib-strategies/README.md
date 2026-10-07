# optionstratlib-strategies

Option strategies of [OptionStratLib](https://github.com/joaquinbejar/OptionStratLib):
vertical spreads, butterflies, condors, straddles, strangles, covered and
protective structures, custom multi-leg strategies, delta neutrality and the
probability analysis of a strategy. The crate depends on
`optionstratlib-core`, `optionstratlib-pricing`, `optionstratlib-market` and
`optionstratlib-analytics`, and on no simulation, backtesting or plotting
code.

| Module       | Contents                                                                                                   |
|--------------|------------------------------------------------------------------------------------------------------------|
| `strategies` | `BullCallSpread`, `IronCondor`, `LongStraddle`, `CustomStrategy`, …; the `Strategies`, `Strategable`, `Optimizable`, `BasicAble` and `Validable` traits; `StrategyConstructor`, `StrategyRequest`; `DeltaNeutrality`; `ProbabilityAnalysis` |
| `error`      | `StrategyError`, `StrategyResult` and the error kinds in `error::strategies`                               |

The optimiser side filter, `FindOptimalSide`, is a market type:
`optionstratlib_market::chains::utils::FindOptimalSide`. Charts of a strategy
and the simulation of a strategy over a price path stay with the
visualization and backtesting layers (`optionstratlib-visualization`,
`optionstratlib-backtest`); the `optionstratlib` facade re-exports them.

## Place in the workspace

- **Depends on** `optionstratlib-core`, `optionstratlib-pricing`,
  `optionstratlib-market` and `optionstratlib-analytics`.
- **Must not depend on** `optionstratlib-simulation`, `-backtest` and
  `-visualization`; `make check-graph` enforces the layering (ADR-0001 D9).
- **In the facade:** `optionstratlib::strategies` and `StrategyError` /
  `StrategyResult` / `AdjustmentError` in `optionstratlib::error`, under the
  facade feature `strategies`. The facade paths are the same types as the
  paths here; the [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

<!-- #553: link the 0.21 to 0.22 migration guide here -->

## Minimal example

```rust
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_strategies::strategies::base::BreakEvenable;
use optionstratlib_strategies::strategies::{BullCallSpread, Validable};
use rust_decimal_macros::dec;

fn main() -> Result<(), StrategyError> {
    // Long the 95 call for 6.50, short the 105 call for 1.50: a debit of 5.
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
    assert!(spread.validate());
    assert_eq!(spread.get_break_even_points()?, &vec![pos_or_panic!(100.0)]);
    Ok(())
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-strategies`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/strategies); `make tree-example-direct-strategies`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                                    |
|----------|---------|---------------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on strategy types and enables `schema` below    |

## License

MIT
