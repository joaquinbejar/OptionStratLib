//! The native exporter end to end: files written for every case, and every
//! documented error.

use optionstratlib_visualization::visualization::{
    GraphConfig, GraphData, Series2D, Surface3D, TraceMode,
};
use osl_prototype_plotters_export::fixtures::{Case, Fixture, TEXT_SERIES, TEXT_TITLE};
use osl_prototype_plotters_export::{
    ExportError, Label, SvgText, render_svg, render_svg_outlined, write_png, write_svg,
};
use rust_decimal_macros::dec;
use std::error::Error;
use std::path::Path;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];

/// Width and height from a PNG's IHDR chunk.
fn png_dimensions(path: &Path) -> Result<(u32, u32), Box<dyn Error>> {
    let bytes = std::fs::read(path)?;
    if bytes.get(..8) != Some(&PNG_SIGNATURE[..]) {
        return Err("not a PNG".into());
    }
    let read = |range: std::ops::Range<usize>| -> Result<u32, Box<dyn Error>> {
        let slice = bytes.get(range).ok_or("truncated IHDR")?;
        Ok(u32::from_be_bytes(slice.try_into()?))
    };
    Ok((read(16..20)?, read(20..24)?))
}

fn series(name: &str, x: Vec<rust_decimal::Decimal>, y: Vec<rust_decimal::Decimal>) -> Series2D {
    Series2D {
        x,
        y,
        name: name.to_string(),
        mode: TraceMode::Lines,
        line_color: None,
        line_width: None,
    }
}

fn small_config() -> GraphConfig {
    GraphConfig {
        width: 320,
        height: 200,
        ..GraphConfig::default()
    }
}

#[test]
fn test_write_png_every_case_matches_the_configured_size() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    for case in Case::ALL {
        let Fixture { data, config } = case.build()?;
        let path = dir.path().join(format!("{}.png", case.slug()));
        write_png(&data, &config, &path)?;
        assert_eq!(
            png_dimensions(&path)?,
            (config.width, config.height),
            "{}",
            case.slug()
        );
    }
    Ok(())
}

#[test]
fn test_render_svg_every_case_embeds_the_font_on_request() -> Result<(), Box<dyn Error>> {
    for case in Case::ALL {
        let Fixture { data, config } = case.build()?;
        let embedded = render_svg(&data, &config, SvgText::EmbeddedFont)?;
        let plain = render_svg(&data, &config, SvgText::ViewerFont)?;
        let size = format!("width=\"{}\" height=\"{}\"", config.width, config.height);
        for document in [&embedded, &plain] {
            assert!(document.starts_with("<svg"), "{}", case.slug());
            assert!(document.contains(&size), "{}", case.slug());
            assert!(document.trim_end().ends_with("</svg>"), "{}", case.slug());
            assert!(
                document.contains("'Open Sans', sans-serif"),
                "{}",
                case.slug()
            );
        }
        assert!(embedded.contains("@font-face"), "{}", case.slug());
        assert!(!plain.contains("@font-face"), "{}", case.slug());
    }
    Ok(())
}

#[test]
fn test_write_svg_creates_missing_directories() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let Fixture { data, config } = Case::Curves.build()?;
    let path = dir.path().join("nested/deeper/curves.svg");
    write_svg(&data, &config, &path, SvgText::ViewerFont)?;
    assert!(std::fs::read_to_string(&path)?.starts_with("<svg"));
    Ok(())
}

#[test]
fn test_write_png_under_a_file_is_an_io_error() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    let blocker = dir.path().join("blocker");
    std::fs::write(&blocker, b"not a directory")?;
    let Fixture { data, config } = Case::Curves.build()?;
    let result = write_png(&data, &config, &blocker.join("chart.png"));
    assert!(matches!(result, Err(ExportError::Io(_))), "{result:?}");
    Ok(())
}

#[test]
fn test_zero_dimensions_are_rejected() {
    let data = GraphData::Series(series("s", vec![dec!(1)], vec![dec!(1)]));
    let config = GraphConfig {
        width: 0,
        ..small_config()
    };
    let result = render_svg(&data, &config, SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::InvalidDimensions {
            width: 0,
            height: 200
        })
    ));
}

