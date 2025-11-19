::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [simulate_heston_volatility]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#238-255){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn simulate_heston_volatility(
    kappa: Decimal,
    theta: Decimal,
    xi: Decimal,
    v0: Decimal,
    dt: Decimal,
    steps: usize,
) -> Result<Vec<Positive>, VolatilityError>
```

Expand description

::: docblock
Simulates stochastic volatility using the Heston model (simplified).

## [§](#arguments){.doc-anchor}Arguments

- `kappa` - Mean reversion speed.
- `theta` - Long-term variance.
- `xi` - Volatility of volatility.
- `v0` - Initial variance.
- `dt` - Time step.
- `steps` - Number of simulation steps.

## [§](#returns){.doc-anchor}Returns

A vector of Decimal values representing the simulated volatility.
:::
::::::
:::::::
