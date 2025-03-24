use crate::Positive;
use crate::constants::{DARK_GREEN, DARK_RED};
use crate::pricing::payoff::Profit;
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use num_traits::ToPrimitive;
use plotters::backend::BitMapBackend;
use plotters::element::{Circle, Text};
use plotters::prelude::RGBColor;
use plotters::prelude::{
    Cartesian2d, ChartContext, Color, DrawingBackend, IntoDrawingArea, IntoFont, LineSeries,
    Ranged, WHITE,
};
use rand::{Rng, rng};
use std::error::Error;
use std::ops::Add;

/// Aplica un degradado a un color base basado en un valor normalizado.
///
/// # Parámetros
/// - `base_color`: El color base del degradado.
/// - `end_color`: El color final del degradado.
/// - `normalized_value`: Un valor normalizado en el rango [0, 1] que determina la posición en el degradado.
///
/// # Retorno
/// Un nuevo `RGBColor` interpolado entre `base_color` y `end_color`.
pub fn apply_shade(base_color: RGBColor, normalized_value: f64) -> RGBColor {
    let end_color = RGBColor(base_color.1, base_color.2, base_color.0);
    let r = base_color.0 as f64 + (end_color.0 as f64 - base_color.0 as f64) * normalized_value;
    let g = base_color.1 as f64 + (end_color.1 as f64 - base_color.1 as f64) * normalized_value;
    let b = base_color.2 as f64 + (end_color.2 as f64 - base_color.2 as f64) * normalized_value;

    RGBColor(r as u8, g as u8, b as u8)
}

/**
Defines the backend for rendering graphs.  Different backends are available depending on the target architecture.

# Backends

* **Bitmap (Native Targets):** Renders the graph to a bitmap image file.  This is available for all targets except WebAssembly
*/
pub enum GraphBackend<'a> {
    /// Bitmap backend.  Writes the graph to an image file.
    Bitmap {
        /// Path to the output image file.
        file_path: &'a str,
        /// Dimensions of the output image (width, height).
        size: (u32, u32),
    },
}

/// Creates a drawing area with a white background.
///
/// # Arguments
///
/// * `$file_path` - The path to the output image file.
/// * `$width` - The width of the drawing area.
/// * `$height` - The height of the drawing area.
///
/// # Returns
///
/// A `DrawingArea` object.
///
/// # Errors
///
/// Returns an error if the drawing area cannot be created.
#[macro_export]
macro_rules! create_drawing_area {
    ($file_path:expr, $width:expr, $height:expr) => {{
        let root = BitMapBackend::new($file_path, ($width, $height)).into_drawing_area();
        root.fill(&WHITE)?;
        root
    }};
}

/// Builds a chart with a title and specified axis ranges.
///
/// # Arguments
///
/// * `$root` - The drawing area to build the chart on.
/// * `$title` - The title of the chart.
/// * `$title_size` - The font size of the title.
/// * `$min_x` - The minimum value for the x-axis.
/// * `$max_x` - The maximum value for the x-axis.
/// * `$min_y` - The minimum value for the y-axis.
/// * `$max_y` - The maximum value for the y-axis.
///
/// # Returns
///
/// A `ChartBuilder` object.        
/// # Errors
///
/// Returns an error if the chart cannot be built.
#[macro_export]
macro_rules! build_chart {
    ($root:expr, $title:expr, $title_size:expr, $min_x:expr, $max_x:expr, $min_y:expr, $max_y:expr) => {
        plotters::prelude::ChartBuilder::on($root)
            .caption($title, ("sans-serif", $title_size))
            .margin(10)
            .top_x_label_area_size(40)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .right_y_label_area_size(60)
            .build_cartesian_2d($min_x..$max_x, $min_y..$max_y)?
    };
}

/// Configures the chart mesh, labels, and draws a horizontal line at y = 0.
///
/// # Arguments
///
/// * `$chart` - The chart to configure.
/// * `$x_labels` - The number of labels for the x-axis.
/// * `$y_labels` - The number of labels for the y-axis.
/// * `$min_x` - The minimum value for the x-axis.
/// * `$max_x` - The maximum value for the x-axis.
///
/// # Errors
///
/// Returns an error if the chart cannot be configured or the line cannot be drawn.
#[macro_export]
macro_rules! configure_chart_and_draw_mesh {
    ($chart:expr, $x_labels:expr, $y_labels:expr, $min_x:expr, $max_x:expr) => {{
        $chart
            .configure_mesh()
            .disable_mesh() // Disable the mesh grid
            .x_labels($x_labels)
            .y_labels($y_labels)
            .draw()?;
        // Draw a horizontal line at y = 0
        $chart.draw_series(plotters::prelude::LineSeries::new(
            vec![($min_x, 0.0), ($max_x, 0.0)],
            &plotters::prelude::BLACK,
        ))?;
    }};
}

