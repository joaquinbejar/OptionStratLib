//! Target crate (ADR-0001 D6, roadmap M1-14): **visualization**. Owns `GraphError`.

use optionstratlib_backtest::error::BacktestError;
use optionstratlib_core::error::DecimalError;
use optionstratlib_math::error::{CurveError, SurfaceError};
use rust_decimal::Decimal;
use thiserror::Error;

/// Represents errors that can occur during graph generation and rendering operations.
///
/// This error type encapsulates failures that may happen when creating, rendering,
/// or saving graphical representations of financial data such as option chains,
/// volatility surfaces, or strategy payoffs.
///
/// The PNG and SVG export (`static_export`) reports data it cannot draw
/// faithfully with its own variants rather than repairing it: a surface
/// with missing or repeated grid points, series whose coordinate arrays
/// differ in length, a colour or a label character it cannot reproduce.
/// The enum is `#[non_exhaustive]`: match it with a wildcard arm, since
/// later releases may add variants.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum GraphError {
    /// Represents errors that occur during the rendering process.
    /// Contains a descriptive message about what went wrong.
    #[error("Render error: {0}")]
    Render(String),

    /// Represents I/O errors that occur when reading from or writing to files
    /// (e.g., when saving graphs to disk).
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Error from curve operations.
    #[error(transparent)]
    Curve(CurveError),

    /// Error from surface operations.
    #[error(transparent)]
    Surface(SurfaceError),

    /// A figure a report derives for display (a total, an average) left the
    /// representable `Decimal` range.
    #[error(transparent)]
    Decimal(Box<DecimalError>),

    /// The backtest figures a report shows could not be computed, for
    /// example when the statistics of a `SimulationStats` accumulator
    /// overflow `Decimal`.
    #[error(transparent)]
    Backtest(Box<BacktestError>),

    /// The configured figure width or height is zero or larger than the
    /// 16384 pixels the static export draws.
    #[error("invalid dimensions {width}x{height}: both must be 1 to 16384 pixels")]
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

    /// A 2D series has a different number of x and y values. Plotly drew
    /// the shorter length; the static export does not drop points.
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

    /// A surface grid point appears more than once. Plotly kept the last
    /// value; the static export does not choose.
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
    /// distinct y, `found` the number of points given. Plotly filled the
    /// gaps with zero; the static export does not invent values.
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

    /// A colour string is none of the CSS Color 4 forms the static export
    /// accepts: a named colour, `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`,
    /// `rgb()`/`rgba()` or `hsl()`/`hsla()`.
    #[error("unsupported colour {color:?}: {reason}")]
    InvalidColor {
        /// The string as given.
        color: String,
        /// Why it was rejected.
        reason: String,
    },

    /// A label uses a character the bundled font has no glyph for. The
    /// static export never substitutes or drops it.
    #[error("the bundled font has no glyph for {character:?} (U+{code:04X}) in label {text:?}")]
    MissingGlyph {
        /// The character without a glyph.
        character: char,
        /// Its code point, for display.
        code: u32,
        /// The whole label.
        text: String,
    },

    /// A label, or the chart title or description an SVG carries, contains
    /// a character XML 1.0 cannot represent: a C0 control other than tab,
    /// line feed and carriage return, U+FFFE or U+FFFF.
    #[error("{character:?} (U+{code:04X}) cannot appear in XML; found in {text:?}")]
    NotXmlText {
        /// The character XML cannot carry.
        character: char,
        /// Its code point, for display.
        code: u32,
        /// The string it was found in.
        text: String,
    },

    /// Text the static export cannot draw: a font style other than the
    /// bundled regular face, or SVG text outside the form it converts.
    #[error("unsupported text: {0}")]
    UnsupportedText(String),
}

impl From<BacktestError> for GraphError {
    #[cold]
    #[inline(never)]
    fn from(err: BacktestError) -> Self {
        GraphError::Backtest(Box::new(err))
    }
}

impl From<DecimalError> for GraphError {
    #[cold]
    #[inline(never)]
    fn from(err: DecimalError) -> Self {
        GraphError::Decimal(Box::new(err))
    }
}

impl From<CurveError> for GraphError {
    #[cold]
    #[inline(never)]
    fn from(err: CurveError) -> Self {
        GraphError::Curve(err)
    }
}

impl From<SurfaceError> for GraphError {
    #[cold]
    #[inline(never)]
    fn from(err: SurfaceError) -> Self {
        GraphError::Surface(err)
    }
}
