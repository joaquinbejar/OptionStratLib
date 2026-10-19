//! A drawing backend wrapper that measures and draws every label with the
//! bundled font through [`crate::text`], so Plotters is built without any
//! font feature: no `ab_glyph`, no `ttf`, no ttf-parser.
//!
//! Plotters measures text through `DrawingBackend::estimate_text_size` (chart
//! titles, axis labels) and draws it through `DrawingBackend::draw_text`, and
//! the prototype uses no element that measures text any other way (only
//! `MultiLineText` does). The wrapper overrides those two methods and
//! delegates everything else to the backend it wraps:
//!
//! * measuring reproduces the anchor box Plotters' ab_glyph text measured,
//!   so the chart layout is the one of #920 and #923;
//! * [`TextMode::Raster`] (PNG) fills the glyph outlines with
//!   `ab_glyph_rasterizer`, keeping side bearings and sub-pixel pen
//!   positions, which Plotters' own ab_glyph rasteriser dropped;
//! * [`TextMode::Svg`] passes the label on to plotters-svg, which writes a
//!   `<text>` element without needing any font data; the outline conversion
//!   ([`crate::outline`]) or the viewer draws it.
//!
//! A text error is stashed in the shared [`TextErrors`] slot and reported as
//! a font error to Plotters, which only carries it as a string; the caller
//! returns the stashed [`ExportError`] instead, typed.

use crate::error::ExportError;
use crate::text::{self, Missing, Segment};
use ab_glyph_rasterizer::{Point, Rasterizer, point};
use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
    FontStyle, FontTransform,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Where the first text error of a drawing goes.
pub(crate) type TextErrors = Rc<RefCell<Option<ExportError>>>;

/// How the wrapped backend gets its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TextMode {
    /// Fill the glyph outlines into the wrapped bitmap.
    Raster,
    /// Hand the label to the wrapped SVG backend as a `<text>` element.
    Svg,
}

/// The error Plotters sees for a text failure; the typed one is stashed.
#[derive(Debug)]
struct TextFailed(String);

impl std::fmt::Display for TextFailed {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for TextFailed {}

/// `inner` with its text measured and drawn by [`crate::text`].
pub(crate) struct TextBackend<B> {
    inner: B,
    mode: TextMode,
    errors: TextErrors,
}

impl<B> TextBackend<B> {
    pub(crate) fn new(inner: B, mode: TextMode, errors: TextErrors) -> Self {
        TextBackend {
            inner,
            mode,
            errors,
        }
    }
}

impl<B: DrawingBackend> TextBackend<B> {
    /// Stashes `error` (the first one wins) and gives Plotters its message.
    fn fail(&self, error: ExportError) -> DrawingErrorKind<B::ErrorType> {
        let message = error.to_string();
        let mut slot = self.errors.borrow_mut();
        if slot.is_none() {
            *slot = Some(error);
        }
        DrawingErrorKind::FontError(Box::new(TextFailed(message)))
    }

    /// Only the regular face is bundled.
    fn check_style<S: BackendTextStyle>(
        &self,
        style: &S,
    ) -> Result<(), DrawingErrorKind<B::ErrorType>> {
        match style.style() {
            FontStyle::Normal => Ok(()),
            other => Err(self.fail(ExportError::UnsupportedSvgText(format!(
                "font style {}: only the regular face is bundled",
                other.as_str()
            )))),
        }
    }

    fn draw_outlines<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: BackendCoord,
    ) -> Result<(), ExportError> {
        let font = text::bundled_font()?;
        let layout = font.layout(text, style.size(), Missing::Error)?;
        let dx = match style.anchor().h_pos {
            HPos::Left => 0,
            HPos::Right => -layout.width,
            HPos::Center => -layout.width / 2,
        };
        let dy = match style.anchor().v_pos {
            VPos::Top => 0,
            VPos::Center => -layout.height / 2,
            VPos::Bottom => -layout.height,
        };
        // Outlines relative to `pos`, turned as plotters-backend turns text
        // (`FontTransform::transform`), then placed at `pos`.
        let transform = style.transform();
        let (px, py) = (f64::from(pos.0), f64::from(pos.1));
        let place = |(x, y): (f64, f64)| {
            let (x, y) = match transform {
                FontTransform::None => (x, y),
                FontTransform::Rotate90 => (-y, x),
                FontTransform::Rotate180 => (-x, -y),
                FontTransform::Rotate270 => (y, -x),
            };
            (px + x, py + y)
        };
        let mut segments = Vec::new();
        let ink = font.trace(&layout, (f64::from(dx), f64::from(dy)), |segment| {
            segments.push(match segment {
                Segment::Move(p) => Segment::Move(place(p)),
                Segment::Line(p) => Segment::Line(place(p)),
                Segment::Quad(c, p) => Segment::Quad(place(c), place(p)),
                Segment::Cubic(c0, c1, p) => Segment::Cubic(place(c0), place(c1), place(p)),
                Segment::Close => Segment::Close,
            });
        })?;
        if ink.is_none() {
            return Ok(());
        }
        self.fill(&segments, style.color())
    }

