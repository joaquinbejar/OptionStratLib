//! # Curve Visualization Module
//!
//! Provides a flexible plotting trait for mathematical curves and collections of curves.
//!
//! ## Features
//! - Generic plotting for single and multiple curves
//! - Customizable plot configuration
//! - Multiple output formats
//! - Error handling
//!
//! ## Usage Examples
//! ```rust
//! // Plot a single curve
//! use rust_decimal::Decimal;
//! use optionstratlib::curves::{Curve, Point2D};
//! let curve = Curve::new(vec![
//!             Point2D::new(Decimal::ZERO, Decimal::ZERO), // p11
//!             Point2D::new(Decimal::ONE, Decimal::ONE),   // p12
//!             Point2D::new(Decimal::ZERO, Decimal::ONE),  // p21
//!             Point2D::new(Decimal::ONE, Decimal::TWO),   // p22
//!         ]);
//! curve.plot()
//!     .title("Single Curve")
//!     .save("single_curve.png")?;
//!
//! // Plot multiple curves
//! let curve1 = curve.clone();
//! let curve2 = curve.clone();
//! let curve3 = curve.clone();
//! let curves = vec![curve1, curve2, curve3];
//! curves.plot()
//!     .title("Curve Comparison")
//!     .save("multiple_curves.png")?;
//! ```

use crate::curves::Curve;
use crate::curves::Point2D;
use crate::error::CurvesError;
use num_traits::ToPrimitive;
use plotters::prelude::*;
use rust_decimal::Decimal;
use std::path::Path;
use rust_decimal_macros::dec;

/// Plot configuration options
#[derive(Clone, Debug)]
pub struct PlotOptions {
    /// Plot title
    pub title: Option<String>,
    /// X-axis label
    pub x_label: Option<String>,
    /// Y-axis label
    pub y_label: Option<String>,
    /// Line colors for each curve
    pub line_colors: Option<Vec<RGBColor>>,
    /// Line width
    pub line_width: u32,
    /// Background color
    pub background_color: RGBColor,
    /// Plot width
    pub width: u32,
    /// Plot height
    pub height: u32,
}

impl PlotOptions {
    /// Default color palette for multiple curves
    fn default_colors() -> Vec<RGBColor> {
        vec![
            RGBColor(0, 0, 255),   // Blue
            RGBColor(255, 0, 0),   // Red
            RGBColor(0, 255, 0),   // Green
            RGBColor(255, 165, 0), // Orange
            RGBColor(128, 0, 128), // Purple
        ]
    }
}

impl Default for PlotOptions {
    fn default() -> Self {
        PlotOptions {
            title: None,
            x_label: None,
            y_label: None,
            line_colors: None,
            line_width: 2,
            background_color: RGBColor(255, 255, 255), // White
            width: 800,  // Dimensión por defecto
            height: 600, // Dimensión por defecto
        }
    }
}

/// Trait for plotting curves
pub trait Plottable {
    /// Creates a plot builder
    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized;
}

/// Plot Builder for configurable curve visualization
pub struct PlotBuilder<T: Plottable> {
    /// Data to be plotted
    data: T,
    /// Plot configuration options
    options: PlotOptions,
}

impl<T: Plottable> PlotBuilder<T> {
    /// Set plot title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.options.title = Some(title.into());
        self
    }

    /// Set x-axis label
    pub fn x_label(mut self, label: impl Into<String>) -> Self {
        self.options.x_label = Some(label.into());
        self
    }

    /// Set y-axis label
    pub fn y_label(mut self, label: impl Into<String>) -> Self {
        self.options.y_label = Some(label.into());
        self
    }

    /// Set line colors
    pub fn line_colors(mut self, colors: Vec<RGBColor>) -> Self {
        self.options.line_colors = Some(colors);
        self
    }

    /// Set line width
    pub fn line_width(mut self, width: u32) -> Self {
        self.options.line_width = width;
        self
    }

    /// Set plot dimensions
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.options.width = width;
        self.options.height = height;
        self
    }

    pub fn save(self, path: impl AsRef<Path>) -> Result<(), CurvesError>
    where
        Self: PlotBuilderExt<T>,
    {
        PlotBuilderExt::save(self, path)
    }
}

/// Plottable implementation for single Curve
impl Plottable for Curve {
    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized,
    {
        PlotBuilder {
            data: self.clone(),
            options: PlotOptions::default(),
        }
    }
}

/// Plottable implementation for Vec<Curve>
impl Plottable for Vec<Curve> {
    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized,
    {
        PlotBuilder {
            data: self.clone(),
            options: PlotOptions::default(),
        }
    }
}

/// Plotting extension methods
pub trait PlotBuilderExt<T: Plottable> {
    /// Save plot to file
    fn save(self, path: impl AsRef<Path>) -> Result<(), CurvesError>;
}

