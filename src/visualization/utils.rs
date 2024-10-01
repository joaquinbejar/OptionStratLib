/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use crate::constants::{DARK_GREEN, DARK_RED};
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::{build_chart, configure_chart_and_draw_mesh, create_drawing_area, draw_line_segments};
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::element::{Circle, Text};
use plotters::prelude::{
    Cartesian2d, ChartContext, Color, DrawingBackend, IntoDrawingArea, IntoFont, LineSeries,
    Ranged, BLACK, WHITE,
};
use std::error::Error;
use std::ops::Add;

#[macro_export]
macro_rules! create_drawing_area {
    ($file_path:expr, $width:expr, $height:expr) => {{
        let root = BitMapBackend::new($file_path, ($width, $height)).into_drawing_area();
        root.fill(&WHITE)?;
        root
    }};
}

#[macro_export]
macro_rules! build_chart {
    ($root:expr, $title:expr, $title_size:expr, $min_x:expr, $max_x:expr, $min_y:expr, $max_y:expr) => {
        ChartBuilder::on($root)
            .caption($title, ("sans-serif", $title_size))
            .margin(10)
            .top_x_label_area_size(40)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .right_y_label_area_size(60)
            .build_cartesian_2d($min_x..$max_x, $min_y..$max_y)?
    };
}

#[macro_export]
macro_rules! configure_chart_and_draw_mesh {
    ($chart:expr, $x_labels:expr, $y_labels:expr, $min_x:expr, $max_x:expr) => {{
        // Configure and draw the mesh grid
        $chart
            .configure_mesh()
            .x_labels($x_labels)
            .y_labels($y_labels)
            .draw()?;

        // Draw a horizontal line at y = 0
        $chart.draw_series(LineSeries::new(vec![($min_x, 0.0), ($max_x, 0.0)], &BLACK))?;
    }};
}

#[macro_export]
macro_rules! draw_line_segments {
    ($chart:expr, $x_axis_data:expr, $y_axis_data:expr, $dark_green:expr, $dark_red:expr) => {
        let mut last_point = None;
        for (&price, &value) in $x_axis_data.iter().zip($y_axis_data.iter()) {
            if let Some((last_price, last_profit)) = last_point {
                let color = if value > 0.0 {
                    &$dark_green
                } else {
                    &$dark_red
                };
                $chart.draw_series(LineSeries::new(
                    vec![(last_price, last_profit), (price, value)],
                    color,
                ))?;
            }
            last_point = Some((price, value));
        }
    };
}

pub trait Graph {
    fn graph(
        &self,
        x_axis_data: &[f64],
        file_path: &str,
        title_size: u32,          // 15
        canvas_size: (u32, u32),  // (1200, 800)
        _label_coors: (i32, i32), // (10, 30)
        _label_interval: usize,   // 10
    ) -> Result<(), Box<dyn Error>> {
        // Generate profit values for each price in the data vector
        let y_axis_data: Vec<f64> = self.get_values(x_axis_data);

        // Determine the range for the X and Y axes
        let (max_x_value, min_x_value, max_y_value, min_y_value) =
            calculate_axis_range(x_axis_data, &y_axis_data);

        // Set up the drawing area with a 1200x800 pixel canvas
        let root = create_drawing_area!(file_path, canvas_size.0, canvas_size.1);

        let mut chart = build_chart!(
            &root,
            self.title(),
            title_size,
            min_x_value,
            max_x_value,
            min_y_value,
            max_y_value
        );

        configure_chart_and_draw_mesh!(chart, 20, 20, min_x_value, max_x_value);

        draw_line_segments!(chart, x_axis_data, y_axis_data, DARK_GREEN, DARK_RED);

        draw_points_on_chart(&mut chart, &self.get_points())?;
        draw_vertical_lines_on_chart(&mut chart, &self.get_vertical_lines())?;
        root.present()?;
        Ok(())
    }

    fn title(&self) -> String;

    fn get_values(&self, data: &[f64]) -> Vec<f64>;

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        panic!("Not implemented");
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        panic!("Not implemented");
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
    x_axis_data: &[f64],
    y_axis_data: &[f64],
) -> (f64, f64, f64, f64) {
    let (min_x_value, max_x_value) = x_axis_data.iter().fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(min_x, max_x), &value| (f64::min(min_x, value), f64::max(max_x, value)),
    );
    let (min_y_temp, max_y_temp) = y_axis_data.iter().fold(
        (f64::INFINITY, f64::NEG_INFINITY),
        |(min_y, max_y), &value| (f64::min(min_y, value), f64::max(max_y, value)),
    );
    let adjusted_max_profit = (max_y_temp * 1.2 - max_y_temp).abs();
    let adjusted_min_profit = (min_y_temp * 1.2 - min_y_temp).abs();
    let margin_value = adjusted_max_profit.max(adjusted_min_profit);
    let max_y_value = max_y_temp + margin_value;
    let min_y_value = min_y_temp - margin_value;
    (max_x_value, min_x_value, max_y_value, min_y_value)
}

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
        let (offset_x, offset_y) = point.label_offset;
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

#[cfg(test)]
mod tests_calculate_axis_range {
    use super::*;

    #[test]
    fn test_calculate_axis_range() {
        let x_data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y_data = vec![-10.0, -5.0, 0.0, 5.0, 10.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, 5.0);
        assert_eq!(min_x, 1.0);
        assert!(max_y > 10.0);
        assert!(min_y < -10.0);
    }

    #[test]
    fn test_calculate_axis_range_single_value() {
        let x_data = vec![1.0];
        let y_data = vec![0.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, 1.0);
        assert_eq!(min_x, 1.0);
        assert_eq!(max_y, 0.0);
        assert_eq!(min_y, 0.0);
    }

    #[test]
    fn test_calculate_axis_range_negative_values() {
        let x_data = vec![-5.0, -3.0, -1.0];
        let y_data = vec![-10.0, -20.0, -30.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, -1.0);
        assert_eq!(min_x, -5.0);
        assert!(max_y > -10.0);
        assert!(min_y < -30.0);
    }

    #[test]
    fn test_calculate_axis_range_zero_values() {
        let x_data = vec![0.0, 0.0, 0.0];
        let y_data = vec![0.0, 0.0, 0.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, 0.0);
        assert_eq!(min_x, 0.0);
        assert_eq!(max_y, 0.0);
        assert_eq!(min_y, 0.0);
    }

    #[test]
    fn test_calculate_axis_range_large_values() {
        let x_data = vec![1e6, 2e6, 3e6];
        let y_data = vec![1e9, 2e9, 3e9];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, 3e6);
        assert_eq!(min_x, 1e6);
        assert!(max_y > 3e9);
        assert!(min_y < 1e9);
    }
}
