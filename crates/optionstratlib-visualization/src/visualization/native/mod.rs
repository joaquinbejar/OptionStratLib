//! The native PNG and SVG export behind `static_export` (#927): charts
//! drawn with Plotters from [`GraphData`] and [`GraphConfig`], with no
//! browser, driver, network or installed font.
//!
//! * [`render`] lays the chart out and draws it, mirroring the Plotly
//!   defaults `Graph::to_plot` uses, and reports data it cannot draw
//!   faithfully as typed [`GraphError`](crate::error::GraphError) variants.
//! * [`text`] reads the bundled Open Sans (SIL OFL 1.1, `assets/OFL.txt`)
//!   with skrifa, and [`backend`] measures and draws every label with it, so
//!   Plotters needs no font feature and no `ttf-parser` is involved.
//! * [`outline`] turns the SVG's text into glyph outlines, so the file looks
//!   the same in every viewer, with each label kept as an accessible name.
//! * [`color`] parses the CSS Color 4 colour strings charts carry.
//!
//! The design and its evidence come from the prototype under
//! `prototypes/plotters-export` (#920, #923, #926).
//!
//! [`GraphData`]: crate::visualization::GraphData
//! [`GraphConfig`]: crate::visualization::GraphConfig

mod backend;
mod color;
mod outline;
mod render;
mod text;

