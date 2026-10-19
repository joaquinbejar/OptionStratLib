//! The bundled font, read with skrifa (fontations): character map,
//! advances, vertical metrics and glyph outlines. Nothing here uses
//! ttf-parser (RUSTSEC-2026-0192, unmaintained), which the advisory itself
//! points to skrifa to replace.
//!
//! Labels are laid out exactly the way Plotters' `ab_glyph` text laid them
//! out for #920 and #923, so the chart layout does not move:
//!
//! * the Plotters font size is ab_glyph's `PxScale`: one font-size unit per
//!   `ascent - descent` font units, computed in `f32` as ab_glyph does;
//! * the anchor box is the advance width truncated to whole pixels, and the
//!   Plotters size divided by [`EM_RATIO`] as its height;
//! * the baseline is half the Plotters size below the top of the box.
//!
//! There is no kerning and no shaping: one glyph per character. The bundled
//! Open Sans keeps its kerning in GPOS only, which ab_glyph never applied
//! either, so this is the same text as before.
//!
//! [`EM_RATIO`]: crate::render

use crate::error::ExportError;
use crate::render::{EM_RATIO, FONT_BYTES};
use skrifa::charmap::Charmap;
use skrifa::instance::{LocationRef, Size};
use skrifa::metrics::GlyphMetrics;
use skrifa::outline::{DrawSettings, OutlineGlyphCollection, OutlinePen};
use skrifa::{FontRef, GlyphId, MetadataProvider};
use std::sync::OnceLock;

/// Whitespace drawn as a space: Plotters lays text out on one line.
pub(crate) const LINE_WHITESPACE: [char; 3] = ['\t', '\n', '\r'];

/// The bundled font, parsed once per process.
pub(crate) struct BundledFont {
    charmap: Charmap<'static>,
    metrics: GlyphMetrics<'static>,
    outlines: OutlineGlyphCollection<'static>,
    /// Ascent minus descent, in font units: ab_glyph's unit of size.
    height: f32,
}

/// How a label treats a character the font has no glyph for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Missing {
    /// Measure it with the `.notdef` advance, as ab_glyph did. Measuring
    /// only: the viewer-font and embedded-font SVGs leave such characters
    /// to the viewer.
    Notdef,
    /// Fail with [`ExportError::MissingGlyph`]: nothing is drawn in its
    /// place.
    Error,
}

/// An axis-aligned box in pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct InkBox {
    pub(crate) left: f64,
    pub(crate) top: f64,
    pub(crate) right: f64,
    pub(crate) bottom: f64,
}

impl InkBox {
    pub(crate) fn union(self, other: InkBox) -> InkBox {
        InkBox {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }
}

/// One piece of a glyph outline, in pixels, y down.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) enum Segment {
    /// Starts a contour.
    Move((f64, f64)),
    /// A straight line to the point.
    Line((f64, f64)),
    /// A quadratic curve: control point, end point.
    Quad((f64, f64), (f64, f64)),
    /// A cubic curve: two control points, end point.
    Cubic((f64, f64), (f64, f64), (f64, f64)),
    /// Closes the contour.
    Close,
}

/// A label laid out like Plotters lays it out.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct LineLayout {
    /// Width of the anchor box: the advances, truncated to whole pixels.
    pub(crate) width: i32,
    /// Height of the anchor box.
    pub(crate) height: i32,
    /// The baseline, below the top of the anchor box.
    pub(crate) baseline: f64,
    /// Each glyph with its pen position from the left of the box.
    glyphs: Vec<(GlyphId, f32)>,
    /// Pixels per font unit.
    factor: f32,
}

#[cold]
fn font_error(reason: impl std::fmt::Display) -> ExportError {
    ExportError::Font(format!("Open Sans Regular: {reason}"))
}

/// The bundled font, parsed once per process.
pub(crate) fn bundled_font() -> Result<&'static BundledFont, ExportError> {
    static FONT: OnceLock<Result<BundledFont, String>> = OnceLock::new();
    FONT.get_or_init(|| {
        let font = FontRef::new(FONT_BYTES).map_err(|error| error.to_string())?;
        let vertical = font.metrics(Size::unscaled(), LocationRef::default());
        Ok(BundledFont {
            charmap: font.charmap(),
            metrics: font.glyph_metrics(Size::unscaled(), LocationRef::default()),
            outlines: font.outline_glyphs(),
            height: vertical.ascent - vertical.descent,
        })
    })
    .as_ref()
    .map_err(font_error)
}

