/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 9/1/25
******************************************************************************/
use crate::curves::{Curve, Point2D};
use crate::error::CurveError;
use crate::geometrics::GeometricObject;
use optionstratlib_core::model::decimal::{d_add, d_div, d_mul, d_sub};
use rust_decimal::Decimal;
use std::collections::BTreeSet;
use tracing::warn;

/// Wraps a checked-arithmetic failure raised while sampling a utility curve.
fn sampling_err(err: optionstratlib_core::error::DecimalError) -> CurveError {
    CurveError::ConstructionError(err.to_string())
}

/// The `i`-th coordinate of a grid from `start` to `end`, given its computed
/// value `start + step * i`.
///
/// The step is `(end - start) / steps` rounded at 28 places, so the computed
/// last coordinate can land 1 to 2 ulps on either side of an `end` with a
/// long mantissa (#799). The last coordinate is therefore `end` itself, and
/// every other one is clamped toward `end`, in whichever direction the range
/// runs, as in the merge grid of #795. Only the last coordinate and an
/// overshooting one can move; every other coordinate is unchanged.
pub(crate) fn grid_coordinate(
    value: Decimal,
    start: Decimal,
    end: Decimal,
    is_last: bool,
) -> Decimal {
    if is_last {
        end
    } else if start <= end {
        value.min(end)
    } else {
        value.max(end)
    }
}

/// Creates a linear curve defined by a starting point, an ending point, and a slope.
///
/// This function generates a 2-dimensional curve by calculating evenly spaced points
/// (10 intervals by default) between the `start` and `end` x-coordinates. For each
/// generated x-coordinate, the corresponding y-coordinate is computed using the provided
/// slope, following the equation:
///
/// ```text
/// y = slope * x
/// ```
///
/// The generated points are then used to construct a `Curve` instance.
///
/// # Parameters
/// - `start`: The starting x-coordinate of the curve (as a `Decimal`).
/// - `end`: The ending x-coordinate of the curve (as a `Decimal`).
///   - Must be greater than the `start` value for the function to work as intended.
/// - `slope`: The slope of the linear curve, which determines the relationship between
///   x and y values.
///
/// # Returns
/// A `Curve` instance containing evenly spaced points along the linear curve determined
/// by the specified parameters.
///
/// # Behavior
/// - The x-coordinates are computed as evenly spaced values between `start` and `end`
///   across 10 steps. Each x-coordinate includes its corresponding `y` value determined
///   by the slope.
/// - Internally uses `Point2D::new` to construct points based on the computed x- and
///   y-coordinate values.
/// - Constructs the final curve using `Curve::from_vector`, with the computed points
///   forming the curve.
///
/// # Constraints
/// - The `end` value must be greater than the `start` value; otherwise, the generated
///   points will result in an incorrect or potentially invalid curve.
/// - The function uses a fixed number (10) of steps to divide the range between `start`
///   and `end`. This ensures uniform spacing between points but limits flexibility
///   for other resolutions.
///
/// # Example Workflow (Internal Overview)
/// 1. Divide the range `[start, end]` into 10 equal steps (`step_size`).
/// 2. Iteratively compute `(x, y)` points using the formula `y = slope * x`.
/// 3. Accumulate these points into a `Vec<Point2D>`.
/// 4. Construct the final `Curve` using `Curve::from_vector`.
///
/// # Usage Notes
/// - This function is best suited for applications requiring a simple linear curve
///   representation between two bounds.
/// - For higher resolution or adaptive step generation, consider modifying the function
///   or implementing a similar utility.
///
/// # Errors
///
/// Returns [`CurveError::ConstructionError`] when the span `end - start`, a
/// sampled abscissa or the ordinate `slope * x` leaves the representable
/// `Decimal` range.
///
/// # See Also
/// - [`Point2D::new`]: Utility used to construct individual points for the curve.
/// - [`Curve::from_vector`]: Used to generate the resulting curve from the constructed points.
///
/// # Example (High-Level Usage Concept)
/// While examples are omitted as requested, the general idea is to pass desired
/// values for `start`, `end`, and `slope` into this function in a practical implementation
/// scenario.
///
/// ```rust
/// use rust_decimal::Decimal;
/// use optionstratlib_math::curves::create_linear_curve;
/// let curve = create_linear_curve(
///     Decimal::new(0, 1),   // start = 0.0
///     Decimal::new(100, 1), // end = 10.0
///     Decimal::new(1, 0)    // slope = 1.0
/// )
/// .expect("the range and slope are well within the Decimal range");
/// ```
///
/// would result in a curve defined by the points:
/// `(0.0, 0.0)`, `(1.0, 1.0)`, ..., `(10.0, 10.0)`.
///
/// From the above, it demonstrates how linearly spaced and
pub fn create_linear_curve(
    start: Decimal,
    end: Decimal,
    slope: Decimal,
) -> Result<Curve, CurveError> {
    let steps = 10;
    let op = "create_linear_curve";
    let span = d_sub(end, start, op).map_err(sampling_err)?;
    let step_size = d_div(span, Decimal::from(steps), op).map_err(sampling_err)?;

    let mut points: Vec<Point2D> = Vec::new();
    for i in 0..=steps {
        let offset = d_mul(step_size, Decimal::from(i), op).map_err(sampling_err)?;
        let x = grid_coordinate(
            d_add(start, offset, op).map_err(sampling_err)?,
            start,
            end,
            i == steps,
        );
        let y = d_mul(slope, x, op).map_err(sampling_err)?;
        points.push(Point2D::new(x, y));
    }

    Ok(Curve::from_vector(points.iter().collect()))
}

