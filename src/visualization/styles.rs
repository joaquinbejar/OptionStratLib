use serde::{Deserialize, Serialize};

/// Represents the visual style of a line in graphical elements.
///
/// This enum defines different line styles that can be used when drawing lines
/// in graphics contexts, charts, or UI elements.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LineStyle {
    /// A continuous unbroken line (default style).
    #[default]
    Solid,
    /// A line composed of small dots.
    Dotted,
    /// A line composed of longer segments with gaps.
    Dashed,
}

/// Represents the color scheme used for visualizations.
///
/// This enum defines various color schemes that can be applied to charts, graphs,
/// or any visual elements that require color differentiation.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColorScheme {
    /// Default color scheme, used when no specific scheme is selected.
    #[default]
    Default,

    /// Viridis color scheme - a perceptually uniform color map designed to be
    /// perceived by viewers with common forms of color blindness.
    Viridis,

    /// Plasma color scheme - a perceptually uniform color map with a purple-orange gradient.
    Plasma,

    /// Custom color scheme defined by a vector of color strings.
    /// Each string should represent a valid color (e.g., hex code, RGB, color name).
    Custom(Vec<String>),

    /// White color scheme - primarily white or light colors.
    White,

    /// High contrast color scheme - designed for maximum visibility and accessibility.
    HighContrast,
}

/// Defines the visual appearance of traces in plots.
///
/// This enum specifies how data points should be displayed in a visualization,
/// such as connecting points with lines, showing individual markers, or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TraceMode {
    /// Display points connected by lines only.
    #[default]
    Lines,
    /// Display individual markers only without connecting lines.
    Markers,
    /// Display both lines connecting points and individual markers.
    LinesMarkers,
    /// Display text labels at points.
    TextLabels,
}

/// Represents different types of plots that can be generated.
///
/// This enum defines the various plot visualization types supported by the system.
/// Each variant corresponds to a specific visualization method for data representation.
///
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlotType {
    /// 2D line plot
    #[default]
    Line2D,
    /// 2D scatter plot with individual points
    Scatter2D,
    /// 3D surface plot for visualizing 3D data
    Surface3D,
    /// Heat map visualization for representing data density or intensity
    Heatmap,
}
