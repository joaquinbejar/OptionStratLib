//! The native exporter end to end: files written for every case, and every
//! documented error.

use optionstratlib_visualization::visualization::{
    GraphConfig, GraphData, Series2D, Surface3D, TraceMode,
};
use osl_prototype_plotters_export::fixtures::{Case, Fixture};
use osl_prototype_plotters_export::{ExportError, SvgText, render_svg, write_png, write_svg};
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
