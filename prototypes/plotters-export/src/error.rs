//! Errors of the native exporter.

use rust_decimal::Decimal;
use thiserror::Error;

/// Why a chart could not be exported.
///
/// Every case the Plotly path papers over (a surface with missing grid
/// points is filled with zeros there, mismatched series lengths are cut to
/// the shorter one by plotly.js) is an error here: the prototype never
/// drops a point or invents a value.
#[derive(Debug, Error)]
pub enum ExportError {
    /// The configured width or height is zero.
    #[error("invalid dimensions {width}x{height}: both must be positive")]
    InvalidDimensions {
        /// Configured width in pixels.
        width: u32,
        /// Configured height in pixels.
        height: u32,
    },

    /// The figure is too small to lay the chart out.
    #[error(
        "{width}x{height} is too small for this chart; it needs at least {min_width}x{min_height}"
    )]
    TooSmall {
        /// Configured width in pixels.
        width: u32,
        /// Configured height in pixels.
        height: u32,
        /// Smallest width the chart is laid out in.
        min_width: u32,
        /// Smallest height the chart is laid out in.
        min_height: u32,
    },

    /// The title, legend or colour bar leave no pixels to draw the chart in.
    #[error("{width}x{height} leaves no room for the chart under its title, legend or colour bar")]
    NoRoom {
        /// Configured width in pixels.
        width: u32,
        /// Configured height in pixels.
        height: u32,
    },

    /// A line width is not a finite number of pixels between 0 and 100.
    #[error("series {name:?} has line width {width}: expected a finite width in 0..=100 px")]
    InvalidLineWidth {
        /// Name of the series.
        name: String,
        /// The width given.
        width: f64,
    },

    /// A 2D series has a different number of x and y values.
    #[error("series {name:?} has {x} x values and {y} y values")]
    LengthMismatch {
        /// Name of the series.
        name: String,
        /// Number of x values.
        x: usize,
        /// Number of y values.
        y: usize,
    },

    /// A surface has a different number of x, y and z values.
    #[error("surface {name:?} has {x} x, {y} y and {z} z values")]
    SurfaceLengthMismatch {
        /// Name of the surface.
        name: String,
        /// Number of x values.
        x: usize,
        /// Number of y values.
        y: usize,
        /// Number of z values.
        z: usize,
    },

    /// A surface needs at least two distinct x and two distinct y values.
    #[error("surface {name:?} has a {columns}x{rows} grid; at least 2x2 is needed")]
    DegenerateSurface {
        /// Name of the surface.
        name: String,
        /// Distinct x values.
        columns: usize,
        /// Distinct y values.
        rows: usize,
    },

    /// A surface grid point appears more than once.
    #[error("surface {name:?} has more than one z value at ({x}, {y})")]
    DuplicateSurfacePoint {
        /// Name of the surface.
        name: String,
        /// x of the repeated point.
        x: Decimal,
        /// y of the repeated point.
        y: Decimal,
    },

    /// A surface grid point is missing: `expected` is distinct x times
    /// distinct y, `found` the number of points given.
    #[error("surface {name:?} is an incomplete grid: {found} of {expected} points")]
    IncompleteSurface {
        /// Name of the surface.
        name: String,
        /// Points a complete grid has.
        expected: usize,
        /// Points the surface has.
        found: usize,
    },

    /// A value has no `f64` representation for drawing.
    #[error("value {0} cannot be represented as f64")]
    NotRepresentable(Decimal),

    /// A colour is not `#RGB` or `#RRGGBB`.
    #[error("unsupported colour {0:?}: expected #RGB or #RRGGBB")]
    InvalidColor(String),

    /// The embedded font could not be registered.
    #[error("embedded font rejected: {0}")]
    Font(String),

    /// A label uses a character the bundled font has no glyph for, so it
    /// cannot be drawn as outlines. The other modes leave such a character to
    /// the viewer (or draw the font's empty-box glyph in PNG); outlines never
    /// substitute or drop it.
    #[error("the bundled font has no glyph for {character:?} (U+{code:04X}) in label {text:?}")]
    MissingGlyph {
        /// The character without a glyph.
        character: char,
        /// Its code point, for display.
        code: u32,
        /// The whole label.
        text: String,
    },

    /// A text element is outside the subset of SVG the outline conversion
    /// understands: the exact `<text>` elements plotters-svg 0.3.7 writes
    /// for the regular style of the bundled font.
    #[error("unsupported SVG text: {0}")]
    UnsupportedSvgText(String),

    /// Plotters failed while drawing or encoding.
    #[error("render failed: {0}")]
    Render(String),

    /// The output file or its directory could not be written.
    #[error("i/o error: {0}")]
    Io(#[from] std::io::Error),
}
