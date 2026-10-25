//! The native renderer: a `GraphData` and its `GraphConfig` drawn with
//! Plotters, with no browser, driver or network.
//!
//! It reads the chart exactly the way `Graph::to_plot` does (legend labels
//! override series names, `ColorScheme` colours series without one) and
//! mirrors the Plotly defaults where the data leaves a choice open: the
//! plotly.js colourway, a 2px line, a 6px marker, a 16px text label and a
//! 10px marker for the zero-width "label" markers `make_scatter` special-
//! cases. Where Plotly would silently fix bad data, it returns an
//! [`GraphError`] instead (see that type).

use super::backend::{TextBackend, TextErrors, TextMode};
use super::color::parse_color;
use super::outline::{self, ChartText, Outlined};
use crate::error::GraphError;
use crate::visualization::{
    GraphConfig, GraphData, Series2D, Surface3D, TraceMode, get_color_from_scheme,
};
use num_traits::ToPrimitive;
use plotters::coord::Shift;
use plotters::coord::types::RangedCoordf64;
use plotters::prelude::*;
use plotters::style::colors::colormaps::ViridisRGB;
use plotters::style::text_anchor::{HPos, Pos, VPos};
use rust_decimal::Decimal;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Family name of the bundled font: every text style names it, and the
/// outline conversion accepts only text in it.
pub(crate) const FONT_FAMILY: &str = "Open Sans";

/// Open Sans Regular (SIL OFL 1.1, `assets/OFL.txt`): the first family in
/// Plotly's default font stack.
pub(crate) const FONT_BYTES: &[u8] = include_bytes!("../../../assets/OpenSans-Regular.ttf");

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
/// Where a legend entry's name starts: after its sample and a gap.
const LEGEND_TEXT_X: i32 = LEGEND_MARGIN_PX + LEGEND_SAMPLE_PX + LEGEND_GAP_PX;

/// 3D view: Plotters' yaw/pitch in radians and its scale factor.
const SURFACE_YAW: f64 = 0.7;
const SURFACE_PITCH: f64 = 0.5;
const SURFACE_SCALE: f64 = 0.8;
/// Tick labels per surface axis.
const SURFACE_TICKS: usize = 6;
/// Clearance kept between two surface labels, in pixels.
const LABEL_GAP_PX: f64 = 2.0;
/// Length of a surface axis tick, in pixels.
const SURFACE_TICK_PX: i32 = 4;
/// Where the x and y axis titles sit beyond the box, as a fraction of the
/// span they are offset along, and how far out they may move, in steps,
/// to clear the tick labels (#940).
const TITLE_OFFSET: f64 = 0.22;
const TITLE_STEP: f64 = 0.02;
const TITLE_STEPS: u32 = 40;
const COLORBAR_WIDTH_PX: u32 = 110;
/// Smallest figure a surface is laid out in: the colour bar plus room for
/// the 3D axes, and a height that leaves an area under the title.
const SURFACE_MIN_WIDTH_PX: u32 = COLORBAR_WIDTH_PX + 130;
const SURFACE_MIN_HEIGHT_PX: u32 = 160;
const COLORBAR_STEPS: i32 = 100;
const COLORBAR_TICKS: i32 = 5;
/// The colour bar's left and right edges, its tick ends and label start,
/// in pixels from the left of its area.
const COLORBAR_LEFT_PX: i32 = 10;
const COLORBAR_RIGHT_PX: i32 = 30;
const COLORBAR_TICK_END_PX: i32 = COLORBAR_RIGHT_PX + 4;
const COLORBAR_LABEL_X: i32 = COLORBAR_RIGHT_PX + 8;

