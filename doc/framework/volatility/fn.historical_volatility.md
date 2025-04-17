::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [historical_volatility]{.fn}Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#53-61){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn historical_volatility(
    returns: &[Decimal],
    window_size: usize,
) -> Result<Vec<Positive>, Box<dyn Error>>
```

Expand description

::: docblock
Calculates historical volatility using a moving window approach.

## [§](#arguments){.doc-anchor}Arguments

- `returns` - A slice of Decimal values representing the returns.
- `window_size` - The size of the moving window.

## [§](#returns){.doc-anchor}Returns

A vector of Decimal values representing the historical volatility for
each window.
:::
::::::
:::::::
