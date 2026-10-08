# optionstratlib-simulation

The simulation engine of [OptionStratLib](https://github.com/joaquinbejar/OptionStratLib):
random walks, stochastic processes, simulators, exit policies and the generic
evaluation and statistics of simulated paths. The crate depends on
`optionstratlib-core` and `optionstratlib-pricing`, and on no option chain,
strategy, backtesting, plotting, I/O or terminal-presentation code.

| Module       | Contents                                                                                                   |
|--------------|------------------------------------------------------------------------------------------------------------|
| `simulation` | `WalkType` (Brownian, geometric Brownian, log-returns, mean-reverting, jump diffusion, GARCH, Heston, telegraph, custom, historical) and `WalkTypeAble`; `WalkParams`, `Step`, `Xstep`, `Ystep`; `walk_steps`, `walk_steps_par`, `generator_positive`; `RandomWalk`, `Simulator`; `generate_ou_process`; `ExitPolicy`; `PathEvaluator`, `PathOutcome`, `PathStatistics`, `evaluate_paths` |
| `error`      | `SimulationError`, `SimulationResult`                                                                      |

Prices along a path are `Positive`, drifts and statistics
`rust_decimal::Decimal`; `f64` stays inside the kernels. The evaluation of a
strategy over simulated paths, the charts of a walk and the chain and series
generators built on these walks stay with the backtesting, visualization and
`synthetic` layers; the `optionstratlib` facade provides them.

## Place in the workspace

- **Depends on** `optionstratlib-core` and `optionstratlib-pricing`.
- **Must not depend on** `optionstratlib-market`, `-analytics`, `-strategies`,
  `-backtest` and `-visualization`; `make check-graph` enforces the layering
  (ADR-0001 D9).
- **In the facade:** `optionstratlib::simulation` and `SimulationError` /
  `SimulationResult` in `optionstratlib::error`, under the facade feature
  `simulation`. The facade paths are the same types as the paths here; the
  [ownership
  map](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/ownership.md)
  lists every one with its feature.

Moving from 0.21? The [0.22 architecture and adoption
guide](https://github.com/joaquinbejar/OptionStratLib/blob/main/docs/migration-0.22.md)
maps each redesign to its replacement workflow; 0.22 keeps no
compatibility with 0.21.

## Minimal example

```rust
use optionstratlib_simulation::error::SimulationError;
use optionstratlib_core::pos_or_panic;
use optionstratlib_simulation::simulation::generate_ou_process;

fn main() -> Result<(), SimulationError> {
    // An Ornstein-Uhlenbeck path of 50 steps mean-reverting to 100.
    let path = generate_ou_process(
        pos_or_panic!(100.0),
        pos_or_panic!(100.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.2),
        pos_or_panic!(0.01),
        50,
    )?;
    assert_eq!(path.len(), 50);
    Ok(())
}
```

## Runnable example

A runnable program that depends on this crate directly, with the smallest
dependency set and no facade, is [`osl-example-direct-simulation`](https://github.com/joaquinbejar/OptionStratLib/tree/main/examples/direct/simulation); `make tree-example-direct-simulation`
asserts its resolved graph.

## Features

| Feature  | Default | Effect                                                                     |
|----------|---------|----------------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on simulation types and enables `schema` in core and pricing |

## License

MIT