/// Writes `data` to `path` as a PNG of `config.width` x `config.height`.
///
/// Creates the parent directory when it is missing. The image is drawn into
/// a [`Scratch`] directory next to `path` and renamed over it only once
/// complete, so a failed export never leaves a partial image (Plotters'
/// bitmap backend writes its file even when drawing fails).
///
/// # Errors
///
/// [`GraphError`] for invalid dimensions or data, a label the font cannot
/// draw or a drawing failure; [`GraphError::Io`] when the file cannot be
/// written or the image cannot be encoded into it.
#[tracing::instrument(skip(data, config), fields(title = %config.title))]
pub(crate) fn write_png(
    data: &GraphData,
    config: &GraphConfig,
    path: &Path,
) -> Result<(), GraphError> {
    let size = dimensions(config)?;
    prepare_parent(path)?;
    let scratch = Scratch::create(path)?;
    let image = scratch.file("image.png");
    let errors = TextErrors::default();
    let result = {
        let backend = TextBackend::new(
            BitMapBackend::new(&image, size),
            TextMode::Raster,
            errors.clone(),
        );
        let root = backend.into_drawing_area();
        draw(&root, data, config).and_then(|()| root.present().map_err(png_error))
        // `root`, and with it the backend that writes `image`, drops here.
    };
    typed_text_error_first(result, &errors)?;
    scratch.publish(&image, path)
}

/// The bitmap backend's error, which `plotters` does not re-export.
type BitMapBackendError = <BitMapBackend<'static> as DrawingBackend>::ErrorType;

/// How many names [`Scratch::create`] tries before giving up.
const SCRATCH_ATTEMPTS: u32 = 64;

/// A directory of one export's own, next to its output, for the file being
/// written. It is created exclusively, so an entry already there under that
/// name (a symlink included) is never reused, and on Unix only its owner can
/// enter it: nothing planted beforehand can redirect the file drawn inside.
/// Its name is short whatever the output is called. Dropping it removes the
/// directory and anything left in it.
struct Scratch {
    dir: std::path::PathBuf,
}

impl Scratch {
    fn create(path: &Path) -> Result<Self, GraphError> {
        use std::sync::atomic::{AtomicU64, Ordering};
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let process = std::process::id();
        Self::create_in(
            parent,
            std::iter::repeat_with(|| {
                format!(
                    ".osl-export-{process}-{}",
                    SEQUENCE.fetch_add(1, Ordering::Relaxed)
                )
            })
            .take(SCRATCH_ATTEMPTS.to_usize().unwrap_or(1)),
        )
    }

    /// The first of `names` that can be created as a new directory in
    /// `parent`.
    fn create_in(parent: &Path, names: impl Iterator<Item = String>) -> Result<Self, GraphError> {
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)]
        std::os::unix::fs::DirBuilderExt::mode(&mut builder, 0o700);
        let mut taken = None;
        for name in names {
            let dir = parent.join(name);
            match builder.create(&dir) {
                Ok(()) => return Ok(Self { dir }),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    taken = Some(error);
                }
                Err(error) => return Err(error.into()),
            }
        }
        Err(taken
            .unwrap_or_else(|| std::io::Error::from(std::io::ErrorKind::AlreadyExists))
            .into())
    }

    /// `name` inside the directory.
    fn file(&self, name: &str) -> std::path::PathBuf {
        self.dir.join(name)
    }

    /// Moves the complete `file` over `path`: an existing `path` is
    /// replaced as a whole, never written through.
    fn publish(self, file: &Path, path: &Path) -> Result<(), GraphError> {
        std::fs::rename(file, path)?;
        Ok(())
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        // Best effort: whatever the export returned stands. The directory is
        // this export's own, so everything in it is this export's file.
        if let Err(error) = std::fs::remove_dir_all(&self.dir)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            tracing::warn!(path = %self.dir.display(), %error, "export scratch directory left behind");
        }
    }
}

/// A PNG export's failure: a failure to write or encode the file is
/// [`GraphError::Io`]. The encoder's own error type belongs to `image`,
/// which this crate does not name, and with the dimensions bounded it fails
/// only writing the file; its kind is that of the first [`std::io::Error`]
/// in the error's source chain, or [`std::io::ErrorKind::Other`] when the
/// chain has none (image 0.24 does not expose the one it wraps).
#[cold]
#[inline(never)]
fn png_error(error: DrawingAreaErrorKind<BitMapBackendError>) -> GraphError {
    use plotters_backend::DrawingErrorKind;
    match error {
        DrawingAreaErrorKind::BackendError(DrawingErrorKind::DrawingError(
            BitMapBackendError::IOError(error),
        )) => GraphError::Io(error),
        DrawingAreaErrorKind::BackendError(DrawingErrorKind::DrawingError(
            BitMapBackendError::ImageError(error),
        )) => {
            let kind =
                std::iter::successors(std::error::Error::source(&error), |cause| cause.source())
                    .find_map(|cause| cause.downcast_ref::<std::io::Error>())
                    .map_or(std::io::ErrorKind::Other, std::io::Error::kind);
            GraphError::Io(std::io::Error::new(kind, error.to_string()))
        }
        other => render_error(other),
    }
}

