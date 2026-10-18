//! The native renderer: a `GraphData` and its `GraphConfig` drawn with
//! Plotters, with no browser, driver or network.
//!
//! It reads the chart exactly the way `Graph::to_plot` does (legend labels
//! override series names, `ColorScheme` colours series without one) and
//! mirrors the Plotly defaults where the data leaves a choice open: the
//! plotly.js colourway, a 2px line, a 6px marker, a 16px text label and a
//! 10px marker for the zero-width "label" markers `make_scatter` special-
//! cases. Where Plotly would silently fix bad data, it returns an
//! [`ExportError`] instead (see that type).

use crate::error::ExportError;
use crate::outline::{self, ChartText, Outlined};
use crate::svg;
use optionstratlib_visualization::visualization::{
    GraphConfig, GraphData, Series2D, Surface3D, TraceMode, get_color_from_scheme,
};
use plotters::coord::Shift;
use plotters::prelude::*;
use plotters::style::colors::colormaps::ViridisRGB;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use plotters::style::{FontStyle, register_font};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::sync::OnceLock;

/// Family name the embedded font is registered under, and the name every
/// text element uses (SVG `font-family` included).
pub const FONT_FAMILY: &str = "Open Sans";

/// Open Sans Regular (SIL OFL 1.1, `assets/OFL.txt`): the first family in
/// Plotly's default font stack.
pub(crate) const FONT_BYTES: &[u8] = include_bytes!("../assets/OpenSans-Regular.ttf");

/// plotters-svg writes `font-size` as the style size divided by this
/// factor, so a CSS size of `n` px is a Plotters size of `n * EM_RATIO`.
pub(crate) const EM_RATIO: f64 = 1.24;

/// Plotly's default sizes, in CSS pixels.
const TITLE_PX: f64 = 17.0;
const AXIS_TITLE_PX: f64 = 14.0;
const TICK_PX: f64 = 12.0;
const LEGEND_PX: f64 = 12.0;
/// `make_scatter` sets 16px for text labels.
const TEXT_LABEL_PX: f64 = 16.0;

/// plotly.js scatter defaults.
const DEFAULT_LINE_WIDTH: f64 = 2.0;
/// Widest line drawn. Plotters' polygon rasteriser panics on an unbounded
/// width (an infinite one became `u32::MAX`), so wider is an error.
const MAX_LINE_WIDTH_PX: f64 = 100.0;
const DEFAULT_MARKER_DIAMETER: u32 = 6;
/// `make_scatter` gives zero-width markers a 10px marker.
const LABEL_MARKER_DIAMETER: u32 = 10;

/// The plotly.js default colourway, used for series with no colour.
const COLORWAY: [&str; 10] = [
    "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b", "#e377c2", "#7f7f7f",
    "#bcbd22", "#17becf",
];

const GRID: RGBColor = RGBColor(235, 235, 235);
const AXIS_LINE: RGBColor = RGBColor(68, 68, 68);
const TEXT: RGBColor = RGBColor(42, 63, 95);
const ZERO_LINE: RGBColor = RGBColor(68, 68, 68);

/// Vertical padding added around the data, as a fraction of its span.
const Y_PADDING: f64 = 0.05;
/// Half-span given to an axis whose values are all equal and zero.
const FLAT_ZERO_HALF_SPAN: f64 = 1.0;
/// Half-span, as a fraction of the value, for an axis whose values are all
/// equal and not zero.
const FLAT_HALF_SPAN: f64 = 0.05;

const LEGEND_ROW_PX: i32 = 22;
const LEGEND_SAMPLE_PX: i32 = 30;
const LEGEND_GAP_PX: i32 = 8;
const LEGEND_MARGIN_PX: i32 = 12;

/// 3D view: Plotters' yaw/pitch in radians and its scale factor.
const SURFACE_YAW: f64 = 0.7;
const SURFACE_PITCH: f64 = 0.5;
const SURFACE_SCALE: f64 = 0.8;
const COLORBAR_WIDTH_PX: u32 = 110;
/// Smallest figure a surface is laid out in: the colour bar plus room for
/// the 3D axes, and a height that leaves an area under the title.
const SURFACE_MIN_WIDTH_PX: u32 = COLORBAR_WIDTH_PX + 130;
const SURFACE_MIN_HEIGHT_PX: u32 = 160;
const COLORBAR_STEPS: i32 = 100;
const COLORBAR_TICKS: i32 = 5;

/// How an SVG carries its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum SvgText {
    /// Embed the font as a base64 `@font-face`, so the SVG renders with the
    /// same glyphs everywhere (about 200 KB per file).
    #[default]
    EmbeddedFont,
    /// Name the family with a `sans-serif` fallback and embed nothing: the
    /// viewer substitutes its own font when it lacks Open Sans, as the
    /// Plotly SVG does.
    ViewerFont,
    /// Draw every label as `<path>` outlines of the embedded font: no font
    /// is named, embedded or needed, and each label keeps its text as an
    /// accessible name (see [`crate::outline`]). A character the font lacks
    /// is an error instead of a viewer's substitute.
    Outlines,
}

