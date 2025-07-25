use crate::visualization::ColorScheme;

#[cfg(feature = "plotly")]
use {
    crate::visualization::GraphConfig,
    crate::visualization::{Label2D, Series2D, Surface3D, TraceMode},
    plotly::common::{Font, Line, Mode},
    plotly::{Scatter, Surface},
    rust_decimal::Decimal,
};

/// Creates a Scatter trace from a Series2D and configuration
#[cfg(feature = "plotly")]
pub fn make_scatter(series: &Series2D) -> Box<Scatter<Decimal, Decimal>> {
    let mode = match series.mode {
        TraceMode::Lines => Mode::Lines,
        TraceMode::Markers => Mode::Markers,
        TraceMode::LinesMarkers => Mode::LinesMarkers,
        TraceMode::TextLabels => Mode::Text,
    };

    // Create the scatter with coordinates and name
    let mut trace = Scatter::new(series.x.clone(), series.y.clone())
        .name(series.name.clone())
        .mode(mode);

    // Configure the line with color and width if specified
    let mut line = Line::new();
    if let Some(w) = series.line_width {
        line = line.width(w);
    }

    if let Some(ref c) = series.line_color {
        line = line.color(c.to_string());
    }

    trace = trace.line(line);

    // Handle text labels mode
    if matches!(series.mode, TraceMode::TextLabels) {
        // Use the name as text for each point
        let text_vec = vec![series.name.clone(); series.x.len()];

        // Set the text for each point
        trace = trace.text_array(text_vec);

        // Configure the mode to show text
        trace = trace.mode(Mode::Text);

        // Configure the text to be visible with appropriate color
        if let Some(ref color) = series.line_color {
            // Increase font size for better visibility
            let text_font = plotly::common::Font::new()
                .color(color.to_string())
                .size(16);
            trace = trace.text_font(text_font);
        }
    } else if matches!(series.mode, TraceMode::Markers) && series.line_width == Some(0.0) {
        // Backward compatibility for old implementation
        // Use the name as text for each point
        let text_vec = vec![series.name.clone(); series.x.len()];

        // Set the text for each point
        trace = trace.text_array(text_vec);

        // Configure the text to be visible with appropriate color
        if let Some(ref color) = series.line_color {
            // Increase font size for better visibility
            let text_font = plotly::common::Font::new()
                .color(color.to_string())
                .size(16);
            trace = trace.text_font(text_font);

            // Make the marker visible with appropriate color
            let marker = plotly::common::Marker::new()
                .size(10)
                .color(color.to_string());
            trace = trace.marker(marker);
        }
    }

    trace
}

/// Pick a color from config based on index
#[cfg(feature = "plotly")]
pub fn pick_color(cfg: &GraphConfig, idx: usize) -> Option<String> {
    get_color_from_scheme(&cfg.color_scheme, idx)
}

/// Creates a 3D surface representation wrapped in a `Box`, based on the input `Surface3D` structure.
///
/// This function converts the input `Surface3D` data into a `Surface` object, which includes the x, y, and z
/// coordinate mappings. If any of the x, y, or z coordinate vectors in the input are empty, it creates a
/// default 2x2 surface with all z values initialized to zero and x/y spanning the range `[0, 1]`.
///
/// # Parameters
/// - `surf`: A reference to a `Surface3D` object, which contains x, y, and z coordinate vectors
///   and an optional name for the surface.
///
/// # Returns
/// - A `Box` containing a `Surface<Decimal, Decimal, Decimal>` object, where:
///   - `x` and `y` axes are unique and sorted.
///   - `z` values are represented in a 2D matrix where rows correspond to `y` values and columns
///     correspond to `x` values.
///
/// # Logic Details
/// 1. **Edge Case Handling**: If any of `surf.x`, `surf.y`, or `surf.z` is empty, create a default 2x2 surface:
///    - All z values are initialized to `Decimal::ZERO`.
///    - x and y coordinates are `[0, 1]`.
///    - The surface name remains as provided in the `surf.name`.
///
/// 2. **Unique and Sorted Axes**:
///    - Extract unique and sorted x and y coordinates using a `BTreeSet`.
///    - These unique x and y coordinates are then stored in vectors (`x_unique` and `y_unique`).
///
/// 3. **Mapping Coordinates**:
///    - Create mappings (`x_to_col` and `y_to_row`) to translate x and y coordinates into column and row
///      indices of the `z_matrix`. Each coordinate is mapped to its respective index in the
#[cfg(feature = "plotly")]
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
#[cfg(feature = "plotly")]
pub fn to_plotly_mode(mode: &TraceMode) -> Mode {
    match mode {
        TraceMode::Lines => Mode::Lines,
        TraceMode::Markers => Mode::Markers,
        TraceMode::LinesMarkers => Mode::LinesMarkers,
        TraceMode::TextLabels => Mode::Text,
    }
}

/// Creates a Scatter trace from a Label2D
///
/// This function takes a Label2D structure and creates a plotly Scatter trace
/// with the text label positioned at the point's coordinates.
#[cfg(feature = "plotly")]
#[allow(dead_code)]
pub fn make_label_scatter(label: &Label2D) -> Box<Scatter<Decimal, Decimal>> {
    // Create the scatter with the point coordinates
    let mut trace = Scatter::new(vec![label.point.x], vec![label.point.y])
        .name(label.point.name.clone())
        .mode(Mode::Text);

    // Set the text for the label
    trace = trace.text_array(vec![label.label.clone()]);

    // Configure the text to be visible with appropriate color
    if let Some(ref color) = label.point.color {
        // Set font properties for better visibility
        let text_font = Font::new().color(color.to_string()).size(16);
        trace = trace.text_font(text_font);
    }

    trace
}

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
