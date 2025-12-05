::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[pricing](index.html)
:::

# Function [probability_keep_under_strike]{.fn} Copy item path

[[Source](../../src/optionstratlib/pricing/utils.rs.html#335-353){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn probability_keep_under_strike(
    option: Options,
    strike: Option<Positive>,
) -> Result<Decimal, DecimalError>
```

Expand description

::: docblock
Calculates the probability that the option will remain under the strike
price.

## [§](#parameters){.doc-anchor}Parameters

- `option`: An `Options` struct that contains various attributes
  necessary for the calculation, such as underlying price, strike price,
  risk-free rate, expiration date, and implied volatility.
- `strike`: An optional `f64` value representing the strike price. If
  `None` is provided, the function uses the `strike_price` from the
  `Options` struct.

## [§](#returns){.doc-anchor}Returns

A `f64` value representing the calculated probability.
:::
::::::
:::::::