#[test]
fn test_empty_series_renders_empty_axes() -> Result<(), Box<dyn Error>> {
    let data = GraphData::Series(Series2D::default());
    let svg = render_svg(&data, &small_config(), SvgText::ViewerFont)?;
    assert!(svg.starts_with("<svg"));
    Ok(())
}

#[test]
fn test_constant_series_renders() -> Result<(), Box<dyn Error>> {
    let data = GraphData::Series(series(
        "flat",
        vec![dec!(1), dec!(2), dec!(3)],
        vec![dec!(5), dec!(5), dec!(5)],
    ));
    let svg = render_svg(&data, &small_config(), SvgText::ViewerFont)?;
    assert!(svg.contains("<polyline"));
    Ok(())
}

#[test]
fn test_mismatched_series_lengths_are_rejected() {
    let data = GraphData::MultiSeries(vec![
        series("ok", vec![dec!(1), dec!(2)], vec![dec!(1), dec!(2)]),
        series("short", vec![dec!(1), dec!(2)], vec![dec!(1)]),
    ]);
    let result = render_svg(&data, &small_config(), SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::LengthMismatch { ref name, x: 2, y: 1 }) if name == "short"
    ));
}

#[test]
fn test_named_colours_are_rejected() {
    let mut red = series("red", vec![dec!(1)], vec![dec!(1)]);
    red.line_color = Some("red".to_string());
    let result = render_svg(
        &GraphData::Series(red),
        &small_config(),
        SvgText::ViewerFont,
    );
    assert!(matches!(result, Err(ExportError::InvalidColor(ref c)) if c == "red"));
}

fn surface(
    x: Vec<rust_decimal::Decimal>,
    y: Vec<rust_decimal::Decimal>,
    z: Vec<rust_decimal::Decimal>,
) -> GraphData {
    GraphData::GraphSurface(Surface3D {
        x,
        y,
        z,
        name: "iv".to_string(),
    })
}

#[test]
fn test_incomplete_surface_is_rejected_not_zero_filled() {
    let data = surface(
        vec![dec!(1), dec!(2), dec!(1)],
        vec![dec!(1), dec!(1), dec!(2)],
        vec![dec!(0.2), dec!(0.3), dec!(0.4)],
    );
    let result = render_svg(&data, &small_config(), SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::IncompleteSurface {
            expected: 4,
            found: 3,
            ..
        })
    ));
}

#[test]
fn test_single_column_surface_is_rejected() {
    let data = surface(
        vec![dec!(1), dec!(1)],
        vec![dec!(1), dec!(2)],
        vec![dec!(0.2), dec!(0.3)],
    );
    let result = render_svg(&data, &small_config(), SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::DegenerateSurface {
            columns: 1,
            rows: 2,
            ..
        })
    ));
}

#[test]
fn test_surface_length_mismatch_is_rejected() {
    let data = surface(vec![dec!(1), dec!(2)], vec![dec!(1)], vec![dec!(0.2)]);
    let result = render_svg(&data, &small_config(), SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::SurfaceLengthMismatch {
            x: 2,
            y: 1,
            z: 1,
            ..
        })
    ));
}

#[test]
fn test_empty_surface_is_rejected() {
    let data = surface(vec![], vec![], vec![]);
    let result = render_svg(&data, &small_config(), SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::DegenerateSurface {
            columns: 0,
            rows: 0,
            ..
        })
    ));
}

#[test]
fn test_small_figures_never_panic() -> Result<(), Box<dyn Error>> {
    const SIZES: [u32; 9] = [1, 2, 5, 20, 40, 120, 159, 239, 240];
    for case in Case::ALL {
        let fixture = case.build()?;
        for width in SIZES {
            for height in SIZES {
                let config = GraphConfig {
                    width,
                    height,
                    ..fixture.config.clone()
                };
                let outcome = std::panic::catch_unwind(|| {
                    render_svg(&fixture.data, &config, SvgText::ViewerFont)
                });
                let Ok(result) = outcome else {
                    return Err(format!("{} panicked at {width}x{height}", case.slug()).into());
                };
                match result {
                    Ok(_) | Err(ExportError::TooSmall { .. } | ExportError::NoRoom { .. }) => {}
                    Err(error) => {
                        return Err(format!("{} at {width}x{height}: {error}", case.slug()).into());
                    }
                }
            }
        }
    }
    Ok(())
}

