/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 20/8/24
******************************************************************************/
use crate::constants::{DARK_GREEN, DARK_RED};
use crate::{
    build_chart, configure_chart_and_draw_mesh, create_drawing_area, draw_line_segments,
    draw_points_with_labels, draw_vertical_lines_and_labels,
};
use plotters::backend::BitMapBackend;
use plotters::chart::ChartBuilder;
use plotters::element::{Circle, EmptyElement, Text};
use plotters::prelude::{IntoDrawingArea, IntoFont, LineSeries, PointSeries, BLACK, WHITE};
use std::error::Error;

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

#[macro_export]
macro_rules! draw_vertical_lines_and_labels {
    ($chart:expr, $vertical_lines:expr, $min_y_value:expr, $max_y_value:expr, $BLACK:expr, $label_position:expr) => {
        for (label, line) in $vertical_lines {
            $chart.draw_series(LineSeries::new(
                vec![(line, $min_y_value), (line, $max_y_value)],
                &$BLACK,
            ))?;

            $chart.draw_series(PointSeries::of_element(
                vec![(line, $max_y_value)],
                5,
                &$BLACK,
                &|coord, _size, _style| {
                    EmptyElement::at(coord)
                        + Text::new(
                            format!("{}: {:.2}", label, line),
                            $label_position,
                            ("sans-serif", 15).into_font(),
                        )
                },
            ))?;
        }
    };
}

#[macro_export]
macro_rules! draw_points_with_labels {
    ($chart:expr, $x_axis_data:expr, $y_axis_data:expr, $dark_green:expr, $dark_red:expr, $label_interval:expr) => {
        for (i, (&price, &value)) in $x_axis_data.iter().zip($y_axis_data.iter()).enumerate() {
            let point_color = if value > 0.0 {
                &$dark_green
            } else {
                &$dark_red
            };
            let label_offset = if value >= 0.0 { (20, 0) } else { (-20, -20) };
            let size = 3;

            if value != 0.0 {
            $chart.draw_series(PointSeries::of_element(
                vec![(price, value)],
                size,
                point_color,
                &|coord, size, style| {
                    let element =
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled());

                    if i % $label_interval == 0 {
                        element
                            + Text::new(
                                format!("{:.2}", value),
                                (label_offset.0, label_offset.1),
                                ("sans-serif", 15).into_font(),
                            )
                    } else {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), 0, style.filled())
                            + Text::new(
                                String::new(),
                                (label_offset.0, label_offset.1),
                                ("sans-serif", 15).into_font(),
                            )
                    }
                },
            ))?;
            }
        }
    };
}

pub trait Graph {
    fn graph(&self, x_axis_data: &[f64], file_path: &str) -> Result<(), Box<dyn Error>> {
        // Generate profit values for each price in the data vector
        let y_axis_data: Vec<f64> = self.get_values(x_axis_data);

        // Determine the range for the X and Y axes
        let (max_x_value, min_x_value, max_y_value, min_y_value) =
            calculate_axis_range(x_axis_data, &y_axis_data);

        // Set up the drawing area with a 1200x800 pixel canvas
        let root = create_drawing_area!(file_path, 1200, 800);

        let mut chart = build_chart!(
            &root,
            self.title(),
            15,
            min_x_value,
            max_x_value,
            min_y_value,
            max_y_value
        );

        configure_chart_and_draw_mesh!(chart, 20, 20, min_x_value, max_x_value);
        draw_line_segments!(chart, x_axis_data, y_axis_data, DARK_GREEN, DARK_RED);

        draw_vertical_lines_and_labels!(
            chart,
            self.get_vertical_lines(),
            min_y_value,
            max_y_value,
            BLACK,
            (10, 30)
        );
        draw_points_with_labels!(chart, x_axis_data, y_axis_data, DARK_GREEN, DARK_RED, 10);

        root.present()?;
        Ok(())
    }

    fn title(&self) -> String;

    // fn get_values(&self, data: &[f64]) -> Vec<f64>;
    fn get_values(&self, data: &[f64]) -> Vec<f64>;

    fn get_vertical_lines(&self) -> Vec<(String, f64)>;
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
    // Determine the range for the X and Y axes
    let max_x_value = x_axis_data
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let min_x_value = x_axis_data.iter().cloned().fold(f64::INFINITY, f64::min);
    let max_y_temp = y_axis_data
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let min_y_temp = y_axis_data.iter().cloned().fold(f64::INFINITY, f64::min);

    let adjusted_max_profit = (max_y_temp * 1.2 - max_y_temp).abs();
    let adjusted_min_profit = (min_y_temp * 1.2 - min_y_temp).abs();

    let margin_value = adjusted_max_profit.max(adjusted_min_profit);

    let max_y_value = max_y_temp + margin_value;
    let min_y_value = min_y_temp - margin_value;

    (max_x_value, min_x_value, max_y_value, min_y_value)
}
