//! Colour strings for the native export (#925, condition 3 of #920).
//!
//! `ColorScheme::Custom` documents "hex code, RGB, color name", and the
//! Plotly path hands every colour string to plotly.js, whose parser accepts
//! the CSS forms. The native renderer accepts the same forms, so moving a
//! chart to it does not narrow what a caller may write:
//!
//! * the 148 CSS Color 4 named colours, case-insensitive, `gray`/`grey`
//!   spellings included, and `transparent` (black with alpha 0);
//! * `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`;
//! * `rgb()` and `rgba()` (the same function), with comma-separated or
//!   space-separated channels, each a number from 0 to 255 or a percentage,
//!   and an optional alpha after a comma or `/`, a number from 0 to 1 or a
//!   percentage;
//! * `hsl()` and `hsla()`, likewise: hue in degrees (an optional `deg`),
//!   saturation and lightness as percentages or plain numbers.
//!
//! Whitespace around the string and inside functions is ignored, and
//! function names are case-insensitive. Out-of-range channels are clamped
//! as CSS clamps them. Anything else is [`ExportError::InvalidColor`], never
//! a silent black: context-dependent keywords (`currentcolor`, `inherit`,
//! `initial`, `unset`, `revert`), system colours, other functions such as
//! `hwb()` or `color()`, and malformed numbers.

use crate::error::ExportError;
use plotters::style::RGBAColor;

/// Keywords whose colour depends on a document or user agent the export
/// does not have.
const CONTEXT_KEYWORDS: [&str; 6] = [
    "currentcolor",
    "inherit",
    "initial",
    "unset",
    "revert",
    "revert-layer",
];