impl LineLayout {
    /// The same layout with another anchor-box height. The outline
    /// conversion takes it from the SVG `font-size` attribute, which is
    /// exactly the height Plotters computed; multiplying that attribute
    /// back by [`EM_RATIO`] and dividing again could land an ulp below a
    /// whole number and truncate differently.
    pub(crate) fn with_height(self, height: i32) -> LineLayout {
        LineLayout { height, ..self }
    }
}

impl BundledFont {
    /// The glyph `character` is drawn with, if the font has one.
    fn glyph(&self, character: char) -> Option<GlyphId> {
        let drawn = if LINE_WHITESPACE.contains(&character) {
            ' '
        } else {
            character
        };
        self.charmap.map(drawn)
    }

    fn advance(&self, glyph: GlyphId) -> f32 {
        self.metrics.advance_width(glyph).unwrap_or_default()
    }

    /// Lays `text` out at the Plotters font size `plotters_size`.
    ///
    /// # Errors
    ///
    /// [`ExportError::MissingGlyph`] for a character without a glyph when
    /// `missing` is [`Missing::Error`].
    pub(crate) fn layout(
        &self,
        text: &str,
        plotters_size: f64,
        missing: Missing,
    ) -> Result<LineLayout, ExportError> {
        // ab_glyph scales in f32, as Plotters does.
        let scale = plotters_size as f32;
        let factor = scale / self.height;
        let mut glyphs = Vec::with_capacity(text.len());
        let mut pen = 0_f32;
        for character in text.chars() {
            let glyph = match (self.glyph(character), missing) {
                (Some(glyph), _) => glyph,
                (None, Missing::Notdef) => GlyphId::NOTDEF,
                (None, Missing::Error) => {
                    return Err(ExportError::MissingGlyph {
                        character,
                        code: u32::from(character),
                        text: text.to_string(),
                    });
                }
            };
            glyphs.push((glyph, pen));
            pen += self.advance(glyph) * factor;
        }
        Ok(LineLayout {
            width: pen as i32,
            height: (plotters_size / EM_RATIO) as i32,
            baseline: f64::from(scale / 2.0),
            glyphs,
            factor,
        })
    }

    /// Calls `segment` with every outline segment of `layout`, the top left
    /// of its anchor box at `origin`, and returns the ink box; `None` when
    /// no glyph has ink (an empty or all-space label).
    ///
    /// # Errors
    ///
    /// [`ExportError::Font`] if a glyph outline cannot be read.
    pub(crate) fn trace(
        &self,
        layout: &LineLayout,
        origin: (f64, f64),
        mut segment: impl FnMut(Segment),
    ) -> Result<Option<InkBox>, ExportError> {
        let factor = f64::from(layout.factor);
        let baseline = origin.1 + layout.baseline;
        let mut ink: Option<InkBox> = None;
        for (glyph, pen) in &layout.glyphs {
            // Glyphs without ink (spaces: an empty outline and box) only
            // advance the pen, as with ab_glyph.
            let bounds = self
                .metrics
                .bounds(*glyph)
                .filter(|b| b.x_min < b.x_max && b.y_min < b.y_max);
            let (Some(outline), Some(bounds)) = (self.outlines.get(*glyph), bounds) else {
                continue;
            };
            let left = origin.0 + f64::from(*pen);
            let place = |x: f32, y: f32| {
                (
                    left + f64::from(x) * factor,
                    baseline - f64::from(y) * factor,
                )
            };
            let mut pen = Pen {
                place: &place,
                current: None,
                emit: &mut segment,
            };
            outline
                .draw(
                    DrawSettings::unhinted(Size::unscaled(), LocationRef::default()),
                    &mut pen,
                )
                .map_err(font_error)?;
            let (left_top, right_bottom) = (
                place(bounds.x_min, bounds.y_max),
                place(bounds.x_max, bounds.y_min),
            );
            let glyph_box = InkBox {
                left: left_top.0,
                top: left_top.1,
                right: right_bottom.0,
                bottom: right_bottom.1,
            };
            ink = Some(ink.map_or(glyph_box, |ink| ink.union(glyph_box)));
        }
        Ok(ink)
    }
}