/// Writes `data` to `path` as a PNG of `config.width` x `config.height`.
///
/// Creates the parent directory when it is missing.
///
/// # Errors
///
/// [`ExportError`] for invalid dimensions or data, a failed font
/// registration, a drawing failure, or a path that cannot be written.
#[tracing::instrument(skip(data, config), fields(title = %config.title))]
pub fn write_png(data: &GraphData, config: &GraphConfig, path: &Path) -> Result<(), ExportError> {
    let size = dimensions(config)?;
    register_embedded_font()?;
    prepare_parent(path)?;
    let root = BitMapBackend::new(path, size).into_drawing_area();
    draw(&root, data, config)?;
    root.present().map_err(render_error)?;
    Ok(())
}

/// Renders `data` as an SVG document of `config.width` x `config.height`.
///
/// # Errors
///
/// [`ExportError`] for invalid dimensions or data, a failed font
/// registration or a drawing failure; with [`SvgText::Outlines`], also a
/// character the font has no glyph for.
#[tracing::instrument(skip(data, config), fields(title = %config.title))]
pub fn render_svg(
    data: &GraphData,
    config: &GraphConfig,
    text: SvgText,
) -> Result<String, ExportError> {
    let document = render_plotters_svg(data, config)?;
    match text {
        SvgText::EmbeddedFont => Ok(svg::embed_font(&document)),
        SvgText::ViewerFont => Ok(svg::with_fallback_family(&document)),
        SvgText::Outlines => {
            Ok(outline::outline_text(&document, &describe(data, config))?.document)
        }
    }
}

/// Renders `data` as an SVG with its text as outlines, like
/// [`render_svg`] with [`SvgText::Outlines`], and also returns where each
/// label's ink is: the comparison of #923 crops the viewers' renders there.
///
/// # Errors
///
/// Everything [`render_svg`] returns for [`SvgText::Outlines`].
pub fn render_svg_outlined(
    data: &GraphData,
    config: &GraphConfig,
) -> Result<Outlined, ExportError> {
    let document = render_plotters_svg(data, config)?;
    outline::outline_text(&document, &describe(data, config))
}

/// The document plotters-svg writes, before any text post-processing.
fn render_plotters_svg(data: &GraphData, config: &GraphConfig) -> Result<String, ExportError> {
    let size = dimensions(config)?;
    register_embedded_font()?;
    let mut document = String::new();
    {
        let root = SVGBackend::with_string(&mut document, size).into_drawing_area();
        draw(&root, data, config)?;
        root.present().map_err(render_error)?;
    }
    Ok(document)
}

/// The chart's accessible name and description: its title, what it plots
/// and its axes, with the names the legend shows.
fn describe(data: &GraphData, config: &GraphConfig) -> ChartText {
    let legend = |index: usize| config.legend.as_ref().and_then(|legend| legend.get(index));
    let mut description = match data {
        GraphData::Series(series) => {
            format!("Chart of 1 series: {}.", legend(0).unwrap_or(&series.name))
        }
        GraphData::MultiSeries(list) => {
            let names: Vec<&str> = list
                .iter()
                .enumerate()
                .map(|(index, series)| legend(index).unwrap_or(&series.name).as_str())
                .collect();
            format!("Chart of {} series: {}.", names.len(), names.join("; "))
        }
        GraphData::GraphSurface(surface) => format!(
            "Surface chart of {}, coloured by value.",
            legend(0).unwrap_or(&surface.name)
        ),
    };
    let axes = [
        ("x", &config.x_label),
        ("y", &config.y_label),
        ("z", &config.z_label),
    ];
    for (axis, label) in axes {
        if let Some(label) = label {
            description.push_str(&format!(" {axis} axis: {label}."));
        }
    }
    let title = if config.title.trim().is_empty() {
        "Chart".to_string()
    } else {
        config.title.clone()
    };
    ChartText { title, description }
}

/// Writes `data` to `path` as an SVG; see [`render_svg`].
///
/// Creates the parent directory when it is missing.
///
/// # Errors
///
/// Everything [`render_svg`] returns, and [`ExportError::Io`] when the file
/// cannot be written.
pub fn write_svg(
    data: &GraphData,
    config: &GraphConfig,
    path: &Path,
    text: SvgText,
) -> Result<(), ExportError> {
    let document = render_svg(data, config, text)?;
    prepare_parent(path)?;
    std::fs::write(path, document)?;
    Ok(())
}

/// Registers the embedded font once per process.
fn register_embedded_font() -> Result<(), ExportError> {
    static REGISTERED: OnceLock<Result<(), String>> = OnceLock::new();
    REGISTERED
        .get_or_init(|| {
            register_font(FONT_FAMILY, FontStyle::Normal, FONT_BYTES)
                .map_err(|_| "Open Sans Regular is not a valid TrueType font".to_string())
        })
        .clone()
        .map_err(ExportError::Font)
}

fn dimensions(config: &GraphConfig) -> Result<(u32, u32), ExportError> {
    if config.width == 0 || config.height == 0 {
        return Err(ExportError::InvalidDimensions {
            width: config.width,
            height: config.height,
        });
    }
    Ok((config.width, config.height))
}

fn prepare_parent(path: &Path) -> Result<(), ExportError> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        std::fs::create_dir_all(parent)?;
    }
    Ok(())
}

#[cold]
#[inline(never)]
fn render_error<E: std::error::Error + Send + Sync>(error: DrawingAreaErrorKind<E>) -> ExportError {
    ExportError::Render(error.to_string())
}

/// A Plotters font of `px` CSS pixels.
fn font(px: f64) -> FontDesc<'static> {
    (FONT_FAMILY, px * EM_RATIO).into_font()
}