/// The CSS Color 4 named colours, sorted by name.
const NAMED: [(&str, u32); 148] = [
    ("aliceblue", 0xF0F8FF),
    ("antiquewhite", 0xFAEBD7),
    ("aqua", 0x00FFFF),
    ("aquamarine", 0x7FFFD4),
    ("azure", 0xF0FFFF),
    ("beige", 0xF5F5DC),
    ("bisque", 0xFFE4C4),
    ("black", 0x000000),
    ("blanchedalmond", 0xFFEBCD),
    ("blue", 0x0000FF),
    ("blueviolet", 0x8A2BE2),
    ("brown", 0xA52A2A),
    ("burlywood", 0xDEB887),
    ("cadetblue", 0x5F9EA0),
    ("chartreuse", 0x7FFF00),
    ("chocolate", 0xD2691E),
    ("coral", 0xFF7F50),
    ("cornflowerblue", 0x6495ED),
    ("cornsilk", 0xFFF8DC),
    ("crimson", 0xDC143C),
    ("cyan", 0x00FFFF),
    ("darkblue", 0x00008B),
    ("darkcyan", 0x008B8B),
    ("darkgoldenrod", 0xB8860B),
    ("darkgray", 0xA9A9A9),
    ("darkgreen", 0x006400),
    ("darkgrey", 0xA9A9A9),
    ("darkkhaki", 0xBDB76B),
    ("darkmagenta", 0x8B008B),
    ("darkolivegreen", 0x556B2F),
    ("darkorange", 0xFF8C00),
    ("darkorchid", 0x9932CC),
    ("darkred", 0x8B0000),
    ("darksalmon", 0xE9967A),
    ("darkseagreen", 0x8FBC8F),
    ("darkslateblue", 0x483D8B),
    ("darkslategray", 0x2F4F4F),
    ("darkslategrey", 0x2F4F4F),
    ("darkturquoise", 0x00CED1),
    ("darkviolet", 0x9400D3),
    ("deeppink", 0xFF1493),
    ("deepskyblue", 0x00BFFF),
    ("dimgray", 0x696969),
    ("dimgrey", 0x696969),
    ("dodgerblue", 0x1E90FF),
    ("firebrick", 0xB22222),
    ("floralwhite", 0xFFFAF0),
    ("forestgreen", 0x228B22),
    ("fuchsia", 0xFF00FF),
    ("gainsboro", 0xDCDCDC),
    ("ghostwhite", 0xF8F8FF),
    ("gold", 0xFFD700),
    ("goldenrod", 0xDAA520),
    ("gray", 0x808080),
    ("green", 0x008000),
    ("greenyellow", 0xADFF2F),
    ("grey", 0x808080),
    ("honeydew", 0xF0FFF0),
    ("hotpink", 0xFF69B4),
    ("indianred", 0xCD5C5C),
    ("indigo", 0x4B0082),
    ("ivory", 0xFFFFF0),
    ("khaki", 0xF0E68C),
    ("lavender", 0xE6E6FA),
    ("lavenderblush", 0xFFF0F5),
    ("lawngreen", 0x7CFC00),
    ("lemonchiffon", 0xFFFACD),
    ("lightblue", 0xADD8E6),
    ("lightcoral", 0xF08080),
    ("lightcyan", 0xE0FFFF),
    ("lightgoldenrodyellow", 0xFAFAD2),
    ("lightgray", 0xD3D3D3),
    ("lightgreen", 0x90EE90),
    ("lightgrey", 0xD3D3D3),
    ("lightpink", 0xFFB6C1),
    ("lightsalmon", 0xFFA07A),
    ("lightseagreen", 0x20B2AA),
    ("lightskyblue", 0x87CEFA),
    ("lightslategray", 0x778899),
    ("lightslategrey", 0x778899),
    ("lightsteelblue", 0xB0C4DE),
    ("lightyellow", 0xFFFFE0),
    ("lime", 0x00FF00),
    ("limegreen", 0x32CD32),
    ("linen", 0xFAF0E6),
    ("magenta", 0xFF00FF),
    ("maroon", 0x800000),
    ("mediumaquamarine", 0x66CDAA),
    ("mediumblue", 0x0000CD),
    ("mediumorchid", 0xBA55D3),
    ("mediumpurple", 0x9370DB),
    ("mediumseagreen", 0x3CB371),
    ("mediumslateblue", 0x7B68EE),
    ("mediumspringgreen", 0x00FA9A),
    ("mediumturquoise", 0x48D1CC),
    ("mediumvioletred", 0xC71585),
    ("midnightblue", 0x191970),
    ("mintcream", 0xF5FFFA),
    ("mistyrose", 0xFFE4E1),
    ("moccasin", 0xFFE4B5),
    ("navajowhite", 0xFFDEAD),
    ("navy", 0x000080),
    ("oldlace", 0xFDF5E6),
    ("olive", 0x808000),
    ("olivedrab", 0x6B8E23),
    ("orange", 0xFFA500),
    ("orangered", 0xFF4500),
    ("orchid", 0xDA70D6),
    ("palegoldenrod", 0xEEE8AA),
    ("palegreen", 0x98FB98),
    ("paleturquoise", 0xAFEEEE),
    ("palevioletred", 0xDB7093),
    ("papayawhip", 0xFFEFD5),
    ("peachpuff", 0xFFDAB9),
    ("peru", 0xCD853F),
    ("pink", 0xFFC0CB),
    ("plum", 0xDDA0DD),
    ("powderblue", 0xB0E0E6),
    ("purple", 0x800080),
    ("rebeccapurple", 0x663399),
    ("red", 0xFF0000),
    ("rosybrown", 0xBC8F8F),
    ("royalblue", 0x4169E1),
    ("saddlebrown", 0x8B4513),
    ("salmon", 0xFA8072),
    ("sandybrown", 0xF4A460),
    ("seagreen", 0x2E8B57),
    ("seashell", 0xFFF5EE),
    ("sienna", 0xA0522D),
    ("silver", 0xC0C0C0),
    ("skyblue", 0x87CEEB),
    ("slateblue", 0x6A5ACD),
    ("slategray", 0x708090),
    ("slategrey", 0x708090),
    ("snow", 0xFFFAFA),
    ("springgreen", 0x00FF7F),
    ("steelblue", 0x4682B4),
    ("tan", 0xD2B48C),
    ("teal", 0x008080),
    ("thistle", 0xD8BFD8),
    ("tomato", 0xFF6347),
    ("turquoise", 0x40E0D0),
    ("violet", 0xEE82EE),
    ("wheat", 0xF5DEB3),
    ("white", 0xFFFFFF),
    ("whitesmoke", 0xF5F5F5),
    ("yellow", 0xFFFF00),
    ("yellowgreen", 0x9ACD32),
];