/// Draws line segments on the chart based on provided data.
///
/// # Arguments
///
/// * `$chart` - The chart to draw on.
/// * `$x_axis_data` - The data for the x-axis.
/// * `$y_axis_data` - The data for the y-axis.
/// * `$dark_green` - The color to use for positive values.
/// * `$dark_red` - The color to use for negative values.
///
/// # Errors
///
/// Returns an error if the line segments cannot be drawn.
#[macro_export]
macro_rules! draw_line_segments {
    ($chart:expr, $x_axis_data:expr, $y_axis_data:expr, $dark_green:expr, $dark_red:expr) => {{
        let mut last_point: Option<(Positive, f64)> = None;
        for (&price, &value) in $x_axis_data.iter().zip($y_axis_data.iter()) {
            if let Some((last_price, last_profit)) = last_point {
                let color = if value > 0.0 {
                    &$dark_green
                } else {
                    &$dark_red
                };
                let points: Vec<(f64, f64)> =
                    vec![(last_price.to_f64(), last_profit), (price.to_f64(), value)];
                $chart.draw_series(plotters::prelude::LineSeries::new(points, color))?;
            }
            last_point = Some((price, value));
        }
        let _ = Ok::<(), Box<dyn std::error::Error>>(());
    }};
}

/// Trait for creating graphs of profit calculations.
/// This trait extends the `Profit` trait, adding the functionality to visualize profit calculations.
pub trait Graph: Profit {
    /// Generates a graph of profit calculations.
    ///
    /// # Arguments
    ///
    /// * `x_axis_data` - A slice of `Positive` values representing the x-axis data points (e.g., prices).
    /// * `backend` - The `GraphBackend` to use for rendering.  This determines whether the graph is rendered to a bitmap file or a canvas element.
    /// * `title_size` - The font size for the graph title.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// * `x_axis_data` is empty.
    /// * No valid y-axis values could be calculated (e.g., all calculations resulted in errors).
    /// * There is an issue during graph creation or rendering with the chosen backend.
    ///
    fn graph(&self, backend: GraphBackend, title_size: u32) -> Result<(), Box<dyn Error>> {
        let x_values = self.get_x_values();
        let x_axis_data: &[Positive] = &x_values;

        if x_axis_data.is_empty() {
            return Err("No valid values to plot".into());
        }

        let y_axis_data: Vec<f64> = self.get_y_values();
        if y_axis_data.is_empty() {
            return Err("No valid values to plot".into());
        }

        let (max_x_value, min_x_value, max_y_value, min_y_value) =
            calculate_axis_range(x_axis_data, &y_axis_data, None);

        // Set up the drawing area
        let root = match backend {
            GraphBackend::Bitmap { file_path, size } => {
                let root = BitMapBackend::new(file_path, size).into_drawing_area();
                root.fill(&WHITE)?;
                root
            }
        };

        let mut chart = build_chart!(
            &root,
            self.title(),
            title_size,
            min_x_value.to_f64(),
            max_x_value.to_f64(),
            min_y_value,
            max_y_value
        );

        configure_chart_and_draw_mesh!(chart, 20, 20, min_x_value.to_f64(), max_x_value.to_f64());
        draw_line_segments!(chart, x_axis_data, y_axis_data, DARK_GREEN, DARK_RED);

        draw_points_on_chart(&mut chart, &self.get_points())?;
        draw_vertical_lines_on_chart(&mut chart, &self.get_vertical_lines())?;
        root.present()?;
        Ok(())
    }

    /// Returns the title of the graph.
    fn title(&self) -> String;

    fn get_x_values(&self) -> Vec<Positive>;

    /// Calculates the y-axis values (profit) corresponding to the provided x-axis data.
    ///
    /// # Arguments
    ///
    /// * `data` - A slice of `Positive` values representing the x-axis data points.
    ///
    /// # Returns
    ///
    /// A vector of `f64` representing the calculated profit values.
    fn get_y_values(&self) -> Vec<f64> {
        let data = self.get_x_values();

        data.iter()
            .filter_map(|&price| {
                self.calculate_profit_at(price)
                    .ok() // Result in Option
                    .and_then(|d| d.to_f64())
            })
            .collect()
    }

    /// Returns a vector of vertical lines to draw on the chart. Default implementation returns an empty vector.
    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        Vec::new()
    }

    /// Returns a vector of points to draw on the chart. Default implementation returns an empty vector.
    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        Vec::new()
    }
}

