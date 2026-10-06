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
visualization and backtesting layers; the `optionstratlib` facade provides
them.

## Features

| Feature  | Default | Effect                                                                    |
|----------|---------|---------------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on strategy types and enables `schema` below    |

## License

MIT