#[test]
fn test_surface_below_its_minimum_size_is_rejected() -> Result<(), Box<dyn Error>> {
    let Fixture { data, config } = Case::Surface.build()?;
    let config = GraphConfig {
        width: 20,
        height: 20,
        ..config
    };
    let result = render_svg(&data, &config, SvgText::ViewerFont);
    assert!(matches!(
        result,
        Err(ExportError::TooSmall {
            width: 20,
            height: 20,
            min_width: 240,
            min_height: 160
        })
    ));
    Ok(())
}

#[test]
fn test_non_finite_or_huge_line_widths_are_rejected() -> Result<(), Box<dyn Error>> {
    let dir = tempfile::tempdir()?;
    for width in [f64::INFINITY, f64::NAN, -1.0, 1e9] {
        let mut line = series("wide", vec![dec!(0), dec!(1)], vec![dec!(0), dec!(1)]);
        line.line_width = Some(width);
        let config = GraphConfig {
            width: 640,
            height: 480,
            ..GraphConfig::default()
        };
        let result = write_png(&GraphData::Series(line), &config, &dir.path().join("w.png"));
        assert!(
            matches!(result, Err(ExportError::InvalidLineWidth { .. })),
            "{width}: {result:?}"
        );
    }
    Ok(())
}

#[test]
fn test_wide_but_flat_surface_figure_is_rejected() {
    let data = surface(
        vec![dec!(1), dec!(2), dec!(1), dec!(2)],
        vec![dec!(1), dec!(1), dec!(2), dec!(2)],
        vec![dec!(0.2), dec!(0.3), dec!(0.4), dec!(0.5)],
    );
    let config = GraphConfig {
        width: 640,
        height: 20,
        ..GraphConfig::default()
    };
    let result = render_svg(&data, &config, SvgText::ViewerFont);
    assert!(
        matches!(result, Err(ExportError::TooSmall { .. })),
        "{result:?}"
    );
}

#[test]
fn test_title_taller_than_the_figure_is_no_room() {
    let data = GraphData::Series(series("s", vec![dec!(0), dec!(1)], vec![dec!(0), dec!(1)]));
    let config = GraphConfig {
        width: 640,
        height: 10,
        ..GraphConfig::default()
    };
    let result = render_svg(&data, &config, SvgText::ViewerFont);
    assert!(
        matches!(result, Err(ExportError::NoRoom { .. })),
        "{result:?}"
    );
}

/// The numeric text elements of an SVG.
fn numeric_labels(svg: &str) -> Vec<f64> {
    svg.split("<text")
        .skip(1)
        .filter_map(|element| {
            let content = element.split_once('>')?.1.split_once("</text>")?.0;
            content.trim().replace('\u{2212}', "-").parse::<f64>().ok()
        })
        .collect()
}

#[test]
fn test_small_values_keep_distinct_tick_labels() -> Result<(), Box<dyn Error>> {
    let data = GraphData::Series(series(
        "tiny",
        vec![dec!(0), dec!(1)],
        vec![dec!(0), dec!(0.00001)],
    ));
    let svg = render_svg(&data, &small_config(), SvgText::ViewerFont)?;
    let small: std::collections::BTreeSet<String> = numeric_labels(&svg)
        .into_iter()
        .filter(|value| *value > 0.0 && *value <= 1.1e-5)
        .map(|value| format!("{value:e}"))
        .collect();
    assert!(small.len() >= 3, "{small:?}");
    Ok(())
}