#[cfg(test)]
pub(crate) use render::render_svg;
pub(crate) use render::{write_png, write_svg};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::GraphError;
    use crate::visualization::{GraphConfig, GraphData, Series2D, Surface3D, TraceMode};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::path::Path;

    type TestResult = Result<(), Box<dyn std::error::Error>>;

    fn series(name: &str, x: &[Decimal], y: &[Decimal], mode: TraceMode) -> Series2D {
        Series2D {
            x: x.to_vec(),
            y: y.to_vec(),
            name: name.to_string(),
            mode,
            line_color: None,
            line_width: None,
        }
    }

    fn multi() -> GraphData {
        let x = [dec!(-2), dec!(0), dec!(1), dec!(3)];
        GraphData::MultiSeries(vec![
            series(
                "Payoff",
                &x,
                &[dec!(-1), dec!(-1), dec!(0.5), dec!(2)],
                TraceMode::Lines,
            ),
            series(
                "Delta",
                &x,
                &[dec!(0), dec!(0.25), dec!(0.75), dec!(1)],
                TraceMode::LinesMarkers,
            ),
            series(
                "Points",
                &x,
                &[dec!(1), dec!(2), dec!(1), dec!(2)],
                TraceMode::Markers,
            ),
            series("BE: 0.5", &[dec!(0.5)], &[dec!(0)], TraceMode::TextLabels),
        ])
    }

    /// A complete 4 x 3 grid with a smile, as `make_surface` expects.
    fn surface() -> GraphData {
        let (mut x, mut y, mut z) = (Vec::new(), Vec::new(), Vec::new());
        for strike in [dec!(80), dec!(90), dec!(100), dec!(110)] {
            for days in [dec!(7), dec!(30), dec!(90)] {
                let smile = (strike - dec!(95)).abs() / dec!(100);
                x.push(strike);
                y.push(days);
                z.push(dec!(0.2) + smile - days / dec!(1000));
            }
        }
        GraphData::GraphSurface(Surface3D {
            x,
            y,
            z,
            name: "Implied volatility".to_string(),
        })
    }

    fn config() -> GraphConfig {
        GraphConfig {
            title: "Bull call spread: $5,820 − $5,750".to_string(),
            width: 800,
            height: 480,
            x_label: Some("Underlying price".to_string()),
            y_label: Some("Profit / loss".to_string()),
            z_label: Some("Volatility".to_string()),
            ..GraphConfig::default()
        }
    }

    /// Width and height from a PNG's IHDR chunk.
    fn png_size(path: &Path) -> Result<(u32, u32), Box<dyn std::error::Error>> {
        let bytes = std::fs::read(path)?;
        let read = |range: std::ops::Range<usize>| -> Result<u32, Box<dyn std::error::Error>> {
            let slice = bytes.get(range).ok_or("truncated PNG")?;
            Ok(u32::from_be_bytes(slice.try_into()?))
        };
        assert_eq!(bytes.get(..8), Some(&b"\x89PNG\r\n\x1a\n"[..]));
        Ok((read(16..20)?, read(20..24)?))
    }

    #[test]
    fn test_every_chart_kind_exports_png_and_outlined_svg() -> TestResult {
        let dir = tempfile::tempdir()?;
        let single = GraphData::Series(series(
            "Call",
            &[dec!(1), dec!(2), dec!(3)],
            &[dec!(0.1), dec!(0.4), dec!(0.2)],
            TraceMode::Lines,
        ));
        for (name, data) in [
            ("single", single),
            ("multi", multi()),
            ("surface", surface()),
        ] {
            let png = dir.path().join(format!("nested/{name}.png"));
            write_png(&data, &config(), &png)?;
            assert_eq!(png_size(&png)?, (800, 480), "{name}");
            let svg_path = dir.path().join(format!("{name}.svg"));
            write_svg(&data, &config(), &svg_path)?;
            let svg = std::fs::read_to_string(&svg_path)?;
            assert!(
                svg.starts_with("<svg width=\"800\" height=\"480\""),
                "{name}"
            );
            for absent in [
                "<text",
                "font-family",
                "@font-face",
                "<image",
                "href",
                "data:",
            ] {
                assert!(!svg.contains(absent), "{name} has {absent}");
            }
            assert!(svg.contains("<title>Bull call spread: $5,820 − $5,750</title>"));
            assert!(svg.contains("aria-label=\"Underlying price\""), "{name}");
        }
        Ok(())
    }

    #[test]
    fn test_surface_svg_names_its_axes_and_colour_scale() -> TestResult {
        let outlined = render_svg(&surface(), &config())?;
        let texts: Vec<&str> = outlined.labels.iter().map(|l| l.text.as_str()).collect();
        for expected in ["Underlying price", "Profit / loss", "Volatility"] {
            assert!(
                texts.contains(&expected),
                "{expected} missing from {texts:?}"
            );
        }
        assert!(
            outlined
                .document
                .contains("<desc>Surface chart of Implied volatility")
        );
        Ok(())
    }

    #[test]
    fn test_bad_data_is_a_typed_error_not_a_repair() -> TestResult {
        let scratch = tempfile::tempdir()?;
        let dir = scratch.path().join("never-written.png");
        let short = GraphData::Series(series(
            "short",
            &[dec!(1), dec!(2)],
            &[dec!(1)],
            TraceMode::Lines,
        ));
        assert!(matches!(
            write_png(&short, &config(), &dir),
            Err(GraphError::LengthMismatch { ref name, x: 2, y: 1 }) if name == "short"
        ));
        let incomplete = GraphData::GraphSurface(Surface3D {
            x: vec![dec!(1), dec!(2), dec!(1)],
            y: vec![dec!(1), dec!(1), dec!(2)],
            z: vec![dec!(0.1), dec!(0.2), dec!(0.3)],
            name: "gappy".to_string(),
        });
        assert!(matches!(
            write_svg(&incomplete, &config(), &dir),
            Err(GraphError::IncompleteSurface {
                expected: 4,
                found: 3,
                ..
            })
        ));
        let repeated = GraphData::GraphSurface(Surface3D {
            x: vec![dec!(1), dec!(1), dec!(2), dec!(2), dec!(1)],
            y: vec![dec!(1), dec!(2), dec!(1), dec!(2), dec!(1)],
            z: vec![dec!(0.1); 5],
            name: "twice".to_string(),
        });
        assert!(matches!(
            write_png(&repeated, &config(), &dir),
            Err(GraphError::DuplicateSurfacePoint { .. })
        ));
        let mut red = series(
            "red",
            &[dec!(1), dec!(2)],
            &[dec!(1), dec!(2)],
            TraceMode::Lines,
        );
        red.line_color = Some("reddish".to_string());
        assert!(matches!(
            write_png(&GraphData::Series(red), &config(), &dir),
            Err(GraphError::InvalidColor { ref color, .. }) if color == "reddish"
        ));
        let zero = GraphConfig {
            width: 0,
            ..config()
        };
        assert!(matches!(
            write_png(&multi(), &zero, &dir),
            Err(GraphError::InvalidDimensions {
                width: 0,
                height: 480
            })
        ));
        // Neither the image nor a partial file is left behind.
        assert_eq!(std::fs::read_dir(scratch.path())?.count(), 0);
        Ok(())
    }

    #[test]
    fn test_text_the_font_or_xml_cannot_carry_is_a_typed_error() -> TestResult {
        let dir = tempfile::tempdir()?;
        let cjk = GraphConfig {
            title: "IV 中".to_string(),
            ..config()
        };
        assert!(matches!(
            write_png(&multi(), &cjk, &dir.path().join("cjk.png")),
            Err(GraphError::MissingGlyph {
                character: '中',
                ..
            })
        ));
        assert!(matches!(
            write_svg(&multi(), &cjk, &dir.path().join("cjk.svg")),
            Err(GraphError::MissingGlyph {
                character: '中',
                ..
            })
        ));
        // A NUL in a series the legend does not show still reaches <desc>.
        let mut hidden = multi();
        if let GraphData::MultiSeries(list) = &mut hidden
            && let Some(first) = list.first_mut()
        {
            first.name = "hidden\0series".to_string();
        }
        let no_legend = GraphConfig {
            show_legend: false,
            ..config()
        };
        assert!(matches!(
            write_svg(&hidden, &no_legend, &dir.path().join("nul.svg")),
            Err(GraphError::NotXmlText {
                character: '\0',
                ..
            })
        ));
        Ok(())
    }

    #[test]
    fn test_small_figures_never_panic() {
        for data in [multi(), surface()] {
            for width in [1, 5, 40, 120, 239, 240] {
                for height in [1, 5, 40, 120, 159, 160] {
                    let config = GraphConfig {
                        width,
                        height,
                        ..config()
                    };
                    let outcome = std::panic::catch_unwind(|| render_svg(&data, &config));
                    match outcome {
                        Ok(
                            Ok(_) | Err(GraphError::TooSmall { .. } | GraphError::NoRoom { .. }),
                        ) => {}
                        Ok(Err(other)) => panic!("{width}x{height}: {other}"),
                        Err(_) => panic!("{width}x{height} panicked"),
                    }
                }
            }
        }
    }

    #[test]
    fn test_tiny_values_keep_distinct_tick_labels() -> TestResult {
        let tiny = GraphData::Series(series(
            "tiny",
            &[dec!(0), dec!(1)],
            &[dec!(0), dec!(0.00001)],
            TraceMode::Lines,
        ));
        let outlined = render_svg(
            &tiny,
            &GraphConfig {
                width: 320,
                height: 200,
                ..config()
            },
        )?;
        let small: std::collections::BTreeSet<String> = outlined
            .labels
            .iter()
            .filter_map(|l| l.text.replace('\u{2212}', "-").parse::<f64>().ok())
            .filter(|v| *v > 0.0 && *v <= 1.1e-5)
            .map(|v| format!("{v:e}"))
            .collect();
        assert!(small.len() >= 3, "{small:?}");
        Ok(())
    }

    #[test]
    fn test_png_title_clipped_at_the_sides_keeps_its_coverage() -> TestResult {
        // The same centred title cut off at 640 px and whole at 2040 px.
        let dir = tempfile::tempdir()?;
        let export = |width: u32| -> Result<(Vec<u8>, usize), Box<dyn std::error::Error>> {
            let config = GraphConfig {
                width,
                height: 480,
                title: "O".repeat(100),
                ..config()
            };
            let path = dir.path().join(format!("{width}.png"));
            write_png(&multi(), &config, &path)?;
            let mut reader =
                png::Decoder::new(std::io::BufReader::new(std::fs::File::open(&path)?))
                    .read_info()?;
            let mut buffer = vec![0; reader.output_buffer_size()];
            let info = reader.next_frame(&mut buffer)?;
            Ok((buffer, info.line_size))
        };
        let ((small, small_stride), (large, large_stride)) = (export(640)?, export(2040)?);
        let offset = (2040 - 640) / 2 * 3;
        let mut worst = 0_u8;
        for row in 0..25 {
            for col in 0..640 * 3 {
                let a = small[row * small_stride + col];
                let b = large[row * large_stride + offset + col];
                worst = worst.max(a.abs_diff(b));
            }
        }
        assert!(worst <= 2, "clipped title differs by {worst}/255");
        Ok(())
    }

    #[test]
    fn test_oversized_figures_are_refused_before_drawing() -> TestResult {
        // Each of these used to overflow inside Plotters: the PNG buffer
        // size, or an `i32` coordinate of the SVG. Nothing is allocated.
        let dir = tempfile::tempdir()?;
        let (png, svg) = (dir.path().join("big.png"), dir.path().join("big.svg"));
        let huge = 1_u32 << 31;
        for (width, height) in [
            (u32::MAX, u32::MAX),
            (huge, 480),
            (640, huge),
            (16_385, 100),
            (100, 16_385),
        ] {
            let config = GraphConfig {
                width,
                height,
                ..config()
            };
            let refused = |result: Result<(), GraphError>| matches!(result, Err(GraphError::InvalidDimensions { width: w, height: h }) if (w, h) == (width, height));
            assert!(
                refused(write_png(&multi(), &config, &png)),
                "{width}x{height}"
            );
            assert!(
                refused(write_svg(&multi(), &config, &svg)),
                "{width}x{height}"
            );
            assert!(refused(render_svg(&multi(), &config).map(|_| ())));
        }
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
        // The largest side is still drawn.
        let edge = GraphConfig {
            width: 16_384,
            height: 200,
            ..config()
        };
        write_png(&multi(), &edge, &png)?;
        assert_eq!(png_size(&png)?, (16_384, 200));
        Ok(())
    }

    #[test]
    fn test_a_long_output_name_is_written() -> TestResult {
        // The temporary file's name does not grow with the output's.
        let dir = tempfile::tempdir()?;
        for extension in ["png", "svg"] {
            let path = dir.path().join(format!("{}.{extension}", "a".repeat(246)));
            if extension == "png" {
                write_png(&multi(), &config(), &path)?;
            } else {
                write_svg(&multi(), &config(), &path)?;
            }
            assert!(path.is_file());
        }
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 2);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_a_symlink_at_the_output_is_replaced_not_written_through() -> TestResult {
        let dir = tempfile::tempdir()?;
        let victim = dir.path().join("victim.txt");
        std::fs::write(&victim, "untouched")?;
        for name in ["out.png", "out.svg"] {
            let path = dir.path().join(name);
            std::os::unix::fs::symlink(&victim, &path)?;
            if name.ends_with("png") {
                write_png(&multi(), &config(), &path)?;
            } else {
                write_svg(&multi(), &config(), &path)?;
            }
            assert!(std::fs::symlink_metadata(&path)?.is_file(), "{name}");
        }
        assert_eq!(std::fs::read_to_string(&victim)?, "untouched");
        // victim.txt, out.png, out.svg: no scratch directory is left.
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 3);
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn test_an_unwritable_directory_is_an_io_error() -> TestResult {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir()?;
        let locked = dir.path().join("locked");
        std::fs::create_dir(&locked)?;
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o555))?;
        // A superuser writes anyway; there is nothing to check then.
        let enforced = std::fs::write(locked.join("probe"), "").is_err();
        let outcomes = [
            write_png(&multi(), &config(), &locked.join("out.png")),
            write_svg(&multi(), &config(), &locked.join("out.svg")),
        ];
        std::fs::set_permissions(&locked, std::fs::Permissions::from_mode(0o755))?;
        if enforced {
            for outcome in outcomes {
                assert!(
                    matches!(&outcome, Err(GraphError::Io(error)) if error.kind() == std::io::ErrorKind::PermissionDenied),
                    "{outcome:?}"
                );
            }
        }
        Ok(())
    }
    /// A complete grid over `xs` x `ys`, rising along both axes.
    fn grid(xs: &[Decimal], ys: &[Decimal]) -> GraphData {
        let (mut x, mut y, mut z) = (Vec::new(), Vec::new(), Vec::new());
        for (i, strike) in xs.iter().enumerate() {
            for (j, depth) in ys.iter().enumerate() {
                x.push(*strike);
                y.push(*depth);
                z.push(Decimal::from(i) * dec!(0.1) + Decimal::from(j) * dec!(0.05));
            }
        }
        GraphData::GraphSurface(Surface3D {
            x,
            y,
            z,
            name: "Delta".to_string(),
        })
    }

    /// Every pair of painted labels whose ink boxes intersect: tick labels,
    /// axis titles and colour bar labels alike (#937, #940).
    fn overlapping(labels: &[outline::Label]) -> Vec<(String, String)> {
        let ticks: Vec<&outline::Label> = labels.iter().collect();
        let mut pairs = Vec::new();
        for (index, a) in ticks.iter().enumerate() {
            for b in ticks.iter().skip(index + 1) {
                let apart = a.right <= b.left
                    || b.right <= a.left
                    || a.bottom <= b.top
                    || b.bottom <= a.top;
                if !apart {
                    pairs.push((a.text.clone(), b.text.clone()));
                }
            }
        }
        pairs
    }

    #[test]
    fn test_surface_labels_never_overlap() -> TestResult {
        let strikes = |end: Decimal| [dec!(5600), dec!(5800), dec!(6000), end];
        let vols = |end: Decimal| [dec!(0.10), dec!(0.20), dec!(0.30), end];
        let mut cases = vec![("fixture".to_string(), surface())];
        for x_end in [dec!(6200), dec!(6190), dec!(6210), dec!(6250)] {
            for y_end in [dec!(0.40), dec!(0.39), dec!(0.41), dec!(0.45)] {
                cases.push((
                    format!("{x_end} x {y_end}"),
                    grid(&strikes(x_end), &vols(y_end)),
                ));
            }
        }
        cases.push((
            "0..1".to_string(),
            grid(
                &[dec!(0), dec!(0.5), dec!(1)],
                &[dec!(0), dec!(0.5), dec!(1)],
            ),
        ));
        cases.push((
            "days".to_string(),
            grid(
                &[dec!(80), dec!(100), dec!(120)],
                &[dec!(1), dec!(180), dec!(365)],
            ),
        ));
        cases.push((
            "negative".to_string(),
            grid(&[dec!(-3), dec!(2), dec!(7)], &[dec!(-1), dec!(0), dec!(1)]),
        ));
        let mut found = Vec::new();
        for (width, height) in [(800, 480), (1280, 720), (640, 420)] {
            let config = GraphConfig {
                width,
                height,
                ..config()
            };
            for (name, data) in &cases {
                let outlined = render_svg(data, &config)?;
                for pair in overlapping(&outlined.labels) {
                    found.push(format!("{name} at {width}x{height}: {pair:?}"));
                }
                let (right, bottom) = (f64::from(width), f64::from(height));
                for label in &outlined.labels {
                    let inside = label.left >= 0.0
                        && label.top >= 0.0
                        && label.right <= right
                        && label.bottom <= bottom;
                    if !inside {
                        found.push(format!(
                            "{name} at {width}x{height}: {} off the image",
                            label.text
                        ));
                    }
                }
            }
        }
        assert!(found.is_empty(), "{found:#?}");
        Ok(())
    }

    #[test]
    fn test_surface_corner_drops_only_the_depth_label() -> TestResult {
        // #937: both axes end on a tick, so their last labels meet at the
        // front corner; the strike stays and so does every other tick.
        let data = grid(
            &[dec!(5600), dec!(5800), dec!(6000), dec!(6200)],
            &[dec!(0.10), dec!(0.20), dec!(0.30), dec!(0.40)],
        );
        let config = GraphConfig {
            z_label: None,
            ..config()
        };
        let outlined = render_svg(&data, &config)?;
        let count = |text: &str| outlined.labels.iter().filter(|l| l.text == text).count();
        for strike in ["5600", "5800", "6000", "6200"] {
            assert_eq!(count(strike), 1, "{strike}");
        }
        // 0.1 to 0.3 on the depth axis, the height axis and the colour bar;
        // 0.4 only on the height axis, its depth twin at the corner dropped.
        for (tick, painted) in [("0.1", 3), ("0.2", 3), ("0.3", 3), ("0.4", 1)] {
            assert_eq!(count(tick), painted, "{tick}");
        }
        Ok(())
    }
}