/// Places skrifa's unscaled outline in pixels and drops lines to the
/// current point, which have no geometry.
struct Pen<'a, P, E> {
    place: &'a P,
    current: Option<(f32, f32)>,
    emit: &'a mut E,
}

impl<P: Fn(f32, f32) -> (f64, f64), E: FnMut(Segment)> OutlinePen for Pen<'_, P, E> {
    fn move_to(&mut self, x: f32, y: f32) {
        self.current = Some((x, y));
        (self.emit)(Segment::Move((self.place)(x, y)));
    }

    fn line_to(&mut self, x: f32, y: f32) {
        if self.current == Some((x, y)) {
            return;
        }
        self.current = Some((x, y));
        (self.emit)(Segment::Line((self.place)(x, y)));
    }

    fn quad_to(&mut self, cx: f32, cy: f32, x: f32, y: f32) {
        self.current = Some((x, y));
        (self.emit)(Segment::Quad((self.place)(cx, cy), (self.place)(x, y)));
    }

    fn curve_to(&mut self, cx0: f32, cy0: f32, cx1: f32, cy1: f32, x: f32, y: f32) {
        self.current = Some((x, y));
        (self.emit)(Segment::Cubic(
            (self.place)(cx0, cy0),
            (self.place)(cx1, cy1),
            (self.place)(x, y),
        ));
    }

    fn close(&mut self) {
        self.current = None;
        (self.emit)(Segment::Close);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundled_font_metrics_are_open_sans() -> Result<(), ExportError> {
        let font = bundled_font()?;
        // hhea ascent 2189, descent -600.
        assert_eq!(font.height, 2789.0);
        assert!(font.glyph('A').is_some());
        assert!(font.glyph('\u{4E2D}').is_none());
        // Line whitespace is drawn with the space glyph.
        assert_eq!(font.glyph('\n'), font.glyph(' '));
        Ok(())
    }

    #[test]
    fn test_layout_measures_like_plotters() -> Result<(), ExportError> {
        let font = bundled_font()?;
        let size = 12.0 * EM_RATIO;
        let layout = font.layout("Strike", size, Missing::Error)?;
        assert!(layout.width > 20 && layout.width < 40, "{}", layout.width);
        assert_eq!(layout.height, (size / EM_RATIO) as i32);
        assert!((layout.baseline - size / 2.0).abs() < 1e-5);
        Ok(())
    }

    #[test]
    fn test_missing_glyphs_follow_the_policy() -> Result<(), ExportError> {
        let font = bundled_font()?;
        let measured = font.layout("IV 中", 14.88, Missing::Notdef)?;
        let plain = font.layout("IV ", 14.88, Missing::Notdef)?;
        assert!(measured.width > plain.width);
        assert!(matches!(
            font.layout("IV 中", 14.88, Missing::Error),
            Err(ExportError::MissingGlyph {
                character: '中',
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn test_trace_closes_every_contour_and_skips_spaces() -> Result<(), ExportError> {
        let font = bundled_font()?;
        let layout = font.layout("o o", 20.0, Missing::Error)?;
        let mut segments = Vec::new();
        let ink = font.trace(&layout, (10.0, 20.0), |segment| segments.push(segment))?;
        let moves = segments
            .iter()
            .filter(|s| matches!(s, Segment::Move(_)))
            .count();
        let closes = segments.iter().filter(|s| **s == Segment::Close).count();
        // Two `o`s, two contours each; the space has none.
        assert_eq!((moves, closes), (4, 4));
        let ink = ink
            .ok_or("ink")
            .map_err(|reason| ExportError::Render(reason.to_string()))?;
        assert!(ink.left >= 10.0 && ink.right > ink.left && ink.bottom > ink.top);
        let blank = font.layout("   ", 20.0, Missing::Error)?;
        assert_eq!(font.trace(&blank, (0.0, 0.0), |_| {})?, None);
        Ok(())
    }
}