/// Creates a constant curve with equidistant points along the x-axis and the same constant value for the y-axis.
///
/// This function generates a simple mathematical curve defined over a fixed range of x-values with an equal spacing
/// between points, where each y-coordinate is set to a constant value specified by the `value` parameter. The curve
/// is represented as a collection of `Point2D` points, which are then used to create a `Curve` object.
///
/// # Parameters
/// - `start`: The starting x-coordinate for the curve, represented as a `Decimal`.
/// - `end`: The ending x-coordinate for the curve, represented as a `Decimal`.
/// - `value`: The constant y-coordinate value applied to all points in the curve, represented as a `Decimal`.
///
/// # Returns
/// A `Curve` instance that represents the constant curve. The returned curve consists of equidistant `Point2D`
/// points between the `start` and `end` x-coordinates, all having the same y-coordinate defined by `value`.
///
/// # Behavior
/// - The function divides the range `[start, end]` into a fixed number of equally spaced steps.
/// - The x-coordinate of each point is calculated based on this step size.
/// - The `value` is used as the y-coordinate for all points.
/// - A `Curve` is created using the generated `Point2D` points via the `Curve::from_vector` method.
///
/// # Details
/// - Internally, this function assumes 10 steps (`steps = 10`) for dividing the x-range. This creates 11 points
///   including both the `start` and `end` x-coordinates.
/// - The calculation of intermediate x-coordinates uses a constant `step_size`, computed as `(end - start) / steps`.
/// - The function ensures that both the `start` and `end` values are included in the resulting curve.
///
/// # Example
/// While this is designed to remain usage-agnostic, in practice, it results in a horizontal line in Cartesian
/// space that is constant in the y-dimension and spans the x-range.
///
/// # Errors
///
/// Returns [`CurveError::ConstructionError`] when the span `end - start` or a
/// sampled abscissa leaves the representable `Decimal` range.
///
/// # See Also
/// - [`Point2D::new`]: Used to create individual points in the resulting curve.
/// - [`Curve::from_vector`]: Used internally to convert the set of constant points into a `Curve` object.
pub fn create_constant_curve(
    start: Decimal,
    end: Decimal,
    value: Decimal,
) -> Result<Curve, CurveError> {
    let steps = 10;
    let op = "create_constant_curve";
    let span = d_sub(end, start, op).map_err(sampling_err)?;
    let step_size = d_div(span, Decimal::from(steps), op).map_err(sampling_err)?;

    let mut point_values: Vec<Point2D> = Vec::new();
    for i in 0..=steps {
        let offset = d_mul(step_size, Decimal::from(i), op).map_err(sampling_err)?;
        let x = grid_coordinate(
            d_add(start, offset, op).map_err(sampling_err)?,
            start,
            end,
            i == steps,
        );
        point_values.push(Point2D::new(x, value));
    }

    let points: Vec<&Point2D> = point_values.iter().collect();

    Ok(Curve::from_vector(points))
}

