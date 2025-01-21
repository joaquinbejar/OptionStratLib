/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/1/25
******************************************************************************/
use crate::error::CurvesError;
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
            line_colors: None,
            line_width: 2,
            background_color: RGBColor(255, 255, 255), // White
            width: 800,                                // Dimensión por defecto
            height: 600,                               // Dimensión por defecto
            curve_name: None,
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

    pub fn save(self, path: impl AsRef<Path>) -> Result<(), CurvesError>
    where
        Self: PlotBuilderExt<T>,
    {
        PlotBuilderExt::save(self, path)
    }
}

/// Plotting extension methods
pub trait PlotBuilderExt<T: Plottable> {
    /// Save plot to file
    fn save(self, path: impl AsRef<Path>) -> Result<(), CurvesError>;
}
