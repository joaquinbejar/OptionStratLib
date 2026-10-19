//! SVG text as glyph outlines (#923).
//!
//! plotters-svg writes every label as a `<text>` element and leaves the
//! glyphs to the viewer, and librsvg ignores the embedded `@font-face` of
//! [`crate::svg::embed_font`]. This module replaces each of those elements
//! with `<path>` outlines of the bundled Open Sans, so the document paints
//! the same glyphs in every viewer and needs no font at all.
//!
//! It converts the exact subset plotters-svg 0.3.7 writes for this crate,
//! not arbitrary SVG: any other text construct is
//! [`ExportError::UnsupportedSvgText`], and a character the font has no
//! glyph for is [`ExportError::MissingGlyph`]. Nothing falls back to a
//! viewer font, a raster image or a dropped label.
//!
//! # Layout
//!
//! Labels are laid out with the metrics Plotters measured the chart with,
//! which are also the ones the PNG is drawn with: ab_glyph at the Plotters
//! font size (the SVG `font-size` times `EM_RATIO`, the factor
//! plotters-svg divided it by, applied once here), each character's
//! horizontal advance plus ab_glyph's kerning, and the anchor box of
//! `plotters-backend`'s default `draw_text`. The room the layout reserved
//! (legend column, label areas) therefore fits the painted text. Unlike the
//! PNG rasteriser, glyphs keep their left side bearings and sub-pixel pen
//! positions. There is no shaping: one glyph per character, no ligatures.
//!
//! A tab, line feed or carriage return is whitespace, not a missing glyph:
//! each advances like one space, on the same line. Plotters has no line
//! breaks, and the PNG and every viewer also draw them as horizontal space
//! (library titles such as the strategies' contain `\n\t`). Any other
//! character without a glyph is an error.
//!
//! # Accessibility
//!
//! Each label becomes `<g role="img" aria-label="…">` around its path, so
//! its text stays readable in the source and is the group's accessible
//! name, and the document gets a `<title>` and a `<desc>`. Selecting,
//! copying, searching or editing the text is lost: that needs `<text>`.

use crate::error::ExportError;
use crate::render::{EM_RATIO, FONT_BYTES, FONT_FAMILY};
use ab_glyph::{Font, FontRef, GlyphId, OutlineCurve, Point, PxScale, ScaleFont};
use serde::Serialize;
use std::sync::OnceLock;

const TEXT_OPEN: &str = "<text";
const TEXT_CLOSE: &str = "</text>";
/// Glyph 0 is `.notdef`, which ab_glyph returns for unmapped characters.
const NOTDEF: GlyphId = GlyphId(0);
/// Whitespace drawn as a space: Plotters lays text out on one line.
const LINE_WHITESPACE: [char; 3] = ['\t', '\n', '\r'];
/// Path coordinates are written with this many decimals (1/100 px).
const DECIMALS: usize = 2;
/// The entities plotters-svg escapes text with, and the only ones accepted.
const ENTITIES: [(&str, char); 5] = [
    ("&lt;", '<'),
    ("&gt;", '>'),
    ("&amp;", '&'),
    ("&quot;", '"'),
    ("&apos;", '\''),
];

/// Accessible name and description of a whole chart.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChartText {
    /// The document `<title>`: the chart's accessible name.
    pub title: String,
    /// The document `<desc>`: the chart's accessible description.
    pub description: String,
}

/// Where a label's ink landed, in SVG user units with its rotation applied.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Label {
    /// The label's text, as the chart has it.
    pub text: String,
    /// Left edge of the ink.
    pub left: f64,
    /// Top edge of the ink.
    pub top: f64,
    /// Right edge of the ink.
    pub right: f64,
    /// Bottom edge of the ink.
    pub bottom: f64,
}

/// An SVG document with its text as outlines, and where each label is.
#[derive(Debug, Clone, PartialEq)]
pub struct Outlined {
    /// The converted document.
    pub document: String,
    /// Every painted label, in document order.
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Anchor {
    Start,
    Middle,
    End,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Baseline {
    Top,
    Center,
    Bottom,
}

/// A quarter turn about the label's anchor point, as plotters-svg writes
/// `FontTransform::Rotate90/180/270`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Rotation {
    Quarter,
    Half,
    ThreeQuarters,
}

/// One `<text>` element of the plotters-svg subset.
#[derive(Debug, Clone, PartialEq)]
struct TextElement<'a> {
    x: i32,
    y: i32,
    anchor: Anchor,
    baseline: Baseline,
    /// The `font-size` attribute: the Plotters size divided by `EM_RATIO`.
    css_size: f64,
    fill: &'a str,
    opacity: &'a str,
    rotation: Option<(Rotation, &'a str)>,
    text: String,
}

/// An axis-aligned box in SVG user units.
#[derive(Debug, Clone, Copy, PartialEq)]
struct InkBox {
    left: f64,
    top: f64,
    right: f64,
    bottom: f64,
}

impl InkBox {
    fn union(self, other: InkBox) -> InkBox {
        InkBox {
            left: self.left.min(other.left),
            top: self.top.min(other.top),
            right: self.right.max(other.right),
            bottom: self.bottom.max(other.bottom),
        }
    }

