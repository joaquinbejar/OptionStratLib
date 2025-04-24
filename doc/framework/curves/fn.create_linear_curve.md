::::::::: width-limiter
:::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[curves](index.html)
:::

# Function [create_linear_curve]{.fn}Copy item path

[[Source](../../src/optionstratlib/curves/utils.rs.html#92-105){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub fn create_linear_curve(
    start: Decimal,
    end: Decimal,
    slope: Decimal,
) -> Curve
```

Expand description

::::: docblock
Creates a linear curve defined by a starting point, an ending point, and
a slope.

This function generates a 2-dimensional curve by calculating evenly
spaced points (10 intervals by default) between the `start` and `end`
x-coordinates. For each generated x-coordinate, the corresponding
y-coordinate is computed using the provided slope, following the
equation:

::: example-wrap
``` language-text
y = slope * x
```
:::

The generated points are then used to construct a `Curve` instance.

## [§](#parameters){.doc-anchor}Parameters

- `start`: The starting x-coordinate of the curve (as a `Decimal`).
- `end`: The ending x-coordinate of the curve (as a `Decimal`).
  - Must be greater than the `start` value for the function to work as
    intended.
- `slope`: The slope of the linear curve, which determines the
  relationship between x and y values.

## [§](#returns){.doc-anchor}Returns

A `Curve` instance containing evenly spaced points along the linear
curve determined by the specified parameters.

## [§](#behavior){.doc-anchor}Behavior

- The x-coordinates are computed as evenly spaced values between `start`
  and `end` across 10 steps. Each x-coordinate includes its
  corresponding `y` value determined by the slope.
- Internally uses `Point2D::new` to construct points based on the
  computed x- and y-coordinate values.
- Constructs the final curve using `Curve::from_vector`, with the
  computed points forming the curve.

## [§](#constraints){.doc-anchor}Constraints

- The `end` value must be greater than the `start` value; otherwise, the
  generated points will result in an incorrect or potentially invalid
  curve.
- The function uses a fixed number (10) of steps to divide the range
  between `start` and `end`. This ensures uniform spacing between points
  but limits flexibility for other resolutions.

## [§](#example-workflow-internal-overview){.doc-anchor}Example Workflow (Internal Overview)

1.  Divide the range `[start, end]` into 10 equal steps (`step_size`).
2.  Iteratively compute `(x, y)` points using the formula
    `y = slope * x`.
3.  Accumulate these points into a `Vec<Point2D>`.
4.  Construct the final `Curve` using `Curve::from_vector`.

## [§](#usage-notes){.doc-anchor}Usage Notes

- This function is best suited for applications requiring a simple
  linear curve representation between two bounds.
- For higher resolution or adaptive step generation, consider modifying
  the function or implementing a similar utility.

## [§](#panics){.doc-anchor}Panics

This function will panic if the calculated `step_size` results in a
division by zero, which could occur if `end` is equal to `start`. The
caller should ensure that `end` is greater than `start` to avoid this
scenario.

## [§](#see-also){.doc-anchor}See Also

- [`Point2D::new`](struct.Point2D.html#method.new "associated function optionstratlib::curves::Point2D::new"):
  Utility used to construct individual points for the curve.
- [`Curve::from_vector`](struct.Curve.html#method.from_vector "associated function optionstratlib::curves::Curve::from_vector"):
  Used to generate the resulting curve from the constructed points.

## [§](#example-high-level-usage-concept){.doc-anchor}Example (High-Level Usage Concept)

While examples are omitted as requested, the general idea is to pass
desired values for `start`, `end`, and `slope` into this function in a
practical implementation scenario.

::: example-wrap
``` {.rust .rust-example-rendered}
use rust_decimal::Decimal;
use optionstratlib::curves::create_linear_curve;
let curve = create_linear_curve(
    Decimal::new(0, 1),   // start = 0.0
    Decimal::new(100, 1), // end = 10.0
    Decimal::new(1, 0)    // slope = 1.0
);
```
:::

would result in a curve defined by the points: `(0.0, 0.0)`,
`(1.0, 1.0)`, ..., `(10.0, 10.0)`.

From the above, it demonstrates how linearly spaced and
:::::
::::::::
:::::::::
