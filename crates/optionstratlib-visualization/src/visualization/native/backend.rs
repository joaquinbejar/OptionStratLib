//! A drawing backend wrapper that measures and draws every label with the
//! bundled font through [`super::text`], so Plotters is built without any
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
//!   ([`super::outline`]) or the viewer draws it.
//!
//! A text error is stashed in the shared [`TextErrors`] slot and reported as
//! a font error to Plotters, which only carries it as a string; the caller
//! returns the stashed [`GraphError`] instead, typed.

use super::text::{self, Horizontal, Missing, Segment, Vertical, whole_pixels};
use crate::error::GraphError;
use ab_glyph_rasterizer::{Point, Rasterizer, point};
use num_traits::ToPrimitive;
use plotters_backend::text_anchor::{HPos, VPos};
use plotters_backend::{
    BackendColor, BackendCoord, BackendStyle, BackendTextStyle, DrawingBackend, DrawingErrorKind,
    FontStyle, FontTransform,
};
use std::cell::RefCell;
use std::rc::Rc;

/// Where the first text error of a drawing goes.
pub(crate) type TextErrors = Rc<RefCell<Option<GraphError>>>;

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

/// `inner` with its text measured and drawn by [`super::text`].
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
    fn fail(&self, error: GraphError) -> DrawingErrorKind<B::ErrorType> {
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
            other => Err(self.fail(GraphError::UnsupportedText(format!(
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
    ) -> Result<(), GraphError> {
        let font = text::bundled_font()?;
        let layout = font.layout(text, style.size(), Missing::Error)?;
        let dx = layout.dx(match style.anchor().h_pos {
            HPos::Left => Horizontal::Start,
            HPos::Center => Horizontal::Middle,
            HPos::Right => Horizontal::End,
        })?;
        let dy = layout.dy(match style.anchor().v_pos {
            VPos::Top => Vertical::Top,
            VPos::Center => Vertical::Center,
            VPos::Bottom => Vertical::Bottom,
        })?;
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
    ///
    /// The rasterizer covers only the visible part of the label, plus a
    /// guard column on each side: visible pixels are its columns
    /// `1..=cols`. It clips rows itself but assumes every x lies within its
    /// width, so a label running past the left or right edge of the canvas
    /// is clipped here first (see [`ClippedLines`]).
    ///
    /// The guards matter because the rasterizer accumulates coverage over
    /// the whole buffer in `f32`: a line ending exactly on its x = 0 can
    /// drift a hair below it, and a contribution there is dropped (first
    /// row) or lands in the previous row, which corrupts every pixel after
    /// it. With the visible area starting at x = 1, such drift stays in the
    /// left guard of the same row, where the running sum still counts it.
    fn fill(&mut self, segments: &[Segment], color: BackendColor) -> Result<(), GraphError> {
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
        let (canvas_width, canvas_height) = (f64::from(width), f64::from(height));
        let (left, top) = (x0.floor().max(0.0), y0.floor().max(0.0));
        let (right, bottom) = (x1.ceil().min(canvas_width), y1.ceil().min(canvas_height));
        if right <= left || bottom <= top {
            return Ok(());
        }
        // Whole pixels inside the canvas, so these conversions are exact.
        let span = |length: f64| {
            length
                .to_usize()
                .ok_or_else(|| GraphError::Render(format!("label span {length} out of range")))
        };
        let (cols, rows) = (span(right - left)?, span(bottom - top)?);
        let local = |(x, y): (f64, f64)| point(narrow(x - left + 1.0), narrow(y - top));
        let guarded = cols
            .checked_add(2)
            .ok_or_else(|| GraphError::Render("label too wide".to_string()))?;
        let mut raster = Rasterizer::new(guarded, rows);
        // A label inside the canvas horizontally goes to the rasterizer as
        // it is; only one running past a side edge needs clipping, to the
        // visible columns' outer edges.
        let edges = (x0 < 0.0 || x1 > canvas_width).then_some((1.0, cols as f32 + 1.0));
        let mut lines = ClippedLines {
            raster: &mut raster,
            edges,
        };
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
                        lines.line(from, p);
                    }
                    current = Some(p);
                }
                Segment::Quad(c, p) => {
                    let p = local(p);
                    if let Some(from) = current {
                        lines.quad(from, local(c), p);
                    }
                    current = Some(p);
                }
                Segment::Cubic(c0, c1, p) => {
                    let p = local(p);
                    if let Some(from) = current {
                        lines.cubic(from, local(c0), local(c1), p, 0);
                    }
                    current = Some(p);
                }
                Segment::Close => {
                    if let (Some(from), Some(to)) = (current, start)
                        && from != to
                    {
                        lines.line(from, to);
                    }
                    current = start;
                }
            }
        }
        let mut pixels = Vec::new();
        raster.for_each_pixel_2d(|x, y, coverage| {
            // Columns 1..=cols are the visible ones; 0 and cols + 1 guard.
            if coverage > 0.0
                && (1..=cols).contains(&(x as usize))
                && let Some(x) = x.checked_sub(1)
            {
                pixels.push((x, y, coverage.min(1.0)));
            }
        });
        let (left, top) = (
            whole_pixels(left, "the label position")?,
            whole_pixels(top, "the label position")?,
        );
        for (x, y, coverage) in pixels {
            let (Some(x), Some(y)) = (
                i32::try_from(x).ok().and_then(|x| left.checked_add(x)),
                i32::try_from(y).ok().and_then(|y| top.checked_add(y)),
            ) else {
                continue;
            };
            let alpha = color.alpha * f64::from(coverage);
            self.inner
                .draw_pixel(
                    (x, y),
                    BackendColor {
                        alpha,
                        rgb: color.rgb,
                    },
                )
                .map_err(|error| GraphError::Render(error.to_string()))?;
        }
        Ok(())
    }
}