/// Calculates the range for the X and Y axes.
///
/// # Arguments
///
/// * `x_axis_data` - A slice of `f64` values representing the x-axis data.
/// * `y_axis_data` - A slice of `f64` values representing the y-axis data.
///
/// # Returns
///
/// A tuple `(f64, f64, f64, f64)` containing the following values:
///
/// * `max_x_value` - The maximum value in `x_axis_data`.
/// * `min_x_value` - The minimum value in `x_axis_data`.
/// * `max_y_value` - The maximum value in `y_axis_data`, adjusted to include a margin.
/// * `min_y_value` - The minimum value in `y_axis_data`, adjusted to include a margin.
///
pub(crate) fn calculate_axis_range(
    x_axis_data: &[Positive],
    y_axis_data: &[f64],
    margin: Option<f64>,
) -> (Positive, Positive, f64, f64) {
    let (min_x_value, max_x_value) = if x_axis_data.is_empty() {
        (Positive::ZERO, Positive::INFINITY)
    } else {
        x_axis_data.iter().fold(
            (Positive::INFINITY, Positive::ZERO),
            |(min_x, max_x), &value| (min_x.min(value), max_x.max(value)),
        )
    };

    let (min_y_temp, max_y_temp) = if y_axis_data.is_empty() {
        (f64::INFINITY, f64::NEG_INFINITY)
    } else {
        y_axis_data.iter().fold(
            (f64::INFINITY, f64::NEG_INFINITY),
            |(min_y, max_y), &value| (f64::min(min_y, value), f64::max(max_y, value)),
        )
    };

    if min_y_temp.is_infinite() || max_y_temp.is_infinite() {
        return (max_x_value, min_x_value, min_y_temp, max_y_temp);
    }

    let margin_value = margin.unwrap_or(1.0);

    let adjusted_max_profit = (max_y_temp * margin_value - max_y_temp).abs();
    let adjusted_min_profit = (min_y_temp * margin_value - min_y_temp).abs();
    let margin_value = adjusted_max_profit.max(adjusted_min_profit);
    let max_y_value = max_y_temp + margin_value;
    let min_y_value = min_y_temp - margin_value;

    (max_x_value, min_x_value, max_y_value, min_y_value)
}

/// Draws chart points and their associated labels on a chart context.
///
/// This function is responsible for rendering a list of chart points onto a given
/// chart context. Each point is represented as a circle, styled with a specific
/// size and color, and labeled with text positioned based on a defined offset.
///
/// # Type Parameters
///
/// - `DB`: The backend responsible for rendering the chart, implementing the `DrawingBackend` trait.
/// - `X`: The type representing the horizontal (x-axis) range of the chart, implementing the `Ranged` trait.
/// - `Y`: The type representing the vertical (y-axis) range of the chart, also implementing the `Ranged` trait.
///
/// # Arguments
///
/// - `ctx`: A mutable reference to a `ChartContext` object, which provides the necessary context
///   for drawing on the chart.
/// - `points`: A slice of `ChartPoint` objects, each representing a point to render on the chart,
///   including its coordinates, styling, and label information.
///
/// # Returns
///
/// Returns `Ok(())` if all points and their labels are successfully drawn.
/// If an error occurs during the rendering process (e.g., backend issues), a boxed `Error` is returned.
///
/// # Constraints
///
/// - The value type of `X` and `Y` must:
///   - Be clonable.
///   - Support addition with a `f64` value (`Add<f64>`).
/// - `X::ValueType` and `Y::ValueType` must additionally be compatible with `Into<(X::ValueType, Y::ValueType)>`.
/// - The error type of the drawing backend (`DB::ErrorType`) must be `'static`.
///
/// # Implementation Details
///
/// - For each point in the `points` slice:
///   1. A circle is drawn according to the point's coordinates, size, and color.
///   2. A textual label is placed near the point, with its position influenced by the specified `label_offset`.
///
/// - Uses the helper method `LabelOffsetType::get_offset` to determine the offset values for positioning the labels.
///
pub fn draw_points_on_chart<DB: DrawingBackend, X, Y>(
    ctx: &mut ChartContext<DB, Cartesian2d<X, Y>>,
    points: &[ChartPoint<(X::ValueType, Y::ValueType)>],
) -> Result<(), Box<dyn Error>>
where
    X: Ranged,
    Y: Ranged,
    X::ValueType: Clone + Add<f64, Output = X::ValueType> + 'static,
    Y::ValueType: Clone + Add<f64, Output = Y::ValueType> + 'static,
    (X::ValueType, Y::ValueType): Clone + Into<(X::ValueType, Y::ValueType)>,
    DB::ErrorType: 'static,
{
    for point in points {
        ctx.draw_series(std::iter::once(Circle::new(
            point.coordinates.clone(),
            point.point_size,
            point.point_color.filled(),
        )))?;
    }

    for point in points {
        let (x, y) = point.coordinates.clone();
        let (offset_x, offset_y) = point.label_offset.get_offset();
        let label_pos = (x.add(offset_x), y.add(offset_y));

        ctx.draw_series(std::iter::once(Text::new(
            point.label.clone(),
            label_pos,
            ("sans-serif", point.font_size)
                .into_font()
                .color(&point.label_color),
        )))?;
    }

    Ok(())
}

