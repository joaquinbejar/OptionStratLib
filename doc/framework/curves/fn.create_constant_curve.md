::::::: width-limiter
:::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[curves](index.html)
:::

# Function [create_constant_curve]{.fn}Copy item path

[[Source](../../src/optionstratlib/curves/utils.rs.html#145-159){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_constant_curve(
    start: Decimal,
    end: Decimal,
    value: Decimal,
) -> Curve
```

Expand description

::: docblock
Creates a constant curve with equidistant points along the x-axis and
the same constant value for the y-axis.

This function generates a simple mathematical curve defined over a fixed
range of x-values with an equal spacing between points, where each
y-coordinate is set to a constant value specified by the `value`
parameter. The curve is represented as a collection of `Point2D` points,
which are then used to create a `Curve` object.

## [§](#parameters){.doc-anchor}Parameters

- `start`: The starting x-coordinate for the curve, represented as a
  `Decimal`.
- `end`: The ending x-coordinate for the curve, represented as a
  `Decimal`.
- `value`: The constant y-coordinate value applied to all points in the
  curve, represented as a `Decimal`.

## [§](#returns){.doc-anchor}Returns

A `Curve` instance that represents the constant curve. The returned
curve consists of equidistant `Point2D` points between the `start` and
`end` x-coordinates, all having the same y-coordinate defined by
`value`.

## [§](#behavior){.doc-anchor}Behavior

- The function divides the range `[start, end]` into a fixed number of
  equally spaced steps.
- The x-coordinate of each point is calculated based on this step size.
- The `value` is used as the y-coordinate for all points.
- A `Curve` is created using the generated `Point2D` points via the
  `Curve::from_vector` method.

## [§](#details){.doc-anchor}Details

- Internally, this function assumes 10 steps (`steps = 10`) for dividing
  the x-range. This creates 11 points including both the `start` and
  `end` x-coordinates.
- The calculation of intermediate x-coordinates uses a constant
  `step_size`, computed as `(end - start) / steps`.
- The function ensures that both the `start` and `end` values are
  included in the resulting curve.

## [§](#example){.doc-anchor}Example

While this is designed to remain usage-agnostic, in practice, it results
in a horizontal line in Cartesian space that is constant in the
y-dimension and spans the x-range.

## [§](#panics){.doc-anchor}Panics

- The function will panic if `steps` is set to zero or if the provided
  `start` and `end` values result in invalid arithmetic operations, such
  as division by zero or overflow of Decimal values.

## [§](#see-also){.doc-anchor}See Also

- [`Point2D::new`](struct.Point2D.html#method.new "associated function optionstratlib::curves::Point2D::new"):
  Used to create individual points in the resulting curve.
- [`Curve::from_vector`](struct.Curve.html#method.from_vector "associated function optionstratlib::curves::Curve::from_vector"):
  Used internally to convert the set of constant points into a `Curve`
  object.
:::
::::::
:::::::
