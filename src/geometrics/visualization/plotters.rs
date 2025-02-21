/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
use plotters::prelude::RGBColor;
use std::path::Path;

/// Plot configuration options
#[derive(Clone, Debug)]
pub struct PlotOptions {
    /// Plot title
    pub title: Option<String>,
    /// X-axis label
    pub x_label: Option<String>,
    /// Y-axis label
    pub y_label: Option<String>,
    /// Z-axis label
    pub z_label: Option<String>,
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
    /// Curve names
    pub curve_name: Option<Vec<String>>,

    pub point_size: Option<u32>,

    pub labels_size: Option<f64>,
}

#[allow(dead_code)]
impl PlotOptions {
    /// Default color palette for multiple curves
    pub(crate) fn default_colors() -> Vec<RGBColor> {
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
            z_label: None,
            line_colors: None,
            line_width: 2,
            background_color: RGBColor(255, 255, 255),
            width: 800,
            height: 600,
            curve_name: None,
            point_size: None,
            labels_size: None,
        }
    }
}

/// Trait for plotting curves
pub trait Plottable {
    type Error;

    /// Creates a plot builder
    fn plot(&self) -> PlotBuilder<Self>
    where
        Self: Sized;
}

/// Plot Builder for configurable curve visualization
#[allow(dead_code)]
pub struct PlotBuilder<T: Plottable> {
    /// Data to be plotted
    pub(crate) data: T,
    /// Plot configuration options
    pub(crate) options: PlotOptions,
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

    pub fn z_label(mut self, label: impl Into<String>) -> Self {
        self.options.z_label = Some(label.into());
        self
    }

    pub fn point_size(mut self, size: u32) -> Self {
        self.options.point_size = Some(size);
        self
    }

    pub fn label_size(mut self, size: f64) -> Self {
        self.options.labels_size = Some(size);
        self
    }

    pub fn curve_name(mut self, label: Vec<String>) -> Self {
        self.options.curve_name = Some(label);
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

    pub fn save(self, path: impl AsRef<Path>) -> Result<(), T::Error>
    where
        Self: PlotBuilderExt<T>,
    {
        PlotBuilderExt::save(self, path)
    }
}

/// Plotting extension methods
pub trait PlotBuilderExt<T: Plottable> {
    /// Save plot to file
    fn save(self, path: impl AsRef<Path>) -> Result<(), T::Error>;
}

#[cfg(test)]
mod tests_plot_builder {
    use super::*;

    /// A mock Plottable implementation for testing
    struct MockPlottable;

    impl Plottable for MockPlottable {
        type Error = std::io::Error;

        fn plot(&self) -> PlotBuilder<Self> {
            PlotBuilder {
                data: MockPlottable,
                options: PlotOptions::default(),
            }
        }
    }

    /// Convenience function to create a test plot builder
    fn create_test_builder() -> PlotBuilder<MockPlottable> {
        MockPlottable.plot()
    }

    #[test]
    fn test_z_label_method() {
        // Test setting z_label with a string
        let builder = create_test_builder().z_label("Z Axis");

        assert_eq!(builder.options.z_label, Some("Z Axis".to_string()));
    }

    #[test]
    fn test_z_label_method_with_different_types() {
        // Test setting z_label with different string-like types
        let builder1 = create_test_builder().z_label(String::from("Z Axis"));
        let builder2 = create_test_builder().z_label("Z Axis");

        assert_eq!(builder1.options.z_label, Some("Z Axis".to_string()));
        assert_eq!(builder2.options.z_label, Some("Z Axis".to_string()));
    }

    #[test]
    fn test_point_size_method() {
        // Test setting point size
        let builder = create_test_builder().point_size(10);

        assert_eq!(builder.options.point_size, Some(10));
    }

    #[test]
    fn test_point_size_method_multiple_calls() {
        // Test that multiple calls override the previous value
        let builder = create_test_builder().point_size(5).point_size(15);

        assert_eq!(builder.options.point_size, Some(15));
    }

    #[test]
    fn test_label_size_method() {
        // Test setting label size
        let builder = create_test_builder().label_size(12.5);

        assert_eq!(builder.options.labels_size, Some(12.5));
    }

    #[test]
    fn test_label_size_method_multiple_calls() {
        // Test that multiple calls override the previous value
        let builder = create_test_builder().label_size(10.0).label_size(20.5);

        assert_eq!(builder.options.labels_size, Some(20.5));
    }

    #[test]
    fn test_chaining_multiple_methods() {
        // Test chaining multiple configuration methods
        let builder = create_test_builder()
            .z_label("Z Axis")
            .point_size(10)
            .label_size(12.5);

        assert_eq!(builder.options.z_label, Some("Z Axis".to_string()));
        assert_eq!(builder.options.point_size, Some(10));
        assert_eq!(builder.options.labels_size, Some(12.5));
    }
}