/// Detects peaks and valleys in a set of points with configurable sensitivity
///
/// # Arguments
///
/// * `points` - A reference to a BTreeSet of Point2D
/// * `min_prominence` - Minimum vertical distance between a peak/valley and surrounding points
/// * `window_size` - Number of points to consider on each side (default: 1)
///
/// # Returns
///
/// A tuple containing two vectors:
/// - The first vector contains the peaks (local maxima)
/// - The second vector contains the valleys (local minima)
///
/// # Errors
///
/// Returns [`CurveError::ConstructionError`] when `2 * window_size + 1`
/// leaves the `usize` range, when a window index falls outside the points,
/// and when a prominence leaves the `Decimal` range (ordinates spanning more
/// than `Decimal::MAX`); the raw operators aborted on each before #788.
pub fn detect_peaks_and_valleys(
    points: &BTreeSet<Point2D>,
    min_prominence: Decimal,
    window_size: usize,
) -> Result<(Vec<Point2D>, Vec<Point2D>), CurveError> {
    let points_vec: Vec<Point2D> = points.iter().cloned().collect();
    let mut peaks = Vec::new();
    let mut valleys = Vec::new();

    // Need at least 2*window_size + 1 points to detect peaks and valleys
    let needed = window_size
        .checked_mul(2)
        .and_then(|width| width.checked_add(1))
        .ok_or_else(|| {
            CurveError::ConstructionError(format!(
                "peak window of {window_size} points on each side is too large"
            ))
        })?;
    if points_vec.len() < needed {
        warn!(
            "Not enough points to detect peaks and valleys with window size {}. Need at least {} points, but got {}.",
            window_size,
            needed,
            points_vec.len()
        );
        return Ok((peaks, valleys));
    }

    // `len >= 2 * window_size + 1` above, so the subtraction stays in range.
    let last = points_vec
        .len()
        .checked_sub(window_size)
        .ok_or_else(|| window_error(window_size, points_vec.len()))?;
    for i in window_size..last {
        let current = window_point(&points_vec, Some(i))?;
        let mut is_peak = true;
        let mut is_valley = true;

        // Check if the current point is higher or lower than all points in the window
        for j in 1..=window_size {
            let before = window_point(&points_vec, i.checked_sub(j))?;
            let after = window_point(&points_vec, i.checked_add(j))?;

            // For a peak, current needs to be higher than all points in window
            if current.y <= before.y || current.y <= after.y {
                is_peak = false;
            }

            // For a valley, current needs to be lower than all points in window
            if current.y >= before.y || current.y >= after.y {
                is_valley = false;
            }

            // No need to check further if neither peak nor valley
            if !is_peak && !is_valley {
                break;
            }
        }

        // Check prominence (how much a peak/valley "stands out")
        if is_peak {
            let prominence = calculate_prominence(&points_vec, i, true)?;
            if prominence >= min_prominence {
                peaks.push(*current);
            }
        } else if is_valley {
            let prominence = calculate_prominence(&points_vec, i, false)?;
            if prominence >= min_prominence {
                valleys.push(*current);
            }
        }
    }

    Ok((peaks, valleys))
}

/// The point at a window index, or the error naming an index outside the
/// points (`None` is an index that overflowed `usize`).
fn window_point(points: &[Point2D], index: Option<usize>) -> Result<&Point2D, CurveError> {
    index
        .and_then(|i| points.get(i))
        .ok_or_else(|| window_error(index.unwrap_or(usize::MAX), points.len()))
}

#[cold]
#[inline(never)]
fn window_error(index: usize, len: usize) -> CurveError {
    CurveError::ConstructionError(format!(
        "peak window index {index} is outside the {len} points"
    ))
}

/// Calculate prominence (vertical distance from a peak/valley to its surroundings)
///
/// # Errors
///
/// Returns [`CurveError::ConstructionError`] when `index` is outside the
/// points or the distance leaves the `Decimal` range.
fn calculate_prominence(
    points: &[Point2D],
    index: usize,
    is_peak: bool,
) -> Result<Decimal, CurveError> {
    let current = window_point(points, Some(index))?.y;

    // Find highest/lowest points to the left and right
    let left_bound = if is_peak {
        points
            .iter()
            .take(index)
            .map(|p| p.y)
            .min()
            .unwrap_or(Decimal::MAX)
    } else {
        points
            .iter()
            .take(index)
            .map(|p| p.y)
            .max()
            .unwrap_or(Decimal::MIN)
    };

    // `skip(index).skip(1)` is the points after `index`, with no `index + 1`.
    let right_bound = if is_peak {
        points
            .iter()
            .skip(index)
            .skip(1)
            .map(|p| p.y)
            .min()
            .unwrap_or(Decimal::MAX)
    } else {
        points
            .iter()
            .skip(index)
            .skip(1)
            .map(|p| p.y)
            .max()
            .unwrap_or(Decimal::MIN)
    };

    // Calculate prominence
    let op = "curves::utils::calculate_prominence";
    if is_peak {
        d_sub(current, Decimal::max(left_bound, right_bound), op).map_err(sampling_err)
    } else {
        d_sub(Decimal::min(left_bound, right_bound), current, op).map_err(sampling_err)
    }
}

