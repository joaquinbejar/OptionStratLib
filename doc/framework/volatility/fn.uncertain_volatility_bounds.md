::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[volatility](index.html)
:::

# Function [uncertain_volatility_bounds]{.fn} Copy item path

[[Source](../../src/optionstratlib/volatility/utils.rs.html#268-288){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn uncertain_volatility_bounds(
    option: &Options,
    min_volatility: Positive,
    max_volatility: Positive,
) -> Result<(Positive, Positive), VolatilityError>
```

Expand description

::: docblock
Calculates bounds for uncertain volatility.

## [§](#arguments){.doc-anchor}Arguments

- `option` - The option for which to calculate bounds.
- `min_volatility` - The minimum possible volatility.
- `max_volatility` - The maximum possible volatility.

## [§](#returns){.doc-anchor}Returns

A tuple of (lower_bound, upper_bound) for the option price.
:::
::::::
:::::::