#[test]
fn test_tiny_nonconstant_series_keeps_its_scale() -> Result<(), Box<dyn Error>> {
    let data = GraphData::Series(series(
        "tiny",
        vec![dec!(0), dec!(1)],
        vec![dec!(0), dec!(0.000000000000000001)],
    ));
    let svg = render_svg(&data, &small_config(), SvgText::ViewerFont)?;
    let labels = numeric_labels(&svg);
    assert!(
        labels.iter().any(|value| *value > 0.0 && *value < 1e-17),
        "{labels:?}"
    );
    assert!(!labels.iter().any(|value| *value <= -0.5), "{labels:?}");
    Ok(())
}

// Text as outlines (#923).

/// The document with every `<text>` element of plotters-svg removed.
fn without_text_elements(svg: &str) -> String {
    let mut kept = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some((before, element)) = rest.split_once("<text ") {
        kept.push_str(before);
        rest = element.split_once("</text>").map_or("", |(_, after)| after);
    }
    kept.push_str(rest);
    kept
}

/// The document with every outlined label and the title/desc removed.
fn without_outlined_labels(svg: &str) -> String {
    let mut kept = String::with_capacity(svg.len());
    let mut rest = svg;
    while let Some((before, group)) = rest.split_once("<g role=\"img\"") {
        kept.push_str(before);
        rest = group.split_once("</g>").map_or("", |(_, after)| after);
    }
    kept.push_str(rest);
    match kept.split_once("<title>") {
        Some((head, tail)) => {
            let tail = tail.split_once("</desc>").map_or("", |(_, after)| after);
            format!("{head}{tail}")
        }
        None => kept,
    }
}

fn label_texts(labels: &[Label]) -> Vec<&str> {
    labels.iter().map(|label| label.text.as_str()).collect()
}

#[test]
fn test_outlines_every_case_paints_text_without_any_font() -> Result<(), Box<dyn Error>> {
    for case in Case::EVERY {
        let Fixture { data, config } = case.build()?;
        let outlined = render_svg_outlined(&data, &config)?;
        let document = &outlined.document;
        assert_eq!(
            *document,
            render_svg(&data, &config, SvgText::Outlines)?,
            "{}",
            case.slug()
        );
        for absent in [
            "<text",
            "font-family",
            "font-size",
            "@font-face",
            "<style",
            "<image",
            "href",
            "data:",
            "url(",
        ] {
            assert!(!document.contains(absent), "{} has {absent}", case.slug());
        }
        let groups = document.matches("<g role=\"img\" aria-label=").count();
        assert_eq!(groups, outlined.labels.len(), "{}", case.slug());
        assert!(!outlined.labels.is_empty(), "{}", case.slug());
        let (width, height) = (f64::from(config.width), f64::from(config.height));
        for label in &outlined.labels {
            assert!(
                label.left < label.right && label.top < label.bottom,
                "{}: {label:?}",
                case.slug()
            );
            assert!(
                label.left >= 0.0
                    && label.top >= 0.0
                    && label.right <= width
                    && label.bottom <= height,
                "{} paints {label:?} outside {width}x{height}",
                case.slug()
            );
        }
        let title = config.title.replace(['\n', '\t'], " ");
        assert!(
            label_texts(&outlined.labels).contains(&title.as_str()),
            "{}",
            case.slug()
        );
        assert!(document.contains("<title>"), "{}", case.slug());
        assert!(document.contains("<desc>"), "{}", case.slug());
    }
    Ok(())
}

#[test]
fn test_outlines_leave_every_non_text_element_unchanged() -> Result<(), Box<dyn Error>> {
    for case in Case::EVERY {
        let Fixture { data, config } = case.build()?;
        let plain = render_svg(&data, &config, SvgText::ViewerFont)?;
        let outlined = render_svg(&data, &config, SvgText::Outlines)?;
        assert_eq!(
            without_outlined_labels(&outlined),
            without_text_elements(&plain),
            "{}",
            case.slug()
        );
    }
    Ok(())
}

