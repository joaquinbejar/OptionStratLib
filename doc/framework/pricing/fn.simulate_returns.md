::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pricing](index.html)
:::

# Function [simulate_returns]{.fn}Copy item path

[[Source](../../src/optionstratlib/pricing/utils.rs.html#35-91){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn simulate_returns(
    mean: Decimal,
    std_dev: Positive,
    length: usize,
    time_step: Decimal,
) -> Result<Vec<Decimal>, DecimalError>
```

Expand description

::: docblock
Simulates stock returns based on a normal distribution using pure
decimal arithmetic.

## [§](#arguments){.doc-anchor}Arguments

- `mean` - The mean return (annualized)
- `std_dev` - The standard deviation of returns (annualized)
- `length` - The number of returns to simulate
- `time_step` - The time step for each return (e.g., 1/252 for daily
  returns assuming 252 trading days)

## [§](#returns){.doc-anchor}Returns

A Result containing either:

- Ok(`Vec<Decimal>`): A vector of simulated returns as Decimal numbers
- Err(DecimalError): If there's an error in decimal calculations
:::
::::::
:::::::
