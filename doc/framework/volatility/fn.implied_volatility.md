::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [implied_volatility]{.fn}Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#108-156){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn implied_volatility(
    market_price: Positive,
    options: &mut Options,
    max_iterations: i64,
) -> Result<Positive, Box<dyn Error>>
```

Expand description

::: docblock
Calculates the implied volatility of an option given its market price.

This function uses the Newton-Raphson method to iteratively approximate
the implied volatility that corresponds to the observed market price of
the option. The implied volatility is updated within the `Options`
struct provided as a mutable reference.

## [§](#parameters){.doc-anchor}Parameters

- `market_price`: The observed market price of the option.
- `options`: A mutable reference to an `Options` struct, which should
  contain the necessary methods and fields such as `implied_volatility`,
  `calculate_price_black_scholes()`, and `vega()`.
- `max_iterations`: The maximum number of iterations allowed for the
  Newton-Raphson method.

## [§](#returns){.doc-anchor}Returns

The function returns the estimated implied volatility of the option.

## [§](#remarks){.doc-anchor}Remarks

- If the price difference between the calculated and market price is
  within the tolerated threshold (`TOLERANCE`), the current implied
  volatility is returned.
- The function ensures that the implied volatility stays positive.
:::
::::::
:::::::