fn to_f64(value: Decimal) -> Result<f64, ExportError> {
    f64::try_from(value).map_err(|_| ExportError::NotRepresentable(value))
}

/// Parses `#RGB` or `#RRGGBB`.
fn parse_color(text: &str) -> Result<RGBColor, ExportError> {
    let invalid = || ExportError::InvalidColor(text.to_string());
    let hex = text.strip_prefix('#').ok_or_else(invalid)?;
    let channel = |digits: &str| u8::from_str_radix(digits, 16).map_err(|_| invalid());
    match hex.len() {
        3 => {
            let mut channels = [0_u8; 3];
            for (slot, index) in channels.iter_mut().zip(0..3) {
                let digit = hex.get(index..=index).ok_or_else(invalid)?;
                *slot = channel(&format!("{digit}{digit}"))?;
            }
            let [r, g, b] = channels;
            Ok(RGBColor(r, g, b))
        }
        6 => {
            let part = |range: std::ops::Range<usize>| hex.get(range).ok_or_else(invalid);
            Ok(RGBColor(
                channel(part(0..2)?)?,
                channel(part(2..4)?)?,
                channel(part(4..6)?)?,
            ))
        }
        _ => Err(invalid()),
    }
}

/// The plotly.js colourway entry for the `index`-th trace.
fn default_color(index: usize) -> Result<RGBColor, ExportError> {
    let entry = index
        .checked_rem(COLORWAY.len())
        .and_then(|slot| COLORWAY.get(slot))
        .ok_or_else(|| ExportError::InvalidColor(format!("colourway entry {index}")))?;
    parse_color(entry)
}

fn draw<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    data: &GraphData,
    config: &GraphConfig,
) -> Result<(), ExportError>
where
    DB::ErrorType: 'static,
{
    root.fill(&WHITE).map_err(render_error)?;
    match data {
        GraphData::Series(series) => {
            let mut series = series.clone();
            if let Some(label) = config.legend.as_ref().and_then(|legend| legend.first()) {
                series.name = label.clone();
            }
            draw_2d(root, &[series], config)
        }
        GraphData::MultiSeries(list) => {
            let mut styled = Vec::with_capacity(list.len());
            for (index, series) in list.iter().enumerate() {
                let mut series = series.clone();
                if series.line_color.is_none() {
                    series.line_color = get_color_from_scheme(&config.color_scheme, index);
                }
                if let Some(label) = config.legend.as_ref().and_then(|legend| legend.get(index)) {
                    series.name = label.clone();
                }
                styled.push(series);
            }
            draw_2d(root, &styled, config)
        }
        GraphData::GraphSurface(surface) => {
            let mut surface = surface.clone();
            if let Some(label) = config.legend.as_ref().and_then(|legend| legend.first()) {
                surface.name = label.clone();
            }
            draw_surface(root, &surface, config)
        }
    }
}

/// Fails when `area`, carved out of `root` by a title, legend or colour bar,
/// has no pixels left. Plotters reports an area squeezed below zero as a huge
/// unsigned size rather than an error, so it is caught here before any
/// geometry is derived from it.
fn ensure_room<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    root: &DrawingArea<DB, Shift>,
) -> Result<(), ExportError> {
    let (width, height) = area.dim_in_pixel();
    let (root_width, root_height) = root.dim_in_pixel();
    if width == 0 || height == 0 || width > root_width || height > root_height {
        return Err(ExportError::NoRoom {
            width: root_width,
            height: root_height,
        });
    }
    Ok(())
}

/// One 2D series, validated and converted for drawing.
struct Trace {
    name: String,
    points: Vec<(f64, f64)>,
    mode: TraceMode,
    color: RGBColor,
    line_width: u32,
    marker_radius: u32,
}

fn prepare_traces(series: &[Series2D]) -> Result<Vec<Trace>, ExportError> {
    let mut traces = Vec::with_capacity(series.len());
    for (index, item) in series.iter().enumerate() {
        if item.x.len() != item.y.len() {
            return Err(ExportError::LengthMismatch {
                name: item.name.clone(),
                x: item.x.len(),
                y: item.y.len(),
            });
        }
        let mut points = Vec::with_capacity(item.x.len());
        for (x, y) in item.x.iter().zip(&item.y) {
            points.push((to_f64(*x)?, to_f64(*y)?));
        }
        let color = match &item.line_color {
            Some(text) => parse_color(text)?,
            None => default_color(index)?,
        };
        let width = item.line_width.unwrap_or(DEFAULT_LINE_WIDTH);
        if !(width.is_finite() && (0.0..=MAX_LINE_WIDTH_PX).contains(&width)) {
            return Err(ExportError::InvalidLineWidth {
                name: item.name.clone(),
                width,
            });
        }
        let diameter = if item.line_width == Some(0.0) {
            LABEL_MARKER_DIAMETER
        } else {
            DEFAULT_MARKER_DIAMETER
        };
        traces.push(Trace {
            name: item.name.clone(),
            points,
            mode: item.mode,
            color,
            // Finite and within 0..=MAX_LINE_WIDTH_PX, checked above.
            line_width: width.round() as u32,
            marker_radius: diameter / 2,
        });
    }
    Ok(traces)
}