/// Draws vertical lines with labels on a given chart using the specified drawing backend.
///
/// This function renders a series of vertical lines on a chart, given their positions,
/// styles, and associated labels. It utilizes a `ChartContext` for rendering the lines and
/// the Plotters crate utilities for styling and layout. Each line is drawn between a specified
/// range on the y-axis and features an optional label placed at a specific offset.
///
/// # Type Parameters
///
/// - `DB`: The type representing the drawing backend, which must implement the `DrawingBackend`
///   trait. This defines how the chart elements are rendered (e.g., as an image, on a canvas, etc.).
/// - `X`: The type representing the x-axis of the chart. It must implement the `Ranged` trait
///   to support scaling and interpolation.
/// - `Y`: The type representing the y-axis of the chart. Similar to `X`, it must implement
///   the `Ranged` trait for compatibility.
///
/// # Function Parameters
///
/// - `ctx`: A mutable reference to a `ChartContext`, which handles the drawing and layout
///   of the chart elements. It is parameterized with the drawing backend `DB` and coordinate system
///   `Cartesian2d<X, Y>`.
/// - `lines`: A slice of `ChartVerticalLine` structures defining the x-coordinate, y-range,
///   style, and label for each vertical line to be drawn.
///
/// # Returns
///
/// - Returns a `Result`:
///     - `Ok(())` on success, indicating that all vertical lines were drawn without errors.
///     - `Err(Box<dyn Error>)` if an error occurs during the drawing operations.
///
/// # Constraints
///
/// - The `X` and `Y` types, as well as their associated value types (`X::ValueType` and `Y::ValueType`),
///   must support cloning (`Clone`) and addition (`Add<f64>`). This enables the function to compute
///   positions and offsets for labels.
/// - Value types for `X` and `Y` must be displayable (`std::fmt::Display`) to render labels correctly
///   on the chart.
/// - Drawing backend errors must be composable as `'static` to integrate seamlessly with the function's
///   return type.
///
/// # Behavior
///
/// 1. **Line Drawing**: For each vertical line in the input slice, a line is drawn from the bottom
///    to the top of the specified y-range using `LineSeries`.
/// 2. **Label Placement**: For each line, a `Text` entity displaying the label is rendered at the
///    specified offset relative to the x-coordinate and the upper y-coordinate of the line.
/// 3. Styling: Uses attributes from `ChartVerticalLine` (`line_style`, `font_size`, and colors)
///    to apply custom styles to the lines and labels.
///
pub fn draw_vertical_lines_on_chart<DB: DrawingBackend, X, Y>(
    ctx: &mut ChartContext<DB, Cartesian2d<X, Y>>,
    lines: &[ChartVerticalLine<X::ValueType, Y::ValueType>],
) -> Result<(), Box<dyn Error>>
where
    X: Ranged,
    Y: Ranged,
    X::ValueType: Clone + Add<f64, Output = X::ValueType>,
    Y::ValueType: Clone + Add<f64, Output = Y::ValueType>,
    <X as Ranged>::ValueType: 'static,
    <Y as Ranged>::ValueType: 'static,
    <DB as DrawingBackend>::ErrorType: 'static,
    <X as Ranged>::ValueType: std::fmt::Display,
    <Y as Ranged>::ValueType: std::fmt::Display,
{
    for line in lines {
        ctx.draw_series(LineSeries::new(
            vec![
                (line.x_coordinate.clone(), line.y_range.0.clone()),
                (line.x_coordinate.clone(), line.y_range.1.clone()),
            ],
            line.line_style,
        ))?;
    }

    for line in lines {
        let (x, y) = (line.x_coordinate.clone(), line.y_range.1.clone());
        let (offset_x, offset_y) = line.label_offset;
        let label_pos = (x.add(offset_x), y.add(offset_y));

        ctx.draw_series(std::iter::once(Text::new(
            line.label.clone(),
            label_pos,
            ("sans-serif", line.font_size)
                .into_font()
                .color(&line.label_color),
        )))?;
    }
    Ok(())
}

