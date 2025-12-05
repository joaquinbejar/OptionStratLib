::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [garch_volatility]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#209-222){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn garch_volatility(
    returns: &[Decimal],
    omega: Decimal,
    alpha: Decimal,
    beta: Decimal,
) -> Result<Vec<Positive>, VolatilityError>
```

Expand description

::: docblock
Calculates GARCH(1,1) volatility (simplified).

## [§](#arguments){.doc-anchor}Arguments

- `returns` - A slice of Decimal values representing the returns.
- `omega`, `alpha`, `beta` - GARCH(1,1) parameters.

## [§](#returns){.doc-anchor}Returns

A vector of Decimal values representing the GARCH(1,1) volatility.
:::
::::::
:::::::
