::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [ewma_volatility]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#70-83){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn ewma_volatility(
    returns: &[Decimal],
    lambda: Decimal,
) -> Result<Vec<Positive>, VolatilityError>
```

Expand description

::: docblock
Calculates EWMA (Exponentially Weighted Moving Average) volatility.

## [§](#arguments){.doc-anchor}Arguments

- `returns` - A slice of Decimal values representing the returns.
- `lambda` - The decay factor (typically 0.94 for daily data).

## [§](#returns){.doc-anchor}Returns

A vector of Decimal values representing the EWMA volatility.
:::
::::::
:::::::
