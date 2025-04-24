::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[utils](../index.html)::[others](index.html)
:::

# Function [calculate_log_returns]{.fn}Copy item path

[[Source](../../../src/optionstratlib/utils/others.rs.html#200-217){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn calculate_log_returns(
    close_prices: &[Positive],
) -> Result<Vec<Positive>, DecimalError>
```

Expand description

::: docblock
## [§](#calculate-logarithmic-returns){.doc-anchor}Calculate Logarithmic Returns

Computes the logarithmic returns from a series of close prices.
Logarithmic returns are calculated as the natural logarithm of the ratio
between consecutive prices.

Logarithmic returns are commonly used in financial analysis because:

- They are additive over time, unlike percentage returns
- They better approximate a normal distribution
- They're suitable for statistical analysis of financial time series

### [§](#parameters){.doc-anchor}Parameters

- `close_prices` - A slice of `Decimal` values representing sequential
  close prices

### [§](#returns){.doc-anchor}Returns

- `Result<Vec<Decimal>, DecimalError>` - A vector of logarithmic returns
  if successful, or an error if the calculation fails

### [§](#errors){.doc-anchor}Errors

This function returns a `DecimalError` in the following cases:

- If any price value is zero or negative
- If there's a failure when converting `Decimal` to `f64` for logarithm
  calculation
- If there's a failure when converting the logarithm result back to
  `Decimal`

### [§](#notes){.doc-anchor}Notes

- Returns an empty vector if fewer than 2 price points are provided
- For consecutive prices P₁ and P₂, the log return is ln(P₂/P₁)
:::
::::::
:::::::