/// The range of `values`, padded by `padding` of its span on each side.
fn axis_range(values: impl Iterator<Item = f64>, padding: f64) -> std::ops::Range<f64> {
    let mut bounds: Option<(f64, f64)> = None;
    for value in values {
        bounds = Some(match bounds {
            Some((low, high)) => (low.min(value), high.max(value)),
            None => (value, value),
        });
    }
    match bounds {
        None => 0.0..1.0,
        // Only equal bounds are flat: a tiny but real span (0 to 1e-18 is a
        // valid Decimal series) keeps its own scale.
        Some((low, high)) if high == low => {
            let half = if low == 0.0 {
                FLAT_ZERO_HALF_SPAN
            } else {
                low.abs() * FLAT_HALF_SPAN
            };
            (low - half)..(high + half)
        }
        Some((low, high)) => {
            let pad = (high - low) * padding;
            (low - pad)..(high + pad)
        }
    }
}

fn draw_2d<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    series: &[Series2D],
    config: &GraphConfig,
) -> Result<(), ExportError>
where
    DB::ErrorType: 'static,
{
    let traces = prepare_traces(series)?;
    let body = root
        .titled(&config.title, font(TITLE_PX).color(&TEXT))
        .map_err(render_error)?;
    ensure_room(&body, root)?;

    let (plot_area, legend_area) = if config.show_legend && !traces.is_empty() {
        let width = legend_width(root, &traces)?;
        let (total, _) = body.dim_in_pixel();
        let split = total.checked_sub(width).unwrap_or(total);
        let (left, right) = body.split_horizontally(split);
        ensure_room(&left, root)?;
        (left, Some(right))
    } else {
        (body, None)
    };

    let x_range = axis_range(
        traces.iter().flat_map(|t| t.points.iter().map(|p| p.0)),
        0.0,
    );
    let y_range = axis_range(
        traces.iter().flat_map(|t| t.points.iter().map(|p| p.1)),
        Y_PADDING,
    );
    // Plotters' key points degrade below spans of about 1e-15, so a tiny axis
    // is drawn in units of its own scale and labelled with the real values.
    let (x_scale, y_scale) = (axis_scale(&x_range), axis_scale(&y_range));
    let scaled = |point: &(f64, f64)| (point.0 / x_scale, point.1 / y_scale);
    let x_range = (x_range.start / x_scale)..(x_range.end / x_scale);
    let y_range = (y_range.start / y_scale)..(y_range.end / y_scale);

    let mut chart = ChartBuilder::on(&plot_area)
        .margin(10)
        .x_label_area_size(if config.x_label.is_some() { 60 } else { 35 })
        .y_label_area_size(if config.y_label.is_some() { 85 } else { 65 })
        .build_cartesian_2d(x_range.clone(), y_range.clone())
        .map_err(render_error)?;

    let x_step = tick_step(&x_range, 8);
    let y_step = tick_step(&y_range, 8);
    let x_format = move |value: &f64| format_number(*value * x_scale, x_step * x_scale);
    let y_format = move |value: &f64| format_number(*value * y_scale, y_step * y_scale);
    let mut mesh = chart.configure_mesh();
    mesh.x_label_formatter(&x_format)
        .y_label_formatter(&y_format)
        .light_line_style(TRANSPARENT)
        .bold_line_style(GRID.stroke_width(1))
        .axis_style(AXIS_LINE.stroke_width(1))
        .label_style(font(TICK_PX).color(&TEXT))
        .axis_desc_style(font(AXIS_TITLE_PX).color(&TEXT))
        .x_labels(8)
        .y_labels(8);
    if let Some(label) = &config.x_label {
        mesh.x_desc(label.as_str());
    }
    if let Some(label) = &config.y_label {
        mesh.y_desc(label.as_str());
    }
    mesh.draw().map_err(render_error)?;

    // Plotly's zero lines: drawn when zero is inside the axis range.
    let zero_line = ZERO_LINE.stroke_width(1);
    if y_range.start < 0.0 && y_range.end > 0.0 {
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(x_range.start, 0.0), (x_range.end, 0.0)],
                zero_line,
            )))
            .map_err(render_error)?;
    }
    if x_range.start < 0.0 && x_range.end > 0.0 {
        chart
            .draw_series(std::iter::once(PathElement::new(
                vec![(0.0, y_range.start), (0.0, y_range.end)],
                zero_line,
            )))
            .map_err(render_error)?;
    }

    for trace in &traces {
        let draws_line = matches!(trace.mode, TraceMode::Lines | TraceMode::LinesMarkers);
        let draws_markers = matches!(trace.mode, TraceMode::Markers | TraceMode::LinesMarkers);
        if draws_line && trace.line_width > 0 {
            let style = trace.color.stroke_width(trace.line_width);
            chart
                .draw_series(LineSeries::new(trace.points.iter().map(scaled), style))
                .map_err(render_error)?;
        }
        if draws_markers {
            let style = trace.color.filled();
            chart
                .draw_series(
                    trace
                        .points
                        .iter()
                        .map(|point| Circle::new(scaled(point), trace.marker_radius, style)),
                )
                .map_err(render_error)?;
        }
        if trace.mode == TraceMode::TextLabels {
            let style = font(TEXT_LABEL_PX)
                .color(&trace.color)
                .pos(Pos::new(HPos::Center, VPos::Center));
            chart
                .draw_series(
                    trace
                        .points
                        .iter()
                        .map(|point| Text::new(trace.name.clone(), scaled(point), style.clone())),
                )
                .map_err(render_error)?;
        }
    }

    if let Some(area) = legend_area {
        draw_legend(&area, &traces)?;
    }
    Ok(())
}

