use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum LineStyle {
    #[default]
    Solid,
    Dotted,
    Dashed,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum ColorScheme {
    #[default]
    Default,
    Viridis,
    Plasma,
    Custom(Vec<String>),
    White,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum TraceMode {
    #[default]
    Lines,
    Markers,
    LinesMarkers,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum PlotType {
    #[default]
    Line2D,
    Scatter2D,
    Surface3D,
    Heatmap,
}