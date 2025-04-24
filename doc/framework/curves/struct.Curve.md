:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: width-limiter
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#main-content .section .content}
:::: main-heading
::: rustdoc-breadcrumbs
[optionstratlib](../index.html)::[curves](index.html)
:::

# Struct [Curve]{.struct}Copy item path

[[Source](../../src/optionstratlib/curves/curve.rs.html#62-71){.src}
]{.sub-heading}
::::

``` {.rust .item-decl}
pub struct Curve {
    pub points: BTreeSet<Point2D>,
    pub x_range: (Decimal, Decimal),
}
```

Expand description

::: docblock
Represents a mathematical curve as a collection of 2D points.

## [§](#overview){.doc-anchor}Overview

The `Curve` struct is a fundamental representation of a curve, defined
as a series of points in a two-dimensional Cartesian coordinate system.
Each curve is associated with an `x_range`, specifying the inclusive
domain of the curve in terms of its x-coordinates.

This structure supports precise mathematical and computational
operations, including interpolation, analysis, transformations, and
intersections. The use of `Decimal` for coordinates ensures
high-precision calculations, making it particularly suitable for
scientific, financial, or mathematical applications.

## [§](#usage){.doc-anchor}Usage

The `Curve` struct acts as the basis for high-level operations provided
within the `crate::curves` module. These include (but are not limited
to):

- Generating statistical analyses (`CurveAnalysisResult`)
- Performing curve interpolation
- Logical manipulations, such as merging curves (`MergeOperation`)
- Visualizing graphs or curve plots using libraries like `plotters`

## [§](#example-applications){.doc-anchor}Example Applications

The `Curve` type fits into mathematical or graphical operations such as:

- Modeling data over a range of x-values
- Comparing curves through transformations or intersections
- Calculating derivatives, integrals, and extrema along the curve

## [§](#constraints){.doc-anchor}Constraints

- All points in the `points` vector must lie within the specified
  `x_range`.
- Methods working with `Curve` data will assume that the `points` vector
  is ordered by the `x`-coordinate. Non-ordered inputs may lead to
  undefined behavior in specific operations.

## [§](#see-also){.doc-anchor}See Also

- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The fundamental data type for representing points in 2D space.
- [`MergeOperation`](../geometrics/enum.MergeOperation.html "enum optionstratlib::geometrics::MergeOperation"):
  Enum for combining multiple curves.
:::

## Fields[§](#fields){.anchor} {#fields .fields .section-header}

[[§](#structfield.points){.anchor
.field}`points: `[`BTreeSet`](https://doc.rust-lang.org/1.86.0/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}`<`[`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}`>`]{#structfield.points
.structfield .section-header}

::: docblock
A ordered set of `Point2D` objects that defines the curve in terms of
its x-y plane coordinates. Points are stored in a `BTreeSet` which
automatically maintains them in sorted order by their x-coordinate.
:::

[[§](#structfield.x_range){.anchor
.field}`x_range: (Decimal, Decimal)`]{#structfield.x_range .structfield
.section-header}

::: docblock
A tuple `(min_x, max_x)` that specifies the minimum and maximum
x-coordinate values for the curve. Operations performed on the curve
should ensure they fall within this range. Both values are of type
`Decimal` to ensure high precision in boundary calculations.
:::

## Implementations[§](#implementations){.anchor} {#implementations .section-header}

::::::: {#implementations-list}
::: {#impl-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#91-122){.src
.rightside}[§](#impl-Curve){.anchor}

### impl [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-curve .code-header}
:::

::::: impl-items
::: {#method.new .section .method}
[Source](../../src/optionstratlib/curves/curve.rs.html#118-121){.src
.rightside}

#### pub fn [new](#method.new){.fn}(points: [BTreeSet](https://doc.rust-lang.org/1.86.0/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\>) -\> Self {#pub-fn-newpoints-btreesetpoint2d---self .code-header}
:::

::: docblock
Creates a new curve from a vector of points.

This constructor initializes a `Curve` instance using a list of 2D
points provided as a `BTreeSet<Point2D>`. Additionally, the x-range of
the curve is calculated and stored. The x-range is determined by
evaluating the minimum and maximum x-coordinates among the provided
points.

##### [§](#parameters){.doc-anchor}Parameters

- `points` (`BTreeSet<Point2D>`): A vector of points that define the
  curve in a two-dimensional Cartesian coordinate plane.

##### [§](#returns){.doc-anchor}Returns

- `Curve`: A newly instantiated curve containing the provided points and
  the computed x-range.

##### [§](#behavior){.doc-anchor}Behavior

- Calculates the x-range of the points using `calculate_range()`.
- Stores the provided points for later use in curve-related
  calculations.

##### [§](#see-also-1){.doc-anchor}See Also

- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The type of points used to define the curve.
- [`crate::curves::Curve::calculate_range`](struct.Curve.html#method.calculate_range "associated function optionstratlib::curves::Curve::calculate_range"):
  Computes the x-range of a set of points.
:::
:::::
:::::::

## Trait Implementations[§](#trait-implementations){.anchor} {#trait-implementations .section-header}

::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#trait-implementations-list}
:::: {#impl-Arithmetic%3CCurve%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1254-1453){.src
.rightside}[§](#impl-Arithmetic%3CCurve%3E-for-Curve){.anchor}

### impl [Arithmetic](../geometrics/trait.Arithmetic.html "trait optionstratlib::geometrics::Arithmetic"){.trait}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-arithmeticcurve-for-curve .code-header}

::: docblock
Implements the `CurveArithmetic` trait for the `Curve` type, providing
functionality for merging multiple curves using a specified mathematical
operation and performing arithmetic operations between two curves.
:::
::::

::::::::: impl-items
::: {#method.merge .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1316-1418){.src
.rightside}[§](#method.merge){.anchor}

#### fn [merge](../geometrics/trait.Arithmetic.html#tymethod.merge){.fn}( curves: &\[&[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\], operation: [MergeOperation](../geometrics/enum.MergeOperation.html "enum optionstratlib::geometrics::MergeOperation"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-merge-curves-curve-operation-mergeoperation---resultcurve-curveerror .code-header}
:::

::: docblock
Merges a collection of curves into a single curve based on the specified
mathematical operation.

##### [§](#parameters-8){.doc-anchor}Parameters {#parameters-8}

- `curves` (`&[&Curve]`): A slice of references to the curves to be
  merged. Each curve must have defined x-ranges and interpolation
  capabilities.
- `operation` (`MergeOperation`): The arithmetic operation to perform on
  the interpolated y-values for the provided curves. Operations include
  addition, subtraction, multiplication, division, and aggregation
  (e.g., max, min).

##### [§](#returns-8){.doc-anchor}Returns {#returns-8}

- `Ok(Curve)`: Returns a new curve resulting from the merging operation.
- `Err(CurvesError)`: If input parameters are invalid or the merge
  operation encounters an error (e.g., incompatible x-ranges or
  interpolation failure), an error is returned.

##### [§](#behavior-3){.doc-anchor}Behavior {#behavior-3}

1.  **Parameter Validation**:

- Verifies that at least one curve is provided in the `curves`
  parameter.
- Returns an error if no curves are included or if x-ranges are
  incompatible.

2.  **Cloning Single Curve**:

- If only one curve is provided, its clone is returned as the result
  without performing any further calculations.

3.  **Range Computation**:

- Computes the intersection of x-ranges across the curves by finding the
  maximum lower bound (`min_x`) and minimum upper bound (`max_x`).
- If the x-range intersection is invalid (i.e., `min_x >= max_x`), an
  error is returned.

4.  **Interpolation and Arithmetic**:

- Divides the x-range into `steps` equally spaced intervals (default:
  100).
- Interpolates the y-values for all curves at each x-value in the range.
- Applies the specified `operation` to the aggregated y-values at each
  x-point.

5.  **Parallel Processing**:

- Uses parallel iteration to perform interpolation and value combination
  efficiently, leveraging the Rayon library.

6.  **Error Handling**:

- Any errors during interpolation or arithmetic operations are
  propagated back to the caller.

##### [§](#errors-6){.doc-anchor}Errors {#errors-6}

- **Invalid Parameter** (`CurvesError`): Returned when no curves are
  provided or x-ranges are incompatible.
- **Interpolation Failure** (`CurvesError`): Raised if interpolation
  fails for a specific curve or x-value.

##### [§](#example-use-case-1){.doc-anchor}Example Use Case {#example-use-case-1}

This function enables combining multiple curves for tasks such as:

- Summing y-values across different curves to compute a composite curve.
- Finding the maximum/minimum y-value at each x-point for a collection
  of curves.
:::

::: {#method.merge_with .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1450-1452){.src
.rightside}[§](#method.merge_with){.anchor}

#### fn [merge_with](../geometrics/trait.Arithmetic.html#tymethod.merge_with){.fn}( &self, other: &[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, operation: [MergeOperation](../geometrics/enum.MergeOperation.html "enum optionstratlib::geometrics::MergeOperation"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-merge_with-self-other-curve-operation-mergeoperation---resultcurve-curveerror .code-header}
:::

::: docblock
Combines the current `Curve` instance with another curve using a
mathematical operation, resulting in a new curve.

##### [§](#parameters-9){.doc-anchor}Parameters {#parameters-9}

- `self` (`&Self`): A reference to the current curve instance.
- `other` (`&Curve`): A reference to the second curve for the arithmetic
  operation.
- `operation` (`MergeOperation`): The operation to apply when merging
  the curves.

##### [§](#returns-9){.doc-anchor}Returns {#returns-9}

- `Ok(Curve)`: Returns a new curve that represents the result of the
  operation.
- `Err(CurvesError)`: If the merge operation fails (e.g., incompatible
  x-ranges or interpolation errors), an error is returned.

##### [§](#behavior-4){.doc-anchor}Behavior {#behavior-4}

This function is a convenience wrapper around `merge_curves` that
operates specifically on two curves. It passes `self` and `other` as an
array to `merge_curves` and applies the desired operation.

##### [§](#errors-7){.doc-anchor}Errors {#errors-7}

- Inherits all errors returned by the `merge_curves` method, including
  parameter validation and interpolation errors.

##### [§](#examples){.doc-anchor}Examples

Use this method to easily perform arithmetic operations between two
curves, such as summing their y-values or finding their pointwise
maximum.
:::

::: {#associatedtype.Error-1 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1255){.src
.rightside}[§](#associatedtype.Error-1){.anchor}

#### type [Error](../geometrics/trait.Arithmetic.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror .code-header}
:::

::: docblock
The error type returned when merging operations fail
:::
:::::::::

::: {#impl-AxisOperations%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1455-1494){.src
.rightside}[§](#impl-AxisOperations%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [AxisOperations](../geometrics/trait.AxisOperations.html "trait optionstratlib::geometrics::AxisOperations"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-axisoperationspoint2d-decimal-for-curve .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-2 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1456){.src
.rightside}[§](#associatedtype.Error-2){.anchor}

#### type [Error](../geometrics/trait.AxisOperations.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-1 .code-header}
:::

::: docblock
The type of error that can occur during point operations
:::

::: {#method.contains_point .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1458-1460){.src
.rightside}[§](#method.contains_point){.anchor}

#### fn [contains_point](../geometrics/trait.AxisOperations.html#tymethod.contains_point){.fn}(&self, x: &Decimal) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-contains_pointself-x-decimal---bool .code-header}
:::

::: docblock
Checks if a coordinate value exists in the structure. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.contains_point)
:::

::: {#method.get_index_values .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1462-1464){.src
.rightside}[§](#method.get_index_values){.anchor}

#### fn [get_index_values](../geometrics/trait.AxisOperations.html#tymethod.get_index_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\> {#fn-get_index_valuesself---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to all index values in the structure.
[Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_index_values)
:::

::: {#method.get_values .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1466-1472){.src
.rightside}[§](#method.get_values){.anchor}

#### fn [get_values](../geometrics/trait.AxisOperations.html#tymethod.get_values){.fn}(&self, x: Decimal) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\> {#fn-get_valuesself-x-decimal---vecdecimal .code-header}
:::

::: docblock
Returns a vector of references to dependent values for a given
coordinate. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_values)
:::

::: {#method.get_closest_point .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1474-1485){.src
.rightside}[§](#method.get_closest_point){.anchor}

#### fn [get_closest_point](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point){.fn}(&self, x: &Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-get_closest_pointself-x-decimal---resultpoint2d-selferror .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_closest_point)
:::

::: {#method.get_point .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1487-1493){.src
.rightside}[§](#method.get_point){.anchor}

#### fn [get_point](../geometrics/trait.AxisOperations.html#tymethod.get_point){.fn}(&self, x: &Decimal) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#fn-get_pointself-x-decimal---optionpoint2d .code-header}
:::

::: docblock
Finds the closest point to the given coordinate value. [Read
more](../geometrics/trait.AxisOperations.html#tymethod.get_point)
:::

::: {#method.merge_indexes .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#85-115){.src
.rightside}[§](#method.merge_indexes){.anchor}

#### fn [merge_indexes](../geometrics/trait.AxisOperations.html#method.merge_indexes){.fn}(&self, axis: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\>) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_indexesself-axis-vecinput---vecinput .code-header}
:::

::: docblock
Merges the index values from the current structure with an additional
set of indices. This combines self.get_index_values() with the provided
axis vector to create a single vector of unique indices. [Read
more](../geometrics/trait.AxisOperations.html#method.merge_indexes)
:::
:::::::::::::::::

:::: {#impl-BiLinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#446-522){.src
.rightside}[§](#impl-BiLinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [BiLinearInterpolation](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-bilinearinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implementation of the `BiLinearInterpolation` trait for the `Curve`
struct.
:::
::::

::: docblock
This function performs bilinear interpolation, which is used to estimate
the value of a function at a given point `x` within a grid defined by at
least 4 points. Bilinear interpolation combines two linear
interpolations: one along the x-axis and another along the y-axis,
within the bounds defined by the four surrounding points.

#### [§](#parameters-3){.doc-anchor}Parameters

- **`x`**: The x-coordinate value for which the interpolation will be
  performed. Must fall within the range of the x-coordinates of the
  curve's points.

#### [§](#returns-3){.doc-anchor}Returns

- **`Ok(Point2D)`**: A `Point2D` instance representing the interpolated
  point at the given x-coordinate, with both x and y provided as
  `Decimal` values.
- **`Err(CurvesError)`**: An error if the interpolation cannot be
  performed due to one of the following reasons:
  - There are fewer than 4 points in the curve.
  - The x-coordinate does not fall within a valid range of points.
  - The bracketing points for the given x-coordinate cannot be
    determined.

#### [§](#function-details){.doc-anchor}Function Details

1.  **Input Validation**:

    - Ensures the curve has at least 4 points, as required for bilinear
      interpolation.
    - Returns an error if the condition is not met.

2.  **Exact Match Check**:

    - If the x-coordinate matches exactly with one of the points in the
      curve, the corresponding `Point2D` is returned directly.

3.  **Bracket Point Search**:

    - Determines the bracketing points (`i` and `j`) for the given
      x-coordinate using the `find_bracket_points` method.

4.  **Grid Point Selection**:

    - Extracts four points from the curve:
      - `p11`: Bottom-left point.
      - `p12`: Bottom-right point.
      - `p21`: Top-left point.
      - `p22`: Top-right point.

5.  **x-Normalization**:

    - Computes a normalized x value (`dx` in the range `[0,1]`), used to
      perform interpolation along the x-axis within the defined grid.

6.  **Linear Interpolation**:

    - First performs interpolation along the x-axis for the bottom and
      top edges of the grid, resulting in partial y-values `bottom` and
      `top`.
    - Then, interpolates along the y-axis between the bottom and top
      edge values, resulting in the final interpolated y-coordinate.

7.  **Output**:

    - Returns the interpolated `Point2D` with the input x-coordinate and
      the computed y-coordinate.

#### [§](#errors-2){.doc-anchor}Errors

- **Insufficient Points**: If the curve contains fewer than 4 points, a
  `CurvesError` with a relevant message is returned.
- **Out-of-Range x**: If the x-coordinate cannot be bracketed by points
  in the curve, a `CurvesError` is returned with an appropriate message.

#### [§](#related-traits){.doc-anchor}Related Traits

- [`BiLinearInterpolation`](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation"):
  The trait defining this method.
- [`Interpolate`](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"):
  Ensures compatibility of the curve with multiple interpolation
  methods.

#### [§](#see-also-4){.doc-anchor}See Also {#see-also-4}

- [`Curve`](struct.Curve.html "struct optionstratlib::curves::Curve"):
  The overarching structure that represents the curve.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The data type used to represent individual points on the curve.
- [`find_bracket_points`](../geometrics/trait.Interpolate.html#method.find_bracket_points "method optionstratlib::geometrics::Interpolate::find_bracket_points"):
  A helper method used to locate the two points that bracket the given
  x-coordinate.
:::

::::: impl-items
::: {#method.bilinear_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#486-521){.src
.rightside}[§](#method.bilinear_interpolate){.anchor}

#### fn [bilinear_interpolate](../geometrics/trait.BiLinearInterpolation.html#tymethod.bilinear_interpolate){.fn}( &self, x: Decimal, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-bilinear_interpolate-self-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

::: docblock
Performs bilinear interpolation to find the value of the curve at a
given `x` coordinate.

##### [§](#parameters-2){.doc-anchor}Parameters {#parameters-2}

- `x`: The x-coordinate at which the interpolation is to be performed.
  This should be a `Decimal` value within the range of the curve's known
  points.

##### [§](#returns-2){.doc-anchor}Returns {#returns-2}

- On success, returns a `Point2D` instance representing the interpolated
  point at the given `x` value.
- On failure, returns a `CurvesError`:
  - `CurvesError::InterpolationError`: If there are fewer than four
    points available for interpolation or if the required conditions for
    interpolation are not met.

##### [§](#function-description){.doc-anchor}Function Description

- The function retrieves the set of points defining the curve using
  `self.get_points()`.
- If fewer than four points exist, the function immediately fails with
  an `InterpolationError`.
- If the exact `x` value is found in the point set, its corresponding
  `Point2D` is returned directly.
- Otherwise, it determines the bracketing points (two pairs of points
  forming a square grid) necessary for bilinear interpolation using
  `self.find_bracket_points()`.
- From the bracketing points, it computes:
  - `dx`: A normalized value representing the relative position of `x`
    between its bracketing x-coordinates in the `[`0,1`]` interval.
  - `bottom`: The interpolated y-value along the bottom edge of the
    grid.
  - `top`: The interpolated y-value along the top edge of the grid.
  - `y`: The final interpolated value along the y-dimension from
    `bottom` to `top`.
- Returns the final interpolated point as `Point2D(x, y)`.

##### [§](#errors-1){.doc-anchor}Errors {#errors-1}

- Returns an error if the curve has fewer than four points, as bilinear
  interpolation requires at least four.
- Returns an error from `self.find_bracket_points()` if `x` cannot be
  bracketed.

##### [§](#notes){.doc-anchor}Notes

- The input `x` should be within the bounds of the curve for
  interpolation to succeed, as specified by the bracketing function.
- This function assumes that the points provided by `get_points` are
  sorted by ascending x-coordinate.

##### [§](#example-use-case){.doc-anchor}Example Use Case {#example-use-case}

This method is useful for calculating intermediate values on a 2D grid
when exact measurements are unavailable. Bilinear interpolation is
particularly applicable for approximating smoother values in a tabular
dataset or a regularly sampled grid.
:::
:::::

::: {#impl-Clone-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#impl-Clone-for-Curve){.anchor}

### impl [Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-clone-for-curve .code-header}
:::

::::::: impl-items
::: {#method.clone .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#method.clone){.anchor}

#### fn [clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone){.fn}(&self) -\> [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#fn-cloneself---curve .code-header}
:::

::: docblock
Returns a copy of the value. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#tymethod.clone)
:::

::: {#method.clone_from .section .method .trait-impl}
[[1.0.0]{.since title="Stable since Rust version 1.0.0"} ·
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#174){.src}]{.rightside}[§](#method.clone_from){.anchor}

#### fn [clone_from](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from){.fn}(&mut self, source: &Self) {#fn-clone_frommut-self-source-self .code-header}
:::

::: docblock
Performs copy-assignment from `source`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html#method.clone_from)
:::
:::::::

:::: {#impl-CubicInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#583-699){.src
.rightside}[§](#impl-CubicInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [CubicInterpolation](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-cubicinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `CubicInterpolation` trait for the `Curve` struct,
providing an algorithm for cubic interpolation utilizing a Catmull-Rom
spline.
:::
::::

::: docblock
#### [§](#method-cubic_interpolate){.doc-anchor}Method: `cubic_interpolate`

##### [§](#parameters-5){.doc-anchor}Parameters

- **`x`**: The x-value at which the interpolation is performed. This
  value must be within the range of x-values in the curve's defined
  points, and it is passed as a `Decimal` to allow for high-precision
  computation.

##### [§](#returns-5){.doc-anchor}Returns

- **`Ok(Point2D)`**: Returns a `Point2D` representing the interpolated x
  and y values.
- **`Err(CurvesError)`**: Returns an error if:
  - There are fewer than 4 points available for interpolation.
  - The x-value is outside the curve's range, or interpolation fails for
    any other reason.

##### [§](#behavior-2){.doc-anchor}Behavior {#behavior-2}

1.  **Point Validation**: Ensures at least four points exist for cubic
    interpolation, as this is a fundamental requirement for computing
    the Catmull-Rom spline.
2.  **Exact Match Check**: If the x-value matches an existing point in
    the curve, the method directly returns the corresponding `Point2D`
    without further computation.
3.  **Bracket Points**: Determines the bracketing points (4 points
    total) around the provided x-value. Depending on the position of the
    x-value in the curve, the method dynamically adjusts the selected
    points to ensure they form a proper bracket:
    - If near the start of the curve, uses the first four points.
    - If near the end, uses the last four points.
    - Else, selects points before and after x to define the bracket.
4.  **Parameter Calculation**: Computes a normalized parameter `t` that
    represents the relative position of the target x-value between `p1`
    and `p2`.
5.  **Catmull-Rom Spline**: Performs cubic interpolation using a
    Catmull-Rom spline, a widely used, smooth spline algorithm. The
    coefficients are calculated based on the relative x position and the
    y-values of the four surrounding points.
6.  **Interpolation**: Calculates the interpolated y-value using the
    cubic formula:

    ::: example-wrap
    ``` language-text
    y(t) = 0.5 * (
        2 * p1.y + (-p0.y + p2.y) * t
        + (2 * p0.y - 5 * p1.y + 4 * p2.y - p3.y) * t^2
        + (-p0.y + 3 * p1.y - 3 * p2.y + p3.y) * t^3
    )
    ```
    :::

    Here, `t` is the normalized x position, and `p0`, `p1`, `p2`, `p3`
    are the four bracketed points.

##### [§](#errors-3){.doc-anchor}Errors {#errors-3}

- Returns an error of type `CurvesError::InterpolationError` if any
  issues are encountered, such as insufficient points or the inability
  to locate bracket points.

##### [§](#example-2){.doc-anchor}Example {#example-2}

This method is part of the `Curve` struct, which defines a set of points
and supports interpolation. It is often used in applications requiring
smooth manifolds or animations.

##### [§](#notes-1){.doc-anchor}Notes

- The computed y-value ensures smooth transitions and continuity between
  interpolated segments.
- Catmull-Rom splines are particularly effective for creating visually
  smooth transitions, making this method suitable for curves,
  animations, and numerical analysis.

#### [§](#see-also-5){.doc-anchor}See Also {#see-also-5}

- [`CubicInterpolation`](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation"):
  The trait defining this method.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents the points used for interpolation.
- [`find_bracket_points`](../geometrics/trait.Interpolate.html#method.find_bracket_points "method optionstratlib::geometrics::Interpolate::find_bracket_points"):
  Determines the bracketing points required for interpolation.
:::

::::: impl-items
::: {#method.cubic_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#651-698){.src
.rightside}[§](#method.cubic_interpolate){.anchor}

#### fn [cubic_interpolate](../geometrics/trait.CubicInterpolation.html#tymethod.cubic_interpolate){.fn}(&self, x: Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-cubic_interpolateself-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

::: docblock
Performs cubic interpolation on a set of points to estimate the
y-coordinate for a given x value using a Catmull-Rom spline.

##### [§](#parameters-4){.doc-anchor}Parameters {#parameters-4}

- `x`: The x-coordinate for which the interpolation is performed. This
  value should lie within the range of the points on the curve.

##### [§](#returns-4){.doc-anchor}Returns {#returns-4}

- `Ok(Point2D)`: A `Point2D` instance representing the interpolated
  position `(x, y)`, where `y` is estimated using cubic interpolation.
- `Err(CurvesError)`: An error indicating issues with the interpolation
  process, such as insufficient points or an out-of-range x value.

##### [§](#requirements){.doc-anchor}Requirements

- The number of points in the curve must be at least 4, as cubic
  interpolation requires four points for accurate calculations.
- The specified `x` value should be inside the range defined by the
  curve's points.
- If the specified x matches an existing point on the curve, the
  interpolated result directly returns that exact point.

##### [§](#functionality){.doc-anchor}Functionality

This method performs cubic interpolation using the general properties of
the Catmull-Rom spline, which is well-suited for smooth curve fitting.
It operates as follows:

1.  **Exact Point Check**: If the x value matches an existing point, the
    method returns that point without further processing.

2.  **Bracketing Points Selection**:

    - Searches for two points that bracket the given x value (using
      `find_bracket_points` from the `Interpolate` trait). The method
      ensures that there are always enough points before and after the
      target x value to perform cubic interpolation.

3.  **Point Selection for Interpolation**:

    - Depending on the position of the target x value, four points
      (`p0, p1, p2, p3`) are selected:
      - When `x` is near the start of the points, select the first four.
      - When `x` is near the end, select the last four.
      - Otherwise, select the two points just before and after the x
        value and include an additional adjacent point on either side.

4.  **Parameter Calculation**:

    - The `t` parameter is derived, representing the normalized position
      of x between `p1` and `p2`.

5.  **Cubic Interpolation**:

    - The interpolated y-coordinate is computed using the Catmull-Rom
      spline formula, leveraging the `t`-value and the y-coordinates of
      the four selected points.

##### [§](#error-handling){.doc-anchor}Error Handling

This method returns an error in the following circumstances:

- If fewer than 4 points are available, it returns a
  `CurvesError::InterpolationError` with a corresponding message.
- If the bracketing points cannot be identified (e.g., when `x` is
  outside the range of points), the appropriate interpolation error is
  propagated.

##### [§](#example-1){.doc-anchor}Example

- Interpolating smoothly along a curve defined by a set of points,
  avoiding sharp transitions between segments.

- Provides a high degree of precision due to the use of the `Decimal`
  type for `x` and `y` calculations.
:::
:::::

::: {#impl-Debug-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#impl-Debug-for-Curve){.anchor}

### impl [Debug](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html "trait core::fmt::Debug"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-debug-for-curve .code-header}
:::

::::: impl-items
::: {#method.fmt .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#method.fmt){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Debug.html#tymethod.fmt)
:::
:::::

::: {#impl-Default-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#82-89){.src
.rightside}[§](#impl-Default-for-Curve){.anchor}

### impl [Default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html "trait core::default::Default"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-default-for-curve .code-header}
:::

::::: impl-items
::: {#method.default .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#83-88){.src
.rightside}[§](#method.default){.anchor}

#### fn [default](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default){.fn}() -\> Self {#fn-default---self .code-header}
:::

::: docblock
Returns the "default value" for a type. [Read
more](https://doc.rust-lang.org/1.86.0/core/default/trait.Default.html#tymethod.default)
:::
:::::

::: {#impl-Deserialize%3C'de%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#impl-Deserialize%3C'de%3E-for-Curve){.anchor}

### impl\<\'de\> [Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#implde-deserializede-for-curve .code-header}
:::

:::::: impl-items
:::: {#method.deserialize .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#method.deserialize){.anchor}

#### fn [deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize){.fn}\<\_\_D\>(\_\_deserializer: \_\_D) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, \_\_D::[Error](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html#associatedtype.Error "type serde::de::Deserializer::Error"){.associatedtype}\> {#fn-deserialize__d__deserializer-__d---resultself-__derror .code-header}

::: where
where \_\_D:
[Deserializer](https://docs.rs/serde/1.0.219/serde/de/trait.Deserializer.html "trait serde::de::Deserializer"){.trait}\<\'de\>,
:::
::::

::: docblock
Deserialize this value from the given Serde deserializer. [Read
more](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html#tymethod.deserialize)
:::
::::::

::: {#impl-Display-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#73-80){.src
.rightside}[§](#impl-Display-for-Curve){.anchor}

### impl [Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-display-for-curve .code-header}
:::

::::: impl-items
::: {#method.fmt-1 .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#74-79){.src
.rightside}[§](#method.fmt-1){.anchor}

#### fn [fmt](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt){.fn}(&self, f: &mut [Formatter](https://doc.rust-lang.org/1.86.0/core/fmt/struct.Formatter.html "struct core::fmt::Formatter"){.struct}\<\'\_\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/fmt/type.Result.html "type core::fmt::Result"){.type} {#fn-fmtself-f-mut-formatter_---result-1 .code-header}
:::

::: docblock
Formats the value using the given formatter. [Read
more](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html#tymethod.fmt)
:::
:::::

::: {#impl-GeometricObject%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#134-193){.src
.rightside}[§](#impl-GeometricObject%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [GeometricObject](../geometrics/trait.GeometricObject.html "trait optionstratlib::geometrics::GeometricObject"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-geometricobjectpoint2d-decimal-for-curve .code-header}
:::

:::::::::::::::::::: impl-items
::: {#associatedtype.Error .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#135){.src
.rightside}[§](#associatedtype.Error){.anchor}

#### type [Error](../geometrics/trait.GeometricObject.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-2 .code-header}
:::

::: docblock
Type alias for any errors that might occur during the construction of
the geometric object.
:::

::: {#method.get_points .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#137-139){.src
.rightside}[§](#method.get_points){.anchor}

#### fn [get_points](../geometrics/trait.GeometricObject.html#tymethod.get_points){.fn}(&self) -\> [BTreeSet](https://doc.rust-lang.org/1.86.0/alloc/collections/btree/set/struct.BTreeSet.html "struct alloc::collections::btree::set::BTreeSet"){.struct}\<&[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> {#fn-get_pointsself---btreesetpoint2d .code-header}
:::

::: docblock
Returns a `BTreeSet` containing references to the points that constitute
the geometric object. The `BTreeSet` ensures that the points are ordered
and unique.
:::

:::: {#method.from_vector .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#141-149){.src
.rightside}[§](#method.from_vector){.anchor}

#### fn [from_vector](../geometrics/trait.GeometricObject.html#tymethod.from_vector){.fn}\<T\>(points: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<T\>) -\> Self {#fn-from_vectortpoints-vect---self .code-header}

::: where
where T:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> +
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::: docblock
Creates a new geometric object from a `Vec` of points. [Read
more](../geometrics/trait.GeometricObject.html#tymethod.from_vector)
:::

:::: {#method.construct .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#151-192){.src
.rightside}[§](#method.construct){.anchor}

#### fn [construct](../geometrics/trait.GeometricObject.html#tymethod.construct){.fn}\<T\>(method: T) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricObject.html#associatedtype.Error "type optionstratlib::geometrics::GeometricObject::Error"){.associatedtype}\> {#fn-constructtmethod-t---resultself-selferror .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
T:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<[ConstructionMethod](../geometrics/enum.ConstructionMethod.html "enum optionstratlib::geometrics::ConstructionMethod"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct},
Decimal\>\>,
:::
::::

::: docblock
Constructs a geometric object using a specific construction method.
[Read more](../geometrics/trait.GeometricObject.html#tymethod.construct)
:::

::: {#method.vector .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#22-24){.src
.rightside}[§](#method.vector){.anchor}

#### fn [vector](../geometrics/trait.GeometricObject.html#method.vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}\> {#fn-vectorself---vecpoint .code-header}
:::

::: docblock
Returns a `Vec` containing references to the points that constitute the
geometric object. This method simply converts the `BTreeSet` from
`get_points` into a `Vec`.
:::

::: {#method.to_vector .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#50-52){.src
.rightside}[§](#method.to_vector){.anchor}

#### fn [to_vector](../geometrics/trait.GeometricObject.html#method.to_vector){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[&Point](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}\> {#fn-to_vectorself---vecpoint .code-header}
:::

::: docblock
Returns the points of the geometric object as a `Vec` of references.
Equivalent to calling the `vector()` method.
:::

:::: {#method.calculate_range .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/utils.rs.html#57-64){.src
.rightside}[§](#method.calculate_range){.anchor}

#### fn [calculate_range](../geometrics/trait.GeometricObject.html#method.calculate_range){.fn}\<I\>(iter: I) -\> (Decimal, Decimal) {#fn-calculate_rangeiiter-i---decimal-decimal .code-header}

::: where
where I:
[Iterator](https://doc.rust-lang.org/1.86.0/core/iter/traits/iterator/trait.Iterator.html "trait core::iter::traits::iterator::Iterator"){.trait}\<Item
= Decimal\>,
:::
::::

::: docblock
Calculates the minimum and maximum decimal values from an iterator of
decimals. [Read
more](../geometrics/trait.GeometricObject.html#method.calculate_range)
:::
::::::::::::::::::::

::: {#impl-GeometricTransformations%3CPoint2D%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1536-1645){.src
.rightside}[§](#impl-GeometricTransformations%3CPoint2D%3E-for-Curve){.anchor}

### impl [GeometricTransformations](../geometrics/trait.GeometricTransformations.html "trait optionstratlib::geometrics::GeometricTransformations"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-geometrictransformationspoint2d-for-curve .code-header}
:::

::::::::::::::::: impl-items
::: {#associatedtype.Error-3 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1537){.src
.rightside}[§](#associatedtype.Error-3){.anchor}

#### type [Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-3 .code-header}
:::

::: docblock
The error type that can be returned by geometric operations.
:::

::: {#method.translate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1539-1554){.src
.rightside}[§](#method.translate){.anchor}

#### fn [translate](../geometrics/trait.GeometricTransformations.html#tymethod.translate){.fn}(&self, deltas: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-translateself-deltas-vecdecimal---resultself-selferror .code-header}
:::

::: docblock
Translates the geometric object by specified amounts along each
dimension. [Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.translate)
:::

::: {#method.scale .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1556-1571){.src
.rightside}[§](#method.scale){.anchor}

#### fn [scale](../geometrics/trait.GeometricTransformations.html#tymethod.scale){.fn}(&self, factors: [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<&Decimal\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Self, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-scaleself-factors-vecdecimal---resultself-selferror .code-header}
:::

::: docblock
Scales the geometric object by specified factors along each dimension.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.scale)
:::

::: {#method.intersect_with .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1573-1589){.src
.rightside}[§](#method.intersect_with){.anchor}

#### fn [intersect_with](../geometrics/trait.GeometricTransformations.html#tymethod.intersect_with){.fn}(&self, other: &Self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}\>, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-intersect_withself-other-self---resultvecpoint2d-selferror .code-header}
:::

::: docblock
Finds all intersection points between this geometric object and another.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.intersect_with)
:::

::: {#method.derivative_at .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1591-1601){.src
.rightside}[§](#method.derivative_at){.anchor}

#### fn [derivative_at](../geometrics/trait.GeometricTransformations.html#tymethod.derivative_at){.fn}(&self, point: &[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\>, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-derivative_atself-point-point2d---resultvecdecimal-selferror .code-header}
:::

::: docblock
Calculates the derivative at a specific point on the geometric object.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.derivative_at)
:::

::: {#method.extrema .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1603-1626){.src
.rightside}[§](#method.extrema){.anchor}

#### fn [extrema](../geometrics/trait.GeometricTransformations.html#tymethod.extrema){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}), Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-extremaself---resultpoint2d-point2d-selferror .code-header}
:::

::: docblock
Finds the extrema (minimum and maximum points) of the geometric object.
[Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.extrema)
:::

::: {#method.measure_under .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1628-1644){.src
.rightside}[§](#method.measure_under){.anchor}

#### fn [measure_under](../geometrics/trait.GeometricTransformations.html#tymethod.measure_under){.fn}(&self, base_value: &Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Decimal, Self::[Error](../geometrics/trait.GeometricTransformations.html#associatedtype.Error "type optionstratlib::geometrics::GeometricTransformations::Error"){.associatedtype}\> {#fn-measure_underself-base_value-decimal---resultdecimal-selferror .code-header}
:::

::: docblock
Calculates the area or volume under the geometric object relative to a
base value. [Read
more](../geometrics/trait.GeometricTransformations.html#tymethod.measure_under)
:::
:::::::::::::::::

:::: {#impl-Index%3Cusize%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#249-258){.src
.rightside}[§](#impl-Index%3Cusize%3E-for-Curve){.anchor}

### impl [Index](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html "trait core::ops::index::Index"){.trait}\<[usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-indexusize-for-curve .code-header}

::: docblock
Allows indexed access to the points in a `Curve` using `usize` indices.
:::
::::

:::: docblock
#### [§](#overview-1){.doc-anchor}Overview

This implementation provides intuitive, array-like access to the points
within a `Curve`. By using the `Index<usize>` trait, users can directly
reference specific points by their index within the internal `points`
collection without manually iterating or managing indices themselves.

#### [§](#behavior-1){.doc-anchor}Behavior {#behavior-1}

- The `index` method fetches the `Point2D` at the specified position in
  the order of the curve's `points` (sorted by the `Point2D` ordering,
  typically based on the `x` values).
- If the specified index exceeds the range of available points, it
  triggers a panic with the message `"Index out of bounds"`.

#### [§](#constraints-1){.doc-anchor}Constraints

- The index must be a valid value between `0` and
  `self.points.len() - 1`.
- The `Curve`'s `points` are internally stored as a `BTreeSet<Point2D>`,
  so indexing reflects the natural order of the set, which is determined
  by the `Ord` trait implementation for `Point2D`.

#### [§](#fields-accessed){.doc-anchor}Fields Accessed

- **`points`**: A `BTreeSet` of `Point2D` structs representing the
  curve's 2D points.

#### [§](#panics){.doc-anchor}Panics

This implementation will panic if:

- The index provided is out of bounds (less than `0` or greater
  than/equal to the number of points in the curve).

#### [§](#use-cases){.doc-anchor}Use Cases

- Quickly accessing specific points on a curve during visualization,
  interpolation, or analysis operations.
- Performing operations that require stepwise access to points, such as
  slicing or filtering points along the curve.

#### [§](#example){.doc-anchor}Example {#example}

Suppose you have a `Curve` instance `curve` with multiple points:

::: {.example-wrap .ignore}
[ⓘ](# "This example is not tested"){.tooltip}

``` {.rust .rust-example-rendered}
let point = curve[0]; // Access the first point
```
:::

#### [§](#important-notes){.doc-anchor}Important Notes

- This indexing implementation provides read-only access
  (`&Self::Output`).
- Modifying the `points` collection or its contents directly is not
  allowed through this implementation, ensuring immutability when using
  indexed access.

#### [§](#type-associations){.doc-anchor}Type Associations

- **Input**:
  - The input type for the `Index` operation is `usize`, the standard
    for indexing.
- **Output**:
  - The output type for the `Index` operation is a reference to
    `Point2D`, specifically `&Point2D`.

#### [§](#key-implementations){.doc-anchor}Key Implementations

- **`Index<usize>`**: Provides indexing-based access to curve points.
::::

::::::: impl-items
::: {#method.index .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#255-257){.src
.rightside}[§](#method.index){.anchor}

#### fn [index](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#tymethod.index){.fn}(&self, index: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> &Self::[Output](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#associatedtype.Output "type core::ops::index::Index::Output"){.associatedtype} {#fn-indexself-index-usize---selfoutput .code-header}
:::

::: docblock
Fetches the `Point2D` at the specified index.

Panics if the index is invalid.
:::

::: {#associatedtype.Output .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#250){.src
.rightside}[§](#associatedtype.Output){.anchor}

#### type [Output](https://doc.rust-lang.org/1.86.0/core/ops/index/trait.Index.html#associatedtype.Output){.associatedtype} = [Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct} {#type-output-point2d .code-header}
:::

::: docblock
The returned type after indexing.
:::
:::::::

:::: {#impl-Interpolate%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#287){.src
.rightside}[§](#impl-Interpolate%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [Interpolate](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-interpolatepoint2d-decimal-for-curve .code-header}

::: docblock
Implementation of the `Interpolate` trait for the `Curve` struct.
:::
::::

::: docblock
This implementation integrates the `get_points` method for the `Curve`
structure, providing access to its internal points. The `Interpolate`
trait ensures compatibility with various interpolation methods such as
Linear, BiLinear, Cubic, and Spline interpolations. By implementing this
trait, `Curve` gains the ability to perform interpolation operations and
access bracketing points.

#### [§](#traits-involved){.doc-anchor}Traits Involved

The `Interpolate` trait is an aggregation of multiple
interpolation-related traits:

- [`LinearInterpolation`](../geometrics/trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation")
- [`BiLinearInterpolation`](../geometrics/trait.BiLinearInterpolation.html "trait optionstratlib::geometrics::BiLinearInterpolation")
- [`CubicInterpolation`](../geometrics/trait.CubicInterpolation.html "trait optionstratlib::geometrics::CubicInterpolation")
- [`SplineInterpolation`](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation")

These underlying traits implement specific interpolation algorithms,
enabling `Curve` to support a robust set of interpolation options
through the associated methods. Depending on the use case and provided
parameters (e.g., interpolation type and target x-coordinate), the
appropriate algorithm is invoked.

#### [§](#see-also-2){.doc-anchor}See Also {#see-also-2}

- [`Curve`](struct.Curve.html "struct optionstratlib::curves::Curve"):
  The underlying mathematical structure being interpolated.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  The fundamental data type for the curve's points.
- [`Interpolate`](../geometrics/trait.Interpolate.html "trait optionstratlib::geometrics::Interpolate"):
  The trait defining interpolation operations.
:::

::::::: impl-items
::: {#method.interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#80-91){.src
.rightside}[§](#method.interpolate){.anchor}

#### fn [interpolate](../geometrics/trait.Interpolate.html#method.interpolate){.fn}( &self, x: Input, interpolation_type: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<Point, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-interpolate-self-x-input-interpolation_type-interpolationtype---resultpoint-interpolationerror .code-header}
:::

::: docblock
Interpolates a value at the given x-coordinate using the specified
interpolation method. [Read
more](../geometrics/trait.Interpolate.html#method.interpolate)
:::

::: {#method.find_bracket_points .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/interpolation/traits.rs.html#110-132){.src
.rightside}[§](#method.find_bracket_points){.anchor}

#### fn [find_bracket_points](../geometrics/trait.Interpolate.html#method.find_bracket_points){.fn}( &self, x: Input, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<([usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}), [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-find_bracket_points-self-x-input---resultusize-usize-interpolationerror .code-header}
:::

::: docblock
Finds the indices of points that bracket the given x-coordinate. [Read
more](../geometrics/trait.Interpolate.html#method.find_bracket_points)
:::
:::::::

::: {#impl-Len-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#124-132){.src
.rightside}[§](#impl-Len-for-Curve){.anchor}

### impl [Len](../utils/trait.Len.html "trait optionstratlib::utils::Len"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-len-for-curve .code-header}
:::

::::::: impl-items
::: {#method.len .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#125-127){.src
.rightside}[§](#method.len){.anchor}

#### fn [len](../utils/trait.Len.html#tymethod.len){.fn}(&self) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#fn-lenself---usize .code-header}
:::

::: docblock
Returns the number of elements in the collection or the size of the
object. [Read more](../utils/trait.Len.html#tymethod.len)
:::

::: {#method.is_empty .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#129-131){.src
.rightside}[§](#method.is_empty){.anchor}

#### fn [is_empty](../utils/trait.Len.html#method.is_empty){.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_emptyself---bool .code-header}
:::

::: docblock
Returns `true` if the collection contains no elements or the object has
zero size. [Read more](../utils/trait.Len.html#method.is_empty)
:::
:::::::

:::: {#impl-LinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#352-370){.src
.rightside}[§](#impl-LinearInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [LinearInterpolation](../geometrics/trait.LinearInterpolation.html "trait optionstratlib::geometrics::LinearInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-linearinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `LinearInterpolation` trait for the `Curve` struct.
:::
::::

::::: docblock
This implementation provides linear interpolation functionality for a
given set of points on a curve. The interpolation computes the `y` value
corresponding to a given `x` value using the linear interpolation
formula. The method ensures that the input `x` is within the range of
the curve's defined points.

::: example-wrap
``` language-text
y = p1.y + (x - p1.x) * (p2.y - p1.y) / (p2.x - p1.x)
```
:::

#### [§](#parameters-1){.doc-anchor}Parameters {#parameters-1}

- `x`: A `Decimal` representing the `x`-coordinate for which the
  corresponding interpolated `y` value is to be computed.

#### [§](#returns-1){.doc-anchor}Returns {#returns-1}

- `Ok(Point2D)`: A `Point2D` instance containing the input `x` value and
  the interpolated `y` value.
- `Err(CurvesError)`: Returns an error of type
  `CurvesError::InterpolationError` in any of the following cases:
  - The curve does not have enough points for interpolation.
  - The provided `x` value is outside the range of the curve's points.
  - Bracketing points for `x` cannot be found.

#### [§](#working-mechanism){.doc-anchor}Working Mechanism

1.  The method calls `find_bracket_points` (implemented in the
    `Interpolate` trait) to locate the index pair `(i, j)` of two points
    that bracket the `x` value.
2.  From the located points `p1` and `p2`, the method calculates the
    interpolated `y` value using the linear interpolation formula.
3.  Finally, a `Point2D` is created and returned with the provided `x`
    and the computed `y` value.

#### [§](#implementation-details){.doc-anchor}Implementation Details

- The function leverages `Decimal` arithmetic for high precision in
  calculations.
- It assumes that the provided points on the curve are sorted in
  ascending order based on their `x` values.

#### [§](#errors){.doc-anchor}Errors {#errors}

This method returns a `CurvesError` in the following cases:

- **Insufficient Points**: When the curve has fewer than two points.
- **Out-of-Range `x`**: When the input `x` value lies outside the range
  of the defined points.
- **No Bracketing Points Found**: When the method fails to find two
  points that bracket the given `x`.

#### [§](#example-how-it-works-internally){.doc-anchor}Example (How it works internally)

Suppose the curve is defined by the following points:

- `p1 = (2.0, 4.0)`
- `p2 = (5.0, 10.0)`

Given `x = 3.0`, the method computes:

::: example-wrap
``` language-text
y = 4.0 + (3.0 - 2.0) * (10.0 - 4.0) / (5.0 - 2.0)
  = 4 + 1 * 6 / 3
  = 4 + 2
  = 6.0
```
:::

It will return `Point2D { x: 3.0, y: 6.0 }`.

#### [§](#see-also-3){.doc-anchor}See Also {#see-also-3}

- `find_bracket_points`: Finds two points that bracket a value.
- `Point2D`: Represents points in 2D space.
- `CurvesError`: Represents errors related to curve operations.
:::::

::::: impl-items
::: {#method.linear_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#359-369){.src
.rightside}[§](#method.linear_interpolate){.anchor}

#### fn [linear_interpolate](../geometrics/trait.LinearInterpolation.html#tymethod.linear_interpolate){.fn}(&self, x: Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-linear_interpolateself-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

::: docblock
##### [§](#method){.doc-anchor}Method

###### [§](#linear_interpolate){.doc-anchor}`linear_interpolate`

Performs linear interpolation for a given `x` value by finding two
consecutive points on the curve (`p1` and `p2`) that bracket the
provided `x`. The `y` value is then calculated using the linear
interpolation formula:
:::
:::::

:::: {#impl-MergeAxisInterpolate%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1496-1534){.src
.rightside}[§](#impl-MergeAxisInterpolate%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [MergeAxisInterpolate](../geometrics/trait.MergeAxisInterpolate.html "trait optionstratlib::geometrics::MergeAxisInterpolate"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-mergeaxisinterpolatepoint2d-decimal-for-curve .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::::: impl-items
::: {#method.merge_axis_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1500-1533){.src
.rightside}[§](#method.merge_axis_interpolate){.anchor}

#### fn [merge_axis_interpolate](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate){.fn}( &self, other: &Self, interpolation: [InterpolationType](../geometrics/enum.InterpolationType.html "enum optionstratlib::geometrics::InterpolationType"){.enum}, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<(Self, Self), Self::[Error](../geometrics/trait.AxisOperations.html#associatedtype.Error "type optionstratlib::geometrics::AxisOperations::Error"){.associatedtype}\> {#fn-merge_axis_interpolate-self-other-self-interpolation-interpolationtype---resultself-self-selferror .code-header}
:::

::: docblock
Interpolates both structures to align them on a common set of index
values. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#tymethod.merge_axis_interpolate)
:::

::: {#method.merge_axis_index .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/operations/axis.rs.html#144-147){.src
.rightside}[§](#method.merge_axis_index){.anchor}

#### fn [merge_axis_index](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index){.fn}\<\'a\>(&\'a self, other: &\'a Self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Input\> {#fn-merge_axis_indexaa-self-other-a-self---vecinput .code-header}
:::

::: docblock
Merges the index values from two structures into a single ordered
vector. [Read
more](../geometrics/trait.MergeAxisInterpolate.html#method.merge_axis_index)
:::
:::::::

:::: {#impl-MetricsExtractor-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#956-1249){.src
.rightside}[§](#impl-MetricsExtractor-for-Curve){.anchor}

### impl [MetricsExtractor](../geometrics/trait.MetricsExtractor.html "trait optionstratlib::geometrics::MetricsExtractor"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-metricsextractor-for-curve .code-header}

::: docblock
A default implementation for the `Curve` type using a provided default
strategy.
:::
::::

::: docblock
This implementation provides a basic approach to computing curve metrics
by using interpolation and statistical methods available in the standard
curve analysis library.

#### [§](#note){.doc-anchor}Note

This is a minimal implementation that may need to be customized or
enhanced based on specific requirements or domain-specific analysis
needs.
:::

::::::::::::::::: impl-items
::: {#method.compute_basic_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#957-1010){.src
.rightside}[§](#method.compute_basic_metrics){.anchor}

#### fn [compute_basic_metrics](../geometrics/trait.MetricsExtractor.html#tymethod.compute_basic_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_basic_metricsself---resultbasicmetrics-metricserror .code-header}
:::

::: docblock
Computes basic statistical metrics for the curve. [Read
more](../geometrics/trait.MetricsExtractor.html#tymethod.compute_basic_metrics)
:::

::: {#method.compute_shape_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1012-1066){.src
.rightside}[§](#method.compute_shape_metrics){.anchor}

#### fn [compute_shape_metrics](../geometrics/trait.MetricsExtractor.html#tymethod.compute_shape_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[ShapeMetrics](../geometrics/struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_shape_metricsself---resultshapemetrics-metricserror .code-header}
:::

::: docblock
Computes shape-related metrics for the curve. [Read
more](../geometrics/trait.MetricsExtractor.html#tymethod.compute_shape_metrics)
:::

::: {#method.compute_range_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1068-1113){.src
.rightside}[§](#method.compute_range_metrics){.anchor}

#### fn [compute_range_metrics](../geometrics/trait.MetricsExtractor.html#tymethod.compute_range_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RangeMetrics](../geometrics/struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_range_metricsself---resultrangemetrics-metricserror .code-header}
:::

::: docblock
Computes range-related metrics for the curve. [Read
more](../geometrics/trait.MetricsExtractor.html#tymethod.compute_range_metrics)
:::

::: {#method.compute_trend_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1115-1188){.src
.rightside}[§](#method.compute_trend_metrics){.anchor}

#### fn [compute_trend_metrics](../geometrics/trait.MetricsExtractor.html#tymethod.compute_trend_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[TrendMetrics](../geometrics/struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_trend_metricsself---resulttrendmetrics-metricserror .code-header}
:::

::: docblock
Computes trend-related metrics for the curve. [Read
more](../geometrics/trait.MetricsExtractor.html#tymethod.compute_trend_metrics)
:::

::: {#method.compute_risk_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#1190-1248){.src
.rightside}[§](#method.compute_risk_metrics){.anchor}

#### fn [compute_risk_metrics](../geometrics/trait.MetricsExtractor.html#tymethod.compute_risk_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[RiskMetrics](../geometrics/struct.RiskMetrics.html "struct optionstratlib::geometrics::RiskMetrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_risk_metricsself---resultriskmetrics-metricserror .code-header}
:::

::: docblock
Computes risk-related metrics for the curve. [Read
more](../geometrics/trait.MetricsExtractor.html#tymethod.compute_risk_metrics)
:::

::: {#method.compute_curve_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#71-79){.src
.rightside}[§](#method.compute_curve_metrics){.anchor}

#### fn [compute_curve_metrics](../geometrics/trait.MetricsExtractor.html#method.compute_curve_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Metrics](../geometrics/struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_curve_metricsself---resultmetrics-metricserror .code-header}
:::

::: docblock
Computes and aggregates all curve metrics into a comprehensive
`CurveMetrics` struct. [Read
more](../geometrics/trait.MetricsExtractor.html#method.compute_curve_metrics)
:::

::: {#method.compute_surface_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/geometrics/analysis/traits.rs.html#104-111){.src
.rightside}[§](#method.compute_surface_metrics){.anchor}

#### fn [compute_surface_metrics](../geometrics/trait.MetricsExtractor.html#method.compute_surface_metrics){.fn}(&self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Metrics](../geometrics/struct.Metrics.html "struct optionstratlib::geometrics::Metrics"){.struct}, [MetricsError](../error/enum.MetricsError.html "enum optionstratlib::error::MetricsError"){.enum}\> {#fn-compute_surface_metricsself---resultmetrics-metricserror .code-header}
:::

::: docblock
Computes comprehensive metrics for a surface representation. [Read
more](../geometrics/trait.MetricsExtractor.html#method.compute_surface_metrics)
:::
:::::::::::::::::

:::: {#impl-PlotBuilderExt%3CCurve%3E-for-PlotBuilder%3CCurve%3E .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#153-233){.src
.rightside}[§](#impl-PlotBuilderExt%3CCurve%3E-for-PlotBuilder%3CCurve%3E){.anchor}

### impl [PlotBuilderExt](../geometrics/trait.PlotBuilderExt.html "trait optionstratlib::geometrics::PlotBuilderExt"){.trait}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> for [PlotBuilder](../geometrics/struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}\> {#impl-plotbuilderextcurve-for-plotbuildercurve .code-header}

::: docblock
Plotting implementation for single Curve
:::
::::

::::: impl-items
::: {#method.save .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#154-232){.src
.rightside}[§](#method.save){.anchor}

#### fn [save](../geometrics/trait.PlotBuilderExt.html#tymethod.save){.fn}(self, path: impl [AsRef](https://doc.rust-lang.org/1.86.0/core/convert/trait.AsRef.html "trait core::convert::AsRef"){.trait}\<[Path](https://doc.rust-lang.org/1.86.0/std/path/struct.Path.html "struct std::path::Path"){.struct}\>) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[()](https://doc.rust-lang.org/1.86.0/std/primitive.unit.html){.primitive}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-saveself-path-impl-asrefpath---result-curveerror .code-header}
:::

::: docblock
Saves the configured plot to a file at the specified path. [Read
more](../geometrics/trait.PlotBuilderExt.html#tymethod.save)
:::
:::::

:::: {#impl-Plottable-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#58-70){.src
.rightside}[§](#impl-Plottable-for-Curve){.anchor}

### impl [Plottable](../geometrics/trait.Plottable.html "trait optionstratlib::geometrics::Plottable"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-plottable-for-curve .code-header}

::: docblock
Plottable implementation for single Curve
:::
::::

:::::::: impl-items
::: {#associatedtype.Error-4 .section .associatedtype .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#59){.src
.rightside}[§](#associatedtype.Error-4){.anchor}

#### type [Error](../geometrics/trait.Plottable.html#associatedtype.Error){.associatedtype} = [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum} {#type-error-curveerror-4 .code-header}
:::

::: docblock
The error type returned by plotting operations. [Read
more](../geometrics/trait.Plottable.html#associatedtype.Error)
:::

:::: {#method.plot .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/visualization/plotters.rs.html#61-69){.src
.rightside}[§](#method.plot){.anchor}

#### fn [plot](../geometrics/trait.Plottable.html#tymethod.plot){.fn}(&self) -\> [PlotBuilder](../geometrics/struct.PlotBuilder.html "struct optionstratlib::geometrics::PlotBuilder"){.struct}\<Self\> {#fn-plotself---plotbuilderself .code-header}

::: where
where Self:
[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::: docblock
Creates a plot builder for configuring and generating visualizations.
[Read more](../geometrics/trait.Plottable.html#tymethod.plot)
:::
::::::::

::: {#impl-Serialize-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#impl-Serialize-for-Curve){.anchor}

### impl [Serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html "trait serde::ser::Serialize"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-serialize-for-curve .code-header}
:::

:::::: impl-items
:::: {#method.serialize .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#61){.src
.rightside}[§](#method.serialize){.anchor}

#### fn [serialize](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize){.fn}\<\_\_S\>(&self, \_\_serializer: \_\_S) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<\_\_S::[Ok](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Ok "type serde::ser::Serializer::Ok"){.associatedtype}, \_\_S::[Error](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html#associatedtype.Error "type serde::ser::Serializer::Error"){.associatedtype}\> {#fn-serialize__sself-__serializer-__s---result__sok-__serror .code-header}

::: where
where \_\_S:
[Serializer](https://docs.rs/serde/1.0.219/serde/ser/trait.Serializer.html "trait serde::ser::Serializer"){.trait},
:::
::::

::: docblock
Serialize this value into the given Serde serializer. [Read
more](https://docs.rs/serde/1.0.219/serde/ser/trait.Serialize.html#tymethod.serialize)
:::
::::::

:::: {#impl-SplineInterpolation%3CPoint2D,+Decimal%3E-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#757-939){.src
.rightside}[§](#impl-SplineInterpolation%3CPoint2D,+Decimal%3E-for-Curve){.anchor}

### impl [SplineInterpolation](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"){.trait}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, Decimal\> for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-splineinterpolationpoint2d-decimal-for-curve .code-header}

::: docblock
Implements the `SplineInterpolation` trait for the `Curve` struct,
providing functionality to perform cubic spline interpolation.
:::
::::

::: docblock
#### [§](#overview-2){.doc-anchor}Overview

This method calculates the interpolated `y` value for a given `x` value
by using cubic spline interpolation on the points in the `Curve`. The
method ensures a smooth transition between points by computing second
derivatives of the curve at each point, and uses those derivatives in
the spline interpolation formula.

#### [§](#parameters-7){.doc-anchor}Parameters {#parameters-7}

- `x`: The x-coordinate at which the curve should be interpolated. This
  value is passed as a `Decimal` for precise calculations.

#### [§](#returns-7){.doc-anchor}Returns {#returns-7}

- On success, returns a `Point2D` instance representing the interpolated
  point.
- On error, returns a `CurvesError` indicating the reason for failure
  (e.g., insufficient points or an out-of-range x-coordinate).

#### [§](#errors-5){.doc-anchor}Errors {#errors-5}

- Returns `CurvesError::InterpolationError` with an appropriate error
  message in the following cases:
  - If the curve contains fewer than three points, as spline
    interpolation requires at least three points.
  - If the given `x` value lies outside the range of x-coordinates
    spanned by the points in the curve.
  - If a valid segment for interpolation cannot be located.

#### [§](#details){.doc-anchor}Details

1.  **Validation**:
    - Ensures that there are at least three points in the curve for
      spline interpolation.
    - Validates that the provided `x` value is within the range of `x`
      values of the curve.
2.  **Exact Match**: If the `x` value matches the x-coordinate of an
    existing point, the corresponding `Point2D` is returned immediately.
3.  **Second Derivative Calculation**:
    - Uses a tridiagonal matrix to compute the second derivatives at
      each point. This step involves setting up the system of equations
      based on the boundary conditions (natural spline) and solving it
      using the Thomas algorithm.
4.  **Segment Identification**:
    - Determines the segment (interval between two consecutive points)
      in which the provided `x` value lies.
5.  **Interpolation**:
    - Computes the interpolated y-coordinate using the cubic spline
      formula, which is based on the second derivatives and the
      positions of the surrounding points.

#### [§](#implementation-notes){.doc-anchor}Implementation Notes

- This implementation uses `Decimal` from the `rust_decimal` crate to
  ensure high precision in calculations, making it suitable for
  scientific and financial applications.
- The Thomas algorithm is employed to solve the tridiagonal matrix
  system efficiently.
- The method assumes natural spline boundary conditions, where the
  second derivatives at the endpoints are set to zero, ensuring a smooth
  and continuous curve shape.

#### [§](#example-usage){.doc-anchor}Example Usage

Refer to the documentation for how to use the `SplineInterpolation`
trait, as examples are not provided inline with this implementation.

#### [§](#see-also-6){.doc-anchor}See Also

- [`SplineInterpolation`](../geometrics/trait.SplineInterpolation.html "trait optionstratlib::geometrics::SplineInterpolation"):
  The trait definition for spline interpolation.
- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents a point in 2D space.
- [`Curve`](struct.Curve.html "struct optionstratlib::curves::Curve"):
  Represents a mathematical curve made up of points for interpolation.
- [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"):
  Enumerates possible errors during curve operations.
:::

:::::: impl-items
::: {#method.spline_interpolate .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#853-938){.src
.rightside}[§](#method.spline_interpolate){.anchor}

#### fn [spline_interpolate](../geometrics/trait.SplineInterpolation.html#tymethod.spline_interpolate){.fn}(&self, x: Decimal) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Point2D](struct.Point2D.html "struct optionstratlib::curves::Point2D"){.struct}, [InterpolationError](../error/enum.InterpolationError.html "enum optionstratlib::error::InterpolationError"){.enum}\> {#fn-spline_interpolateself-x-decimal---resultpoint2d-interpolationerror .code-header}
:::

:::: docblock
Performs cubic spline interpolation for a given x-coordinate and returns
the interpolated `Point2D` value. This function computes the second
derivatives of the curve points, solves a tridiagonal system to derive
the interpolation parameters, and evaluates the spline function for the
provided `x` value.

##### [§](#parameters-6){.doc-anchor}Parameters {#parameters-6}

- `x`:
  - The x-coordinate at which the interpolation is to be performed.
  - Must be of type `Decimal`.

##### [§](#returns-6){.doc-anchor}Returns {#returns-6}

- `Ok(Point2D)`:

  - The `Point2D` instance representing the interpolated point at the
    given `x` value.
  - The interpolated `y` value is calculated based on the cubic spline
    interpolation algorithm.

- `Err(CurvesError)`:

  - Returned when an error occurs during the interpolation process, such
    as:
    - Insufficient points provided (less than 3 points).
    - The given `x` is outside the valid range of the points.
    - Unable to determine the correct segment for interpolation.

##### [§](#errors-4){.doc-anchor}Errors {#errors-4}

- `CurvesError::InterpolationError`:
  - Occurs under the following conditions:
    - **"Need at least three points for spline interpolation"**:
      Requires at least 3 points to perform cubic spline interpolation.
    - **"x is outside the range of points"**: The provided `x` value
      lies outside the domain of the curve points.
    - **"Could not find valid segment for interpolation"**: Spline
      interpolation fails due to an invalid segment determination.

##### [§](#pre-conditions){.doc-anchor}Pre-conditions

- The curve must contain at least three points for cubic spline
  interpolation.
- The `x` value must fall within the range of the curve's x-coordinates.

##### [§](#implementation-details-1){.doc-anchor}Implementation Details

- **Inputs**:
  - Uses the `get_points` method of the curve to retrieve the list of
    `Point2D` instances that define the interpolation curve.
  - Computes the second derivatives (`m`) for each point using the
    Thomas algorithm to solve a tridiagonal system.
- **Boundary Conditions**:
  - Natural spline boundary conditions are used, with the second
    derivatives on the boundary set to zero.
- **Interpolation**:
  - Determines the segment `[x_i, x_{i+1}]` to which the input `x`
    belongs.
  - Uses the cubic spline equation to calculate the interpolated `y`
    value.

##### [§](#mathematical-formulation){.doc-anchor}Mathematical Formulation

Let `x_i`, `x_{i+1}`, `y_i`, `y_{i+1}` refer to the points of the
segment where `x` lies. The cubic spline function at `x` is computed as
follows:

::: example-wrap
``` language-text
S(x) = m_i * (x_{i+1} - x)^3 / (6 * h)
     + m_{i+1} * (x - x_i)^3 / (6 * h)
     + (y_i / h - h * m_i / 6) * (x_{i+1} - x)
     + (y_{i+1} / h - h * m_{i+1} / 6) * (x - x_i)
```
:::

Where:

- `m_i`, `m_{i+1}` are the second derivatives at `x_i` and `x_{i+1}`.
- `h = x_{i+1} - x_i` is the distance between the two points.
- `(x_{i+1} - x)` and `(x - x_i)` are the relative distances within the
  segment.

##### [§](#example-usages-non-code){.doc-anchor}Example Usages (Non-code)

This method is typically used for high-precision curve fitting or
graphical rendering where smooth transitions between points are
essential. Common applications include:

- Signal processing.
- Data interpolation for missing values.
- Smooth graphical representations of mathematical functions.

##### [§](#related-types){.doc-anchor}Related Types

- [`Point2D`](struct.Point2D.html "struct optionstratlib::curves::Point2D"):
  Represents a 2D point and is used as input/output for this function.
- [`CurveError`](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError")
  Represents any error encountered during interpolation.

##### [§](#performance){.doc-anchor}Performance

- The function operates with `O(n)` complexity, where `n` is the number
  of points. The tridiagonal system is solved efficiently using the
  Thomas algorithm.

##### [§](#notes-2){.doc-anchor}Notes

- Natural spline interpolation may introduce minor deviations beyond the
  range of existing data points due to its boundary conditions. For
  strictly constrained results, consider alternative interpolation
  methods, such as linear or cubic Hermite interpolation.
::::
::::::

::: {#impl-StatisticalCurve-for-Curve .section .impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#941-945){.src
.rightside}[§](#impl-StatisticalCurve-for-Curve){.anchor}

### impl [StatisticalCurve](trait.StatisticalCurve.html "trait optionstratlib::curves::StatisticalCurve"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-statisticalcurve-for-curve .code-header}
:::

::::::::::: impl-items
::: {#method.get_x_values .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/curve.rs.html#942-944){.src
.rightside}[§](#method.get_x_values){.anchor}

#### fn [get_x_values](trait.StatisticalCurve.html#tymethod.get_x_values){.fn}(&self) -\> [Vec](https://doc.rust-lang.org/1.86.0/alloc/vec/struct.Vec.html "struct alloc::vec::Vec"){.struct}\<Decimal\> {#fn-get_x_valuesself---vecdecimal .code-header}
:::

::: docblock
Retrieves the x-axis values for the statistical curve. [Read
more](trait.StatisticalCurve.html#tymethod.get_x_values)
:::

::: {#method.generate_statistical_curve .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/traits.rs.html#113-223){.src
.rightside}[§](#method.generate_statistical_curve){.anchor}

#### fn [generate_statistical_curve](trait.StatisticalCurve.html#method.generate_statistical_curve){.fn}( &self, basic_metrics: &[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, shape_metrics: &[ShapeMetrics](../geometrics/struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, range_metrics: &[RangeMetrics](../geometrics/struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, trend_metrics: &[TrendMetrics](../geometrics/struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, num_points: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, seed: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.86.0/std/primitive.u64.html){.primitive}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-generate_statistical_curve-self-basic_metrics-basicmetrics-shape_metrics-shapemetrics-range_metrics-rangemetrics-trend_metrics-trendmetrics-num_points-usize-seed-optionu64---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a statistical curve with properties that match the provided
metrics. [Read
more](trait.StatisticalCurve.html#method.generate_statistical_curve)
:::

::: {#method.generate_refined_statistical_curve .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/traits.rs.html#245-286){.src
.rightside}[§](#method.generate_refined_statistical_curve){.anchor}

#### fn [generate_refined_statistical_curve](trait.StatisticalCurve.html#method.generate_refined_statistical_curve){.fn}( &self, basic_metrics: &[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, shape_metrics: &[ShapeMetrics](../geometrics/struct.ShapeMetrics.html "struct optionstratlib::geometrics::ShapeMetrics"){.struct}, range_metrics: &[RangeMetrics](../geometrics/struct.RangeMetrics.html "struct optionstratlib::geometrics::RangeMetrics"){.struct}, trend_metrics: &[TrendMetrics](../geometrics/struct.TrendMetrics.html "struct optionstratlib::geometrics::TrendMetrics"){.struct}, num_points: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, max_attempts: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}, tolerance: Decimal, seed: [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<[u64](https://doc.rust-lang.org/1.86.0/std/primitive.u64.html){.primitive}\>, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-generate_refined_statistical_curve-self-basic_metrics-basicmetrics-shape_metrics-shapemetrics-range_metrics-rangemetrics-trend_metrics-trendmetrics-num_points-usize-max_attempts-usize-tolerance-decimal-seed-optionu64---resultcurve-curveerror .code-header}
:::

::: docblock
Generates a refined statistical curve that iteratively adjusts to better
match the target metrics. [Read
more](trait.StatisticalCurve.html#method.generate_refined_statistical_curve)
:::

::: {#method.verify_curve_metrics .section .method .trait-impl}
[Source](../../src/optionstratlib/curves/traits.rs.html#298-313){.src
.rightside}[§](#method.verify_curve_metrics){.anchor}

#### fn [verify_curve_metrics](trait.StatisticalCurve.html#method.verify_curve_metrics){.fn}( &self, curve: &[Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct}, target_metrics: &[BasicMetrics](../geometrics/struct.BasicMetrics.html "struct optionstratlib::geometrics::BasicMetrics"){.struct}, tolerance: Decimal, ) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<[bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}, [CurveError](../error/curves/enum.CurveError.html "enum optionstratlib::error::curves::CurveError"){.enum}\> {#fn-verify_curve_metrics-self-curve-curve-target_metrics-basicmetrics-tolerance-decimal---resultbool-curveerror .code-header}
:::

::: docblock
Verifies if the metrics of the generated curve match the target metrics
within the specified tolerance. [Read
more](trait.StatisticalCurve.html#method.verify_curve_metrics)
:::
:::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::

## Auto Trait Implementations[§](#synthetic-implementations){.anchor} {#synthetic-implementations .section-header}

::::::::: {#synthetic-implementations-list}
::: {#impl-Freeze-for-Curve .section .impl}
[§](#impl-Freeze-for-Curve){.anchor}

### impl [Freeze](https://doc.rust-lang.org/1.86.0/core/marker/trait.Freeze.html "trait core::marker::Freeze"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-freeze-for-curve .code-header}
:::

::: {#impl-RefUnwindSafe-for-Curve .section .impl}
[§](#impl-RefUnwindSafe-for-Curve){.anchor}

### impl [RefUnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.RefUnwindSafe.html "trait core::panic::unwind_safe::RefUnwindSafe"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-refunwindsafe-for-curve .code-header}
:::

::: {#impl-Send-for-Curve .section .impl}
[§](#impl-Send-for-Curve){.anchor}

### impl [Send](https://doc.rust-lang.org/1.86.0/core/marker/trait.Send.html "trait core::marker::Send"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-send-for-curve .code-header}
:::

::: {#impl-Sync-for-Curve .section .impl}
[§](#impl-Sync-for-Curve){.anchor}

### impl [Sync](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sync.html "trait core::marker::Sync"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-sync-for-curve .code-header}
:::

::: {#impl-Unpin-for-Curve .section .impl}
[§](#impl-Unpin-for-Curve){.anchor}

### impl [Unpin](https://doc.rust-lang.org/1.86.0/core/marker/trait.Unpin.html "trait core::marker::Unpin"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-unpin-for-curve .code-header}
:::

::: {#impl-UnwindSafe-for-Curve .section .impl}
[§](#impl-UnwindSafe-for-Curve){.anchor}

### impl [UnwindSafe](https://doc.rust-lang.org/1.86.0/core/panic/unwind_safe/trait.UnwindSafe.html "trait core::panic::unwind_safe::UnwindSafe"){.trait} for [Curve](struct.Curve.html "struct optionstratlib::curves::Curve"){.struct} {#impl-unwindsafe-for-curve .code-header}
:::
:::::::::

## Blanket Implementations[§](#blanket-implementations){.anchor} {#blanket-implementations .section-header}

:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::: {#blanket-implementations-list}
:::: {#impl-Any-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#138){.src
.rightside}[§](#impl-Any-for-T){.anchor}

### impl\<T\> [Any](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html "trait core::any::Any"){.trait} for T {#implt-any-for-t .code-header}

::: where
where T: \'static +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.type_id .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/any.rs.html#139){.src
.rightside}[§](#method.type_id){.anchor}

#### fn [type_id](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id){.fn}(&self) -\> [TypeId](https://doc.rust-lang.org/1.86.0/core/any/struct.TypeId.html "struct core::any::TypeId"){.struct} {#fn-type_idself---typeid .code-header}
:::

::: docblock
Gets the `TypeId` of `self`. [Read
more](https://doc.rust-lang.org/1.86.0/core/any/trait.Any.html#tymethod.type_id)
:::
:::::

:::: {#impl-Borrow%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#209){.src
.rightside}[§](#impl-Borrow%3CT%3E-for-T){.anchor}

### impl\<T\> [Borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html "trait core::borrow::Borrow"){.trait}\<T\> for T {#implt-borrowt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#211){.src
.rightside}[§](#method.borrow){.anchor}

#### fn [borrow](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow){.fn}(&self) -\> [&T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrowself---t .code-header}
:::

::: docblock
Immutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.Borrow.html#tymethod.borrow)
:::
:::::

:::: {#impl-BorrowMut%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#217){.src
.rightside}[§](#impl-BorrowMut%3CT%3E-for-T){.anchor}

### impl\<T\> [BorrowMut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html "trait core::borrow::BorrowMut"){.trait}\<T\> for T {#implt-borrowmutt-for-t .code-header}

::: where
where T:
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.borrow_mut .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/borrow.rs.html#218){.src
.rightside}[§](#method.borrow_mut){.anchor}

#### fn [borrow_mut](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut){.fn}(&mut self) -\> [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#fn-borrow_mutmut-self---mut-t .code-header}
:::

::: docblock
Mutably borrows from an owned value. [Read
more](https://doc.rust-lang.org/1.86.0/core/borrow/trait.BorrowMut.html#tymethod.borrow_mut)
:::
:::::

:::: {#impl-CloneToUninit-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#273){.src
.rightside}[§](#impl-CloneToUninit-for-T){.anchor}

### impl\<T\> [CloneToUninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html "trait core::clone::CloneToUninit"){.trait} for T {#implt-clonetouninit-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

:::::: impl-items
::: {#method.clone_to_uninit .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/clone.rs.html#275){.src
.rightside}[§](#method.clone_to_uninit){.anchor}

#### unsafe fn [clone_to_uninit](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit){.fn}(&self, dst: [\*mut](https://doc.rust-lang.org/1.86.0/std/primitive.pointer.html){.primitive} [u8](https://doc.rust-lang.org/1.86.0/std/primitive.u8.html){.primitive}) {#unsafe-fn-clone_to_uninitself-dst-mut-u8 .code-header}
:::

[]{.item-info}

::: {.stab .unstable}
🔬This is a nightly-only experimental API. (`clone_to_uninit`)
:::

::: docblock
Performs copy-assignment from `self` to `dst`. [Read
more](https://doc.rust-lang.org/1.86.0/core/clone/trait.CloneToUninit.html#tymethod.clone_to_uninit)
:::
::::::

::: {#impl-From%3CT%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#767){.src
.rightside}[§](#impl-From%3CT%3E-for-T){.anchor}

### impl\<T\> [From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\> for T {#implt-fromt-for-t .code-header}
:::

::::: impl-items
::: {#method.from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#770){.src
.rightside}[§](#method.from){.anchor}

#### fn [from](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html#tymethod.from){.fn}(t: T) -\> T {#fn-fromt-t---t .code-header}
:::

::: docblock
Returns the argument unchanged.
:::
:::::

::: {#impl-Instrument-for-T .section .impl}
[§](#impl-Instrument-for-T){.anchor}

### impl\<T\> Instrument for T {#implt-instrument-for-t .code-header}
:::

::::::: impl-items
::: {#method.instrument .section .method .trait-impl}
[§](#method.instrument){.anchor}

#### fn [instrument]{.fn}(self, span: Span) -\> Instrumented\<Self\> {#fn-instrumentself-span-span---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the provided \[`Span`\], returning an
`Instrumented` wrapper. Read more
:::

::: {#method.in_current_span .section .method .trait-impl}
[§](#method.in_current_span){.anchor}

#### fn [in_current_span]{.fn}(self) -\> Instrumented\<Self\> {#fn-in_current_spanself---instrumentedself .code-header}
:::

::: docblock
Instruments this type with the [current](super::Span::current())
[`Span`](crate::Span), returning an `Instrumented` wrapper. Read more
:::
:::::::

:::: {#impl-Into%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#750-752){.src
.rightside}[§](#impl-Into%3CU%3E-for-T){.anchor}

### impl\<T, U\> [Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<U\> for T {#implt-u-intou-for-t .code-header}

::: where
where U:
[From](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From"){.trait}\<T\>,
:::
::::

::::: impl-items
::: {#method.into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#760){.src
.rightside}[§](#method.into){.anchor}

#### fn [into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html#tymethod.into){.fn}(self) -\> U {#fn-intoself---u .code-header}
:::

::: docblock
Calls `U::from(self)`.

That is, this conversion is whatever the implementation of
[`From`](https://doc.rust-lang.org/1.86.0/core/convert/trait.From.html "trait core::convert::From")`<T> for U`
chooses to do.
:::
:::::

::: {#impl-IntoEither-for-T .section .impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#64){.src
.rightside}[§](#impl-IntoEither-for-T){.anchor}

### impl\<T\> [IntoEither](https://docs.rs/either/1/either/into_either/trait.IntoEither.html "trait either::into_either::IntoEither"){.trait} for T {#implt-intoeither-for-t .code-header}
:::

:::::::: impl-items
::: {#method.into_either .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#29){.src
.rightside}[§](#method.into_either){.anchor}

#### fn [into_either](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either){.fn}(self, into_left: [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive}) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_eitherself-into_left-bool---eitherself-self .code-header}
:::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left` is `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either)
:::

:::: {#method.into_either_with .section .method .trait-impl}
[Source](https://docs.rs/either/1/src/either/into_either.rs.html#55-57){.src
.rightside}[§](#method.into_either_with){.anchor}

#### fn [into_either_with](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with){.fn}\<F\>(self, into_left: F) -\> [Either](https://docs.rs/either/1/either/enum.Either.html "enum either::Either"){.enum}\<Self, Self\> {#fn-into_either_withfself-into_left-f---eitherself-self .code-header}

::: where
where F:
[FnOnce](https://doc.rust-lang.org/1.86.0/core/ops/function/trait.FnOnce.html "trait core::ops::function::FnOnce"){.trait}(&Self)
-\>
[bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive},
:::
::::

::: docblock
Converts `self` into a
[`Left`](https://docs.rs/either/1/either/enum.Either.html#variant.Left "variant either::Either::Left")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
if `into_left(&self)` returns `true`. Converts `self` into a
[`Right`](https://docs.rs/either/1/either/enum.Either.html#variant.Right "variant either::Either::Right")
variant of
[`Either<Self, Self>`](https://docs.rs/either/1/either/enum.Either.html "enum either::Either")
otherwise. [Read
more](https://docs.rs/either/1/either/into_either/trait.IntoEither.html#method.into_either_with)
:::
::::::::

::: {#impl-Pointable-for-T .section .impl}
[§](#impl-Pointable-for-T){.anchor}

### impl\<T\> Pointable for T {#implt-pointable-for-t .code-header}
:::

::::::::::::::: impl-items
::: {#associatedconstant.ALIGN .section .associatedconstant .trait-impl}
[§](#associatedconstant.ALIGN){.anchor}

#### const [ALIGN]{.constant}: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#const-align-usize .code-header}
:::

::: docblock
The alignment of pointer.
:::

::: {#associatedtype.Init .section .associatedtype .trait-impl}
[§](#associatedtype.Init){.anchor}

#### type [Init]{.associatedtype} = T {#type-init-t .code-header}
:::

::: docblock
The type for initializers.
:::

::: {#method.init .section .method .trait-impl}
[§](#method.init){.anchor}

#### unsafe fn [init]{.fn}(init: \<T as Pointable\>::Init) -\> [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive} {#unsafe-fn-initinit-t-as-pointableinit---usize .code-header}
:::

::: docblock
Initializes a with the given initializer. Read more
:::

::: {#method.deref .section .method .trait-impl}
[§](#method.deref){.anchor}

#### unsafe fn [deref]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-derefaptr-usize---a-t .code-header}
:::

::: docblock
Dereferences the given pointer. Read more
:::

::: {#method.deref_mut .section .method .trait-impl}
[§](#method.deref_mut){.anchor}

#### unsafe fn [deref_mut]{.fn}\<\'a\>(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) -\> [&\'a mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive} {#unsafe-fn-deref_mutaptr-usize---a-mut-t .code-header}
:::

::: docblock
Mutably dereferences the given pointer. Read more
:::

::: {#method.drop .section .method .trait-impl}
[§](#method.drop){.anchor}

#### unsafe fn [drop]{.fn}(ptr: [usize](https://doc.rust-lang.org/1.86.0/std/primitive.usize.html){.primitive}) {#unsafe-fn-dropptr-usize .code-header}
:::

::: docblock
Drops the object pointed to by the given pointer. Read more
:::
:::::::::::::::

::: {#impl-Same-for-T .section .impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#34){.src
.rightside}[§](#impl-Same-for-T){.anchor}

### impl\<T\> [Same](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html "trait typenum::type_operators::Same"){.trait} for T {#implt-same-for-t .code-header}
:::

::::: impl-items
::: {#associatedtype.Output-1 .section .associatedtype .trait-impl}
[Source](https://docs.rs/typenum/1.18.0/src/typenum/type_operators.rs.html#35){.src
.rightside}[§](#associatedtype.Output-1){.anchor}

#### type [Output](https://docs.rs/typenum/1.18.0/typenum/type_operators/trait.Same.html#associatedtype.Output){.associatedtype} = T {#type-output-t .code-header}
:::

::: docblock
Should always be `Self`
:::
:::::

:::: {#impl-SupersetOf%3CSS%3E-for-SP .section .impl}
[§](#impl-SupersetOf%3CSS%3E-for-SP){.anchor}

### impl\<SS, SP\> SupersetOf\<SS\> for SP {#implss-sp-supersetofss-for-sp .code-header}

::: where
where SS: SubsetOf\<SP\>,
:::
::::

::::::::::: impl-items
::: {#method.to_subset .section .method .trait-impl}
[§](#method.to_subset){.anchor}

#### fn [to_subset]{.fn}(&self) -\> [Option](https://doc.rust-lang.org/1.86.0/core/option/enum.Option.html "enum core::option::Option"){.enum}\<SS\> {#fn-to_subsetself---optionss .code-header}
:::

::: docblock
The inverse inclusion map: attempts to construct `self` from the
equivalent element of its superset. Read more
:::

::: {#method.is_in_subset .section .method .trait-impl}
[§](#method.is_in_subset){.anchor}

#### fn [is_in_subset]{.fn}(&self) -\> [bool](https://doc.rust-lang.org/1.86.0/std/primitive.bool.html){.primitive} {#fn-is_in_subsetself---bool .code-header}
:::

::: docblock
Checks if `self` is actually part of its subset `T` (and can be
converted to it).
:::

::: {#method.to_subset_unchecked .section .method .trait-impl}
[§](#method.to_subset_unchecked){.anchor}

#### fn [to_subset_unchecked]{.fn}(&self) -\> SS {#fn-to_subset_uncheckedself---ss .code-header}
:::

::: docblock
Use with care! Same as `self.to_subset` but without any property checks.
Always succeeds.
:::

::: {#method.from_subset .section .method .trait-impl}
[§](#method.from_subset){.anchor}

#### fn [from_subset]{.fn}(element: [&SS](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) -\> SP {#fn-from_subsetelement-ss---sp .code-header}
:::

::: docblock
The inclusion map: converts `self` to the equivalent element of its
superset.
:::
:::::::::::

:::: {#impl-ToOwned-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#82-84){.src
.rightside}[§](#impl-ToOwned-for-T){.anchor}

### impl\<T\> [ToOwned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html "trait alloc::borrow::ToOwned"){.trait} for T {#implt-toowned-for-t .code-header}

::: where
where T:
[Clone](https://doc.rust-lang.org/1.86.0/core/clone/trait.Clone.html "trait core::clone::Clone"){.trait},
:::
::::

::::::::: impl-items
::: {#associatedtype.Owned .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#86){.src
.rightside}[§](#associatedtype.Owned){.anchor}

#### type [Owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#associatedtype.Owned){.associatedtype} = T {#type-owned-t .code-header}
:::

::: docblock
The resulting type after obtaining ownership.
:::

::: {#method.to_owned .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#87){.src
.rightside}[§](#method.to_owned){.anchor}

#### fn [to_owned](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned){.fn}(&self) -\> T {#fn-to_ownedself---t .code-header}
:::

::: docblock
Creates owned data from borrowed data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#tymethod.to_owned)
:::

::: {#method.clone_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/borrow.rs.html#91){.src
.rightside}[§](#method.clone_into){.anchor}

#### fn [clone_into](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into){.fn}(&self, target: [&mut T](https://doc.rust-lang.org/1.86.0/std/primitive.reference.html){.primitive}) {#fn-clone_intoself-target-mut-t .code-header}
:::

::: docblock
Uses borrowed data to replace owned data, usually by cloning. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/borrow/trait.ToOwned.html#method.clone_into)
:::
:::::::::

:::: {#impl-ToString-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2758){.src
.rightside}[§](#impl-ToString-for-T){.anchor}

### impl\<T\> [ToString](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html "trait alloc::string::ToString"){.trait} for T {#implt-tostring-for-t .code-header}

::: where
where T:
[Display](https://doc.rust-lang.org/1.86.0/core/fmt/trait.Display.html "trait core::fmt::Display"){.trait} +
?[Sized](https://doc.rust-lang.org/1.86.0/core/marker/trait.Sized.html "trait core::marker::Sized"){.trait},
:::
::::

::::: impl-items
::: {#method.to_string .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/alloc/string.rs.html#2760){.src
.rightside}[§](#method.to_string){.anchor}

#### fn [to_string](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string){.fn}(&self) -\> [String](https://doc.rust-lang.org/1.86.0/alloc/string/struct.String.html "struct alloc::string::String"){.struct} {#fn-to_stringself---string .code-header}
:::

::: docblock
Converts the given value to a `String`. [Read
more](https://doc.rust-lang.org/1.86.0/alloc/string/trait.ToString.html#tymethod.to_string)
:::
:::::

:::: {#impl-TryFrom%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#807-809){.src
.rightside}[§](#impl-TryFrom%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\> for T {#implt-u-tryfromu-for-t .code-header}

::: where
where U:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-6 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#811){.src
.rightside}[§](#associatedtype.Error-6){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error){.associatedtype} = [Infallible](https://doc.rust-lang.org/1.86.0/core/convert/enum.Infallible.html "enum core::convert::Infallible"){.enum} {#type-error-infallible .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_from .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#814){.src
.rightside}[§](#method.try_from){.anchor}

#### fn [try_from](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#tymethod.try_from){.fn}(value: U) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<T, \<T as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<U\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_fromvalue-u---resultt-t-as-tryfromuerror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-TryInto%3CU%3E-for-T .section .impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#792-794){.src
.rightside}[§](#impl-TryInto%3CU%3E-for-T){.anchor}

### impl\<T, U\> [TryInto](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html "trait core::convert::TryInto"){.trait}\<U\> for T {#implt-u-tryintou-for-t .code-header}

::: where
where U:
[TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>,
:::
::::

::::::: impl-items
::: {#associatedtype.Error-5 .section .associatedtype .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#796){.src
.rightside}[§](#associatedtype.Error-5){.anchor}

#### type [Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#associatedtype.Error){.associatedtype} = \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype} {#type-error-u-as-tryfromterror .code-header}
:::

::: docblock
The type returned in the event of a conversion error.
:::

::: {#method.try_into .section .method .trait-impl}
[Source](https://doc.rust-lang.org/1.86.0/src/core/convert/mod.rs.html#799){.src
.rightside}[§](#method.try_into){.anchor}

#### fn [try_into](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryInto.html#tymethod.try_into){.fn}(self) -\> [Result](https://doc.rust-lang.org/1.86.0/core/result/enum.Result.html "enum core::result::Result"){.enum}\<U, \<U as [TryFrom](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html "trait core::convert::TryFrom"){.trait}\<T\>\>::[Error](https://doc.rust-lang.org/1.86.0/core/convert/trait.TryFrom.html#associatedtype.Error "type core::convert::TryFrom::Error"){.associatedtype}\> {#fn-try_intoself---resultu-u-as-tryfromterror .code-header}
:::

::: docblock
Performs the conversion.
:::
:::::::

:::: {#impl-VZip%3CV%3E-for-T .section .impl}
[§](#impl-VZip%3CV%3E-for-T){.anchor}

### impl\<V, T\> VZip\<V\> for T {#implv-t-vzipv-for-t .code-header}

::: where
where V: MultiLane\<T\>,
:::
::::

:::: impl-items
::: {#method.vzip .section .method .trait-impl}
[§](#method.vzip){.anchor}

#### fn [vzip]{.fn}(self) -\> V {#fn-vzipself---v .code-header}
:::
::::

::: {#impl-WithSubscriber-for-T .section .impl}
[§](#impl-WithSubscriber-for-T){.anchor}

### impl\<T\> WithSubscriber for T {#implt-withsubscriber-for-t .code-header}
:::

:::::::: impl-items
:::: {#method.with_subscriber .section .method .trait-impl}
[§](#method.with_subscriber){.anchor}

#### fn [with_subscriber]{.fn}\<S\>(self, subscriber: S) -\> WithDispatch\<Self\> {#fn-with_subscribersself-subscriber-s---withdispatchself .code-header}

::: where
where S:
[Into](https://doc.rust-lang.org/1.86.0/core/convert/trait.Into.html "trait core::convert::Into"){.trait}\<Dispatch\>,
:::
::::

::: docblock
Attaches the provided [`Subscriber`](super::Subscriber) to this type,
returning a \[`WithDispatch`\] wrapper. Read more
:::

::: {#method.with_current_subscriber .section .method .trait-impl}
[§](#method.with_current_subscriber){.anchor}

#### fn [with_current_subscriber]{.fn}(self) -\> WithDispatch\<Self\> {#fn-with_current_subscriberself---withdispatchself .code-header}
:::

::: docblock
Attaches the current
[default](crate::dispatcher#setting-the-default-subscriber)
[`Subscriber`](super::Subscriber) to this type, returning a
\[`WithDispatch`\] wrapper. Read more
:::
::::::::

:::: {#impl-DeserializeOwned-for-T .section .impl}
[Source](https://docs.rs/serde/1.0.219/src/serde/de/mod.rs.html#614){.src
.rightside}[§](#impl-DeserializeOwned-for-T){.anchor}

### impl\<T\> [DeserializeOwned](https://docs.rs/serde/1.0.219/serde/de/trait.DeserializeOwned.html "trait serde::de::DeserializeOwned"){.trait} for T {#implt-deserializeowned-for-t .code-header}

::: where
where T: for\<\'de\>
[Deserialize](https://docs.rs/serde/1.0.219/serde/de/trait.Deserialize.html "trait serde::de::Deserialize"){.trait}\<\'de\>,
:::
::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
:::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::::