/// Feeds outline segments to the rasterizer, clipping them to x within
/// `edges` (the visible columns' outer edges) when set.
///
/// The rasterizer accumulates signed coverage along each row, so a piece
/// of the outline left of the first visible column only matters through
/// its height: moving it onto the left edge keeps the coverage of every
/// visible pixel. A piece right of the last visible column covers nothing
/// visible: moving it onto the right edge puts its coverage in the right
/// guard column. Segments are cut where they cross an edge, not clamped,
/// and curves are flattened with the rasterizer's own subdivision first,
/// so the shape is unchanged.
struct ClippedLines<'a> {
    raster: &'a mut Rasterizer,
    edges: Option<(f32, f32)>,
}

/// `value` as `f32`, as `as f32` gives it, clamped past `f32`'s range.
fn narrow(value: f64) -> f32 {
    value
        .to_f32()
        .unwrap_or(if value < 0.0 { f32::MIN } else { f32::MAX })
}

fn lerp(t: f32, p0: Point, p1: Point) -> Point {
    point(p0.x + t * (p1.x - p0.x), p0.y + t * (p1.y - p0.y))
}

/// As the rasterizer's own (crate-private) `Point::distance_to`.
fn distance(a: Point, b: Point) -> f32 {
    let (dx, dy) = (a.x - b.x, a.y - b.y);
    (dx * dx + dy * dy).sqrt() // scan-banned: allow -- f32 `sqrt` of a sum of squares, never negative
}