/// Plotting implementation for single Curve
impl PlotBuilderExt<Curve> for PlotBuilder<Curve> {
    fn save(self, path: impl AsRef<Path>) -> Result<(), CurvesError> {
        // Convert points to f64
        let points: Vec<(f64, f64)> = self
            .data
            .points
            .iter()
            .map(|p| (p.x.to_f64().unwrap_or(0.0), p.y.to_f64().unwrap_or(0.0)))
            .collect();

        // Determine plot range
        let x_min = points.iter().map(|p| p.0).fold(f64::INFINITY, f64::min);
        let x_max = points.iter().map(|p| p.0).fold(f64::NEG_INFINITY, f64::max);
        let y_min = points.iter().map(|p| p.1).fold(f64::INFINITY, f64::min);
        let y_max = points.iter().map(|p| p.1).fold(f64::NEG_INFINITY, f64::max);

        // Create drawing area
        let root = BitMapBackend::new(path.as_ref(), (self.options.width, self.options.height))
            .into_drawing_area();

        root.fill(&self.options.background_color)
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                self.options.title.unwrap_or_default(),
                ("Arial", 30).into_font(),
            )
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        // Configure axes
        chart
            .configure_mesh()
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc(self.options.x_label.as_deref().unwrap_or("X"))
            .y_desc(self.options.y_label.as_deref().unwrap_or("Y"))
            .draw()
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        // Draw the curve
        let color = self
            .options
            .line_colors
            .unwrap_or_else(|| vec![PlotOptions::default_colors()[0]])
            .first()
            .cloned()
            .unwrap_or(RGBColor(0, 0, 255));

        chart
            .draw_series(LineSeries::new(
                points,
                color.stroke_width(self.options.line_width),
            ))
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        root.present().map_err(|e| CurvesError::StdError {
            reason: e.to_string(),
        })?;

        Ok(())
    }
}

/// Plotting implementation for Vec<Curve>
impl PlotBuilderExt<Vec<Curve>> for PlotBuilder<Vec<Curve>> {
    fn save(self, path: impl AsRef<Path>) -> Result<(), CurvesError> {

        // Prepare all curve points
        let all_curve_points: Vec<Vec<(f64, f64)>> = self
            .data
            .iter()
            .map(|curve| {
                curve
                    .points
                    .iter()
                    .map(|p| (p.x.to_f64().unwrap_or(0.0), p.y.to_f64().unwrap_or(0.0)))
                    .collect()
            })
            .collect();

        // Determine overall plot range
        let x_min = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.0))
            .fold(f64::INFINITY, f64::min);
        let x_max = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.0))
            .fold(f64::NEG_INFINITY, f64::max);
        let y_min = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.1))
            .fold(f64::INFINITY, f64::min);
        let y_max = all_curve_points
            .iter()
            .flat_map(|points| points.iter().map(|p| p.1))
            .fold(f64::NEG_INFINITY, f64::max);

        // Create drawing area
        let root = BitMapBackend::new(path.as_ref(), (self.options.width, self.options.height))
            .into_drawing_area();

        root.fill(&self.options.background_color)
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        let mut chart = ChartBuilder::on(&root)
            .caption(
                self.options.title.unwrap_or_default(),
                ("Arial", 30).into_font(),
            )
            .margin(5)
            .x_label_area_size(30)
            .y_label_area_size(30)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        // Configure axes
        chart
            .configure_mesh()
            .x_label_formatter(&|v| format!("{:.2}", v))
            .y_label_formatter(&|v| format!("{:.2}", v))
            .x_desc(self.options.x_label.as_deref().unwrap_or("X"))
            .y_desc(self.options.y_label.as_deref().unwrap_or("Y"))
            .draw()
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        // Determine colors
        let colors = self
            .options
            .line_colors
            .unwrap_or_else(PlotOptions::default_colors);

        // Draw curves
        for (i, points) in all_curve_points.into_iter().enumerate() {
            chart
                .draw_series(LineSeries::new(
                    points,
                    colors[i % colors.len()].stroke_width(self.options.line_width),
                ))
                .map_err(|e| CurvesError::StdError {
                    reason: e.to_string(),
                })?
                .label(format!("Curve {}", i + 1));
        }

        // Add legend
        chart
            .configure_series_labels()
            .border_style(&BLACK)
            .draw()
            .map_err(|e| CurvesError::StdError {
                reason: e.to_string(),
            })?;

        root.present().map_err(|e| CurvesError::StdError {
            reason: e.to_string(),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;
    #[test]
    fn test_single_curve_plot() {
        let points = vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ];
        let curve = Curve::new(points);

        // Plot single curve
        curve.plot()
            .title("Test Curve")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600)  // Añade dimensiones explícitas
            .save("single_curve_test.png")
            .expect("Single curve plot failed");
    }

    #[test]
    fn test_multiple_curves_plot() {
        let points1 = vec![
            Point2D::new(dec!(0.0), dec!(0.0)),
            Point2D::new(dec!(1.0), dec!(1.0)),
            Point2D::new(dec!(2.0), dec!(4.0)),
        ];
        let points2 = vec![
            Point2D::new(dec!(0.0), dec!(1.0)),
            Point2D::new(dec!(1.0), dec!(2.0)),
            Point2D::new(dec!(2.0), dec!(5.0)),
        ];
        let curve1 = Curve::new(points1);
        let curve2 = Curve::new(points2);

        // Plot multiple curves
        vec![curve1, curve2].plot()
            .title("Multiple Curves")
            .x_label("X Axis")
            .y_label("Y Axis")
            .dimensions(800, 600)  // Añade dimensiones explícitas
            .save("multiple_curves_test.png")
            .expect("Multiple curves plot failed");
    }
}
