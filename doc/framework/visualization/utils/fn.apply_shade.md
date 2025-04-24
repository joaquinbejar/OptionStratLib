::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)::[utils](index.html)
:::

# Function [apply_shade]{.fn}Copy item path

[[Source](../../../src/optionstratlib/visualization/utils.rs.html#34-41){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn apply_shade(base_color: RGBColor, normalized_value: f64) -> RGBColor
```

Expand description

::: docblock
Applies a color gradient effect by interpolating between a base color
and a derived color.

This function creates a smooth color transition based on a normalized
value between 0.0 and 1.0. It uses the base color's RGB components in a
rotated order (G, B, R) to create the end color, then interpolates
between the base and end colors according to the normalized value.

## [§](#arguments){.doc-anchor}Arguments

- `base_color` - The starting RGB color used as the base for the
  gradient effect.
- `normalized_value` - A value between 0.0 and 1.0 that determines the
  interpolation position between the base color and the end color. A
  value of 0.0 will return the base color, while 1.0 will return the end
  color.

## [§](#returns){.doc-anchor}Returns

A new `RGBColor` instance representing the interpolated color.
:::
::::::
:::::::
