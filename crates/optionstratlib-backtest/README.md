# optionstratlib-backtest

Strategy backtests over simulated paths for
[OptionStratLib](https://github.com/joaquinbejar/OptionStratLib): a strategy
is evaluated on every path of a simulator, each path ends by an exit policy,
and the run is summarised with its statistics, report types and metrics. The
crate depends on `optionstratlib-core`, `optionstratlib-pricing`,
`optionstratlib-simulation`, `optionstratlib-analytics` and
`optionstratlib-strategies`, and on no option-chain I/O, plotting or async
code.

| Module        | Contents                                                                 |
|---------------|--------------------------------------------------------------------------|
| `backtesting` | `Simulate`, adapters from the engine's path results, statistics, reports, metrics |
| `error`       | `BacktestError`                                                          |

## Example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-backtest`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/backtest); `make tree-example-direct-backtest`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                                |
|----------|---------|-----------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on backtest types and enables `schema` below |

## License

MIT
