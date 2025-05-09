use crate::visualization::config::GraphConfig;
use crate::visualization::model::{Series2D, Surface3D};
use crate::visualization::styles::{ColorScheme, TraceMode};
use plotly::common::{Line, Mode};
use plotly::{Scatter, Surface};
use rust_decimal::Decimal;

/// Creates a Scatter trace from a Series2D and configuration
pub fn make_scatter(series: &Series2D) -> Box<Scatter<Decimal, Decimal>> {
    let mode = match series.mode {
        TraceMode::Lines => Mode::Lines,
        TraceMode::Markers => Mode::Markers,
        TraceMode::LinesMarkers => Mode::LinesMarkers,
    };

    let mut trace = Scatter::new(series.x.clone(), series.y.clone())
        .name(series.name.clone())
        .mode(mode);

    let mut line = Line::new();
    if let Some(w) = series.line_width {
        line = line.width(w);
    }

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
    if surf.x.is_empty() || surf.y.is_empty() || surf.z.is_empty() {
        let z_matrix = vec![
            vec![Decimal::ZERO, Decimal::ZERO],
            vec![Decimal::ZERO, Decimal::ZERO],
        ];
        return Surface::new(z_matrix)
            .x(vec![Decimal::ZERO, Decimal::ONE])
            .y(vec![Decimal::ZERO, Decimal::ONE])
            .name(&surf.name);
    }

    let mut x_set = std::collections::BTreeSet::new();
    let mut y_set = std::collections::BTreeSet::new();

    let indices: Vec<_> = (0..surf.x.len().min(surf.y.len()).min(surf.z.len())).collect();

    for &i in &indices {
        x_set.insert(surf.x[i]);
        y_set.insert(surf.y[i]);
    }

    let x_unique: Vec<_> = x_set.into_iter().collect();
    let y_unique: Vec<_> = y_set.into_iter().collect();

    let mut x_to_col = std::collections::HashMap::new();
    let mut y_to_row = std::collections::HashMap::new();

    for (col, &x) in x_unique.iter().enumerate() {
        x_to_col.insert(x, col);
    }

    for (row, &y) in y_unique.iter().enumerate() {
        y_to_row.insert(y, row);
    }

    let mut z_matrix = vec![vec![Decimal::ZERO; x_unique.len()]; y_unique.len()];

    for i in indices {
        let x = surf.x[i];
        let y = surf.y[i];
        let z = surf.z[i];

        if let (Some(&row), Some(&col)) = (y_to_row.get(&y), x_to_col.get(&x)) {
            z_matrix[row][col] = z;
        }
    }

    Surface::new(z_matrix)
        .x(x_unique)
        .y(y_unique)
        .name(&surf.name)
}

/// Utility function to convert TraceMode to Mode
pub fn to_plotly_mode(mode: &TraceMode) -> Mode {
    match mode {
        TraceMode::Lines => Mode::Lines,
        TraceMode::Markers => Mode::Markers,
        TraceMode::LinesMarkers => Mode::LinesMarkers,
    }
}

/// Get color from a color scheme based on index
/// Get color from a color scheme based on index
pub fn get_color_from_scheme(scheme: &ColorScheme, idx: usize) -> Option<String> {
    match scheme {
        ColorScheme::Default => None,
        ColorScheme::Viridis => {
            // Expanded and shuffled viridis color scheme (40 colors)
            let colors = vec![
                "#481567", "#33638D", "#20A387", "#95D840", "#440154", "#39568C", "#1F968B",
                "#73D055", "#482677", "#2D708E", "#29AF7F", "#B8DE29", "#453781", "#287D8E",
                "#3CBB75", "#DCE319", "#404788", "#238A8D", "#55C667", "#FDE725", "#46337E",
                "#1E9C89", "#79D151", "#471365", "#375B8D", "#23A884", "#A1DB32", "#45287C",
                "#307B8E", "#34B778", "#C6DE2F", "#432D7A", "#288A8D", "#42C675", "#E3E419",
                "#3F4889", "#21968A", "#5DC864", "#F0E51B", "#461C74",
            ];
            let color = colors.get(idx % colors.len()).unwrap();
            Some(color.to_string())
        }
        ColorScheme::Plasma => {
            // Expanded and shuffled plasma color scheme (40 colors)
            let colors = vec![
                "#0D0887", "#A01C9C", "#F2844B", "#2A0593", "#B02A8F", "#F89540", "#41049D",
                "#BF3983", "#FCA636", "#5601A4", "#CB4678", "#FDB92F", "#6A00A8", "#D6556D",
                "#FECE2F", "#7E03A8", "#E16462", "#FCFD35", "#8F0DA4", "#EA7457", "#330597",
                "#B93779", "#F68F46", "#4C02A1", "#CC3F76", "#FBA238", "#6501AB", "#DC5267",
                "#FDC229", "#7B04A7", "#E36159", "#FED330", "#8D0BA2", "#E7704F", "#FEE54F",
                "#9E189B", "#EC7F45", "#FEF06F", "#1C0377", "#AC2294",
            ];
            let color = colors.get(idx % colors.len()).unwrap();
            Some(color.to_string())
        }
        ColorScheme::Custom(list) => list.get(idx % list.len()).cloned(),
        ColorScheme::White => Some("#FFFFFF".to_string()),
        ColorScheme::HighContrast => {
            let colors = vec![
                "#FF0000", "#00FF00", "#0000FF", "#FFFF00", "#FF00FF", "#00FFFF", "#1F77B4",
                "#FF7F0E", "#2CA02C", "#D62728", "#9467BD", "#8C564B", "#E377C2", "#7F7F7F",
                "#BCBD22", "#17BECF", "#800000", "#008000", "#000080", "#808000", "#800080",
                "#008080", "#FF8C00", "#90EE90", "#ADD8E6", "#FFA07A", "#DA70D6", "#FFD700",
                "#FF1493", "#4B0082", "#006400", "#483D8B", "#CD853F", "#8B4513", "#4682B4",
                "#9ACD32", "#B22222", "#A52A2A", "#6A5ACD", "#778899", "#FF6347", "#7CFC00",
                "#87CEFA", "#FFA500", "#9932CC", "#008B8B",
            ];
            let color = colors.get(idx % colors.len()).unwrap();
            Some(color.to_string())
        }
    }
}
