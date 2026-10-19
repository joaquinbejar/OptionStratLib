//! Colour strings for the native export (#925, condition 3 of #920).
//!
//! `ColorScheme::Custom` documents "hex code, RGB, color name", and the
//! Plotly path hands every colour string to plotly.js. The native renderer
//! accepts the CSS Color 4 forms listed below, which cover the documented
//! ones; strings that only plotly.js's laxer parser takes (such as
//! `hsl(0, 100, 50)`) become a typed error instead. That is a behaviour
//! change of the migration, for #927 and #930 to document:
//!
//! * the 148 CSS Color 4 named colours, case-insensitive, `gray`/`grey`
//!   spellings included, and `transparent` (black with alpha 0);
//! * `#rgb`, `#rgba`, `#rrggbb` and `#rrggbbaa`;
//! * `rgb()` and `rgba()` (the same function) and `hsl()` and `hsla()`, in
//!   CSS Color 4's two syntaxes:
//!   * legacy: `rgb(r, g, b[, a])`, the three channels all numbers (0 to
//!     255) or all percentages; `hsl(h, s%, l%[, a])`;
//!   * modern: `rgb(r g b[ / a])`, each channel a number or a percentage;
//!     `hsl(h s l[ / a])`, saturation and lightness numbers or percentages;
//!
//!   the hue is a number of degrees, optionally with `deg`; the alpha a
//!   number from 0 to 1 or a percentage.
//!
//! Numbers follow CSS's own token grammar: an optional sign, digits with an
//! optional fraction (digits must follow a point), an optional exponent; a
//! percentage is a number immediately followed by `%`. A percentage channel
//! is `p * 255 / 100`, so `50%` and `127.5` round alike.
//!
//! Whitespace around the string and between components is ignored, and
//! names are case-insensitive; a function name must touch its parenthesis.
//! Out-of-range channels are clamped as CSS clamps them. Anything else is
//! [`GraphError::InvalidColor`], never a silent black: context-dependent
//! keywords (`currentcolor`, `inherit`, `initial`, `unset`, `revert`),
//! system colours, other functions such as `hwb()` or `color()`, hue units
//! other than `deg`, and malformed numbers or component lists. plotly.js's
//! own parser is more lenient than CSS (it takes `hsl(0, 100, 50)`); such
//! strings are errors here, reported rather than guessed.

use crate::error::GraphError;
use num_traits::ToPrimitive;
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
fn invalid(color: &str, reason: &'static str) -> GraphError {
    GraphError::InvalidColor {
        color: color.to_string(),
        reason: reason.to_string(),
    }
}

/// Parses a colour string as the module documentation describes.
///
/// # Errors
///
/// [`GraphError::InvalidColor`] for anything outside those forms.
pub(crate) fn parse_color(text: &str) -> Result<RGBAColor, GraphError> {
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
        return match name {
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

fn parse_hex(text: &str, digits: &str) -> Result<RGBAColor, GraphError> {
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err(invalid(text, "hex colours use the digits 0-9 and a-f"));
    }
    let value = |range: std::ops::RangeInclusive<usize>| {
        digits
            .get(range)
            .and_then(|part| u8::from_str_radix(part, 16).ok())
            .ok_or_else(|| invalid(text, "malformed hex colour"))
    };
    // `#rgb` doubles each digit: `#f80` is `#ff8800`; a digit is at most 15.
    let short = |index: usize| {
        value(index..=index)?
            .checked_mul(17)
            .ok_or_else(|| invalid(text, "malformed hex colour"))
    };
    let alpha = |byte: u8| f64::from(byte) / 255.0;
    match digits.len() {
        3 => Ok(RGBAColor(short(0)?, short(1)?, short(2)?, 1.0)),
        4 => Ok(RGBAColor(short(0)?, short(1)?, short(2)?, alpha(short(3)?))),
        6 => Ok(RGBAColor(value(0..=1)?, value(2..=3)?, value(4..=5)?, 1.0)),
        8 => Ok(RGBAColor(
            value(0..=1)?,
            value(2..=3)?,
            value(4..=5)?,
            alpha(value(6..=7)?),
        )),
        _ => Err(invalid(text, "hex colours have 3, 4, 6 or 8 digits")),
    }
}

/// Which CSS Color 4 syntax a colour function uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Syntax {
    /// `f(a, b, c[, alpha])`.
    Legacy,
    /// `f(a b c[ / alpha])`.
    Modern,
}

