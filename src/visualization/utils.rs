use plotly::{common, Scatter, Surface};
use crate::visualization::config::GraphConfig;
use crate::visualization::model::{Series2D, Surface3D};
use crate::visualization::styles::{ColorScheme, TraceMode};
use rust_decimal::Decimal;

/// Creates a Scatter trace from a Series2D and configuration
pub(crate) fn make_scatter(series: &Series2D) -> Box<Scatter<Decimal, Decimal>> {
    use plotly::common::{Line, Mode};

    let mode = match series.mode {
        TraceMode::Lines => Mode::Lines,
        TraceMode::Markers => Mode::Markers,
        TraceMode::LinesMarkers => Mode::LinesMarkers,
    };

    let mut trace = Scatter::new(series.x.clone(), series.y.clone())
        .name(series.name.clone())
        .mode(mode);

    let mut line = Line::new();
    if let Some(w) = series.line_width { line = line.width(w); }

    if let Some(ref c) = series.line_color {
        line = line.color(c.to_string()); 
    }

    trace = trace.line(line);

    trace
}

/// Pick a color from config based on index
pub fn pick_color(cfg: &GraphConfig, idx: usize) -> Option<String> {
    get_color_from_scheme(&cfg.color_scheme, idx)
}

/// Creates a Surface from a Surface3D
pub fn make_surface(surf: &Surface3D) -> Box<Surface<Decimal, Decimal, Decimal>> {
    let n_x = surf.x.len();
    let n_y = surf.y.len();
    assert!(surf.z.len() >= n_x * n_y, "not enough z values");

    let mut z_matrix = Vec::with_capacity(n_y);
    for i in 0..n_y {
        let start = i * n_x;
        let end = start + n_x;
        let row = surf.z[start..end].to_vec();
        z_matrix.push(row);
    }
    
    Surface::new(z_matrix)
        .x(surf.x.clone())
        .y(surf.y.clone())
        .name(&surf.name)
}


/// Utility function to convert TraceMode to plotly::common::Mode
pub fn to_plotly_mode(mode: &TraceMode) -> common::Mode {
    match mode {
        TraceMode::Lines => plotly::common::Mode::Lines,
        TraceMode::Markers => plotly::common::Mode::Markers,
        TraceMode::LinesMarkers => plotly::common::Mode::LinesMarkers,
    }
}

/// Get color from a color scheme based on index
pub fn get_color_from_scheme(scheme: &ColorScheme, idx: usize) -> Option<String> {
    match scheme {
        ColorScheme::Default => None,
        ColorScheme::Viridis => {
            // Approximation of viridis color scheme
            let colors = vec![
                "#440154", "#481567", "#482677", "#453781", "#404788",
                "#39568C", "#33638D", "#2D708E", "#287D8E", "#238A8D",
                "#1F968B", "#20A387", "#29AF7F", "#3CBB75", "#55C667",
                "#73D055", "#95D840", "#B8DE29", "#DCE319", "#FDE725"
            ];
            let color = colors.get(idx % colors.len()).unwrap();
            Some(color.to_string())
        },
        ColorScheme::Plasma => {
            // Approximation of plasma color scheme
            let colors = vec![
                "#0D0887", "#2A0593", "#41049D", "#5601A4", "#6A00A8",
                "#7E03A8", "#8F0DA4", "#A01C9C", "#B02A8F", "#BF3983",
                "#CB4678", "#D6556D", "#E16462", "#EA7457", "#F2844B",
                "#F89540", "#FCA636", "#FDB92F", "#FECE2F", "#FCFD35"
            ];
            let color = colors.get(idx % colors.len()).unwrap();
            Some(color.to_string())
        },
        ColorScheme::Custom(list) => list.get(idx % list.len()).cloned(),
        ColorScheme::White => Some("#FFFFFF".to_string()),
    }
}