#[test]
fn test_outlines_keep_every_label_of_the_text_fixture() -> Result<(), Box<dyn Error>> {
    let Fixture { data, config } = Case::Text.build()?;
    let outlined = render_svg_outlined(&data, &config)?;
    let texts = label_texts(&outlined.labels);
    for expected in TEXT_SERIES.into_iter().chain([
        TEXT_TITLE,
        "Strike − spot ($)",
        "Δ per $1 (Γ = 1.5e−7)",
        "Aa",
    ]) {
        assert!(
            texts.contains(&expected),
            "{expected:?} missing from {texts:?}"
        );
    }
    // The text-label series: two point labels plus its legend entry.
    let escaped = TEXT_SERIES[3];
    assert_eq!(texts.iter().filter(|text| **text == escaped).count(), 3);
    assert!(
        outlined
            .document
            .contains("aria-label=\"&amp; &lt; &gt; &quot; &apos; $\"")
    );
    // Scientific ticks with a true minus.
    assert!(
        texts
            .iter()
            .any(|text| text.starts_with('\u{2212}') && text.contains('e'))
    );
    // Padding spaces stay in the label and in its advance.
    assert!(texts.contains(&"  AV To fi  "));
    Ok(())
}

#[test]
fn test_outlines_rotate_the_y_axis_title() -> Result<(), Box<dyn Error>> {
    let Fixture { data, config } = Case::Text.build()?;
    let outlined = render_svg_outlined(&data, &config)?;
    let title = outlined
        .labels
        .iter()
        .find(|label| label.text.starts_with('Δ'))
        .ok_or("no y axis title")?;
    assert!(
        title.bottom - title.top > 3.0 * (title.right - title.left),
        "{title:?}"
    );
    assert!(
        outlined
            .document
            .contains("aria-label=\"Δ per $1 (Γ = 1.5e−7)\" fill=\"#2A3F5F\" opacity=\"1\" transform=\"rotate(270, ")
    );
    Ok(())
}

#[test]
fn test_outlines_labels_never_overlap() -> Result<(), Box<dyn Error>> {
    for case in Case::EVERY {
        let Fixture { data, config } = case.build()?;
        let labels = render_svg_outlined(&data, &config)?.labels;
        for (index, a) in labels.iter().enumerate() {
            for b in labels.iter().skip(index + 1) {
                let apart = a.right <= b.left
                    || b.right <= a.left
                    || a.bottom <= b.top
                    || b.bottom <= a.top;
                assert!(apart, "{}: {a:?} overlaps {b:?}", case.slug());
            }
        }
    }
    Ok(())
}

#[test]
fn test_outlines_fit_the_reserved_legend_column() -> Result<(), Box<dyn Error>> {
    for case in [Case::Curves, Case::Text] {
        let Fixture { data, config } = case.build()?;
        let GraphData::MultiSeries(list) = &data else {
            return Err("a multi-series fixture".into());
        };
        let outlined = render_svg_outlined(&data, &config)?;
        for series in list {
            let entry = outlined
                .labels
                .iter()
                .rev()
                .find(|label| label.text == series.name)
                .ok_or("legend entry missing")?;
            assert!(
                entry.right <= f64::from(config.width),
                "{}: {entry:?}",
                case.slug()
            );
        }
    }
    Ok(())
}

#[test]
fn test_outlines_reject_a_character_the_font_lacks() -> Result<(), Box<dyn Error>> {
    let Fixture { data, mut config } = Case::Curves.build()?;
    config.title = "IV 中".to_string();
    let result = render_svg(&data, &config, SvgText::Outlines);
    assert!(
        matches!(
            result,
            Err(ExportError::MissingGlyph { character: '中', code: 0x4E2D, ref text }) if text == "IV 中"
        ),
        "{result:?}"
    );
    // The other modes leave the character to the viewer.
    assert!(render_svg(&data, &config, SvgText::ViewerFont)?.contains("IV 中"));
    Ok(())
}