    /// The box after `rotation` about `(cx, cy)`, in SVG's y-down sense.
    fn rotated(self, rotation: Option<Rotation>, cx: f64, cy: f64) -> InkBox {
        let turn = |(x, y): (f64, f64)| {
            let (dx, dy) = (x - cx, y - cy);
            match rotation {
                None => (x, y),
                Some(Rotation::Quarter) => (cx - dy, cy + dx),
                Some(Rotation::Half) => (cx - dx, cy - dy),
                Some(Rotation::ThreeQuarters) => (cx + dy, cy - dx),
            }
        };
        let (x0, y0) = turn((self.left, self.top));
        let (x1, y1) = turn((self.right, self.bottom));
        InkBox {
            left: x0.min(x1),
            top: y0.min(y1),
            right: x0.max(x1),
            bottom: y0.max(y1),
        }
    }
}

#[cold]
fn unsupported(reason: impl Into<String>) -> ExportError {
    ExportError::UnsupportedSvgText(reason.into())
}

/// The bundled font, parsed once per process.
fn bundled_font() -> Result<&'static FontRef<'static>, ExportError> {
    static FONT: OnceLock<Result<FontRef<'static>, String>> = OnceLock::new();
    FONT.get_or_init(|| {
        FontRef::try_from_slice(FONT_BYTES)
            .map_err(|_| "Open Sans Regular is not a valid OpenType font".to_string())
    })
    .as_ref()
    .map_err(|reason| ExportError::Font(reason.clone()))
}

/// Replaces every `<text>` element of a plotters-svg `document` with the
/// outlines of its glyphs, and gives the document `chart`'s title and
/// description.
///
/// # Errors
///
/// [`ExportError::UnsupportedSvgText`] for a text element outside the
/// plotters-svg subset (or a document with no `<svg>` element),
/// [`ExportError::MissingGlyph`] for a character the bundled font cannot
/// draw, [`ExportError::NotXmlText`] for a label, title or description
/// with a character XML 1.0 cannot carry, and [`ExportError::Font`] if the
/// bundled font does not parse.
pub fn outline_text(document: &str, chart: &ChartText) -> Result<Outlined, ExportError> {
    let font = bundled_font()?;
    let mut converted = String::with_capacity(document.len());
    let mut labels = Vec::new();
    let mut rest = document;
    while let Some(start) = rest.find(TEXT_OPEN) {
        let (before, element) = rest
            .split_at_checked(start)
            .ok_or_else(|| unsupported("text element at an invalid offset"))?;
        converted.push_str(before);
        let (element, after) = split_element(element)?;
        if let Some((markup, label)) = outline_element(&element, font)? {
            converted.push_str(&markup);
            labels.push(label);
        }
        rest = after;
    }
    converted.push_str(rest);
    let document = with_chart_text(&converted, chart)?;
    Ok(Outlined { document, labels })
}

/// Splits `<text …>content</text>` off the front of `source`, returning the
/// parsed element and what follows it.
fn split_element(source: &str) -> Result<(TextElement<'_>, &str), ExportError> {
    let after_name = source
        .strip_prefix(TEXT_OPEN)
        .ok_or_else(|| unsupported("expected a text element"))?;
    if !after_name.starts_with(' ') {
        return Err(unsupported(
            "an element whose name starts with \"text\" but is not <text>",
        ));
    }
    let (attributes, after_tag) = after_name
        .split_once('>')
        .ok_or_else(|| unsupported("unterminated <text> tag"))?;
    if attributes.ends_with('/') {
        return Err(unsupported("an empty <text/> element"));
    }
    let (content, after) = after_tag
        .split_once(TEXT_CLOSE)
        .ok_or_else(|| unsupported("a <text> element without </text>"))?;
    Ok((parse_element(attributes, content)?, after))
}

/// Reads the ` name="value"` pairs of a plotters-svg tag.
fn parse_attributes(source: &str) -> Result<Vec<(&str, &str)>, ExportError> {
    let mut attributes = Vec::new();
    let mut rest = source;
    while !rest.is_empty() {
        let unspaced = rest
            .strip_prefix(' ')
            .ok_or_else(|| unsupported(format!("unexpected text {rest:?} in a <text> tag")))?;
        let (name, value_and_rest) = unspaced
            .split_once("=\"")
            .ok_or_else(|| unsupported(format!("malformed attribute in {unspaced:?}")))?;
        if name.is_empty() || !name.chars().all(|c| c.is_ascii_lowercase() || c == '-') {
            return Err(unsupported(format!("attribute name {name:?}")));
        }
        let (value, after) = value_and_rest
            .split_once('"')
            .ok_or_else(|| unsupported(format!("unterminated value of {name:?}")))?;
        attributes.push((name, value));
        rest = after;
    }
    Ok(attributes)
}

fn set_once<T>(slot: &mut Option<T>, name: &str, value: T) -> Result<(), ExportError> {
    match slot.replace(value) {
        Some(_) => Err(unsupported(format!("duplicate attribute {name:?}"))),
        None => Ok(()),
    }
}

fn required<T>(slot: Option<T>, name: &str) -> Result<T, ExportError> {
    slot.ok_or_else(|| unsupported(format!("missing attribute {name:?}")))
}

fn parse_coordinate(name: &str, value: &str) -> Result<i32, ExportError> {
    value
        .parse()
        .map_err(|_| unsupported(format!("{name}={value:?} is not an integer pixel")))
}

fn parse_unit_interval(name: &str, value: &str) -> Result<f64, ExportError> {
    match value.parse::<f64>() {
        Ok(number) if (0.0..=1.0).contains(&number) => Ok(number),
        _ => Err(unsupported(format!("{name}={value:?} is not in 0..=1"))),
    }
}

fn parse_fill(value: &str) -> Result<&str, ExportError> {
    let digits = value.strip_prefix('#').unwrap_or_default();
    if digits.len() == 6 && digits.chars().all(|c| c.is_ascii_hexdigit()) {
        Ok(value)
    } else {
        Err(unsupported(format!("fill={value:?} is not #RRGGBB")))
    }
}

/// `rotate(A, X, Y)` with a quarter-turn angle about the element's own
/// anchor point, the only transform plotters-svg writes on text.
fn parse_rotation(value: &str, x: i32, y: i32) -> Result<Rotation, ExportError> {
    let invalid = || unsupported(format!("transform={value:?}"));
    let arguments = value
        .strip_prefix("rotate(")
        .and_then(|rest| rest.strip_suffix(')'))
        .ok_or_else(invalid)?;
    let mut parts = arguments.split(", ");
    let (Some(angle), Some(cx), Some(cy), None) =
        (parts.next(), parts.next(), parts.next(), parts.next())
    else {
        return Err(invalid());
    };
    if cx.parse::<i32>().ok() != Some(x) || cy.parse::<i32>().ok() != Some(y) {
        return Err(invalid());
    }
    match angle {
        "90" => Ok(Rotation::Quarter),
        "180" => Ok(Rotation::Half),
        "270" => Ok(Rotation::ThreeQuarters),
        _ => Err(invalid()),
    }
}

fn parse_element<'a>(attributes: &'a str, content: &str) -> Result<TextElement<'a>, ExportError> {
    let (mut x, mut y, mut baseline, mut anchor, mut family) = (None, None, None, None, None);
    let (mut size, mut opacity, mut fill, mut transform) = (None, None, None, None);
    for (name, value) in parse_attributes(attributes)? {
        match name {
            "x" => set_once(&mut x, name, parse_coordinate(name, value)?)?,
            "y" => set_once(&mut y, name, parse_coordinate(name, value)?)?,
            "dy" => {
                let parsed = match value {
                    "0.76em" => Baseline::Top,
                    "0.5ex" => Baseline::Center,
                    "-0.5ex" => Baseline::Bottom,
                    other => return Err(unsupported(format!("dy={other:?}"))),
                };
                set_once(&mut baseline, name, parsed)?;
            }
            "text-anchor" => {
                let parsed = match value {
                    "start" => Anchor::Start,
                    "middle" => Anchor::Middle,
                    "end" => Anchor::End,
                    other => return Err(unsupported(format!("text-anchor={other:?}"))),
                };
                set_once(&mut anchor, name, parsed)?;
            }
            "font-family" if value == FONT_FAMILY => set_once(&mut family, name, value)?,
            "font-size" => {
                let parsed = match value.parse::<f64>() {
                    Ok(number) if number.is_finite() && number > 0.0 => number,
                    _ => return Err(unsupported(format!("font-size={value:?}"))),
                };
                set_once(&mut size, name, parsed)?;
            }
            "opacity" => {
                parse_unit_interval(name, value)?;
                set_once(&mut opacity, name, value)?;
            }
            "fill" => set_once(&mut fill, name, parse_fill(value)?)?,
            "transform" => set_once(&mut transform, name, value)?,
            // `font-weight`, `font-style`, another family and anything else:
            // only the regular face is bundled, so nothing can be drawn
            // faithfully for them.
            other => return Err(unsupported(format!("attribute {other}={value:?}"))),
        }
    }
    let (x, y) = (required(x, "x")?, required(y, "y")?);
    required(family, "font-family")?;
    let rotation = match transform {
        Some(value) => Some((parse_rotation(value, x, y)?, value)),
        None => None,
    };
    // plotters-svg writes the text between a newline after the tag and one
    // before `</text>`; those two are layout, not part of the label.
    let escaped = content
        .strip_prefix('\n')
        .and_then(|inner| inner.strip_suffix('\n'))
        .ok_or_else(|| unsupported("text content not in plotters-svg's layout"))?;
    Ok(TextElement {
        x,
        y,
        anchor: required(anchor, "text-anchor")?,
        baseline: required(baseline, "dy")?,
        css_size: required(size, "font-size")?,
        fill: required(fill, "fill")?,
        opacity: required(opacity, "opacity")?,
        rotation,
        text: unescape(escaped)?,
    })
}