/// The three channels of a colour function, its optional alpha and its
/// syntax. Each syntax is checked on its own terms: the legacy one takes
/// three or four comma-separated components, the modern one exactly three
/// space-separated channels and an alpha only after a slash.
fn components<'a>(
    text: &str,
    arguments: &'a str,
) -> Result<(Syntax, [&'a str; 3], Option<&'a str>), GraphError> {
    let malformed = || {
        invalid(
            text,
            "expected `r, g, b[, a]` or `r g b[ / a]`: three channels and an optional alpha",
        )
    };
    if arguments.contains(',') {
        if arguments.contains('/') {
            return Err(invalid(text, "mixes commas and a slash"));
        }
        let parts: Vec<&str> = arguments.split(',').map(str::trim).collect();
        return match parts.as_slice() {
            [a, b, c] => Ok((Syntax::Legacy, [a, b, c], None)),
            [a, b, c, alpha] => Ok((Syntax::Legacy, [a, b, c], Some(alpha))),
            _ => Err(malformed()),
        };
    }
    let (channels, alpha) = match arguments.split_once('/') {
        Some((channels, alpha)) => {
            let alpha = alpha.trim();
            if alpha.is_empty() || alpha.contains(char::is_whitespace) || alpha.contains('/') {
                return Err(malformed());
            }
            (channels, Some(alpha))
        }
        None => (arguments, None),
    };
    let channels: Vec<&str> = channels.split_whitespace().collect();
    match channels.as_slice() {
        [a, b, c] => Ok((Syntax::Modern, [a, b, c], alpha)),
        _ => Err(malformed()),
    }
}

/// A CSS `<number>` token: an optional sign, then digits with an optional
/// fraction, or a point and digits, then an optional exponent.
fn css_number(token: &str) -> Option<f64> {
    let mut chars = token.chars().peekable();
    let digits = |chars: &mut std::iter::Peekable<std::str::Chars<'_>>| {
        std::iter::from_fn(|| chars.next_if(char::is_ascii_digit)).count()
    };
    chars.next_if(|c| matches!(c, '+' | '-'));
    let integer = digits(&mut chars);
    let fraction = match chars.next_if_eq(&'.') {
        // Digits must follow a point.
        Some(_) => match digits(&mut chars) {
            0 => return None,
            count => count,
        },
        None => 0,
    };
    if integer == 0 && fraction == 0 {
        return None;
    }
    if chars.next_if(|c| matches!(c, 'e' | 'E')).is_some() {
        chars.next_if(|c| matches!(c, '+' | '-'));
        if digits(&mut chars) == 0 {
            return None;
        }
    }
    if chars.next().is_some() {
        return None;
    }
    token.parse::<f64>().ok().filter(|value| value.is_finite())
}

/// A component's value: a CSS number or percentage.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Value {
    Number(f64),
    Percentage(f64),
}

fn value(text: &str, token: &str) -> Result<Value, GraphError> {
    let parsed = match token.strip_suffix('%') {
        Some(number) => css_number(number).map(Value::Percentage),
        None => css_number(token).map(Value::Number),
    };
    parsed.ok_or_else(|| invalid(text, "a component is not a CSS number or percentage"))
}

/// A channel clamped to 0..=255 and rounded, as CSS does.
fn channel(value: f64) -> u8 {
    // Clamped to 0..=255 first, so the conversion is exact; NaN is 0, as
    // `as u8` makes it.
    value.clamp(0.0, 255.0).round().to_u8().unwrap_or_default()
}

/// An rgb() channel in 0..=255: a number as is, a percentage of 255.
fn rgb_channel(value: Value) -> u8 {
    match value {
        Value::Number(number) => channel(number),
        // Multiplying first keeps `50%` exactly 127.5, as the number is.
        Value::Percentage(percent) => channel(percent * 255.0 / 100.0),
    }
}

fn alpha(text: &str, token: Option<&str>) -> Result<f64, GraphError> {
    let Some(token) = token else {
        return Ok(1.0);
    };
    Ok(match value(text, token)? {
        Value::Number(number) => number.clamp(0.0, 1.0),
        Value::Percentage(percent) => (percent / 100.0).clamp(0.0, 1.0),
    })
}

