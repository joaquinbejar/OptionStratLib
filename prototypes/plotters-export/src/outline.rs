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
//! Labels are laid out by [`crate::text`] with the metrics Plotters
//! measured the chart with, which are also the ones the PNG is drawn with:
//! the Plotters font size is the SVG `font-size` times `EM_RATIO`, the
//! factor plotters-svg divided it by, applied once here; each character
//! advances by its glyph's advance; the anchor box is that of
//! `plotters-backend`'s default `draw_text`. The room the layout reserved
//! (legend column, label areas) therefore fits the painted text. Glyphs
//! keep their side bearings and sub-pixel pen positions. There is no
//! kerning and no shaping: one glyph per character, no ligatures.
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
use crate::render::{EM_RATIO, FONT_FAMILY};
use crate::text::{BundledFont, InkBox, LINE_WHITESPACE, Missing, Segment, bundled_font};
use serde::Serialize;

const TEXT_OPEN: &str = "<text";
const TEXT_CLOSE: &str = "</text>";
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

/// `ink` after `rotation` about `(cx, cy)`, in SVG's y-down sense.
fn rotated(ink: InkBox, rotation: Option<Rotation>, cx: f64, cy: f64) -> InkBox {
    let turn = |(x, y): (f64, f64)| {
        let (dx, dy) = (x - cx, y - cy);
        match rotation {
            None => (x, y),
            Some(Rotation::Quarter) => (cx - dy, cy + dx),
            Some(Rotation::Half) => (cx - dx, cy - dy),
            Some(Rotation::ThreeQuarters) => (cx + dy, cy - dx),
        }
    };
    let (x0, y0) = turn((ink.left, ink.top));
    let (x1, y1) = turn((ink.right, ink.bottom));
    InkBox {
        left: x0.min(x1),
        top: y0.min(y1),
        right: x0.max(x1),
        bottom: y0.max(y1),
    }
}

#[cold]
fn unsupported(reason: impl Into<String>) -> ExportError {
    ExportError::UnsupportedSvgText(reason.into())
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

/// Appends one outline segment as path data.
fn push_segment(out: &mut String, segment: Segment) {
    match segment {
        Segment::Move(p) => push_point(out, 'M', p),
        Segment::Line(p) => push_point(out, 'L', p),
        Segment::Quad(c, p) => {
            push_point(out, 'Q', c);
            out.push(' ');
            push_number_pair(out, p);
        }
        Segment::Cubic(c0, c1, p) => {
            push_point(out, 'C', c0);
            out.push(' ');
            push_number_pair(out, c1);
            out.push(' ');
            push_number_pair(out, p);
        }
        Segment::Close => out.push('Z'),
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
    font: &BundledFont,
) -> Result<Option<(String, InkBox)>, ExportError> {
    // The height is the attribute's own: it is the height Plotters measured.
    let layout = font
        .layout(&element.text, element.css_size * EM_RATIO, Missing::Error)?
        .with_height(element.css_size as i32);
    let dx = match element.anchor {
        Anchor::Start => 0,
        Anchor::Middle => -layout.width / 2,
        Anchor::End => -layout.width,
    };
    let dy = match element.baseline {
        Baseline::Top => 0,
        Baseline::Center => -layout.height / 2,
        Baseline::Bottom => -layout.height,
    };
    let origin = (
        f64::from(element.x) + f64::from(dx),
        f64::from(element.y) + f64::from(dy),
    );
    let mut path = String::new();
    let ink = font.trace(&layout, origin, |segment| push_segment(&mut path, segment))?;
    Ok(ink.map(|ink| (path, ink)))
}

/// The markup replacing `element`, with its label; `None` for a label with
/// nothing to paint or announce.
fn outline_element(
    element: &TextElement<'_>,
    font: &BundledFont,
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
    let ink = rotated(
        ink,
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
    fn test_pairs_are_not_kerned() -> Result<(), ExportError> {
        // No kerning: a pair's label is exactly as wide as its two glyphs
        // drawn apart, as with ab_glyph (Open Sans 3.003 keeps its kerning
        // in GPOS only, which ab_glyph never read either).
        let (Some(one), Some(two)) = (
            convert(&text_element("", "A"))?.labels.first().cloned(),
            convert(&text_element("", "AV"))?.labels.first().cloned(),
        ) else {
            panic!("labels");
        };
        let font = bundled_font()?;
        let v = font.layout("V", 12.0 * EM_RATIO, Missing::Error)?;
        let pair = font.layout("AV", 12.0 * EM_RATIO, Missing::Error)?;
        let a = font.layout("A", 12.0 * EM_RATIO, Missing::Error)?;
        assert!((pair.width - a.width - v.width).abs() <= 1, "{pair:?}");
        assert!(two.right > one.right);
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