/// Decodes the five entities plotters-svg escapes with; markup or any other
/// entity is unsupported.
fn unescape(escaped: &str) -> Result<String, ExportError> {
    let mut text = String::with_capacity(escaped.len());
    let mut rest = escaped;
    loop {
        let (plain, entity) = match rest.find('&') {
            Some(at) => rest
                .split_at_checked(at)
                .ok_or_else(|| unsupported("entity at an invalid offset"))?,
            None => (rest, ""),
        };
        if plain.contains('<') {
            return Err(unsupported("markup inside a <text> element"));
        }
        text.push_str(plain);
        if entity.is_empty() {
            return Ok(text);
        }
        let (name, character) = ENTITIES
            .iter()
            .find(|(name, _)| entity.starts_with(name))
            .ok_or_else(|| unsupported(format!("entity at {entity:?}")))?;
        text.push(*character);
        rest = entity
            .get(name.len()..)
            .ok_or_else(|| unsupported("entity at an invalid offset"))?;
    }
}

/// Whether XML 1.0 can carry `character` at all: its `Char` production
/// excludes the C0 controls other than tab, line feed and carriage return,
/// and U+FFFE/U+FFFF (Rust `char`s are never surrogates).
fn is_xml_char(character: char) -> bool {
    matches!(character, '\t' | '\n' | '\r' | '\u{20}'..='\u{FFFD}' | '\u{10000}'..)
}