/// Creates a random, visually distinguishable color.
///
/// Uses HSL color space to generate colors with:
/// - Random hue (0-360)
/// - High saturation (60-90%)
/// - Medium lightness (35-65%)
///
/// This approach helps ensure colors are:
/// 1. Visually distinct from each other
/// 2. Saturated enough to be visible
/// 3. Neither too dark nor too light
pub fn random_color() -> RGBColor {
    let mut thread_rng = rng();

    // Generate HSL values
    let h = thread_rng.random_range(0.0..360.0); // Hue: Full range for maximum variety
    let s = thread_rng.random_range(0.6..0.9); // Saturation: 60-90% for vivid colors
    let l = thread_rng.random_range(0.35..0.65); // Lightness: 35-65% for medium brightness

    // Convert HSL to RGB
    let rgb = hsl_to_rgb(h, s, l);
    RGBColor(rgb.0, rgb.1, rgb.2)
}

/// Converts HSL color values to RGB.
///
/// Parameters:
/// - h: Hue (0-360)
/// - s: Saturation (0-1)
/// - l: Lightness (0-1)
///
/// Returns:
/// Tuple of (r, g, b) where each value is 0-255
fn hsl_to_rgb(h: f64, s: f64, l: f64) -> (u8, u8, u8) {
    // Helper function for hue to RGB conversion
    let hue_to_rgb = |p: f64, q: f64, mut t: f64| -> f64 {
        if t < 0.0 {
            t += 1.0
        }
        if t > 1.0 {
            t -= 1.0
        }

        if t < 1.0 / 6.0 {
            return p + (q - p) * 6.0 * t;
        }
        if t < 1.0 / 2.0 {
            return q;
        }
        if t < 2.0 / 3.0 {
            return p + (q - p) * (2.0 / 3.0 - t) * 6.0;
        }
        p
    };

    // Edge case: no saturation means a gray
    if s == 0.0 {
        let gray = (l * 255.0) as u8;
        return (gray, gray, gray);
    }

    // Calculate helper values
    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    // Convert hue to RGB channels
    let r = hue_to_rgb(p, q, (h / 360.0) + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h / 360.0);
    let b = hue_to_rgb(p, q, (h / 360.0) - 1.0 / 3.0);

    // Convert to 0-255 range
    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

#[cfg(test)]
mod tests_calculate_axis_range {
    use super::*;
    use crate::pos;

    #[test]

    fn test_calculate_axis_range() {
        let x_data = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0), pos!(5.0)];
        let y_data = vec![-10.0, -5.0, 0.0, 5.0, 10.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data, Some(1.2));

        assert_eq!(max_x, 5.0);
        assert_eq!(min_x, 1.0);
        assert!(max_y > 10.0);
        assert!(min_y < -10.0);
    }

    #[test]

    fn test_calculate_axis_range_single_value() {
        let x_data = vec![pos!(1.0)];
        let y_data = vec![0.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data, Some(1.2));

        assert_eq!(max_x, pos!(1.0));
        assert_eq!(min_x, pos!(1.0));
        assert_eq!(max_y, 0.0);
        assert_eq!(min_y, 0.0);
    }

    #[test]

    fn test_calculate_axis_range_zero_values() {
        let x_data = vec![Positive::ZERO, Positive::ZERO, Positive::ZERO];
        let y_data = vec![0.0, 0.0, 0.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data, Some(1.2));

        assert_eq!(max_x, Positive::ZERO);
        assert_eq!(min_x, Positive::ZERO);
        assert_eq!(max_y, 0.0);
        assert_eq!(min_y, 0.0);
    }

    #[test]

    fn test_calculate_axis_range_large_values() {
        let x_data = vec![pos!(1e6), pos!(2e6), pos!(3e6)];
        let y_data = vec![1e9, 2e9, 3e9];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data, Some(1.2));

        assert_eq!(max_x, 3e6);
        assert_eq!(min_x, 1e6);
        assert!(max_y > 3e9);
        assert!(min_y < 1e9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Positive;
    use crate::pos;
    use crate::visualization::model::LabelOffsetType;
    use plotters::style::RGBColor;
    use rust_decimal::Decimal;
    use std::error::Error;

    struct MockGraph;

    impl Profit for MockGraph {
        fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
            Ok((price * 2.0).to_dec())
        }
    }

    impl Graph for MockGraph {
        fn title(&self) -> String {
            "Mock Graph".to_string()
        }

        fn get_x_values(&self) -> Vec<Positive> {
            vec![Positive::ZERO, pos!(50.0), pos!(100.0)]
        }

        fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
            vec![ChartVerticalLine {
                x_coordinate: 50.0,
                y_range: (-100.0, 100.0),
                label: "Test Line".to_string(),
                label_offset: (0.0, 0.0),
                line_color: RGBColor(0, 0, 0),
                label_color: RGBColor(0, 0, 0),
                line_style: plotters::style::ShapeStyle::from(&RGBColor(0, 0, 0)).stroke_width(1),
                font_size: 12,
            }]
        }

        fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
            vec![ChartPoint {
                coordinates: (50.0, 0.0),
                label: "Test Point".to_string(),
                label_offset: LabelOffsetType::Relative(0.0, 0.0),
                point_color: RGBColor(0, 0, 0),
                label_color: RGBColor(0, 0, 0),
                point_size: 5,
                font_size: 12,
            }]
        }
    }

    #[test]

    fn test_graph_trait() -> Result<(), Box<dyn Error>> {
        let mock_graph = MockGraph;
        mock_graph.graph(
            GraphBackend::Bitmap {
                file_path: "test_graph.png",
                size: (800, 600),
            },
            20,
        )?;
        std::fs::remove_file("test_graph.png")?;
        Ok(())
    }

    #[test]
    fn test_get_values() {
        let mock_graph = MockGraph;
        let values = mock_graph.get_y_values();
        assert_eq!(values, vec![0.0, 100.0, 200.0]);
    }

    #[test]

    fn test_default_get_vertical_lines() {
        struct DefaultGraph;
        impl Profit for DefaultGraph {
            fn calculate_profit_at(&self, _: Positive) -> Result<Decimal, Box<dyn Error>> {
                Ok(Decimal::ZERO)
            }
        }
        impl Graph for DefaultGraph {
            fn title(&self) -> String {
                "Default".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                unimplemented!()
            }
        }
        let graph = DefaultGraph;
        graph.get_vertical_lines();
    }

    #[test]

    fn test_default_get_points() {
        struct DefaultGraph;
        impl Profit for DefaultGraph {
            fn calculate_profit_at(&self, _: Positive) -> Result<Decimal, Box<dyn Error>> {
                Ok(Decimal::ZERO)
            }
        }
        impl Graph for DefaultGraph {
            fn title(&self) -> String {
                "Default".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                unimplemented!()
            }
        }
        let graph = DefaultGraph;
        graph.get_points();
    }

    #[test]

    fn test_draw_points_on_chart() -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[test]

    fn test_draw_vertical_lines_on_chart() -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    #[test]

    fn test_calculate_axis_range_empty() {
        let x_data: Vec<Positive> = vec![];
        let y_data: Vec<f64> = vec![];
        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data, Some(1.2));
        assert_eq!(min_x, Positive::ZERO);
        assert_eq!(max_x, Positive::INFINITY);
        assert_eq!(min_y, f64::NEG_INFINITY);
        assert_eq!(max_y, f64::INFINITY);
    }
}