#[cold]
fn invalid(color: &str, reason: &'static str) -> ExportError {
    ExportError::InvalidColor {
        color: color.to_string(),
        reason,
    }
}

/// Parses a colour string as the module documentation describes.
///
/// # Errors
///
/// [`ExportError::InvalidColor`] for anything outside those forms.
pub(crate) fn parse_color(text: &str) -> Result<RGBAColor, ExportError> {
    let trimmed = text.trim();
    let lower = trimmed.to_ascii_lowercase();
    if let Some(hex) = lower.strip_prefix('#') {
        return parse_hex(text, hex);
    }
    if let Some(open) = lower.find('(') {
        let (name, rest) = lower
            .split_at_checked(open)
            .ok_or_else(|| invalid(text, "malformed function"))?;
        let arguments = rest
            .strip_prefix('(')
            .and_then(|inner| inner.strip_suffix(')'))
            .ok_or_else(|| invalid(text, "a function must end with its closing parenthesis"))?;
        return match name.trim_end() {
            "rgb" | "rgba" => parse_rgb(text, arguments),
            "hsl" | "hsla" => parse_hsl(text, arguments),
            _ => Err(invalid(
                text,
                "only rgb(), rgba(), hsl() and hsla() are supported",
            )),
        };
    }
    if lower == "transparent" {
        return Ok(RGBAColor(0, 0, 0, 0.0));
    }
    if CONTEXT_KEYWORDS.contains(&lower.as_str()) {
        return Err(invalid(
            text,
            "a context-dependent keyword has no colour in an exported image",
        ));
    }
    match NAMED.binary_search_by(|(name, _)| (*name).cmp(lower.as_str())) {
        Ok(index) => {
            let (_, value) = NAMED
                .get(index)
                .ok_or_else(|| invalid(text, "named colour lookup"))?;
            let [_, r, g, b] = value.to_be_bytes();
            Ok(RGBAColor(r, g, b, 1.0))
        }
        Err(_) => Err(invalid(text, "not a CSS named colour")),
    }
}

fn parse_hex(text: &str, digits: &str) -> Result<RGBAColor, ExportError> {
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid(text, "hex colours use the digits 0-9 and a-f"));
    }
    let value = |range: std::ops::Range<usize>| {
        digits
            .get(range)
            .and_then(|part| u8::from_str_radix(part, 16).ok())
            .ok_or_else(|| invalid(text, "malformed hex colour"))
    };
    // `#rgb` doubles each digit: `#f80` is `#ff8800`.
    let short = |index: usize| value(index..index + 1).map(|digit| digit * 17);
    let alpha = |byte: u8| f64::from(byte) / 255.0;
    match digits.len() {
        3 => Ok(RGBAColor(short(0)?, short(1)?, short(2)?, 1.0)),
        4 => Ok(RGBAColor(short(0)?, short(1)?, short(2)?, alpha(short(3)?))),
        6 => Ok(RGBAColor(value(0..2)?, value(2..4)?, value(4..6)?, 1.0)),
        8 => Ok(RGBAColor(
            value(0..2)?,
            value(2..4)?,
            value(4..6)?,
            alpha(value(6..8)?),
        )),
        _ => Err(invalid(text, "hex colours have 3, 4, 6 or 8 digits")),
    }
}

