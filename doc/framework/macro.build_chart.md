::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](index.html)
:::

# Macro [build_chart]{.macro}Copy item path

[[Source](../src/optionstratlib/visualization/utils.rs.html#103-114){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
macro_rules! build_chart {
    ($root:expr, $title:expr, $title_size:expr, $min_x:expr, $max_x:expr, $min_y:expr, $max_y:expr) => { ... };
}
```

Expand description

::: docblock
Builds a chart with a title and specified axis ranges.

## [§](#arguments){.doc-anchor}Arguments

- `$root` - The drawing area to build the chart on.
- `$title` - The title of the chart.
- `$title_size` - The font size of the title.
- `$min_x` - The minimum value for the x-axis.
- `$max_x` - The maximum value for the x-axis.
- `$min_y` - The minimum value for the y-axis.
- `$max_y` - The maximum value for the y-axis.

## [§](#returns){.doc-anchor}Returns

A `ChartBuilder` object.

## [§](#errors){.doc-anchor}Errors

Returns an error if the chart cannot be built.
:::
::::::
:::::::