/// Fails for a string XML cannot carry, before any of it is written.
fn ensure_xml_text(text: &str) -> Result<(), ExportError> {
    match text.chars().find(|c| !is_xml_char(*c)) {
        Some(character) => Err(ExportError::NotXmlText {
            character,
            code: u32::from(character),
            text: text.to_string(),
        }),
        None => Ok(()),
    }
}

/// Appends `text` escaped for an attribute value or element content; the
/// caller has checked it with [`ensure_xml_text`].
fn push_escaped(out: &mut String, text: &str) {
    for character in text.chars() {
        match ENTITIES.iter().find(|(_, escaped)| *escaped == character) {
            Some((entity, _)) => out.push_str(entity),
            None => out.push(character),
        }
    }
}

/// Appends `value` with [`DECIMALS`] decimals and no trailing zeros.
fn push_number(out: &mut String, value: f64) {
    let text = format!("{value:.DECIMALS$}");
    let trimmed = text.trim_end_matches('0').trim_end_matches('.');
    out.push_str(if trimmed == "-0" { "0" } else { trimmed });
}

fn push_point(out: &mut String, command: char, (x, y): (f64, f64)) {
    out.push(command);
    push_number(out, x);
    out.push(' ');
    push_number(out, y);
}

/// Appends one glyph's contours as path data. ab_glyph lists contours as
/// consecutive curves and closes each with a line back to its start; a new
/// contour starts wherever a curve does not begin at the previous end.
fn push_glyph(out: &mut String, curves: &[OutlineCurve], place: impl Fn(Point) -> (f64, f64)) {
    let mut contour_start: Option<Point> = None;
    let mut last: Option<Point> = None;
    for curve in curves {
        let (start, end) = match curve {
            OutlineCurve::Line(p0, p1) | OutlineCurve::Quad(p0, _, p1) => (*p0, *p1),
            OutlineCurve::Cubic(p0, _, _, p1) => (*p0, *p1),
        };
        if last != Some(start) {
            if contour_start.is_some() {
                out.push('Z');
            }
            push_point(out, 'M', place(start));
            contour_start = Some(start);
        }
        match curve {
            OutlineCurve::Line(_, p1) if Some(*p1) == contour_start => {
                // The closing line: `Z` draws it.
                out.push('Z');
                contour_start = None;
                last = None;
                continue;
            }
            OutlineCurve::Line(_, p1) => push_point(out, 'L', place(*p1)),
            OutlineCurve::Quad(_, c, p1) => {
                push_point(out, 'Q', place(*c));
                out.push(' ');
                push_number_pair(out, place(*p1));
            }
            OutlineCurve::Cubic(_, c0, c1, p1) => {
                push_point(out, 'C', place(*c0));
                out.push(' ');
                push_number_pair(out, place(*c1));
                out.push(' ');
                push_number_pair(out, place(*p1));
            }
        }
        last = Some(end);
    }
    if contour_start.is_some() {
        out.push('Z');
    }
}

