:::::::: width-limiter
::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [generate_ou_process]{.fn}Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#411-431){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn generate_ou_process(
    x0: Positive,
    mu: Positive,
    theta: Positive,
    volatility: Positive,
    dt: Positive,
    steps: usize,
) -> Vec<Positive>
```

Expand description

:::: docblock
Generates a mean-reverting Ornstein-Uhlenbeck process time series

This function simulates a discrete-time Ornstein-Uhlenbeck stochastic
process, which is commonly used in financial mathematics to model
mean-reverting processes such as interest rates, volatility, or
commodity prices. The process follows the stochastic differential
equation:

dX_t = θ(μ - X_t)dt + σdW_t

Where:

- θ (theta) is the speed of reversion to the mean
- μ (mu) is the long-term mean level
- σ (sigma) is the volatility or intensity of random fluctuations
- W_t is a Wiener process (standard Brownian motion)

## [§](#arguments){.doc-anchor}Arguments

- `x0` - Initial value of the process
- `mu` - Long-term mean the process reverts to
- `theta` - Speed of mean reversion (higher values cause faster
  reversion)
- `sigma` - Volatility parameter controlling the intensity of random
  fluctuations
- `dt` - Time step size for the simulation
- `steps` - Number of time steps to simulate

## [§](#returns){.doc-anchor}Returns

A vector containing the simulated values of the Ornstein-Uhlenbeck
process at each time step

## [§](#examples){.doc-anchor}Examples

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal_macros::dec;
use optionstratlib::pos;
use optionstratlib::volatility::generate_ou_process;

// Simulate an OU process with initial value 1.0, mean 1.5,
// reversion speed 0.1, volatility 0.2, time step 0.01, for 1000 steps
let process = generate_ou_process(
    pos!(1.0),       // initial value
    pos!(1.5),       // long-term mean
    pos!(0.1),       // speed of reversion
    pos!(0.2),       // volatility
    pos!(0.01),      // time step
    1000             // number of steps
);
```
:::
::::
:::::::
::::::::