/// Renders `data` as an SVG document of `config.width` x `config.height`,
/// every label drawn as outlines of the bundled font (see [`super::outline`]),
/// and returns where each label's ink is.
///
/// # Errors
///
/// [`GraphError`] for invalid dimensions or data or a drawing failure, a
/// character the font has no glyph for ([`GraphError::MissingGlyph`]) or a
/// label, title or description XML cannot carry
/// ([`GraphError::NotXmlText`]).
pub(crate) fn render_svg(data: &GraphData, config: &GraphConfig) -> Result<Outlined, GraphError> {
    let document = render_plotters_svg(data, config)?;
    outline::outline_text(&document, &describe(data, config))
}

/// The document plotters-svg writes, before any text post-processing.
fn render_plotters_svg(data: &GraphData, config: &GraphConfig) -> Result<String, GraphError> {
    let size = dimensions(config)?;
    let errors = TextErrors::default();
    let mut document = String::new();
    let result = {
        let backend = TextBackend::new(
            SVGBackend::with_string(&mut document, size),
            TextMode::Svg,
            errors.clone(),
        );
        let root = backend.into_drawing_area();
        draw(&root, data, config).and_then(|()| root.present().map_err(render_error))
    };
    typed_text_error_first(result, &errors)?;
    Ok(document)
}