fn push_number_pair(out: &mut String, (x, y): (f64, f64)) {
    push_number(out, x);
    out.push(' ');
    push_number(out, y);
}

/// Lays `element` out and returns its path data with the ink box, both
/// before rotation; `None` when no glyph has ink (an empty or all-space
/// label).
fn layout(
    element: &TextElement<'_>,
    font: &FontRef<'static>,
) -> Result<Option<(String, InkBox)>, ExportError> {
    let plotters_size = element.css_size * EM_RATIO;
    // ab_glyph scales in f32, as Plotters does.
    let scale = plotters_size as f32;
    let scaled = font.as_scaled(PxScale::from(scale));
    let mut glyphs = Vec::with_capacity(element.text.len());
    for character in element.text.chars() {
        let drawn = if LINE_WHITESPACE.contains(&character) {
            ' '
        } else {
            character
        };
        let id = font.glyph_id(drawn);
        if id == NOTDEF {
            return Err(ExportError::MissingGlyph {
                character,
                code: u32::from(character),
                text: element.text.clone(),
            });
        }
        glyphs.push(id);
    }

    // The anchor box, exactly as plotters' ab_glyph `estimate_layout` and
    // plotters-backend's default `draw_text` compute it: the advance width
    // truncated to whole pixels, the size in CSS pixels as the height.
    let mut advance = 0_f32;
    let mut previous = None;
    for id in &glyphs {
        advance += scaled.h_advance(*id);
        if let Some(previous) = previous {
            advance += scaled.kern(previous, *id);
        }
        previous = Some(*id);
    }
    let (width, height) = (advance as i32, element.css_size as i32);
    let dx = match element.anchor {
        Anchor::Start => 0,
        Anchor::Middle => -width / 2,
        Anchor::End => -width,
    };
    let dy = match element.baseline {
        Baseline::Top => 0,
        Baseline::Center => -height / 2,
        Baseline::Bottom => -height,
    };
    let left = f64::from(element.x) + f64::from(dx);
    // Plotters puts the baseline half the font size below the box's top.
    let baseline = f64::from(element.y) + f64::from(dy) + f64::from(scale / 2.0);
    let (h_factor, v_factor) = (
        f64::from(scaled.h_scale_factor()),
        f64::from(scaled.v_scale_factor()),
    );

    let mut path = String::new();
    let mut ink: Option<InkBox> = None;
    let mut pen = 0_f32;
    let mut previous = None;
    for id in glyphs {
        if let Some(previous) = previous {
            pen += scaled.kern(previous, id);
        }
        previous = Some(id);
        let origin = left + f64::from(pen);
        pen += scaled.h_advance(id);
        // Glyphs without ink (spaces) only advance the pen.
        let Some(outline) = font.outline(id) else {
            continue;
        };
        let place = |point: Point| {
            (
                origin + f64::from(point.x) * h_factor,
                baseline - f64::from(point.y) * v_factor,
            )
        };
        push_glyph(&mut path, &outline.curves, place);
        // ab_glyph's unscaled bounds: `min` is (x_min, y_max) and `max` is
        // (x_max, y_min), y up.
        let (left_top, right_bottom) = (place(outline.bounds.min), place(outline.bounds.max));
        let glyph = InkBox {
            left: left_top.0,
            top: left_top.1,
            right: right_bottom.0,
            bottom: right_bottom.1,
        };
        ink = Some(ink.map_or(glyph, |ink| ink.union(glyph)));
    }
    Ok(ink.map(|ink| (path, ink)))
}