/// The components of a colour function and its optional alpha, in either
/// `a, b, c[, alpha]` or `a b c[ / alpha]` form.
fn components<'a>(
    text: &str,
    arguments: &'a str,
) -> Result<([&'a str; 3], Option<&'a str>), ExportError> {
    let parts: Vec<&str> = if arguments.contains(',') {
        if arguments.contains('/') {
            return Err(invalid(text, "mixes commas and a slash"));
        }
        arguments.split(',').map(str::trim).collect()
    } else {
        let (channels, alpha) = match arguments.split_once('/') {
            Some((channels, alpha)) => (channels, Some(alpha.trim())),
            None => (arguments, None),
        };
        let mut parts: Vec<&str> = channels.split_whitespace().collect();
        if let Some(alpha) = alpha {
            parts.push(alpha);
        }
        parts
    };
    match parts.as_slice() {
        [a, b, c] => Ok(([a, b, c], None)),
        [a, b, c, alpha] => Ok(([a, b, c], Some(alpha))),
        _ => Err(invalid(
            text,
            "a colour function takes three channels and an optional alpha",
        )),
    }
}

/// A finite number, with `%` turned into a fraction of `full`.
fn number(text: &str, part: &str, full: f64) -> Result<f64, ExportError> {
    let (digits, scale) = match part.strip_suffix('%') {
        Some(digits) => (digits, full / 100.0),
        None => (part, 1.0),
    };
    match digits.trim().parse::<f64>() {
        Ok(value) if value.is_finite() && !digits.trim().is_empty() => Ok(value * scale),
        _ => Err(invalid(text, "a channel is not a finite number")),
    }
}

/// A channel clamped to 0..=255 and rounded, as CSS does.
fn channel(value: f64) -> u8 {
    // Clamped to 0..=255 first, so the conversion is exact.
    value.clamp(0.0, 255.0).round() as u8
}

fn alpha(text: &str, part: Option<&str>) -> Result<f64, ExportError> {
    match part {
        Some(part) => Ok(number(text, part, 1.0)?.clamp(0.0, 1.0)),
        None => Ok(1.0),
    }
}

fn parse_rgb(text: &str, arguments: &str) -> Result<RGBAColor, ExportError> {
    let ([r, g, b], a) = components(text, arguments)?;
    Ok(RGBAColor(
        channel(number(text, r, 255.0)?),
        channel(number(text, g, 255.0)?),
        channel(number(text, b, 255.0)?),
        alpha(text, a)?,
    ))
}

fn parse_hsl(text: &str, arguments: &str) -> Result<RGBAColor, ExportError> {
    let ([h, s, l], a) = components(text, arguments)?;
    let hue = number(text, h.strip_suffix("deg").unwrap_or(h), 1.0)?.rem_euclid(360.0);
    // Saturation and lightness as percentages; a plain number counts as one.
    let saturation = number(text, s, 100.0)?.clamp(0.0, 100.0) / 100.0;
    let lightness = number(text, l, 100.0)?.clamp(0.0, 100.0) / 100.0;
    // CSS Color 4, `hslToRgb`.
    let convert = |n: f64| {
        let k = (n + hue / 30.0).rem_euclid(12.0);
        let a = saturation * lightness.min(1.0 - lightness);
        lightness - a * (k - 3.0).min(9.0 - k).clamp(-1.0, 1.0)
    };
    Ok(RGBAColor(
        channel(convert(0.0) * 255.0),
        channel(convert(8.0) * 255.0),
        channel(convert(4.0) * 255.0),
        alpha(text, a)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rgba(text: &str) -> (u8, u8, u8, f64) {
        match parse_color(text) {
            Ok(RGBAColor(r, g, b, a)) => (r, g, b, a),
            Err(error) => panic!("{text:?}: {error}"),
        }
    }

    #[test]
    fn test_named_table_is_sorted_and_unique() {
        assert!(
            NAMED
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a.0 < b.0))
        );
    }

    #[test]
    fn test_named_table_matches_an_independent_reference() -> Result<(), serde_json::Error> {
        // Generated from Pillow's CSS colour table, not from `NAMED`.
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("../tests/data/css-named-colors.json"))?;
        let Some(colors) = reference.get("colors").and_then(|c| c.as_object()) else {
            panic!("reference without colours");
        };
        assert_eq!(colors.len(), NAMED.len());
        for (name, expected) in colors {
            let channels: Vec<u64> = expected
                .as_array()
                .map(|values| {
                    values
                        .iter()
                        .filter_map(serde_json::Value::as_u64)
                        .collect()
                })
                .unwrap_or_default();
            let (r, g, b, a) = rgba(name);
            assert_eq!(
                vec![u64::from(r), u64::from(g), u64::from(b)],
                channels,
                "{name}"
            );
            assert_eq!(a, 1.0, "{name}");
        }
        Ok(())
    }

    #[test]
    fn test_names_are_case_insensitive_with_both_gray_spellings() {
        assert_eq!(rgba("Red"), (255, 0, 0, 1.0));
        assert_eq!(rgba("  REBECCAPURPLE "), (102, 51, 153, 1.0));
        assert_eq!(rgba("gray"), rgba("grey"));
        assert_eq!(rgba("DarkSlateGray"), rgba("darkslategrey"));
        assert_eq!(rgba("transparent"), (0, 0, 0, 0.0));
    }

    #[test]
    fn test_hex_forms() {
        assert_eq!(rgba("#f80"), (255, 136, 0, 1.0));
        assert_eq!(rgba("#FF8800"), (255, 136, 0, 1.0));
        assert_eq!(rgba("#f808").3, f64::from(0x88_u8) / 255.0);
        assert_eq!(rgba("#ff880080").3, f64::from(0x80_u8) / 255.0);
    }

    #[test]
    fn test_rgb_forms_are_equivalent() {
        let red = (255, 0, 0, 1.0);
        for text in [
            "rgb(255, 0, 0)",
            "RGB(255,0,0)",
            "rgba(255, 0, 0, 1)",
            "rgb(100%, 0%, 0%)",
            "rgb(255 0 0)",
            "rgb(255 0 0 / 100%)",
            "rgb( 255 , 0 , 0 )",
            "rgb(300, -5, 0)",
        ] {
            assert_eq!(rgba(text), red, "{text}");
        }
        assert_eq!(rgba("rgba(0, 128, 255, 0.5)"), (0, 128, 255, 0.5));
        assert_eq!(rgba("rgb(0 128 255 / 25%)"), (0, 128, 255, 0.25));
        assert_eq!(rgba("rgb(12.6, 0, 0)").0, 13);
    }

    #[test]
    fn test_hsl_forms() {
        assert_eq!(rgba("hsl(0, 100%, 50%)"), (255, 0, 0, 1.0));
        assert_eq!(rgba("hsl(120deg 100% 25%)"), (0, 128, 0, 1.0));
        assert_eq!(rgba("hsla(240, 100%, 50%, 0.5)"), (0, 0, 255, 0.5));
        assert_eq!(rgba("hsl(-120, 100, 50)"), (0, 0, 255, 1.0));
        assert_eq!(rgba("hsl(270, 50%, 40%)"), rgba("rebeccapurple"));
    }

    #[test]
    fn test_unsupported_and_malformed_colours_fail() {
        for text in [
            "",
            "#",
            "#12",
            "#12345",
            "#ggg",
            "rgb(1, 2)",
            "rgb(1, 2, 3, 4, 5)",
            "rgb(a, b, c)",
            "rgb(1, 2, 3 / 0.5)",
            "rgb(1 2 3",
            "rgb(NaN, 0, 0)",
            "rgb(inf, 0, 0)",
            "hwb(0 0% 0%)",
            "color(srgb 1 0 0)",
            "currentColor",
            "inherit",
            "Canvas",
            "reddish",
        ] {
            assert!(
                matches!(parse_color(text), Err(ExportError::InvalidColor { .. })),
                "{text:?} should be rejected"
            );
        }
    }
}