fn parse_rgb(text: &str, arguments: &str) -> Result<RGBAColor, GraphError> {
    let (syntax, [r, g, b], a) = components(text, arguments)?;
    let [r, g, b] = [value(text, r)?, value(text, g)?, value(text, b)?];
    let percentages = [r, g, b]
        .iter()
        .filter(|v| matches!(v, Value::Percentage(_)))
        .count();
    if syntax == Syntax::Legacy && percentages != 0 && percentages != 3 {
        return Err(invalid(
            text,
            "legacy rgb() takes three numbers or three percentages, not a mix",
        ));
    }
    Ok(RGBAColor(
        rgb_channel(r),
        rgb_channel(g),
        rgb_channel(b),
        alpha(text, a)?,
    ))
}

fn parse_hsl(text: &str, arguments: &str) -> Result<RGBAColor, GraphError> {
    let (syntax, [h, s, l], a) = components(text, arguments)?;
    let hue = match value(text, h.strip_suffix("deg").unwrap_or(h))? {
        Value::Number(degrees) => degrees.rem_euclid(360.0),
        Value::Percentage(_) => return Err(invalid(text, "a hue is a number of degrees")),
    };
    // Legacy syntax wants percentages; the modern one also takes numbers,
    // read as percentages.
    let percent = |token: &str| -> Result<f64, GraphError> {
        match (value(text, token)?, syntax) {
            (Value::Percentage(percent), _) | (Value::Number(percent), Syntax::Modern) => {
                Ok(percent.clamp(0.0, 100.0) / 100.0)
            }
            (Value::Number(_), Syntax::Legacy) => Err(invalid(
                text,
                "legacy hsl() takes saturation and lightness as percentages",
            )),
        }
    };
    let (saturation, lightness) = (percent(s)?, percent(l)?);
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
        // Generated from Pillow's CSS colour table, not from `NAMED`. It sits
        // beside this module rather than under `tests/`, which the published
        // crate leaves out, so the packaged crate's tests have it too (#929).
        let reference: serde_json::Value =
            serde_json::from_str(include_str!("css-named-colors.json"))?;
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
    fn test_percentages_round_like_the_equivalent_numbers() {
        assert_eq!(rgba("rgb(50%, 50%, 50%)"), rgba("rgb(127.5, 127.5, 127.5)"));
        assert_eq!(rgba("rgb(50%, 50%, 50%)").0, 128);
        assert_eq!(rgba("rgb(90%, 0%, 0%)").0, 230);
        assert_eq!(rgba("rgb(10% 30% 90%)"), rgba("rgb(25.5 76.5 229.5)"));
        // Modern syntax may mix numbers and percentages; CSS number tokens.
        assert_eq!(rgba("rgb(100% 0 0)"), (255, 0, 0, 1.0));
        assert_eq!(rgba("rgb(1e2 +.5 1E+1)"), (100, 1, 10, 1.0));
    }

    #[test]
    fn test_hsl_forms() {
        assert_eq!(rgba("hsl(0, 100%, 50%)"), (255, 0, 0, 1.0));
        assert_eq!(rgba("hsl(120deg 100% 25%)"), (0, 128, 0, 1.0));
        assert_eq!(rgba("hsla(240, 100%, 50%, 0.5)"), (0, 0, 255, 0.5));
        assert_eq!(rgba("hsl(-120 100 50)"), (0, 0, 255, 1.0));
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
            // Component lists (CSS Color 4, legacy and modern syntax).
            "rgb()",
            "rgb(255 0 0 0.5)",
            "rgb(255 0 / 50%)",
            "hsl(0 100% / 50%)",
            "rgb(255 0 0 / 0.5 1)",
            "rgb(255 0 0 /)",
            "rgb(100%, 0, 0)",
            "hsl(0, 100, 50)",
            "hsl(120%, 100%, 50%)",
            "hsl(0.5turn 100% 50%)",
            "rgb (0, 0, 0)",
            // Number tokens.
            "rgb(1., 0, 0)",
            "rgb(1.e2, 0, 0)",
            "rgb(50 %, 0%, 0%)",
            "rgb(1e, 0, 0)",
            "rgb(--1, 0, 0)",
            "rgb(0x10, 0, 0)",
            "rgb(1e309 0 0)",
            "currentColor",
            "inherit",
            "Canvas",
            "reddish",
        ] {
            assert!(
                matches!(parse_color(text), Err(GraphError::InvalidColor { .. })),
                "{text:?} should be rejected"
            );
        }
    }
}