#[cfg(test)]
mod tests_utils {
    use crate::curves::Point2D;
    use crate::curves::utils::{calculate_prominence, detect_peaks_and_valleys};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::collections::BTreeSet;

    #[test]
    fn test_detect_peaks_and_valleys_insufficient_points() {
        // Test when there are not enough points for the window size
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(3.0)),
        ]);

        // Use window_size = 1, which requires at least 3 points (2*1+1)
        let (peaks, valleys) = detect_peaks_and_valleys(&points, dec!(0.1), 1).unwrap();

        // Should return empty vectors with a warning log
        assert!(peaks.is_empty());
        assert!(valleys.is_empty());
    }

    #[test]
    fn test_calculate_prominence() {
        // Test calculate_prominence function (lines 185-189, 191)
        let points = vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(2.0)), // peak
            Point2D::new(dec!(2.0), dec!(1.0)),
            Point2D::new(dec!(3.0), dec!(-1.0)), // valley
            Point2D::new(dec!(4.0), dec!(0.0)),
        ];

        // Test prominence for a peak
        let peak_prominence = calculate_prominence(&points, 1, true).unwrap();
        assert_eq!(peak_prominence, dec!(2.0));

        // Test prominence for a valley
        let valley_prominence = calculate_prominence(&points, 3, false).unwrap();
        assert_eq!(valley_prominence, dec!(1.0));
    }

    #[test]
    fn test_detect_peaks_and_valleys_with_prominence() {
        // Create a curve with clear peaks and valleys
        let points = BTreeSet::from_iter(vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(3.0)),
            Point2D::new(dec!(2.0), dec!(-2.0)),
            Point2D::new(dec!(3.0), dec!(2.0)),
            Point2D::new(dec!(4.0), dec!(-1.0)),
            Point2D::new(dec!(5.0), dec!(0.0)),
        ]);

        // With low prominence threshold, should detect all peaks and valleys
        let (peaks, valleys) = detect_peaks_and_valleys(&points, dec!(0.1), 1).unwrap();
        assert_eq!(peaks.len(), 2);
        assert_eq!(valleys.len(), 2);

        // With high prominence threshold, should only detect the most prominent peaks
        let (peaks, valleys) = detect_peaks_and_valleys(&points, dec!(4.0), 1).unwrap();
        assert!(peaks.is_empty());
        assert!(!valleys.is_empty());

        // With medium prominence threshold
        let (peaks, valleys) = detect_peaks_and_valleys(&points, dec!(2.0), 1).unwrap();
        assert_eq!(peaks.len(), 2);
        assert_eq!(valleys.len(), 1);
        assert_eq!(peaks[0].y, dec!(3.0));
        assert_eq!(valleys[0].y, dec!(-2.0));
    }

    // A peak at `Decimal::MAX` beside a valley at `Decimal::MIN` has a
    // prominence beyond the `Decimal` range; the subtraction aborted with
    // `Subtraction overflowed` before #788.
    #[test]
    fn test_prominence_overflow_is_error() {
        let points = vec![
            Point2D::new(dec!(0.0), Decimal::MIN),
            Point2D::new(dec!(1.0), Decimal::MAX),
            Point2D::new(dec!(2.0), Decimal::MIN),
        ];
        assert!(calculate_prominence(&points, 1, true).is_err());
        assert!(calculate_prominence(&points, 3, true).is_err());
        let set: BTreeSet<Point2D> = points.into_iter().collect();
        assert!(detect_peaks_and_valleys(&set, dec!(0.1), 1).is_err());
    }

    // `2 * window_size + 1` overflowed `usize` before #788.
    #[test]
    fn test_window_size_overflow_is_error() {
        let points = BTreeSet::from_iter(vec![Point2D::new(dec!(1.0), dec!(2.0))]);
        assert!(detect_peaks_and_valleys(&points, dec!(0.1), usize::MAX).is_err());
    }
}

/// The generated abscissas stay inside `[start, end]` and the last one is
/// exactly `end`: the rounded step used to put it 1 to 2 ulps off a
/// long-mantissa `end` (#799).
#[cfg(test)]
mod tests_generators_stay_inside_the_range {
    use super::*;
    use rust_decimal_macros::dec;

