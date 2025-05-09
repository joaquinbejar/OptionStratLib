use serde::{Deserialize, Serialize};
use crate::visualization::styles::{ColorScheme, LineStyle};

#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct GraphConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub z_label: Option<String>,
    pub line_style: LineStyle,
    pub color_scheme: ColorScheme,
    pub show_legend: bool,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            title: "Chart".into(),
            width: 1280,
            height: 720,
            x_label: None,
            y_label: None,
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            show_legend: true,
        }
    }
}