/// Width of the legend column: the widest name plus its sample and margins,
/// at most a third of the figure.
fn legend_width<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    traces: &[Trace],
) -> Result<u32, ExportError>
where
    DB::ErrorType: 'static,
{
    let style = TextStyle::from(font(LEGEND_PX));
    let mut widest = 0_u32;
    for trace in traces {
        let (width, _) = root
            .estimate_text_size(&trace.name, &style)
            .map_err(render_error)?;
        widest = widest.max(width);
    }
    let chrome = (LEGEND_MARGIN_PX * 2 + LEGEND_SAMPLE_PX + LEGEND_GAP_PX).unsigned_abs();
    let (total, _) = root.dim_in_pixel();
    Ok(widest.checked_add(chrome).unwrap_or(total).min(total / 3))
}

fn draw_legend<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    traces: &[Trace],
) -> Result<(), ExportError>
where
    DB::ErrorType: 'static,
{
    let text_style = font(LEGEND_PX)
        .color(&TEXT)
        .pos(Pos::new(HPos::Left, VPos::Center));
    let sample_start = LEGEND_MARGIN_PX;
    let sample_end = sample_start + LEGEND_SAMPLE_PX;
    let sample_mid = sample_start + LEGEND_SAMPLE_PX / 2;
    let mut row = LEGEND_MARGIN_PX + LEGEND_ROW_PX / 2;
    for trace in traces {
        match trace.mode {
            TraceMode::Lines | TraceMode::LinesMarkers | TraceMode::Markers => {
                if trace.mode != TraceMode::Markers && trace.line_width > 0 {
                    area.draw(&PathElement::new(
                        vec![(sample_start, row), (sample_end, row)],
                        trace.color.stroke_width(trace.line_width.min(4)),
                    ))
                    .map_err(render_error)?;
                }
                if trace.mode != TraceMode::Lines {
                    area.draw(&Circle::new(
                        (sample_mid, row),
                        trace.marker_radius,
                        trace.color.filled(),
                    ))
                    .map_err(render_error)?;
                }
            }
            TraceMode::TextLabels => {
                let style = font(LEGEND_PX)
                    .color(&trace.color)
                    .pos(Pos::new(HPos::Center, VPos::Center));
                area.draw(&Text::new("Aa", (sample_mid, row), style))
                    .map_err(render_error)?;
            }
        }
        area.draw(&Text::new(
            trace.name.as_str(),
            (sample_end + LEGEND_GAP_PX, row),
            text_style.clone(),
        ))
        .map_err(render_error)?;
        row += LEGEND_ROW_PX;
    }
    Ok(())
}

/// A complete surface grid: `z[i][j]` is the value at `xs[i]`, `ys[j]`.
struct Grid {
    xs: Vec<f64>,
    ys: Vec<f64>,
    z: Vec<Vec<f64>>,
}

impl Grid {
    #[cfg(test)]
    fn z_at(&self, i: usize, j: usize) -> Option<f64> {
        self.z.get(i).and_then(|column| column.get(j)).copied()
    }
}

/// Validates `surface` as a complete rectangular grid. Plotly's
/// `make_surface` fills missing points with zero and keeps the last of
/// duplicated ones; both are errors here.
fn surface_grid(surface: &Surface3D) -> Result<Grid, ExportError> {
    let (x, y, z) = (surface.x.len(), surface.y.len(), surface.z.len());
    if x != y || y != z {
        return Err(ExportError::SurfaceLengthMismatch {
            name: surface.name.clone(),
            x,
            y,
            z,
        });
    }
    let mut cells: BTreeMap<(Decimal, Decimal), Decimal> = BTreeMap::new();
    for ((x, y), z) in surface.x.iter().zip(&surface.y).zip(&surface.z) {
        if cells.insert((*x, *y), *z).is_some() {
            return Err(ExportError::DuplicateSurfacePoint {
                name: surface.name.clone(),
                x: *x,
                y: *y,
            });
        }
    }
    let xs: BTreeSet<Decimal> = cells.keys().map(|(x, _)| *x).collect();
    let ys: BTreeSet<Decimal> = cells.keys().map(|(_, y)| *y).collect();
    if xs.len() < 2 || ys.len() < 2 {
        return Err(ExportError::DegenerateSurface {
            name: surface.name.clone(),
            columns: xs.len(),
            rows: ys.len(),
        });
    }
    let expected = xs.len().checked_mul(ys.len());
    let incomplete = || ExportError::IncompleteSurface {
        name: surface.name.clone(),
        expected: expected.unwrap_or(usize::MAX),
        found: cells.len(),
    };
    if expected != Some(cells.len()) {
        return Err(incomplete());
    }
    let mut columns = Vec::with_capacity(xs.len());
    for x in &xs {
        let mut column = Vec::with_capacity(ys.len());
        for y in &ys {
            let value = cells.get(&(*x, *y)).ok_or_else(incomplete)?;
            column.push(to_f64(*value)?);
        }
        columns.push(column);
    }
    Ok(Grid {
        xs: xs.into_iter().map(to_f64).collect::<Result<_, _>>()?,
        ys: ys.into_iter().map(to_f64).collect::<Result<_, _>>()?,
        z: columns,
    })
}

