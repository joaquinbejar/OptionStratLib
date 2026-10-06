use crate::visualization::styles::{ColorScheme, LineStyle};
use serde::{Deserialize, Serialize};

/// Represents the configuration parameters for a graph or chart visualization.
///
/// This struct encapsulates all the settings needed to define how a graph should be
/// displayed, including its dimensions, labels, styling options, and legend configuration.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphConfig {
    /// The main title of the graph to be displayed
    pub title: String,

    /// The width of the graph in pixels
    pub width: u32,

    /// The height of the graph in pixels
    pub height: u32,

    /// Optional label for the x-axis
    pub x_label: Option<String>,

    /// Optional label for the y-axis
    pub y_label: Option<String>,

    /// Optional label for the z-axis (for 3D graphs)
    pub z_label: Option<String>,

    /// The style of lines used in the graph (solid, dotted, dashed)
    pub line_style: LineStyle,

    /// The color scheme applied to the graph elements
    pub color_scheme: ColorScheme,

    /// Optional list of labels for the legend entries
    pub legend: Option<Vec<String>>,

    /// Flag indicating whether to display the legend
    pub show_legend: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        let title = "Graph".to_string();
        let legend = None;
        Self {
            title,
            width: 1280,
            height: 720,
            x_label: None,
            y_label: None,
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend,
            show_legend: true,
        }
    }
}
