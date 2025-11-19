::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)
:::

# Module simulation Copy item path

[[Source](../../src/optionstratlib/simulation/mod.rs.html#1-137){.src}
]{.sub-heading}
::::

Expand description

::: docblock
- `simulation` - Simulation techniques for scenario analysis.

Framework for Monte Carlo and other simulation methods to model
potential market scenarios and their impact on options strategies.
Includes path generation algorithms and statistical analysis of
simulation results.

## [§](#random-walk-simulation-library){.doc-anchor}Random Walk Simulation Library

This library provides tools for simulating and analyzing random walk
processes and other stochastic models. It includes implementations of
various random walk algorithms, statistical utilities, and visualization
capabilities.

The library is organized into several modules:

- `model`: Contains the data structures and types that represent
  stochastic processes
- `simulator`: Provides simulation engines and algorithms for running
  the models
- `utils`: Utility functions and helpers for statistical analysis and
  data manipulation
- `walk`: Public API for creating and running random walk simulations

### [§](#core-components){.doc-anchor}Core Components

### [§](#mathematical-background){.doc-anchor}Mathematical Background

The random walk implementation follows the geometric Brownian motion
model with:

1.  Price changes: dS = μSdt + σSdW

    - S: Asset price
    - μ: Drift (mean return)
    - σ: Volatility
    - dW: Wiener process increment

2.  Volatility updates: σ(t) \~ N(σ, σ_change)

    - Stochastic volatility component
    - Updates based on volatility_window

### [§](#features){.doc-anchor}Features

- Geometric Brownian motion simulation
- Stochastic volatility modeling
- Real-time volatility estimation
- Integration with option pricing parameters
- Visualization support
- Iterator interface for sequential processing

### [§](#performance-considerations){.doc-anchor}Performance Considerations

- Time Complexity: O(n) for generation, where n is the number of steps
- Space Complexity: O(n) for storing the price path
- Volatility calculation: O(w) where w is the volatility window size

### [§](#implementation-notes){.doc-anchor}Implementation Notes

- All prices are strictly positive (enforced by Positive)
- Volatility is estimated using rolling windows
- The iterator provides option pricing parameters for each step
- Thread-safe random number generation
- Supports various time frames (daily, weekly, monthly)
:::

## Re-exports[§](#reexports){.anchor} {#reexports .section-header}

`pub use exit::`[`ExitPolicy`](exit/enum.ExitPolicy.html "enum optionstratlib::simulation::exit::ExitPolicy"){.enum}`;`

`pub use exit::`[`check_exit_policy`](exit/fn.check_exit_policy.html "fn optionstratlib::simulation::exit::check_exit_policy"){.fn}`;`

## Modules[§](#modules){.anchor} {#modules .section-header}

[exit](exit/index.html "mod optionstratlib::simulation::exit"){.mod}
:   Module containing exit policy definitions for option trading
    strategies.

[randomwalk](randomwalk/index.html "mod optionstratlib::simulation::randomwalk"){.mod}
:   Random Walk Module

[simulator](simulator/index.html "mod optionstratlib::simulation::simulator"){.mod}
:   Provides simulation engines and algorithms for running stochastic
    models.

[steps](steps/index.html "mod optionstratlib::simulation::steps"){.mod}
:   Module containing functionality for stepping through data or
    calculations.

## Structs[§](#structs){.anchor} {#structs .section-header}

[SimulationStats](struct.SimulationStats.html "struct optionstratlib::simulation::SimulationStats"){.struct}
:   Statistics for tracking Short Put strategy performance across
    multiple simulations.

[WalkParams](struct.WalkParams.html "struct optionstratlib::simulation::WalkParams"){.struct}
:   Parameters for stochastic process simulations (random walks).

## Enums[§](#enums){.anchor} {#enums .section-header}

[WalkType](enum.WalkType.html "enum optionstratlib::simulation::WalkType"){.enum}
:   Enum defining different types of random walks

## Traits[§](#traits){.anchor} {#traits .section-header}

[Simulate](trait.Simulate.html "trait optionstratlib::simulation::Simulate"){.trait}
:   Trait for simulating trading strategies across multiple price paths.

[WalkTypeAble](trait.WalkTypeAble.html "trait optionstratlib::simulation::WalkTypeAble"){.trait}
:   Trait for implementing various random walk models and stochastic
    processes.
::::::
:::::::
