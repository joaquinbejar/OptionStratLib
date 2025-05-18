::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)::[utils](index.html)
:::

# Function [random_color]{.fn}Copy item path

[[Source](../../../src/optionstratlib/visualization/utils.rs.html#524-535){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn random_color() -> ColorScheme
```

Expand description

::: docblock
Creates a random, visually distinguishable color.

Uses HSL color space to generate colors with:

- Random hue (0-360)
- High saturation (60-90%)
- Medium lightness (35-65%)

This approach helps ensure colors are:

1.  Visually distinct from each other
2.  Saturated enough to be visible
3.  Neither too dark nor too light
:::
::::::
:::::::
