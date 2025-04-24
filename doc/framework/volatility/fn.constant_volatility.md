::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [constant_volatility]{.fn}Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#29-41){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn constant_volatility(
    returns: &[Decimal],
) -> Result<Positive, Box<dyn Error>>
```

Expand description

::: docblock
Calculates the constant volatility from a series of returns.

## [§](#arguments){.doc-anchor}Arguments

- `returns` - A slice of Decimal values representing the returns.

## [§](#returns){.doc-anchor}Returns

The calculated volatility as an Decimal.
:::
::::::
:::::::