#[test]
fn test_outlines_reject_a_hidden_name_xml_cannot_carry() -> Result<(), Box<dyn Error>> {
    // The name is never drawn (no legend), but it enters <desc>.
    let Fixture {
        mut data,
        mut config,
    } = Case::Curves.build()?;
    let GraphData::MultiSeries(list) = &mut data else {
        return Err("a multi-series fixture".into());
    };
    let first = list.first_mut().ok_or("no series")?;
    first.name = "hidden\0series".to_string();
    config.show_legend = false;
    config.legend = None;
    let result = render_svg_outlined(&data, &config);
    assert!(
        matches!(
            result,
            Err(ExportError::NotXmlText {
                character: '\0',
                code: 0,
                ..
            })
        ),
        "{result:?}"
    );
    Ok(())
}

#[test]
fn test_outlines_every_case_is_xml_text() -> Result<(), Box<dyn Error>> {
    for case in Case::EVERY {
        let Fixture { data, config } = case.build()?;
        let document = render_svg(&data, &config, SvgText::Outlines)?;
        let forbidden = |c: char| matches!(c, '\0'..='\u{8}' | '\u{B}' | '\u{C}' | '\u{E}'..='\u{1F}' | '\u{FFFE}' | '\u{FFFF}');
        assert!(!document.chars().any(forbidden), "{}", case.slug());
    }
    Ok(())
}

#[test]
fn test_outlines_small_figures_never_panic() -> Result<(), Box<dyn Error>> {
    const SIZES: [u32; 6] = [1, 5, 40, 120, 239, 240];
    for case in Case::EVERY {
        let fixture = case.build()?;
        for width in SIZES {
            for height in SIZES {
                let config = GraphConfig {
                    width,
                    height,
                    ..fixture.config.clone()
                };
                let outcome = std::panic::catch_unwind(|| {
                    render_svg(&fixture.data, &config, SvgText::Outlines)
                });
                let Ok(result) = outcome else {
                    return Err(format!("{} panicked at {width}x{height}", case.slug()).into());
                };
                match result {
                    Ok(_) | Err(ExportError::TooSmall { .. } | ExportError::NoRoom { .. }) => {}
                    Err(error) => {
                        return Err(format!("{} at {width}x{height}: {error}", case.slug()).into());
                    }
                }
            }
        }
    }
    Ok(())
}

/// The numeric labels of an outlines SVG, read from their accessible names.
fn numeric_outline_labels(labels: &[Label]) -> Vec<f64> {
    labels
        .iter()
        .filter_map(|label| {
            label
                .text
                .trim()
                .replace('\u{2212}', "-")
                .parse::<f64>()
                .ok()
        })
        .collect()
}

#[test]
fn test_outlines_keep_distinct_small_tick_labels() -> Result<(), Box<dyn Error>> {
    let data = GraphData::Series(series(
        "tiny",
        vec![dec!(0), dec!(1)],
        vec![dec!(0), dec!(0.00001)],
    ));
    let outlined = render_svg_outlined(&data, &small_config())?;
    let small: std::collections::BTreeSet<String> = numeric_outline_labels(&outlined.labels)
        .into_iter()
        .filter(|value| *value > 0.0 && *value <= 1.1e-5)
        .map(|value| format!("{value:e}"))
        .collect();
    assert!(small.len() >= 3, "{small:?}");
    // Distinct values are also painted apart.
    let ticks: Vec<&Label> = outlined
        .labels
        .iter()
        .filter(|label| label.text.contains("e-"))
        .collect();
    for pair in ticks.windows(2) {
        let [above, below] = pair else { continue };
        assert!(
            above.bottom <= below.top || below.bottom <= above.top,
            "{pair:?}"
        );
    }
    Ok(())
}

#[test]
fn test_outlines_keep_a_tiny_series_scale() -> Result<(), Box<dyn Error>> {
    let data = GraphData::Series(series(
        "tiny",
        vec![dec!(0), dec!(1)],
        vec![dec!(0), dec!(0.000000000000000001)],
    ));
    let outlined = render_svg_outlined(&data, &small_config())?;
    let labels = numeric_outline_labels(&outlined.labels);
    assert!(
        labels.iter().any(|value| *value > 0.0 && *value < 1e-17),
        "{labels:?}"
    );
    assert!(!labels.iter().any(|value| *value <= -0.5), "{labels:?}");
    Ok(())
}