/// The markup replacing `element`, with its label; `None` for a label with
/// nothing to paint or announce.
fn outline_element(
    element: &TextElement<'_>,
    font: &FontRef<'static>,
) -> Result<Option<(String, Label)>, ExportError> {
    let Some((path, ink)) = layout(element, font)? else {
        return Ok(None);
    };
    // The accessible name reads line whitespace as the space it is drawn
    // as; XML would turn it into one in an attribute anyway.
    let text: String = element
        .text
        .chars()
        .map(|c| if LINE_WHITESPACE.contains(&c) { ' ' } else { c })
        .collect();
    ensure_xml_text(&text)?;
    let mut markup = String::with_capacity(path.len() + text.len() + 128);
    markup.push_str("<g role=\"img\" aria-label=\"");
    push_escaped(&mut markup, &text);
    markup.push_str("\" fill=\"");
    markup.push_str(element.fill);
    markup.push_str("\" opacity=\"");
    markup.push_str(element.opacity);
    markup.push('"');
    if let Some((_, transform)) = element.rotation {
        markup.push_str(" transform=\"");
        markup.push_str(transform);
        markup.push('"');
    }
    markup.push_str("><path d=\"");
    markup.push_str(&path);
    markup.push_str("\"/></g>");
    let ink = ink.rotated(
        element.rotation.map(|(rotation, _)| rotation),
        f64::from(element.x),
        f64::from(element.y),
    );
    Ok(Some((
        markup,
        Label {
            text,
            left: ink.left,
            top: ink.top,
            right: ink.right,
            bottom: ink.bottom,
        },
    )))
}

