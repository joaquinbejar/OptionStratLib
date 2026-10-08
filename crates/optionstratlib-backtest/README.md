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

## Place in the workspace

- **Depends on** `optionstratlib-core`, `optionstratlib-pricing`,
  `optionstratlib-simulation`, `optionstratlib-analytics` and
  `optionstratlib-strategies`.
- **Must not depend on** `optionstratlib-visualization`; `make check-graph`
  enforces the layering (ADR-0001 D9).
- **In the facade:** `optionstratlib::backtesting` and `BacktestError` in
  `optionstratlib::error`, under the facade feature `backtest`. The facade
  paths are the same types as the paths here; the [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

Moving from 0.21? The [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
maps each redesign to its replacement workflow; 0.22 keeps no
compatibility with 0.21.

## Minimal example

```rust
use optionstratlib_backtest::backtesting::SimulationStatsResult;

fn main() {
    // A run's statistics start empty; `Simulate::simulate` fills them from a
    // `Simulator`'s paths (see the runnable example below).
    let stats = SimulationStatsResult::default();
    assert_eq!(stats.total_simulations, 0);
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-backtest`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/backtest); `make tree-example-direct-backtest`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                                |
|----------|---------|-----------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on backtest types and enables `schema` below |

## License

MIT
