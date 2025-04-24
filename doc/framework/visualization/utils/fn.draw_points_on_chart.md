::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../../index.html)::[visualization](../index.html)::[utils](index.html)
:::

# Function [draw_points_on_chart]{.fn}Copy item path

[[Source](../../../src/optionstratlib/visualization/utils.rs.html#386-421){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn draw_points_on_chart<DB: DrawingBackend, X, Y>(
    ctx: &mut ChartContext<'_, DB, Cartesian2d<X, Y>>,
    points: &[ChartPoint<(X::ValueType, Y::ValueType)>],
) -> Result<(), Box<dyn Error>>where
    X: Ranged,
    Y: Ranged,
    X::ValueType: Clone + Add<f64, Output = X::ValueType> + 'static,
    Y::ValueType: Clone + Add<f64, Output = Y::ValueType> + 'static,
    (X::ValueType, Y::ValueType): Clone + Into<(X::ValueType, Y::ValueType)>,
    DB::ErrorType: 'static,
```

Expand description

::: docblock
Draws chart points and their associated labels on a chart context.

This function is responsible for rendering a list of chart points onto a
given chart context. Each point is represented as a circle, styled with
a specific size and color, and labeled with text positioned based on a
defined offset.

## [§](#type-parameters){.doc-anchor}Type Parameters

- `DB`: The backend responsible for rendering the chart, implementing
  the `DrawingBackend` trait.
- `X`: The type representing the horizontal (x-axis) range of the chart,
  implementing the `Ranged` trait.
- `Y`: The type representing the vertical (y-axis) range of the chart,
  also implementing the `Ranged` trait.

## [§](#arguments){.doc-anchor}Arguments

- `ctx`: A mutable reference to a `ChartContext` object, which provides
  the necessary context for drawing on the chart.
- `points`: A slice of `ChartPoint` objects, each representing a point
  to render on the chart, including its coordinates, styling, and label
  information.

## [§](#returns){.doc-anchor}Returns

Returns `Ok(())` if all points and their labels are successfully drawn.
If an error occurs during the rendering process (e.g., backend issues),
a boxed `Error` is returned.

## [§](#constraints){.doc-anchor}Constraints

- The value type of `X` and `Y` must:
  - Be clonable.
  - Support addition with a `f64` value (`Add<f64>`).
- `X::ValueType` and `Y::ValueType` must additionally be compatible with
  `Into<(X::ValueType, Y::ValueType)>`.
- The error type of the drawing backend (`DB::ErrorType`) must be
  `'static`.

## [§](#implementation-details){.doc-anchor}Implementation Details

- For each point in the `points` slice:

  1.  A circle is drawn according to the point's coordinates, size, and
      color.
  2.  A textual label is placed near the point, with its position
      influenced by the specified `label_offset`.

- Uses the helper method `LabelOffsetType::get_offset` to determine the
  offset values for positioning the labels.
:::
::::::
:::::::