/// One grid cell of the surface: its corners in Plotters order
/// `(x, height, depth)`, its mean height and its distance from the viewer.
struct Cell {
    corners: [(f64, f64, f64); 4],
    height: f64,
    depth: i32,
}

fn draw_surface<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    surface: &Surface3D,
    config: &GraphConfig,
) -> Result<(), ExportError>
where
    DB::ErrorType: 'static,
{
    let grid = surface_grid(surface)?;
    if config.width < SURFACE_MIN_WIDTH_PX || config.height < SURFACE_MIN_HEIGHT_PX {
        return Err(ExportError::TooSmall {
            width: config.width,
            height: config.height,
            min_width: SURFACE_MIN_WIDTH_PX,
            min_height: SURFACE_MIN_HEIGHT_PX,
        });
    }
    let body = root
        .titled(&config.title, font(TITLE_PX).color(&TEXT))
        .map_err(render_error)?;
    ensure_room(&body, root)?;
    let (total, _) = body.dim_in_pixel();
    let split = total.checked_sub(COLORBAR_WIDTH_PX).unwrap_or(total);
    let (plot_area, bar_area) = body.split_horizontally(split);
    ensure_room(&plot_area, root)?;
    ensure_room(&bar_area, root)?;

    let x_range = axis_range(grid.xs.iter().copied(), 0.0);
    let y_range = axis_range(grid.ys.iter().copied(), 0.0);
    let z_range = axis_range(grid.z.iter().flatten().copied(), 0.0);
    let (z_low, z_high) = (z_range.start, z_range.end);

    // Plotters' second axis is the vertical one: the surface value goes
    // there and the data's y axis becomes the depth.
    let mut chart = ChartBuilder::on(&plot_area)
        .margin(20)
        .build_cartesian_3d(x_range.clone(), z_range.clone(), y_range.clone())
        .map_err(render_error)?;
    chart.with_projection(|mut builder| {
        builder.yaw = SURFACE_YAW;
        builder.pitch = SURFACE_PITCH;
        builder.scale = SURFACE_SCALE;
        builder.into_matrix()
    });
    let x_step = tick_step(&x_range, 6);
    let y_step = tick_step(&y_range, 6);
    let z_step = tick_step(&z_range, 6);
    let x_format = move |value: &f64| format_number(*value, x_step);
    let height_format = move |value: &f64| format_number(*value, z_step);
    let depth_format = move |value: &f64| format_number(*value, y_step);
    chart
        .configure_axes()
        .x_formatter(&x_format)
        .y_formatter(&height_format)
        .z_formatter(&depth_format)
        .x_labels(6)
        .y_labels(6)
        .z_labels(6)
        .light_grid_style(GRID)
        .bold_grid_style(GRID.mix(0.0))
        .axis_panel_style(RGBColor(248, 248, 250).filled())
        .label_style(font(TICK_PX).color(&TEXT))
        .max_light_lines(4)
        .tick_size(4)
        .draw()
        .map_err(render_error)?;

    let mut cells = Vec::with_capacity(
        grid.xs
            .windows(2)
            .len()
            .checked_mul(grid.ys.windows(2).len())
            .unwrap_or_default(),
    );
    {
        let coord = chart.as_coord_spec();
        // Adjacent x columns of the grid, then adjacent y rows within them.
        for (xs, columns) in grid.xs.windows(2).zip(grid.z.windows(2)) {
            let ([x0, x1], [near, far]) = (xs, columns) else {
                continue;
            };
            for (ys, (near_z, far_z)) in grid.ys.windows(2).zip(near.windows(2).zip(far.windows(2)))
            {
                let ([y0, y1], [z00, z01], [z10, z11]) = (ys, near_z, far_z) else {
                    continue;
                };
                let (a, b, c, d) = (
                    (*x0, *z00, *y0),
                    (*x1, *z10, *y0),
                    (*x1, *z11, *y1),
                    (*x0, *z01, *y1),
                );
                let height = (a.1 + b.1 + c.1 + d.1) / 4.0;
                let centre_x = (a.0 + c.0) / 2.0;
                let centre_depth = (a.2 + c.2) / 2.0;
                cells.push(Cell {
                    corners: [a, b, c, d],
                    height,
                    depth: coord.projected_depth(&centre_x, &height, &centre_depth),
                });
            }
        }
    }
    // Painter's algorithm: the farthest cell first, so nearer cells cover it.
    cells.sort_by_key(|cell| std::cmp::Reverse(cell.depth));
    let edge = BLACK.mix(0.18).stroke_width(1);
    for cell in &cells {
        let fill = ViridisRGB.get_color_normalized(cell.height, z_low, z_high);
        chart
            .draw_series(std::iter::once(Polygon::new(
                cell.corners.to_vec(),
                fill.filled(),
            )))
            .map_err(render_error)?;
        let [a, b, c, d] = cell.corners;
        chart
            .draw_series(std::iter::once(PathElement::new(vec![a, b, c, d, a], edge)))
            .map_err(render_error)?;
    }

    let axis_title = font(AXIS_TITLE_PX)
        .color(&TEXT)
        .pos(Pos::new(HPos::Center, VPos::Center));
    let x_mid = (x_range.start + x_range.end) / 2.0;
    let y_mid = (y_range.start + y_range.end) / 2.0;
    let y_span = y_range.end - y_range.start;
    let x_span = x_range.end - x_range.start;
    let mut titles = Vec::with_capacity(3);
    if let Some(label) = &config.x_label {
        titles.push((label, (x_mid, z_low, y_range.end + y_span * 0.22)));
    }
    if let Some(label) = &config.y_label {
        titles.push((label, (x_range.end + x_span * 0.22, z_low, y_mid)));
    }
    if let Some(label) = &config.z_label {
        titles.push((
            label,
            (
                x_range.start - x_span * 0.12,
                z_high,
                y_range.start - y_span * 0.12,
            ),
        ));
    }
    chart
        .draw_series(
            titles
                .into_iter()
                .map(|(label, at)| Text::new(label.clone(), at, axis_title.clone())),
        )
        .map_err(render_error)?;

    draw_colorbar(&bar_area, z_low, z_high, config.z_label.as_deref())
}

