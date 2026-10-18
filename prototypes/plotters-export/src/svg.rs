//! SVG text portability.
//!
//! plotters-svg writes every text element as `<text font-family="Open
//! Sans">`: glyphs are the viewer's business. A viewer without Open Sans
//! falls back to its default font (often a serif), whose metrics differ from
//! the ones the layout was measured with, so labels can shift or clip. The
//! two post-processing steps here are the options measured in the report.

use crate::render::{FONT_BYTES, FONT_FAMILY};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// The attribute plotters-svg writes on every text element.
fn plotters_family_attribute() -> String {
    format!("font-family=\"{FONT_FAMILY}\"")
}

/// The same attribute with a generic fallback after the named family.
fn fallback_family_attribute() -> String {
    format!("font-family=\"'{FONT_FAMILY}', sans-serif\"")
}

/// Adds a `sans-serif` fallback to every text element, so a viewer without
/// Open Sans picks a sans-serif rather than its default font.
#[must_use]
pub fn with_fallback_family(document: &str) -> String {
    document.replace(&plotters_family_attribute(), &fallback_family_attribute())
}

/// Embeds the font as a base64 `@font-face` right after the opening `<svg>`
/// tag and adds the `sans-serif` fallback. A document with no `<svg` tag is
/// returned with the fallback only.
#[must_use]
pub fn embed_font(document: &str) -> String {
    let document = with_fallback_family(document);
    let Some(tag_end) = document
        .find("<svg")
        .and_then(|start| document.get(start..)?.find('>').map(|end| start + end + 1))
    else {
        return document;
    };
    let (head, tail) = document.split_at(tag_end);
    let encoded = STANDARD.encode(FONT_BYTES);
    format!(
        "{head}<defs><style>@font-face{{font-family:'{FONT_FAMILY}';\
         src:url(data:font/ttf;base64,{encoded}) format('truetype');}}</style></defs>{tail}"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const DOCUMENT: &str = "<svg width=\"10\" height=\"10\" xmlns=\"http://www.w3.org/2000/svg\">\
        <text font-family=\"Open Sans\">a</text></svg>";

    #[test]
    fn test_with_fallback_family_adds_sans_serif() {
        let svg = with_fallback_family(DOCUMENT);
        assert!(svg.contains("font-family=\"'Open Sans', sans-serif\""));
        assert!(!svg.contains("font-family=\"Open Sans\""));
    }

    #[test]
    fn test_embed_font_inserts_the_face_inside_the_svg_element() {
        let svg = embed_font(DOCUMENT);
        let defs = svg.find("<defs><style>@font-face");
        let open = svg.find("xmlns=\"http://www.w3.org/2000/svg\">");
        assert!(matches!((open, defs), (Some(open), Some(defs)) if defs > open));
        assert!(svg.contains("data:font/ttf;base64,"));
        assert!(svg.ends_with("</svg>"));
    }

    #[test]
    fn test_embed_font_without_svg_tag_only_adds_the_fallback() {
        let svg = embed_font("<text font-family=\"Open Sans\">a</text>");
        assert!(!svg.contains("@font-face"));
        assert!(svg.contains("sans-serif"));
    }
}
