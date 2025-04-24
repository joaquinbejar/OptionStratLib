::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [draw_line_segments]{.macro}Copy item path

[[Source](../src/optionstratlib/visualization/utils.rs.html#160-178){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! draw_line_segments {
    ($chart:expr, $x_axis_data:expr, $y_axis_data:expr, $dark_green:expr, $dark_red:expr) => { ... };
}
```

Expand description

::: docblock
Draws line segments on the chart based on provided data.

## [§](#arguments){.doc-anchor}Arguments

- `$chart` - The chart to draw on.
- `$x_axis_data` - The data for the x-axis.
- `$y_axis_data` - The data for the y-axis.
- `$dark_green` - The color to use for positive values.
- `$dark_red` - The color to use for negative values.

## [§](#errors){.doc-anchor}Errors

Returns an error if the line segments cannot be drawn.
:::
::::::
:::::::
