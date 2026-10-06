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

## Features

| Feature  | Default | Effect                                                                     |
|----------|---------|----------------------------------------------------------------------------|
| `schema` | no      | Derives `utoipa::ToSchema` on simulation types and enables `schema` in core and pricing |

## License

MIT