#[cfg(test)]
mod tests_extended {
    use super::*;
    use crate::pos;
    use crate::visualization::model::LabelOffsetType;
    use crate::visualization::model::{ChartPoint, ChartVerticalLine};

    use plotters::prelude::PathElement;
    use plotters::style::RGBColor;
    use rust_decimal::Decimal;

    #[allow(dead_code)]
    struct MockGraph;

    impl Profit for MockGraph {
        fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
            Ok((price * 2.0).to_dec())
        }
    }

    impl Graph for MockGraph {
        fn title(&self) -> String {
            "Mock Graph".to_string()
        }

        fn get_x_values(&self) -> Vec<Positive> {
            vec![pos!(50.0)]
        }

        fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
            vec![ChartVerticalLine {
                x_coordinate: 50.0,
                y_range: (-100.0, 100.0),
                label: "Test Line".to_string(),
                label_offset: (0.0, 0.0),
                line_color: RGBColor(0, 0, 0),
                label_color: RGBColor(0, 0, 0),
                line_style: plotters::style::ShapeStyle::from(&RGBColor(0, 0, 0)).stroke_width(1),
                font_size: 12,
            }]
        }

        fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
            vec![ChartPoint {
                coordinates: (50.0, 0.0),
                label: "Test Point".to_string(),
                label_offset: LabelOffsetType::Relative(0.0, 0.0),
                point_color: RGBColor(0, 0, 0),
                label_color: RGBColor(0, 0, 0),
                point_size: 5,
                font_size: 12,
            }]
        }
    }

    #[test]
    fn test_graph_with_empty_data() -> Result<(), Box<dyn Error>> {
        let mock_graph = MockGraph;
        let result = mock_graph.graph(
            GraphBackend::Bitmap {
                file_path: "test_empty_graph.png",
                size: (800, 600),
            },
            20,
        );

        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No valid values to plot")
        );
        Ok(())
    }

    #[test]
    fn test_graph_with_single_point() -> Result<(), Box<dyn Error>> {
        let mock_graph = MockGraph;
        mock_graph.graph(
            GraphBackend::Bitmap {
                file_path: "test_single_point.png",
                size: (800, 600),
            },
            20,
        )?;
        std::fs::remove_file("test_single_point.png")?;
        Ok(())
    }

    #[test]
    fn test_graph_with_negative_values() -> Result<(), Box<dyn Error>> {
        struct NegativeGraph;

        impl Profit for NegativeGraph {
            fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
                Ok(price.to_dec() * Decimal::from(-1))
            }
        }

        impl Graph for NegativeGraph {
            fn title(&self) -> String {
                "Negative Graph".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                unimplemented!()
            }
        }

        let graph = NegativeGraph;
        graph.graph(
            GraphBackend::Bitmap {
                file_path: "test_negative.png",
                size: (800, 600),
            },
            20,
        )?;
        std::fs::remove_file("test_negative.png")?;
        Ok(())
    }

    #[test]
    fn test_multiple_vertical_lines() -> Result<(), Box<dyn Error>> {
        struct MultiLineGraph;

        impl Profit for MultiLineGraph {
            fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
                Ok(price.to_dec())
            }
        }

        impl Graph for MultiLineGraph {
            fn title(&self) -> String {
                "Multi Line Graph".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                unimplemented!()
            }

            fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
                vec![
                    ChartVerticalLine {
                        x_coordinate: 25.0,
                        y_range: (-50.0, 50.0),
                        label: "Line 1".to_string(),
                        label_offset: (5.0, 5.0),
                        line_color: RGBColor(0, 0, 0),
                        label_color: RGBColor(0, 0, 0),
                        line_style: plotters::style::ShapeStyle::from(&RGBColor(0, 0, 0))
                            .stroke_width(1),
                        font_size: 12,
                    },
                    ChartVerticalLine {
                        x_coordinate: 75.0,
                        y_range: (-50.0, 50.0),
                        label: "Line 2".to_string(),
                        label_offset: (-5.0, -5.0),
                        line_color: RGBColor(0, 0, 0),
                        label_color: RGBColor(0, 0, 0),
                        line_style: plotters::style::ShapeStyle::from(&RGBColor(0, 0, 0))
                            .stroke_width(1),
                        font_size: 12,
                    },
                ]
            }
        }

        let graph = MultiLineGraph;
        graph.graph(
            GraphBackend::Bitmap {
                file_path: "test_multi_line.png",
                size: (800, 600),
            },
            20,
        )?;
        std::fs::remove_file("test_multi_line.png")?;
        Ok(())
    }

    #[test]
    fn test_multiple_points() -> Result<(), Box<dyn Error>> {
        struct MultiPointGraph;

        impl Profit for MultiPointGraph {
            fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
                Ok(price.to_dec())
            }
        }

        impl Graph for MultiPointGraph {
            fn title(&self) -> String {
                "Multi Point Graph".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                vec![pos!(0.0), pos!(50.0), pos!(100.0)]
            }

            fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
                vec![
                    ChartPoint {
                        coordinates: (25.0, 25.0),
                        label: "Point 1".to_string(),
                        label_offset: LabelOffsetType::Absolute(5.0, 5.0),
                        point_color: RGBColor(0, 0, 0),
                        label_color: RGBColor(0, 0, 0),
                        point_size: 5,
                        font_size: 12,
                    },
                    ChartPoint {
                        coordinates: (75.0, 75.0),
                        label: "Point 2".to_string(),
                        label_offset: LabelOffsetType::Absolute(-5.0, -5.0),
                        point_color: RGBColor(0, 0, 0),
                        label_color: RGBColor(0, 0, 0),
                        point_size: 5,
                        font_size: 12,
                    },
                ]
            }
        }

        let graph = MultiPointGraph;
        graph.graph(
            GraphBackend::Bitmap {
                file_path: "test_multi_point.png",
                size: (800, 600),
            },
            20,
        )?;
        std::fs::remove_file("test_multi_point.png")?;
        Ok(())
    }

    #[test]
    fn test_get_values_error_handling() {
        struct ErrorGraph;

        impl Profit for ErrorGraph {
            fn calculate_profit_at(&self, _: Positive) -> Result<Decimal, Box<dyn Error>> {
                Err("Test error".into())
            }
        }

        impl Graph for ErrorGraph {
            fn title(&self) -> String {
                "Error Graph".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                vec![pos!(1.0), pos!(2.0), pos!(3.0)]
            }
        }

        let graph = ErrorGraph;
        let values = graph.get_y_values();

        assert!(values.is_empty());

        struct MixedErrorGraph;

        impl Profit for MixedErrorGraph {
            fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
                if price > pos!(1.5) {
                    Err("Test error".into())
                } else {
                    Ok(price.to_dec())
                }
            }
        }

        impl Graph for MixedErrorGraph {
            fn title(&self) -> String {
                "Mixed Error Graph".to_string()
            }

            fn get_x_values(&self) -> Vec<Positive> {
                unimplemented!()
            }
        }

        let mixed_graph = MixedErrorGraph;
        let values = mixed_graph.get_y_values();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0], 1.0);
    }

    #[test]
    fn test_custom_canvas_sizes() -> Result<(), Box<dyn Error>> {
        let mock_graph = MockGraph;
        let sizes = vec![(400, 300), (1920, 1080), (300, 300)];

        for (width, height) in sizes {
            mock_graph.graph(
                GraphBackend::Bitmap {
                    file_path: &format!("test_size_{}x{}.png", width, height),
                    size: (width, height),
                },
                20,
            )?;
            std::fs::remove_file(format!("test_size_{}x{}.png", width, height))?;
        }

        Ok(())
    }

    #[test]
    fn test_bitmap_backend_initialization() {
        let backend = GraphBackend::Bitmap {
            file_path: "test_chart.png",
            size: (800, 600),
        };

        let GraphBackend::Bitmap { file_path, size } = backend;
        let root = BitMapBackend::new(&file_path, size).into_drawing_area();
        assert!(root.fill(&WHITE).is_ok());
        drop(root);
        assert!(std::fs::remove_file("test_chart.png").is_ok());
    }

    #[test]
    fn test_chart_initialization() -> Result<(), Box<dyn Error>> {
        let root = BitMapBackend::new("test_chart_next.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let title = "Test Chart";
        let title_size = 20;
        let min_x_value = 0.0;
        let max_x_value = 100.0;
        let min_y_value = 0.0;
        let max_y_value = 100.0;

        let chart = build_chart!(
            &root,
            title,
            title_size,
            min_x_value,
            max_x_value,
            min_y_value,
            max_y_value
        );

        assert!(chart.plotting_area().dim_in_pixel().0 > 0);
        assert!(chart.plotting_area().dim_in_pixel().1 > 0);

        drop(chart);
        drop(root);
        assert!(std::fs::remove_file("test_chart_next.png").is_ok());

        Ok(())
    }

    #[test]
    fn test_point_rendering() {
        let root = BitMapBackend::new("test_chart_points.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let points = vec![ChartPoint {
            coordinates: (10.0, 20.0),
            point_size: 5,
            point_color: DARK_GREEN,
            label: "Point A".to_string(),
            label_offset: LabelOffsetType::Relative(2.0, 2.0),
            font_size: 12,
            label_color: DARK_GREEN,
        }];

        for point in points {
            // Convertir coordenadas para el círculo
            let (x, y) = (point.coordinates.0 as i32, point.coordinates.1 as i32);
            root.draw(&Circle::new(
                (x, y),
                point.point_size,
                point.point_color.filled(),
            ))
            .unwrap();

            // Convertir coordenadas para el texto
            let label_pos =
                if let LabelOffsetType::Relative(offset_x, offset_y) = point.label_offset {
                    (
                        (point.coordinates.0 + offset_x) as i32,
                        (point.coordinates.1 + offset_y) as i32,
                    )
                } else {
                    (x, y)
                };

            root.draw(&Text::new(
                point.label.clone(),
                label_pos,
                ("sans-serif", point.font_size)
                    .into_font()
                    .color(&point.label_color),
            ))
            .unwrap();
        }

        drop(root);
        assert!(
            std::fs::remove_file("test_chart_points.png").is_ok(),
            "Failed to clean up test file"
        );
    }

    #[test]
    fn test_line_rendering() {
        let root = BitMapBackend::new("test_chart_lines.png", (800, 600)).into_drawing_area();
        root.fill(&WHITE).unwrap();

        let lines = vec![ChartVerticalLine {
            x_coordinate: 50.0,
            y_range: (0.0, 100.0),
            line_style: DARK_RED.stroke_width(2),
            label: "Line A".to_string(),
            label_offset: (5.0, 5.0),
            font_size: 12,
            label_color: DARK_RED,
            line_color: Default::default(),
        }];

        for line in lines {
            let x = line.x_coordinate as i32;
            let y0 = line.y_range.0 as i32;
            let y1 = line.y_range.1 as i32;

            root.draw(&PathElement::new(vec![(x, y0), (x, y1)], line.line_style))
                .unwrap();

            let label_x = (line.x_coordinate + line.label_offset.0) as i32;
            let label_y = (line.y_range.1 + line.label_offset.1) as i32;

            root.draw(&Text::new(
                line.label.clone(),
                (label_x, label_y),
                ("sans-serif", line.font_size)
                    .into_font()
                    .color(&line.label_color),
            ))
            .unwrap();
        }

        drop(root);
        assert!(
            std::fs::remove_file("test_chart_lines.png").is_ok(),
            "Failed to clean up test file"
        );
    }
}
