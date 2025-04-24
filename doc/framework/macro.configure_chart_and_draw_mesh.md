::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [configure_chart_and_draw_mesh]{.macro}Copy item path

[[Source](../src/optionstratlib/visualization/utils.rs.html#130-144){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! configure_chart_and_draw_mesh {
    ($chart:expr, $x_labels:expr, $y_labels:expr, $min_x:expr, $max_x:expr) => { ... };
}
```

Expand description

::: docblock
Configures the chart mesh, labels, and draws a horizontal line at y = 0.

## [§](#arguments){.doc-anchor}Arguments

- `$chart` - The chart to configure.
- `$x_labels` - The number of labels for the x-axis.
- `$y_labels` - The number of labels for the y-axis.
- `$min_x` - The minimum value for the x-axis.
- `$max_x` - The maximum value for the x-axis.

## [§](#errors){.doc-anchor}Errors

Returns an error if the chart cannot be configured or the line cannot be
drawn.
:::
::::::
:::::::
