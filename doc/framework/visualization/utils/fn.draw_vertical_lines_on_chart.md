::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)::[utils](index.html)
:::

# Function [draw_vertical_lines_on_chart]{.fn}Copy item path

[[Source](../../../src/optionstratlib/visualization/utils.rs.html#472-511){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn draw_vertical_lines_on_chart<DB: DrawingBackend, X, Y>(
    ctx: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    lines: &[ChartVerticalLine<X::ValueType, Y::ValueType>],
) -> Result<(), Box<dyn Error>>where
    X: Ranged,
    Y: Ranged,
    X::ValueType: Clone + Add<f64, Output = X::ValueType>,
    Y::ValueType: Clone + Add<f64, Output = Y::ValueType>,
    <X as Ranged>::ValueType: 'static + Display,
    <Y as Ranged>::ValueType: 'static + Display,
    <DB as DrawingBackend>::ErrorType: 'static,
```

Expand description

::: docblock
Draws vertical lines with labels on a given chart using the specified
drawing backend.

This function renders a series of vertical lines on a chart, given their
positions, styles, and associated labels. It utilizes a `ChartContext`
for rendering the lines and the Plotters crate utilities for styling and
layout. Each line is drawn between a specified range on the y-axis and
features an optional label placed at a specific offset.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `DB`: The type representing the drawing backend, which must implement
  the `DrawingBackend` trait. This defines how the chart elements are
  rendered (e.g., as an image, on a canvas, etc.).
- `X`: The type representing the x-axis of the chart. It must implement
  the `Ranged` trait to support scaling and interpolation.
- `Y`: The type representing the y-axis of the chart. Similar to `X`, it
  must implement the `Ranged` trait for compatibility.

## [§](#function-parameters){.doc-anchor}Function Parameters

- `ctx`: A mutable reference to a `ChartContext`, which handles the
  drawing and layout of the chart elements. It is parameterized with the
  drawing backend `DB` and coordinate system `Cartesian2d<X, Y>`.
- `lines`: A slice of `ChartVerticalLine` structures defining the
  x-coordinate, y-range, style, and label for each vertical line to be
  drawn.

## [§](#returns){.doc-anchor}Returns

- Returns a `Result`:
  - `Ok(())` on success, indicating that all vertical lines were drawn
    without errors.
  - `Err(Box<dyn Error>)` if an error occurs during the drawing
    operations.

## [§](#constraints){.doc-anchor}Constraints

- The `X` and `Y` types, as well as their associated value types
  (`X::ValueType` and `Y::ValueType`), must support cloning (`Clone`)
  and addition (`Add<f64>`). This enables the function to compute
  positions and offsets for labels.
- Value types for `X` and `Y` must be displayable (`std::fmt::Display`)
  to render labels correctly on the chart.
- Drawing backend errors must be composable as `'static` to integrate
  seamlessly with the function's return type.

## [§](#behavior){.doc-anchor}Behavior

1.  **Line Drawing**: For each vertical line in the input slice, a line
    is drawn from the bottom to the top of the specified y-range using
    `LineSeries`.
2.  **Label Placement**: For each line, a `Text` entity displaying the
    label is rendered at the specified offset relative to the
    x-coordinate and the upper y-coordinate of the line.
3.  Styling: Uses attributes from `ChartVerticalLine` (`line_style`,
    `font_size`, and colors) to apply custom styles to the lines and
    labels.
:::
::::::
:::::::
