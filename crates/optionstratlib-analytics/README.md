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

## Example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-analytics`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/analytics); `make tree-example-direct-analytics`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                                         |
|----------|---------|--------------------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on analytics types and enables `schema` below        |

## License

MIT
