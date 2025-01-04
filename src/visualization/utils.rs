/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use crate::constants::{DARK_GREEN, DARK_RED};
use crate::pricing::payoff::Profit;
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::{create_drawing_area, pos, Positive};
use plotters::backend::BitMapBackend;
use plotters::element::{Circle, Text};
use plotters::prelude::ChartBuilder;
use plotters::prelude::BLACK;
use plotters::prelude::{
    Cartesian2d, ChartContext, Color, DrawingBackend, IntoDrawingArea, IntoFont, LineSeries,
    Ranged, WHITE,
};
use std::error::Error;
use std::ops::Add;
use num_traits::ToPrimitive;

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
            .disable_mesh() // Disable the mesh grid
            .x_labels($x_labels)
            .y_labels($y_labels)
            .draw()?;
        // Draw a horizontal line at y = 0
        $chart.draw_series(LineSeries::new(vec![($min_x, 0.0), ($max_x, 0.0)], &BLACK))?;
    }};
}

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

                $chart.draw_series(LineSeries::new(points, color))?;
            }
            last_point = Some((price, value));
        }
        let _ = Ok::<(), Box<dyn std::error::Error>>(());
    }};
}

pub trait Graph: Profit {
    fn graph(
        &self,
        x_axis_data: &[Positive], // TODO: it should be Optional
        file_path: &str,
        title_size: u32,         // 15
        canvas_size: (u32, u32), // (1200, 800)
    ) -> Result<(), Box<dyn Error>> {
        // Generate profit values for each price in the data vector
        let y_axis_data: Vec<f64> = self.get_values(x_axis_data);

        let x_axis_point = if x_axis_data.is_empty() {
            &mut (0..y_axis_data.len())
                .map(|i| pos!(i as f64))
                .collect::<Vec<Positive>>()
        } else {
            x_axis_data
        };

        // Determine the range for the X and Y axes
        let (max_x_value, min_x_value, max_y_value, min_y_value) =
            calculate_axis_range(x_axis_point, &y_axis_data);

        // Set up the drawing area with a 1200x800 pixel canvas
        let root = create_drawing_area!(file_path, canvas_size.0, canvas_size.1);

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
        draw_line_segments!(chart, x_axis_point, y_axis_data, DARK_GREEN, DARK_RED);

        draw_points_on_chart(&mut chart, &self.get_points())?;
        draw_vertical_lines_on_chart(&mut chart, &self.get_vertical_lines())?;
        root.present()?;
        Ok(())
    }

    fn title(&self) -> String;

    fn get_values(&self, data: &[Positive]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price).unwrap().to_f64().unwrap())
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        Vec::new()
    }

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
        let x_data = vec![pos!(1.0), pos!(2.0), pos!(3.0), pos!(4.0), pos!(5.0)];
        let y_data = vec![-10.0, -5.0, 0.0, 5.0, 10.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, 5.0);
        assert_eq!(min_x, 1.0);
        assert!(max_y > 10.0);
        assert!(min_y < -10.0);
    }

    #[test]
    fn test_calculate_axis_range_single_value() {
        let x_data = vec![pos!(1.0)];
        let y_data = vec![0.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, pos!(1.0));
        assert_eq!(min_x, pos!(1.0));
        assert_eq!(max_y, 0.0);
        assert_eq!(min_y, 0.0);
    }

    #[test]
    fn test_calculate_axis_range_zero_values() {
        let x_data = vec![pos!(0.0), pos!(0.0), pos!(0.0)];
        let y_data = vec![0.0, 0.0, 0.0];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, Positive::ZERO);
        assert_eq!(min_x, Positive::ZERO);
        assert_eq!(max_y, 0.0);
        assert_eq!(min_y, 0.0);
    }

    #[test]
    fn test_calculate_axis_range_large_values() {
        let x_data = vec![pos!(1e6), pos!(2e6), pos!(3e6)];
        let y_data = vec![1e9, 2e9, 3e9];

        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);

        assert_eq!(max_x, 3e6);
        assert_eq!(min_x, 1e6);
        assert!(max_y > 3e9);
        assert!(min_y < 1e9);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;
    use crate::visualization::model::LabelOffsetType;
    use crate::Positive;
    use plotters::style::RGBColor;
    use std::error::Error;
    use rust_decimal::Decimal;

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
        let x_axis_data = vec![pos!(0.0), pos!(50.0), pos!(100.0)];
        mock_graph.graph(&x_axis_data, "test_graph.png", 20, (800, 600))?;
        std::fs::remove_file("test_graph.png")?;
        Ok(())
    }

    #[test]
    fn test_get_values() {
        let mock_graph = MockGraph;
        let x_axis_data = vec![pos!(0.0), pos!(50.0), pos!(100.0)];
        let values = mock_graph.get_values(&x_axis_data);
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
        let (max_x, min_x, max_y, min_y) = calculate_axis_range(&x_data, &y_data);
        assert_eq!(min_x, Positive::ZERO);
        assert_eq!(max_x, Positive::INFINITY);
        assert_eq!(min_y, f64::NEG_INFINITY);
        assert_eq!(max_y, f64::INFINITY);
    }
}