/// A vertical Viridis scale from `low` (bottom) to `high` (top).
fn draw_colorbar<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    low: f64,
    high: f64,
    title: Option<&str>,
) -> Result<(), ExportError>
where
    DB::ErrorType: 'static,
{
    // Geometry in i64: `dim_in_pixel` of an area squeezed below zero by a
    // caption comes back as a huge u32, which must not overflow here.
    let (_, height) = area.dim_in_pixel();
    let height = i64::from(height);
    let top = height / 6;
    let bottom = height - height / 6;
    let (left, right) = (10, 30);
    let span = (bottom - top).max(1);
    let steps = i64::from(COLORBAR_STEPS);
    let pixel = |value: i64| {
        i32::try_from(value)
            .map_err(|_| ExportError::Render(format!("colour bar coordinate {value} out of range")))
    };
    for step in 0..steps {
        let y0 = pixel(bottom - span * step / steps)?;
        let y1 = pixel(bottom - span * (step + 1) / steps)?;
        let fraction = step as f64 / (steps - 1) as f64;
        let color = ViridisRGB.get_color(fraction);
        area.draw(&Rectangle::new([(left, y1), (right, y0)], color.filled()))
            .map_err(render_error)?;
    }
    let tick_style = font(TICK_PX)
        .color(&TEXT)
        .pos(Pos::new(HPos::Left, VPos::Center));
    for tick in 0..COLORBAR_TICKS {
        let fraction = f64::from(tick) / f64::from(COLORBAR_TICKS - 1);
        let value = low + (high - low) * fraction;
        let y = pixel(bottom - (span as f64 * fraction).round() as i64)?;
        area.draw(&PathElement::new(
            vec![(right, y), (right + 4, y)],
            AXIS_LINE.stroke_width(1),
        ))
        .map_err(render_error)?;
        area.draw(&Text::new(
            format_number(value, (high - low) / f64::from(COLORBAR_TICKS - 1)),
            (right + 8, y),
            tick_style.clone(),
        ))
        .map_err(render_error)?;
    }
    if let Some(title) = title {
        let style = font(TICK_PX)
            .color(&TEXT)
            .pos(Pos::new(HPos::Left, VPos::Bottom));
        area.draw(&Text::new(title, (left, pixel(top - 12)?), style))
            .map_err(render_error)?;
    }
    Ok(())
}

/// `value` with enough precision to tell apart ticks `step` apart, without
/// trailing zeros and with a true minus sign, as Plotly writes ticks.
///
/// Fixed notation while the step is at least 1e-4; below that, or for
/// magnitudes from 1e15, scientific notation with as many mantissa digits as
/// `value / step` needs, so ticks of a 0 to 0.00001 axis stay distinct.
fn format_number(value: f64, step: f64) -> String {
    const SMALLEST_FIXED_STEP: f64 = 1e-4;
    const LARGEST_FIXED_MAGNITUDE: f64 = 1e15;
    const MAX_DIGITS: f64 = 15.0;
    let step = step.abs();
    if !value.is_finite() || !step.is_finite() || step == 0.0 {
        return with_true_minus(value.to_string());
    }
    if value == 0.0 {
        return "0".to_string();
    }
    let magnitude = value.abs().max(step);
    let text = if step >= SMALLEST_FIXED_STEP && magnitude < LARGEST_FIXED_MAGNITUDE {
        let decimals = (-step.log10()).ceil().clamp(0.0, MAX_DIGITS) as usize;
        trim_fraction(&format!("{value:.decimals$}"))
    } else {
        let digits = (magnitude / step).log10().ceil().clamp(0.0, MAX_DIGITS) as usize;
        let scientific = format!("{value:.digits$e}");
        match scientific.split_once('e') {
            Some((mantissa, exponent)) => format!("{}e{exponent}", trim_fraction(mantissa)),
            None => scientific,
        }
    };
    with_true_minus(text)
}

/// `text` without trailing fractional zeros or a bare decimal point.
fn trim_fraction(text: &str) -> String {
    if text.contains('.') {
        text.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        text.to_string()
    }
}

/// A leading `-` as U+2212, and `-0` as `0`.
fn with_true_minus(text: String) -> String {
    match text.strip_prefix('-') {
        Some("0") => "0".to_string(),
        Some(rest) => format!("\u{2212}{rest}"),
        None => text,
    }
}

/// Axes spanning less than this are drawn rescaled.
const SMALLEST_DRAWN_SPAN: f64 = 1e-9;