/// Inserts `<title>` and `<desc>` as the first children of the `<svg>`
/// element.
fn with_chart_text(document: &str, chart: &ChartText) -> Result<String, ExportError> {
    // The description names series that may not be drawn (no legend), so
    // its text has not been through the glyph check.
    ensure_xml_text(&chart.title)?;
    ensure_xml_text(&chart.description)?;
    let tag_end = document
        .find("<svg")
        .and_then(|start| document.get(start..)?.find('>').map(|end| start + end + 1))
        .ok_or_else(|| unsupported("a document without an <svg> element"))?;
    let (head, tail) = document
        .split_at_checked(tag_end)
        .ok_or_else(|| unsupported("<svg> tag at an invalid offset"))?;
    let mut out = String::with_capacity(document.len() + chart.title.len() + 64);
    out.push_str(head);
    out.push_str("<title>");
    push_escaped(&mut out, &chart.title);
    out.push_str("</title><desc>");
    push_escaped(&mut out, &chart.description);
    out.push_str("</desc>");
    out.push_str(tail);
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG_OPEN: &str = "<svg width=\"200\" height=\"100\" viewBox=\"0 0 200 100\" xmlns=\"http://www.w3.org/2000/svg\">\n";

    /// A text element exactly as plotters-svg 0.3.7 writes it.
    fn text_element(attributes: &str, content: &str) -> String {
        format!(
            "<text x=\"20\" y=\"30\" dy=\"0.5ex\" text-anchor=\"start\" font-family=\"Open Sans\" \
             font-size=\"12\" opacity=\"1\" fill=\"#2A3F5F\"{attributes}>\n{content}\n</text>\n"
        )
    }

    fn document(elements: &str) -> String {
        format!("{SVG_OPEN}{elements}</svg>\n")
    }

    fn chart() -> ChartText {
        ChartText {
            title: "Title & <name>".to_string(),
            description: "Two \"series\"".to_string(),
        }
    }

    fn convert(elements: &str) -> Result<Outlined, ExportError> {
        outline_text(&document(elements), &chart())
    }

    fn single_label(outlined: &Outlined) -> &Label {
        assert_eq!(outlined.labels.len(), 1, "{:?}", outlined.labels);
        let Some(label) = outlined.labels.first() else {
            panic!("no label");
        };
        label
    }

    #[test]
    fn test_outline_text_replaces_text_with_a_labelled_path() -> Result<(), ExportError> {
        let outlined = convert(&text_element("", "Strike"))?;
        assert!(!outlined.document.contains("<text"));
        assert!(!outlined.document.contains("font-family"));
        assert!(outlined.document.contains(
            "<g role=\"img\" aria-label=\"Strike\" fill=\"#2A3F5F\" opacity=\"1\"><path d=\"M"
        ));
        assert_eq!(single_label(&outlined).text, "Strike");
        Ok(())
    }

    #[test]
    fn test_outline_text_escapes_the_label_and_adds_title_and_desc() -> Result<(), ExportError> {
        let outlined = convert(&text_element("", "&amp; &lt; &gt; &quot; &apos; $"))?;
        assert_eq!(single_label(&outlined).text, "& < > \" ' $");
        assert!(
            outlined
                .document
                .contains("aria-label=\"&amp; &lt; &gt; &quot; &apos; $\"")
        );
        assert!(outlined.document.contains(
            "xmlns=\"http://www.w3.org/2000/svg\"><title>Title &amp; &lt;name&gt;</title>\
             <desc>Two &quot;series&quot;</desc>\n"
        ));
        Ok(())
    }

    #[test]
    fn test_outline_text_rejects_a_missing_glyph() {
        let result = convert(&text_element("", "IV 中"));
        assert!(matches!(
            result,
            Err(ExportError::MissingGlyph { character: '中', code: 0x4E2D, ref text }) if text == "IV 中"
        ));
    }

    #[test]
    fn test_outline_text_rejects_constructs_outside_the_subset() {
        let cases = [
            text_element(" font-weight=\"bold\"", "a"),
            text_element(" transform=\"rotate(45, 20, 30)\"", "a"),
            text_element(" transform=\"rotate(90, 0, 0)\"", "a"),
            text_element("", "<tspan>a</tspan>"),
            text_element("", "&nbsp;"),
            text_element(" x=\"1\"", "a"),
            "<text x=\"1\" y=\"2\">a</text>".to_string(),
            "<textPath>a</textPath>".to_string(),
            text_element("", "a").replace("Open Sans", "Arial"),
            text_element("", "a").replace("\n</text>", "</text>"),
            text_element("", "a").replace("</text>", ""),
        ];
        for case in cases {
            let result = convert(&case);
            assert!(
                matches!(result, Err(ExportError::UnsupportedSvgText(_))),
                "{case:?} gave {result:?}"
            );
        }
    }

    #[test]
    fn test_spaces_keep_their_advance_and_blank_labels_paint_nothing() -> Result<(), ExportError> {
        let plain = convert(&text_element("", "AV"))?;
        let padded = convert(&text_element("", "  AV"))?;
        let (plain, padded) = (single_label(&plain), single_label(&padded));
        // Two spaces of Open Sans at 12 px: 2 x 532/2048 em.
        let shift = padded.left - plain.left;
        assert!(shift > 4.0 && shift < 7.0, "shift {shift}");
        assert!((padded.right - plain.right - shift).abs() < 1e-9);

        let blank = convert(&format!(
            "{}{}",
            text_element("", "   "),
            text_element("", "")
        ))?;
        assert!(blank.labels.is_empty());
        assert!(!blank.document.contains("<g role"));
        Ok(())
    }

    #[test]
    fn test_metadata_xml_cannot_carry_is_an_error() {
        for bad in ['\0', '\u{B}', '\u{1F}', '\u{FFFE}', '\u{FFFF}'] {
            let chart = ChartText {
                title: "Chart".to_string(),
                description: format!("hidden{bad}series"),
            };
            let result = outline_text(&document(&text_element("", "a")), &chart);
            assert!(
                matches!(result, Err(ExportError::NotXmlText { character, .. }) if character == bad),
                "{bad:?}: {result:?}"
            );
        }
        let chart = ChartText {
            title: "Tab\tand\nbreaks\r are XML".to_string(),
            description: "\u{10FFFF} \u{FFFD}".to_string(),
        };
        assert!(outline_text(&document(&text_element("", "a")), &chart).is_ok());
    }

    #[test]
    fn test_line_whitespace_advances_like_a_space() -> Result<(), ExportError> {
        // A space, a line feed and a tab: three space advances.
        let spaced = convert(&text_element("", "Strategy:   Underlying"))?;
        let broken = convert(&text_element("", "Strategy: \n\tUnderlying"))?;
        let (spaced, broken) = (single_label(&spaced), single_label(&broken));
        assert_eq!(broken.text, "Strategy:   Underlying");
        assert!((spaced.left - broken.left).abs() < 1e-9);
        assert!((spaced.right - broken.right).abs() < 1e-9);
        let bell = convert(&text_element("", "a\u{7}b"));
        assert!(matches!(
            bell,
            Err(ExportError::MissingGlyph {
                character: '\u{7}',
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn test_anchors_place_the_layout_box_like_plotters() -> Result<(), ExportError> {
        let start = convert(&text_element("", "Strike"))?;
        let middle = convert(&text_element("", "Strike").replace("\"start\"", "\"middle\""))?;
        let end = convert(&text_element("", "Strike").replace("\"start\"", "\"end\""))?;
        let (start, middle, end) = (
            single_label(&start),
            single_label(&middle),
            single_label(&end),
        );
        let width = start.left - end.left;
        assert!(width > 20.0 && width < 40.0, "width {width}");
        let half = start.left - middle.left;
        assert!((half - (width / 2.0).trunc()).abs() <= 1.0, "half {half}");
        Ok(())
    }

    #[test]
    fn test_baselines_follow_plotters_vertical_anchors() -> Result<(), ExportError> {
        let top = convert(&text_element("", "H").replace("0.5ex", "0.76em"))?;
        let center = convert(&text_element("", "H"))?;
        let bottom = convert(&text_element("", "H").replace("0.5ex", "-0.5ex"))?;
        let (top, center, bottom) = (
            single_label(&top),
            single_label(&center),
            single_label(&bottom),
        );
        // At 12 px the layout box is 12 px high: the center anchor moves the
        // text up by 6, the bottom anchor by 12.
        assert!((top.top - center.top - 6.0).abs() < 1e-9);
        assert!((top.top - bottom.top - 12.0).abs() < 1e-9);
        // The cap height of H sits under the anchor line for a top anchor.
        assert!(top.top > 29.0 && top.top < 31.0, "top {}", top.top);
        Ok(())
    }

    #[test]
    fn test_rotation_is_kept_and_rotates_the_ink_box() -> Result<(), ExportError> {
        let flat = convert(&text_element("", "Value"))?;
        let turned = convert(&text_element(" transform=\"rotate(270, 20, 30)\"", "Value"))?;
        assert!(
            turned
                .document
                .contains(" transform=\"rotate(270, 20, 30)\"><path")
        );
        let (flat, turned) = (single_label(&flat), single_label(&turned));
        let close = |a: f64, b: f64| (a - b).abs() < 1e-9;
        assert!(close(flat.right - flat.left, turned.bottom - turned.top));
        assert!(close(flat.bottom - flat.top, turned.right - turned.left));
        // A quarter turn counter-clockwise: text reads upwards from the anchor.
        assert!(close(turned.bottom, 30.0 - (flat.left - 20.0)));
        Ok(())
    }

    #[test]
    fn test_bundled_font_kerns_nothing_through_ab_glyph() -> Result<(), ExportError> {
        // Open Sans 3.003 keeps its kerning in GPOS only, and ab_glyph reads
        // the legacy `kern` table: Plotters' layout, its PNG and these
        // outlines kern no pair, while browsers apply GPOS to `<text>`.
        let font = bundled_font()?;
        let scaled = font.as_scaled(PxScale::from(20.0));
        for pair in ["AV", "To", "Te", "Yo", "LT", "P5", "VA"] {
            let mut glyphs = pair.chars().map(|c| font.glyph_id(c));
            let (Some(first), Some(second)) = (glyphs.next(), glyphs.next()) else {
                panic!("two characters");
            };
            assert_ne!(first, NOTDEF);
            assert_eq!(scaled.kern(first, second), 0.0, "{pair}");
        }
        Ok(())
    }

    #[test]
    fn test_push_number_trims_and_rounds() {
        let mut out = String::new();
        for value in [1.0, -0.001, 12.345_678, -3.1, 0.105] {
            push_number(&mut out, value);
            out.push(' ');
        }
        assert_eq!(out, "1 0 12.35 -3.1 0.1 ");
    }

    #[test]
    fn test_unescape_and_escape_round_trip() -> Result<(), ExportError> {
        let text = "a & b < c > d \" e ' f";
        let mut escaped = String::new();
        push_escaped(&mut escaped, text);
        assert_eq!(unescape(&escaped)?, text);
        Ok(())
    }
}