    /// Ends whose tenth has more digits than a `Decimal` keeps.
    fn long_mantissa_ends() -> Vec<Decimal> {
        vec![
            dec!(10.00000000000000000000000002),
            dec!(3.333333333333333333333333333),
            dec!(7.777777777777777777777777777),
            dec!(1.234567890123456789012345678),
            dec!(9.999999999999999999999999999),
            dec!(0.1234567890123456789012345678),
            dec!(123.4567890123456789012345678),
        ]
    }

    fn assert_inside(curve: &Curve, start: Decimal, end: Decimal) {
        let (lo, hi) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };
        for p in &curve.points {
            assert!(p.x >= lo && p.x <= hi, "x = {} outside [{lo}, {hi}]", p.x);
        }
    }

    #[test]
    fn test_parametric_curve_stays_inside_the_range() {
        use crate::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};
        for end in long_mantissa_ends() {
            for (t_start, t_end) in [(Decimal::ZERO, end), (end, Decimal::ZERO)] {
                let params = ConstructionParams::D2 {
                    t_start,
                    t_end,
                    steps: 10,
                };
                let f = |t: Decimal| -> Result<Point2D, CurveError> { Ok(Point2D::new(t, t)) };
                match Curve::construct(ConstructionMethod::Parametric {
                    f: Box::new(f),
                    params,
                }) {
                    Ok(curve) => assert_inside(&curve, t_start, t_end),
                    Err(e) => panic!("parametric curve must build: {e:?}"),
                }
            }
        }
    }

    #[test]
    fn test_parametric_surface_stays_inside_the_range() {
        use crate::error::SurfaceError;
        use crate::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};
        use crate::surfaces::{Point3D, Surface};
        for end in long_mantissa_ends() {
            let params = ConstructionParams::D3 {
                x_start: Decimal::ZERO,
                x_end: end,
                y_start: end,
                y_end: Decimal::ZERO,
                x_steps: 10,
                y_steps: 10,
            };
            let f = |p: Point2D| -> Result<Point3D, SurfaceError> {
                Ok(Point3D::new(p.x, p.y, Decimal::ZERO))
            };
            match Surface::construct(ConstructionMethod::Parametric {
                f: Box::new(f),
                params,
            }) {
                Ok(surface) => {
                    for p in &surface.points {
                        assert!(p.x >= Decimal::ZERO && p.x <= end, "x = {} past {end}", p.x);
                        assert!(p.y >= Decimal::ZERO && p.y <= end, "y = {} past {end}", p.y);
                    }
                    assert!(surface.points.iter().any(|p| p.x == end), "no x at {end}");
                    assert!(
                        surface.points.iter().any(|p| p.y == Decimal::ZERO),
                        "no y at 0"
                    );
                }
                Err(e) => panic!("parametric surface must build: {e:?}"),
            }
        }
    }

    #[test]
    fn test_grid_coordinate_pins_the_last_point_and_clamps_the_rest() {
        let (start, end) = (Decimal::ZERO, dec!(1));
        assert_eq!(grid_coordinate(dec!(0.5), start, end, false), dec!(0.5));
        assert_eq!(
            grid_coordinate(dec!(1.0000000000000000000000000002), start, end, false),
            end
        );
        // The last coordinate is `end` whether it fell short or overshot.
        assert_eq!(
            grid_coordinate(dec!(0.9999999999999999999999999998), start, end, true),
            end
        );
        assert_eq!(
            grid_coordinate(dec!(1.0000000000000000000000000002), start, end, true),
            end
        );
        // A descending range clamps the other way.
        assert_eq!(
            grid_coordinate(dec!(-0.0000000000000000000000000002), end, start, false),
            start
        );
        assert_eq!(grid_coordinate(dec!(0.5), end, start, false), dec!(0.5));
    }

    #[test]
    fn test_linear_and_constant_curves_stay_inside_the_range() {
        for end in long_mantissa_ends() {
            for (start, end) in [(Decimal::ZERO, end), (end, Decimal::ZERO)] {
                match create_linear_curve(start, end, dec!(2)) {
                    Ok(curve) => assert_inside(&curve, start, end),
                    Err(e) => panic!("linear curve must build: {e:?}"),
                }
                match create_constant_curve(start, end, dec!(5)) {
                    Ok(curve) => assert_inside(&curve, start, end),
                    Err(e) => panic!("constant curve must build: {e:?}"),
                }
            }
        }
    }
}