/// The unit a 2D axis is drawn in: 1, or for a span under
/// `SMALLEST_DRAWN_SPAN` the power of ten just below it.
fn axis_scale(range: &std::ops::Range<f64>) -> f64 {
    let span = range.end - range.start;
    if span > 0.0 && span < SMALLEST_DRAWN_SPAN {
        10_f64.powi(span.log10().floor() as i32)
    } else {
        1.0
    }
}

/// Tick spacing Plotters will use for `range` with about `count` ticks.
fn tick_step(range: &std::ops::Range<f64>, count: u32) -> f64 {
    (range.end - range.start) / f64::from(count.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_parse_color_accepts_short_and_long_hex() {
        assert!(matches!(parse_color("#fff"), Ok(RGBColor(255, 255, 255))));
        assert!(matches!(parse_color("#2ca02c"), Ok(RGBColor(44, 160, 44))));
    }

    #[test]
    fn test_parse_color_rejects_names_and_bad_digits() {
        for text in ["red", "#12", "#gggggg", "2ca02c", "#2ca02cff"] {
            assert!(
                matches!(parse_color(text), Err(ExportError::InvalidColor(_))),
                "{text} should be rejected"
            );
        }
    }

    #[test]
    fn test_axis_range_pads_and_widens_flat_axes() {
        let padded = axis_range([0.0, 10.0].into_iter(), 0.05);
        assert!((padded.start + 0.5).abs() < 1e-12 && (padded.end - 10.5).abs() < 1e-12);
        let flat = axis_range([4.0, 4.0].into_iter(), 0.05);
        assert!(flat.start < 4.0 && flat.end > 4.0);
        let zero = axis_range([0.0].into_iter(), 0.05);
        assert!((zero.start + 1.0).abs() < 1e-12 && (zero.end - 1.0).abs() < 1e-12);
        let empty = axis_range(std::iter::empty(), 0.05);
        assert!((empty.start).abs() < 1e-12 && (empty.end - 1.0).abs() < 1e-12);
    }

    #[test]
    fn test_surface_grid_rejects_missing_and_duplicate_points() {
        let incomplete = Surface3D {
            x: vec![dec!(1), dec!(2), dec!(1)],
            y: vec![dec!(1), dec!(1), dec!(2)],
            z: vec![dec!(0.1), dec!(0.2), dec!(0.3)],
            name: "iv".into(),
        };
        assert!(matches!(
            surface_grid(&incomplete),
            Err(ExportError::IncompleteSurface {
                expected: 4,
                found: 3,
                ..
            })
        ));
        let duplicate = Surface3D {
            x: vec![dec!(1), dec!(1)],
            y: vec![dec!(1), dec!(1)],
            z: vec![dec!(0.1), dec!(0.2)],
            name: "iv".into(),
        };
        assert!(matches!(
            surface_grid(&duplicate),
            Err(ExportError::DuplicateSurfacePoint { .. })
        ));
    }

    #[test]
    fn test_surface_grid_orders_a_shuffled_complete_grid() -> Result<(), ExportError> {
        let surface = Surface3D {
            x: vec![dec!(2), dec!(1), dec!(2), dec!(1)],
            y: vec![dec!(20), dec!(10), dec!(10), dec!(20)],
            z: vec![dec!(4), dec!(1), dec!(3), dec!(2)],
            name: "iv".into(),
        };
        let grid = surface_grid(&surface)?;
        assert_eq!(grid.xs, vec![1.0, 2.0]);
        assert_eq!(grid.ys, vec![10.0, 20.0]);
        assert_eq!(grid.z_at(0, 0), Some(1.0));
        assert_eq!(grid.z_at(0, 1), Some(2.0));
        assert_eq!(grid.z_at(1, 0), Some(3.0));
        assert_eq!(grid.z_at(1, 1), Some(4.0));
        Ok(())
    }

    #[test]
    fn test_axis_range_keeps_a_tiny_real_span() {
        let range = axis_range([0.0, 1e-18].into_iter(), 0.05);
        assert!(range.start < 0.0 && range.end > 1e-18);
        assert!(range.end - range.start < 2e-18);
    }

    #[test]
    fn test_format_number_keeps_small_ticks_distinct() {
        let ticks: Vec<String> = [0.0, 2e-6, 4e-6, 6e-6, 8e-6, 1e-5]
            .iter()
            .map(|value| format_number(*value, 2e-6))
            .collect();
        let distinct: std::collections::BTreeSet<&String> = ticks.iter().collect();
        assert_eq!(distinct.len(), ticks.len(), "{ticks:?}");
        assert_eq!(format_number(2e-6, 2e-6), "2e-6");
        assert_eq!(format_number(-2e-6, 2e-6), "\u{2212}2e-6");
        assert_ne!(format_number(1.0, 2e-7), format_number(1.0000002, 2e-7));
        assert_eq!(format_number(f64::INFINITY, 1.0), "inf");
    }

    #[test]
    fn test_format_number_trims_zeros_and_uses_a_true_minus() {
        assert_eq!(format_number(20.0, 10.0), "20");
        assert_eq!(format_number(-20.0, 10.0), "\u{2212}20");
        assert_eq!(format_number(0.25, 0.05), "0.25");
        assert_eq!(format_number(0.3, 0.05), "0.3");
        assert_eq!(format_number(-0.0001, 10.0), "0");
        assert_eq!(format_number(12.7, 25.0), "13");
    }
}