    /// Fills closed `segments` (nonzero winding, as TrueType) with `color`,
    /// antialiased, clipped to the backend.
    fn fill(&mut self, segments: &[Segment], color: BackendColor) -> Result<(), ExportError> {
        let points = segments.iter().flat_map(|segment| match segment {
            Segment::Move(p) | Segment::Line(p) => vec![*p],
            Segment::Quad(c, p) => vec![*c, *p],
            Segment::Cubic(c0, c1, p) => vec![*c0, *c1, *p],
            Segment::Close => Vec::new(),
        });
        let (mut x0, mut y0, mut x1, mut y1) = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for (x, y) in points {
            (x0, y0, x1, y1) = (x0.min(x), y0.min(y), x1.max(x), y1.max(y));
        }
        let (width, height) = self.inner.get_size();
        let (left, top) = (x0.floor().max(0.0), y0.floor().max(0.0));
        let (right, bottom) = (
            x1.ceil().min(f64::from(width)),
            y1.ceil().min(f64::from(height)),
        );
        if right <= left || bottom <= top {
            return Ok(());
        }
        // Whole pixels inside the canvas, so these conversions are exact.
        let (cols, rows) = ((right - left) as usize, (bottom - top) as usize);
        let local = |(x, y): (f64, f64)| point((x - left) as f32, (y - top) as f32);
        let mut raster = Rasterizer::new(cols, rows);
        let (mut start, mut current): (Option<Point>, Option<Point>) = (None, None);
        for segment in segments {
            match *segment {
                Segment::Move(p) => {
                    start = Some(local(p));
                    current = start;
                }
                Segment::Line(p) => {
                    let p = local(p);
                    if let Some(from) = current {
                        raster.draw_line(from, p);
                    }
                    current = Some(p);
                }
                Segment::Quad(c, p) => {
                    let p = local(p);
                    if let Some(from) = current {
                        raster.draw_quad(from, local(c), p);
                    }
                    current = Some(p);
                }
                Segment::Cubic(c0, c1, p) => {
                    let p = local(p);
                    if let Some(from) = current {
                        raster.draw_cubic(from, local(c0), local(c1), p);
                    }
                    current = Some(p);
                }
                Segment::Close => {
                    if let (Some(from), Some(to)) = (current, start)
                        && from != to
                    {
                        raster.draw_line(from, to);
                    }
                    current = start;
                }
            }
        }
        let mut pixels = Vec::new();
        raster.for_each_pixel_2d(|x, y, coverage| {
            if coverage > 0.0 {
                pixels.push((x, y, coverage.min(1.0)));
            }
        });
        let (left, top) = (left as i32, top as i32);
        for (x, y, coverage) in pixels {
            let (Ok(x), Ok(y)) = (i32::try_from(x), i32::try_from(y)) else {
                continue;
            };
            let alpha = color.alpha * f64::from(coverage);
            self.inner
                .draw_pixel(
                    (left + x, top + y),
                    BackendColor {
                        alpha,
                        rgb: color.rgb,
                    },
                )
                .map_err(|error| ExportError::Render(error.to_string()))?;
        }
        Ok(())
    }
}

impl<B: DrawingBackend> DrawingBackend for TextBackend<B> {
    type ErrorType = B::ErrorType;

    fn get_size(&self) -> (u32, u32) {
        self.inner.get_size()
    }

    fn ensure_prepared(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.ensure_prepared()
    }

    fn present(&mut self) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.present()
    }

    fn draw_pixel(
        &mut self,
        point: BackendCoord,
        color: BackendColor,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.draw_pixel(point, color)
    }

    fn draw_line<S: BackendStyle>(
        &mut self,
        from: BackendCoord,
        to: BackendCoord,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.draw_line(from, to, style)
    }

    fn draw_rect<S: BackendStyle>(
        &mut self,
        upper_left: BackendCoord,
        bottom_right: BackendCoord,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.draw_rect(upper_left, bottom_right, style, fill)
    }

    fn draw_path<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        path: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.draw_path(path, style)
    }

    fn draw_circle<S: BackendStyle>(
        &mut self,
        center: BackendCoord,
        radius: u32,
        style: &S,
        fill: bool,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.draw_circle(center, radius, style, fill)
    }

    fn fill_polygon<S: BackendStyle, I: IntoIterator<Item = BackendCoord>>(
        &mut self,
        vert: I,
        style: &S,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.fill_polygon(vert, style)
    }

    fn draw_text<S: BackendTextStyle>(
        &mut self,
        text: &str,
        style: &S,
        pos: BackendCoord,
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        if style.color().alpha == 0.0 {
            return Ok(());
        }
        self.check_style(style)?;
        match self.mode {
            TextMode::Svg => self.inner.draw_text(text, style, pos),
            TextMode::Raster => self
                .draw_outlines(text, style, pos)
                .map_err(|error| self.fail(error)),
        }
    }

    fn estimate_text_size<S: BackendTextStyle>(
        &self,
        text: &str,
        style: &S,
    ) -> Result<(u32, u32), DrawingErrorKind<Self::ErrorType>> {
        let layout = text::bundled_font()
            .and_then(|font| font.layout(text, style.size(), Missing::Notdef))
            .map_err(|error| self.fail(error))?;
        Ok((layout.width.unsigned_abs(), layout.height.unsigned_abs()))
    }

    fn blit_bitmap(
        &mut self,
        pos: BackendCoord,
        size: (u32, u32),
        src: &[u8],
    ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
        self.inner.blit_bitmap(pos, size, src)
    }
}