/// A text failure reaches Plotters' error only as a message; the typed one
/// the text backend stashed wins.
fn typed_text_error_first(
    result: Result<(), GraphError>,
    errors: &TextErrors,
) -> Result<(), GraphError> {
    match errors.borrow_mut().take() {
        Some(error) => Err(error),
        None => result,
    }
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
/// Creates the parent directory when it is missing, and like
/// [`write_png`] writes through a [`Scratch`] directory, so `path` is
/// replaced only by a complete document.
///
/// # Errors
///
/// Everything [`render_svg`] returns, and [`GraphError::Io`] when the file
/// cannot be written.
pub(crate) fn write_svg(
    data: &GraphData,
    config: &GraphConfig,
    path: &Path,
) -> Result<(), GraphError> {
    let document = render_svg(data, config)?.document;
    prepare_parent(path)?;
    let scratch = Scratch::create(path)?;
    let file = scratch.file("image.svg");
    std::fs::write(&file, document)?;
    scratch.publish(&file, path)
}

/// The largest width or height drawn, in pixels. It bounds the PNG buffer
/// (3 bytes a pixel, about 805 MB at 16384 x 16384) and keeps every
/// coordinate far inside the `i32` Plotters draws with, which a larger
/// figure overflows inside Plotters before any of this crate's checks run.
const MAX_SIDE_PX: u32 = 16_384;

fn dimensions(config: &GraphConfig) -> Result<(u32, u32), GraphError> {
    let side = 1..=MAX_SIDE_PX;
    if !side.contains(&config.width) || !side.contains(&config.height) {
        return Err(GraphError::InvalidDimensions {
            width: config.width,
            height: config.height,
        });
    }
    Ok((config.width, config.height))
}

fn prepare_parent(path: &Path) -> Result<(), GraphError> {
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
fn render_error<E: std::error::Error + Send + Sync>(error: DrawingAreaErrorKind<E>) -> GraphError {
    GraphError::Render(error.to_string())
}

/// A Plotters font of `px` CSS pixels.
fn font(px: f64) -> FontDesc<'static> {
    (FONT_FAMILY, px * EM_RATIO).into_font()
}

fn to_f64(value: Decimal) -> Result<f64, GraphError> {
    f64::try_from(value).map_err(|_| GraphError::NotRepresentable(value))
}

/// The plotly.js colourway entry for the `index`-th trace.
fn default_color(index: usize) -> Result<RGBAColor, GraphError> {
    let entry = index
        .checked_rem(COLORWAY.len())
        .and_then(|slot| COLORWAY.get(slot))
        .ok_or_else(|| GraphError::InvalidColor {
            color: format!("colourway entry {index}"),
            reason: "the colourway is empty".to_string(),
        })?;
    parse_color(entry)
}

fn draw<DB: DrawingBackend>(
    root: &DrawingArea<DB, Shift>,
    data: &GraphData,
    config: &GraphConfig,
) -> Result<(), GraphError>
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
) -> Result<(), GraphError> {
    let (width, height) = area.dim_in_pixel();
    let (root_width, root_height) = root.dim_in_pixel();
    if width == 0 || height == 0 || width > root_width || height > root_height {
        return Err(GraphError::NoRoom {
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
    color: RGBAColor,
    line_width: u32,
    marker_radius: u32,
}

fn prepare_traces(series: &[Series2D]) -> Result<Vec<Trace>, GraphError> {
    let mut traces = Vec::with_capacity(series.len());
    for (index, item) in series.iter().enumerate() {
        if item.x.len() != item.y.len() {
            return Err(GraphError::LengthMismatch {
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
            return Err(GraphError::InvalidLineWidth {
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
            line_width: width
                .round()
                .to_u32()
                .ok_or_else(|| GraphError::InvalidLineWidth {
                    name: item.name.clone(),
                    width,
                })?,
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
) -> Result<(), GraphError>
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
) -> Result<u32, GraphError>
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
) -> Result<(), GraphError>
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
            (LEGEND_TEXT_X, row),
            text_style.clone(),
        ))
        .map_err(render_error)?;
        row = row
            .checked_add(LEGEND_ROW_PX)
            .ok_or_else(|| GraphError::Render("legend row out of range".to_string()))?;
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
fn surface_grid(surface: &Surface3D) -> Result<Grid, GraphError> {
    let (x, y, z) = (surface.x.len(), surface.y.len(), surface.z.len());
    if x != y || y != z {
        return Err(GraphError::SurfaceLengthMismatch {
            name: surface.name.clone(),
            x,
            y,
            z,
        });
    }
    let mut cells: BTreeMap<(Decimal, Decimal), Decimal> = BTreeMap::new();
    for ((x, y), z) in surface.x.iter().zip(&surface.y).zip(&surface.z) {
        if cells.insert((*x, *y), *z).is_some() {
            return Err(GraphError::DuplicateSurfacePoint {
                name: surface.name.clone(),
                x: *x,
                y: *y,
            });
        }
    }
    let xs: BTreeSet<Decimal> = cells.keys().map(|(x, _)| *x).collect();
    let ys: BTreeSet<Decimal> = cells.keys().map(|(_, y)| *y).collect();
    if xs.len() < 2 || ys.len() < 2 {
        return Err(GraphError::DegenerateSurface {
            name: surface.name.clone(),
            columns: xs.len(),
            rows: ys.len(),
        });
    }
    let expected = xs.len().checked_mul(ys.len());
    let incomplete = || GraphError::IncompleteSurface {
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
) -> Result<(), GraphError>
where
    DB::ErrorType: 'static,
{
    let grid = surface_grid(surface)?;
    if config.width < SURFACE_MIN_WIDTH_PX || config.height < SURFACE_MIN_HEIGHT_PX {
        return Err(GraphError::TooSmall {
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
    let hidden_depth = corner_clash(
        &plot_area,
        chart.as_coord_spec(),
        (&x_range, &y_range, z_low),
        (&x_format, &|value: &f64| format_number(*value, y_step)),
    )?;
    let depth_format = move |value: &f64| match hidden_depth {
        Some(hidden) if (value - hidden).abs() <= y_step * 1e-9 => String::new(),
        _ => format_number(*value, y_step),
    };
    chart
        .configure_axes()
        .x_formatter(&x_format)
        .y_formatter(&height_format)
        .z_formatter(&depth_format)
        .x_labels(SURFACE_TICKS)
        .y_labels(SURFACE_TICKS)
        .z_labels(SURFACE_TICKS)
        .light_grid_style(GRID)
        .bold_grid_style(GRID.mix(0.0))
        .axis_panel_style(RGBColor(248, 248, 250).filled())
        .label_style(font(TICK_PX).color(&TEXT))
        .max_light_lines(4)
        .tick_size(SURFACE_TICK_PX)
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
    let coord = chart.as_coord_spec();
    let ticks = surface_tick_boxes(
        &plot_area,
        coord,
        chart.plotting_area().get_pixel_range(),
        (&x_range, &y_range, z_low),
        (&x_format, &depth_format),
    )?;
    let (area_x, area_y) = plot_area.get_pixel_range();
    let area = PixelBox {
        left: f64::from(area_x.start),
        top: f64::from(area_y.start),
        right: f64::from(area_x.end),
        bottom: f64::from(area_y.end),
    };
    let title_style = TextStyle::from(font(AXIS_TITLE_PX));
    // An axis title fits at an offset when its box stays in the plot area
    // and clear of every tick label.
    let fits = |label: &str, at: (f64, f64, f64)| -> Result<bool, GraphError> {
        let size = plot_area
            .estimate_text_size(label, &title_style)
            .map_err(render_error)?;
        let title = PixelBox::centred(coord.translate(&at), size);
        Ok(title.inside(&area) && ticks.iter().all(|tick| title.clear_of(tick)))
    };
    let mut titles = Vec::with_capacity(3);
    if let Some(label) = &config.x_label {
        let at = |offset: f64| (x_mid, z_low, y_range.end + y_span * offset);
        let offset = title_offset(|offset| fits(label, at(offset)))?;
        titles.push((label, at(offset)));
    }
    if let Some(label) = &config.y_label {
        let at = |offset: f64| (x_range.end + x_span * offset, z_low, y_mid);
        let offset = title_offset(|offset| fits(label, at(offset)))?;
        titles.push((label, at(offset)));
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
) -> Result<(), GraphError>
where
    DB::ErrorType: 'static,
{
    // Geometry in i64: `dim_in_pixel` of an area squeezed below zero by a
    // caption comes back as a huge u32, which must not overflow here.
    let (_, height) = area.dim_in_pixel();
    let height = i64::from(height);
    let geometry = || GraphError::Render("colour bar geometry out of range".to_string());
    let top = height.checked_div(6).ok_or_else(geometry)?;
    let bottom = height.checked_sub(top).ok_or_else(geometry)?;
    let (left, right) = (COLORBAR_LEFT_PX, COLORBAR_RIGHT_PX);
    let span = bottom.checked_sub(top).ok_or_else(geometry)?.max(1);
    let steps = i64::from(COLORBAR_STEPS);
    let pixel = |value: i64| {
        i32::try_from(value)
            .map_err(|_| GraphError::Render(format!("colour bar coordinate {value} out of range")))
    };
    // `bottom - span * k / steps`, the k-th step's edge.
    let edge = |k: i64| {
        span.checked_mul(k)
            .and_then(|offset| offset.checked_div(steps))
            .and_then(|offset| bottom.checked_sub(offset))
            .ok_or_else(geometry)
            .and_then(pixel)
    };
    for step in 0..steps {
        let y0 = edge(step)?;
        let y1 = edge(step.checked_add(1).ok_or_else(geometry)?)?;
        let fraction = step as f64 / f64::from(COLORBAR_STEPS - 1);
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
        let offset = (span as f64 * fraction)
            .round()
            .to_i64()
            .ok_or_else(geometry)?;
        let y = pixel(bottom.checked_sub(offset).ok_or_else(geometry)?)?;
        area.draw(&PathElement::new(
            vec![(right, y), (COLORBAR_TICK_END_PX, y)],
            AXIS_LINE.stroke_width(1),
        ))
        .map_err(render_error)?;
        area.draw(&Text::new(
            format_number(value, (high - low) / f64::from(COLORBAR_TICKS - 1)),
            (COLORBAR_LABEL_X, y),
            tick_style.clone(),
        ))
        .map_err(render_error)?;
    }
    if let Some(title) = title {
        let style = font(TICK_PX)
            .color(&TEXT)
            .pos(Pos::new(HPos::Left, VPos::Bottom));
        let above = top.checked_sub(12).ok_or_else(geometry)?;
        area.draw(&Text::new(title, (left, pixel(above)?), style))
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
        // Clamped to 0..=MAX_DIGITS, so the conversion is exact.
        let decimals = (-step.log10())
            .ceil()
            .clamp(0.0, MAX_DIGITS)
            .to_usize()
            .unwrap_or_default();
        trim_fraction(&format!("{value:.decimals$}"))
    } else {
        let digits = (magnitude / step)
            .log10()
            .ceil()
            .clamp(0.0, MAX_DIGITS)
            .to_usize()
            .unwrap_or_default();
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
        // The span is in (0, 1e-9): its logarithm is a small negative number.
        10_f64.powi(span.log10().floor().to_i32().unwrap_or_default())
    } else {
        1.0
    }
}

/// How an axis writes a tick value.
type TickFormat<'a> = &'a dyn Fn(&f64) -> String;

/// The depth tick whose label would land on the last x tick's label at the
/// front corner of the surface (#937), so it can be left blank.
///
/// Plotters draws the x and depth axes into the same front corner, each
/// label hanging off its tick. When both axes have a tick at or near that
/// corner, their labels are drawn over each other; the x label is kept and
/// the depth one dropped, which leaves every other label where it was.
///
/// # Errors
///
/// [`GraphError`] when a label cannot be measured.
fn corner_clash<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    coord: &impl CoordTranslate<From = (f64, f64, f64)>,
    (x_range, depth_range, height): (&std::ops::Range<f64>, &std::ops::Range<f64>, f64),
    (x_format, depth_format): (TickFormat<'_>, TickFormat<'_>),
) -> Result<Option<f64>, GraphError>
where
    DB::ErrorType: 'static,
{
    // The ticks Plotters draws: the same key points `configure_axes` asks
    // for with six labels per axis.
    let last_tick = |range: &std::ops::Range<f64>| {
        RangedCoordf64::from(range.clone())
            .key_points(SURFACE_TICKS)
            .into_iter()
            .reduce(f64::max)
    };
    let (Some(x_tick), Some(depth_tick)) = (last_tick(x_range), last_tick(depth_range)) else {
        return Ok(None);
    };
    // From the fixed viewpoint the two axes meet at (x end, depth end), on
    // the floor of the box.
    let x_anchor = coord.translate(&(x_tick, height, depth_range.end));
    let depth_anchor = coord.translate(&(x_range.end, height, depth_tick));
    let style = TextStyle::from(font(TICK_PX));
    let measure = |text: &str| area.estimate_text_size(text, &style).map_err(render_error);
    let (x_width, x_height) = measure(&x_format(&x_tick))?;
    let (depth_width, depth_height) = measure(&depth_format(&depth_tick))?;
    let apart_x = (f64::from(x_anchor.0) - f64::from(depth_anchor.0)).abs();
    let apart_y = (f64::from(x_anchor.1) - f64::from(depth_anchor.1)).abs();
    let room_x = (f64::from(x_width) + f64::from(depth_width)) / 2.0 + LABEL_GAP_PX;
    let room_y = f64::from(x_height.max(depth_height)) + LABEL_GAP_PX;
    Ok((apart_x < room_x && apart_y < room_y).then_some(depth_tick))
}

/// A label's box in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
struct PixelBox {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl PixelBox {
    /// A box of `size` centred on `at`.
    fn centred(at: (i32, i32), (width, height): (u32, u32)) -> Self {
        let (x, y) = (f64::from(at.0), f64::from(at.1));
        let (half_width, half_height) = (f64::from(width) / 2.0, f64::from(height) / 2.0);
        Self {
            left: x - half_width,
            top: y - half_height,
            right: x + half_width,
            bottom: y + half_height,
        }
    }

    /// At least [`LABEL_GAP_PX`] away from `other` on some side.
    fn clear_of(&self, other: &Self) -> bool {
        self.right + LABEL_GAP_PX <= other.left
            || other.right + LABEL_GAP_PX <= self.left
            || self.bottom + LABEL_GAP_PX <= other.top
            || other.bottom + LABEL_GAP_PX <= self.top
    }

    /// Wholly within `area`.
    fn inside(&self, area: &Self) -> bool {
        self.left >= area.left
            && self.top >= area.top
            && self.right <= area.right
            && self.bottom <= area.bottom
    }
}

/// Where Plotters puts the x and depth tick labels of a surface, in pixels.
///
/// Mirrors `draw_axis_ticks`: each label hangs off its tick, two tick
/// lengths away, on the side of the axis that faces away from the centre of
/// the plotting area. From the fixed viewpoint the x axis runs along the
/// front edge (depth end) and the depth axis along the right edge (x end),
/// both on the floor of the box. Blank labels are left out.
///
/// # Errors
///
/// [`GraphError`] when a label cannot be measured.
fn surface_tick_boxes<DB: DrawingBackend>(
    area: &DrawingArea<DB, Shift>,
    coord: &impl CoordTranslate<From = (f64, f64, f64)>,
    (pixels_x, pixels_y): (std::ops::Range<i32>, std::ops::Range<i32>),
    (x_range, depth_range, height): (&std::ops::Range<f64>, &std::ops::Range<f64>, f64),
    (x_format, depth_format): (TickFormat<'_>, TickFormat<'_>),
) -> Result<Vec<PixelBox>, GraphError>
where
    DB::ErrorType: 'static,
{
    let style = TextStyle::from(font(TICK_PX));
    let pixel = |at: (i32, i32)| (f64::from(at.0), f64::from(at.1));
    let centre = (
        (f64::from(pixels_x.start) + f64::from(pixels_x.end)) / 2.0,
        (f64::from(pixels_y.start) + f64::from(pixels_y.end)) / 2.0,
    );
    let tick = f64::from(SURFACE_TICK_PX);
    let x_axis: [(f64, f64, f64); 2] = [
        (x_range.start, height, depth_range.end),
        (x_range.end, height, depth_range.end),
    ];
    let depth_axis: [(f64, f64, f64); 2] = [
        (x_range.end, height, depth_range.start),
        (x_range.end, height, depth_range.end),
    ];
    let x_ticks: Vec<((f64, f64, f64), String)> = RangedCoordf64::from(x_range.clone())
        .key_points(SURFACE_TICKS)
        .into_iter()
        .map(|x| ((x, height, depth_range.end), x_format(&x)))
        .collect();
    let depth_ticks: Vec<((f64, f64, f64), String)> = RangedCoordf64::from(depth_range.clone())
        .key_points(SURFACE_TICKS)
        .into_iter()
        .map(|depth| ((x_range.end, height, depth), depth_format(&depth)))
        .collect();
    let mut boxes = Vec::new();
    for ([begin, end], ticks) in [(x_axis, x_ticks), (depth_axis, depth_ticks)] {
        let (begin, end) = (pixel(coord.translate(&begin)), pixel(coord.translate(&end)));
        let axis = (end.0 - begin.0, end.1 - begin.1);
        let x_dir = if begin.0 < centre.0 {
            (-tick, 0.0)
        } else {
            (tick, 0.0)
        };
        let y_dir = if begin.1 < centre.1 {
            (0.0, -tick)
        } else {
            (0.0, tick)
        };
        let score = |dir: (f64, f64)| (dir.0 * axis.0 + dir.1 * axis.1).abs();
        let dir = if score(x_dir) < score(y_dir) {
            x_dir
        } else {
            y_dir
        };
        for (at, text) in ticks {
            if text.is_empty() {
                continue;
            }
            let (width, label_height) = area
                .estimate_text_size(&text, &style)
                .map_err(render_error)?;
            let anchor = pixel(coord.translate(&at));
            let (x, y) = (anchor.0 + dir.0 * 2.0, anchor.1 + dir.1 * 2.0);
            let (width, label_height) = (f64::from(width), f64::from(label_height));
            boxes.push(if dir.0 == 0.0 && dir.1 > 0.0 {
                PixelBox {
                    left: x - width / 2.0,
                    top: y,
                    right: x + width / 2.0,
                    bottom: y + label_height,
                }
            } else if dir.0 == 0.0 {
                PixelBox {
                    left: x - width / 2.0,
                    top: y - label_height,
                    right: x + width / 2.0,
                    bottom: y,
                }
            } else if dir.0 < 0.0 {
                PixelBox {
                    left: x - width,
                    top: y - label_height / 2.0,
                    right: x,
                    bottom: y + label_height / 2.0,
                }
            } else {
                PixelBox {
                    left: x,
                    top: y - label_height / 2.0,
                    right: x + width,
                    bottom: y + label_height / 2.0,
                }
            });
        }
    }
    Ok(boxes)
}

/// How far beyond the box an axis title sits, as a fraction of the span it
/// is offset along: [`TITLE_OFFSET`] when that `fits`, otherwise the
/// nearest step out that does (#940). When none does, the title stays at
/// [`TITLE_OFFSET`].
///
/// # Errors
///
/// What `fits` returns.
fn title_offset(fits: impl Fn(f64) -> Result<bool, GraphError>) -> Result<f64, GraphError> {
    for step in 0..=TITLE_STEPS {
        let offset = TITLE_OFFSET + f64::from(step) * TITLE_STEP;
        if fits(offset)? {
            return Ok(offset);
        }
    }
    Ok(TITLE_OFFSET)
}

/// Tick spacing Plotters will use for `range` with about `count` ticks.
fn tick_step(range: &std::ops::Range<f64>, count: u32) -> f64 {
    (range.end - range.start) / f64::from(count.max(1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[cfg(unix)]
    #[test]
    fn test_scratch_never_reuses_an_existing_entry() -> Result<(), Box<dyn std::error::Error>> {
        let dir = tempfile::tempdir()?;
        let victim = dir.path().join("victim.txt");
        std::fs::write(&victim, "untouched")?;
        std::os::unix::fs::symlink(&victim, dir.path().join("link"))?;
        std::fs::create_dir(dir.path().join("dir"))?;
        let names = ["link", "dir", "free"].map(str::to_string).into_iter();
        let scratch = Scratch::create_in(dir.path(), names)?;
        assert_eq!(scratch.dir, dir.path().join("free"));
        std::fs::write(scratch.file("image.png"), "partial")?;
        drop(scratch);
        assert!(!dir.path().join("free").exists());
        assert_eq!(std::fs::read_to_string(&victim)?, "untouched");
        // Every name taken: an `AlreadyExists` I/O error, not a reuse.
        let taken = Scratch::create_in(dir.path(), ["link", "dir"].map(str::to_string).into_iter());
        assert!(matches!(
            taken,
            Err(GraphError::Io(ref error)) if error.kind() == std::io::ErrorKind::AlreadyExists
        ));
        Ok(())
    }

    #[test]
    fn test_png_write_failures_are_io_errors() {
        let denied = DrawingAreaErrorKind::BackendError(
            plotters_backend::DrawingErrorKind::DrawingError(BitMapBackendError::IOError(
                std::io::Error::from(std::io::ErrorKind::PermissionDenied),
            )),
        );
        assert!(matches!(
            png_error(denied),
            GraphError::Io(ref error) if error.kind() == std::io::ErrorKind::PermissionDenied
        ));
        let buffer = DrawingAreaErrorKind::BackendError(
            plotters_backend::DrawingErrorKind::DrawingError(BitMapBackendError::InvalidBuffer),
        );
        assert!(matches!(png_error(buffer), GraphError::Render(_)));
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
            Err(GraphError::IncompleteSurface {
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
            Err(GraphError::DuplicateSurfacePoint { .. })
        ));
    }

    #[test]
    fn test_surface_grid_orders_a_shuffled_complete_grid() -> Result<(), GraphError> {
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
    fn test_title_offset_keeps_the_usual_offset_when_it_fits() -> Result<(), GraphError> {
        assert_eq!(title_offset(|_| Ok(true))?, TITLE_OFFSET);
        Ok(())
    }

    #[test]
    fn test_title_offset_steps_out_to_the_first_fit() -> Result<(), GraphError> {
        let offset = title_offset(|offset| Ok(offset > 0.3))?;
        assert!((offset - 0.32).abs() < 1e-12, "{offset}");
        // Nothing fits: the title stays where it always was.
        assert_eq!(title_offset(|_| Ok(false))?, TITLE_OFFSET);
        assert!(title_offset(|_| Err(GraphError::Render("x".to_string()))).is_err());
        Ok(())
    }

    #[test]
    fn test_pixel_box_clearance_and_containment() {
        let a = PixelBox::centred((10, 10), (10, 4));
        let touching = PixelBox::centred((20, 10), (10, 4));
        let apart = PixelBox::centred((23, 10), (10, 4));
        assert!(!a.clear_of(&touching));
        assert!(a.clear_of(&apart) && apart.clear_of(&a));
        let area = PixelBox::centred((10, 10), (20, 20));
        assert!(a.inside(&area));
        assert!(!apart.inside(&area));
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
