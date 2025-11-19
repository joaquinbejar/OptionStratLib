::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[pricing](../index.html)
:::

# Module telegraph Copy item path

[[Source](../../../src/optionstratlib/pricing/telegraph.rs.html#6-581){.src}
]{.sub-heading}
::::

Expand description

::: docblock
Telegraph process model for asset price movement.

Implements a telegraph process model which can be used as an alternative
to geometric Brownian motion for modeling asset price movements with
specific jump characteristics.

The telegraph model is particularly useful for capturing market regimes
with distinct volatility states.

## [§](#telegraph-process){.doc-anchor}Telegraph Process

A Telegraph Process (also known as a two-state process) is a stochastic
process that alternates between two states, typically represented as +1
and -1.

### [§](#key-parameters){.doc-anchor}Key Parameters

- `lambda_up`: Transition rate from state -1 to +1
- `lambda_down`: Transition rate from state +1 to -1

These parameters are always positive (λ_up, λ_down \> 0) and typically
range from 0 to 10 in practice.

### [§](#algorithm){.doc-anchor}Algorithm

1.  The process starts in one of the two states (+1 or -1), usually
    chosen randomly.

2.  At each time step dt:

    - If the current state is +1, there's a probability of changing to
      -1.
    - If the current state is -1, there's a probability of changing to
      +1.

3.  The probability of change in an interval dt is calculated as:
    P(change) = 1 - e\^(-λ \* dt) Where λ is λ_up if the current state
    is -1, or λ_down if the current state is +1.

### [§](#parameter-interpretation){.doc-anchor}Parameter Interpretation

- Higher values indicate more frequent changes between states.
- Lower values indicate that the process tends to remain in a state for
  longer.

Typical value ranges:

- Infrequent changes: 0.1 to 1
- Moderate changes: 1 to 5
- Very frequent changes: 5 to 10

### [§](#relationship-between-parameters){.doc-anchor}Relationship Between Parameters

- If λ_up = λ_down, the process is symmetric.
- If λ_up \> λ_down, the process tends to spend more time in the +1
  state.
- If λ_up \< λ_down, the process tends to spend more time in the -1
  state.

### [§](#use-in-financial-modeling){.doc-anchor}Use in Financial Modeling

In the context of financial options, the Telegraph Process can be used
to model:

- Changes in volatility (high/low volatility regime)
- Changes in market direction (bullish/bearish trend)
- Changes in interest rates (high/low)

### [§](#parameter-estimation){.doc-anchor}Parameter Estimation

Parameters can be estimated from historical data:

1.  Classify historical periods into +1 and -1 states based on a
    threshold.
2.  Calculate the average duration of each state.
3.  Estimate λ_up as 1 / (average duration of -1 state).
4.  Estimate λ_down as 1 / (average duration of +1 state).

### [§](#advantages){.doc-anchor}Advantages

- Allows modeling of abrupt changes in the market.
- Captures "regime change" behaviors that continuous models can't easily
  represent.
- Relatively simple to implement and understand.

### [§](#considerations){.doc-anchor}Considerations

- The choice of λ_up and λ_down significantly affects the model's
  behavior.
- These parameters may need to be calibrated with historical or market
  data.
- In more advanced models, λ_up and λ_down could be dynamically adjusted
  based on changing market conditions.

Remember that the choice of these parameters depends heavily on the
specific asset being modeled and the time horizon of your analysis. It's
common to experiment with different values and validate results against
real data to find the best configuration for your specific model.
:::

## Structs[§](#structs){.anchor} {#structs .section-header}

[TelegraphProcess](struct.TelegraphProcess.html "struct optionstratlib::pricing::telegraph::TelegraphProcess"){.struct}
:   Represents a Telegraph Process, a two-state continuous-time Markov
    chain model used to simulate stochastic processes with discrete
    state transitions.

## Functions[§](#functions){.anchor} {#functions .section-header}

[telegraph](fn.telegraph.html "fn optionstratlib::pricing::telegraph::telegraph"){.fn}
:   Prices an option using the Telegraph process simulation method.
::::::
:::::::