impl ClippedLines<'_> {
    fn line(&mut self, p0: Point, p1: Point) {
        let Some((left, right)) = self.edges else {
            self.raster.draw_line(p0, p1);
            return;
        };
        let dx = p1.x - p0.x;
        let mut cuts = vec![0.0_f32, 1.0];
        if dx != 0.0 {
            for edge in [left, right] {
                let t = (edge - p0.x) / dx;
                if t > 0.0 && t < 1.0 {
                    cuts.push(t);
                }
            }
        }
        cuts.sort_by(f32::total_cmp);
        for pair in cuts.windows(2) {
            let [t0, t1] = pair else {
                continue;
            };
            let (mut a, mut b) = (lerp(*t0, p0, p1), lerp(*t1, p0, p1));
            let middle = 0.5 * (a.x + b.x);
            let x = if middle < left {
                Some(left)
            } else if middle > right {
                Some(right)
            } else {
                None
            };
            match x {
                // Outside: onto the edge.
                Some(x) => (a.x, b.x) = (x, x),
                // Inside: only rounding can stray past an edge.
                None => (a.x, b.x) = (a.x.clamp(left, right), b.x.clamp(left, right)),
            }
            self.raster.draw_line(a, b);
        }
    }

    /// `Rasterizer::draw_quad`'s subdivision, through [`ClippedLines::line`].
    fn quad(&mut self, p0: Point, p1: Point, p2: Point) {
        if self.edges.is_none() {
            self.raster.draw_quad(p0, p1, p2);
            return;
        }
        let devx = p0.x - 2.0 * p1.x + p2.x;
        let devy = p0.y - 2.0 * p1.y + p2.y;
        let devsq = devx * devx + devy * devy;
        if devsq < 0.333 {
            self.line(p0, p2);
            return;
        }
        // At least 1; a degenerate curve too large to subdivide is a line.
        // `devsq` is a sum of squares, so both f32 roots are of non-negative
        // values.
        let Some(n) = (3.0 * devsq)
            .sqrt() // scan-banned: allow -- f32 `sqrt` of a non-negative value
            .sqrt() // scan-banned: allow -- f32 `sqrt` of a non-negative value
            .floor()
            .to_usize()
            .and_then(|n| n.checked_add(1))
        else {
            self.line(p0, p2);
            return;
        };
        let step = (n as f32).recip();
        let (mut p, mut t) = (p0, 0.0);
        for _ in 1..n {
            t += step;
            let next = lerp(t, lerp(t, p0, p1), lerp(t, p1, p2));
            self.line(p, next);
            p = next;
        }
        self.line(p, p2);
    }

    /// `Rasterizer::draw_cubic`'s subdivision, through [`ClippedLines::line`].
    fn cubic(&mut self, p0: Point, p1: Point, p2: Point, p3: Point, depth: u8) {
        if self.edges.is_none() {
            self.raster.draw_cubic(p0, p1, p2, p3);
            return;
        }
        const FLATNESS_SQUARED: f32 = 0.35 * 0.35;
        const MAX_DEPTH: u8 = 16;
        let long = distance(p0, p1) + distance(p1, p2) + distance(p2, p3);
        let short = distance(p0, p3);
        if depth < MAX_DEPTH && long * long - short * short > FLATNESS_SQUARED {
            let (p01, p12, p23) = (lerp(0.5, p0, p1), lerp(0.5, p1, p2), lerp(0.5, p2, p3));
            let (pa, pb) = (lerp(0.5, p01, p12), lerp(0.5, p12, p23));
            let middle = lerp(0.5, pa, pb);
            // `depth < MAX_DEPTH`, so this cannot overflow.
            let deeper = depth.checked_add(1).unwrap_or(MAX_DEPTH);
            self.cubic(p0, p01, pa, middle, deeper);
            self.cubic(middle, pb, p23, p3, deeper);
        } else {
            self.line(p0, p3);
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use plotters::prelude::{BLACK, BitMapBackend, IntoFont, TextStyle};
    use plotters::style::Color;
    use plotters_backend::text_anchor::Pos;

    const WIDTH: u32 = 60;
    const HEIGHT: u32 = 40;
    /// Margin of the large canvas, which holds every test label whole.
    const PAD: u32 = 40;

    /// `FontTransform` is neither `Copy` nor `Debug`: tests pass makers.
    type Transform = fn() -> FontTransform;

    /// The red channel of `text` drawn black at `pos` on a white canvas,
    /// at the Plotters font size `size`.
    fn draw(
        text: &str,
        size: f64,
        pos: BackendCoord,
        transform: Transform,
        (width, height): (u32, u32),
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut buffer = vec![255_u8; (width * height * 3) as usize];
        {
            let inner = BitMapBackend::with_buffer(&mut buffer, (width, height));
            let mut backend = TextBackend::new(inner, TextMode::Raster, TextErrors::default());
            let black = BLACK.to_rgba();
            let style = TextStyle::from(("Open Sans", size).into_font())
                .color(&black)
                .transform(transform())
                .pos(Pos::new(HPos::Left, VPos::Top));
            backend.draw_text(text, &style, pos)?;
            backend.present()?;
        }
        Ok(buffer.chunks(3).map(|pixel| pixel[0]).collect())
    }

    #[test]
    fn test_labels_past_any_edge_are_clipped_not_distorted()
    -> Result<(), Box<dyn std::error::Error>> {
        let transforms: [(&str, Transform); 4] = [
            ("none", || FontTransform::None),
            ("90", || FontTransform::Rotate90),
            ("180", || FontTransform::Rotate180),
            ("270", || FontTransform::Rotate270),
        ];
        // Past the left, right, top and bottom edges, and two at once; then
        // each offset a few pixels past the left edge, where a line ending
        // on the visible edge used to lose coverage for the whole label.
        let mut positions = vec![
            (-25, 10),
            (45, 10),
            (10, -25),
            (10, 35),
            (-25, -25),
            (50, 30),
        ];
        positions.extend((-8..=-1).map(|x| (x, 10)));
        // The chart's tick and title sizes, and a larger one.
        let labels = [("Wg", 20.0), ("OA", 12.0 * 1.24), ("OA", 17.0 * 1.24)];
        let pad = PAD as i32;
        let mut inked = 0;
        for (name, transform) in transforms {
            for (text, size) in labels {
                for &(x, y) in &positions {
                    let small = draw(text, size, (x, y), transform, (WIDTH, HEIGHT))?;
                    let large = draw(
                        text,
                        size,
                        (x + pad, y + pad),
                        transform,
                        (WIDTH + 2 * PAD, HEIGHT + 2 * PAD),
                    )?;
                    for row in 0..HEIGHT as usize {
                        for col in 0..WIDTH as usize {
                            let clipped = small[row * WIDTH as usize + col];
                            let whole = large[(row + PAD as usize) * (WIDTH + 2 * PAD) as usize
                                + col
                                + PAD as usize];
                            assert!(
                                clipped.abs_diff(whole) <= 2,
                                "{text:?} at {size}, rotation {name}, at ({x}, {y}): pixel ({col}, {row}) is {clipped}, {whole} unclipped"
                            );
                            inked += usize::from(clipped < 128);
                        }
                    }
                }
            }
        }
        assert!(
            inked > 500,
            "only {inked} inked pixels: the labels missed the canvas"
        );
        Ok(())
    }
}
