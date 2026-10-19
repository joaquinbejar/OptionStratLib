//! The PNG/SVG export through the public `Graph` API (#927): drawn natively
//! from `graph_data()`/`graph_config()`, with typed errors that reach every
//! entry point unchanged, no browser and no process-wide lock.
#![cfg(feature = "static_export")]

use super::artifacts::{assert_png_artifact, assert_svg_artifact};
use optionstratlib_math::curves::{Curve, Point2D};
use optionstratlib_math::surfaces::{Point3D, Surface};
use optionstratlib_visualization::error::GraphError;
use optionstratlib_visualization::visualization::{
    Graph, GraphConfig, GraphData, OutputType, Plottable, Series2D, Surface3D, TraceMode,
};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::{Command, Stdio};

struct Chart {
    data: GraphData,
    title: &'static str,
}

impl Graph for Chart {
    fn graph_data(&self) -> GraphData {
        self.data.clone()
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: self.title.to_string(),
            width: 640,
            height: 400,
            ..GraphConfig::default()
        }
    }
}

fn line(y: Vec<Decimal>) -> GraphData {
    GraphData::Series(Series2D {
        x: vec![dec!(1), dec!(2), dec!(3)],
        y,
        name: "line".to_string(),
        mode: TraceMode::Lines,
        line_color: None,
        line_width: None,
    })
}

fn surface(x: Vec<Decimal>, y: Vec<Decimal>, z: Vec<Decimal>) -> GraphData {
    GraphData::GraphSurface(Surface3D {
        x,
        y,
        z,
        name: "surface".to_string(),
    })
}

/// One chart the export refuses, the entry points that reach the check and
/// the variant they must all return.
struct Refused {
    chart: Chart,
    /// `false` for a check only the SVG writer makes.
    png: bool,
    expected: fn(&GraphError) -> bool,
}

fn refused() -> Vec<Refused> {
    let series = |line_color: Option<&str>, line_width: Option<f64>| {
        GraphData::Series(Series2D {
            x: vec![dec!(1), dec!(2)],
            y: vec![dec!(1), dec!(2)],
            name: "line".to_string(),
            mode: TraceMode::Lines,
            line_color: line_color.map(str::to_string),
            line_width,
        })
    };
    let chart = |data: GraphData, title: &'static str| Chart { data, title };
    vec![
        // Two y values for three x values: Plotly drew two points.
        Refused {
            chart: chart(line(vec![dec!(1), dec!(2)]), "short"),
            png: true,
            expected: |e| matches!(e, GraphError::LengthMismatch { x: 3, y: 2, .. }),
        },
        Refused {
            chart: chart(
                surface(
                    vec![dec!(1), dec!(2)],
                    vec![dec!(1), dec!(2)],
                    vec![dec!(1)],
                ),
                "uneven",
            ),
            png: true,
            expected: |e| {
                matches!(
                    e,
                    GraphError::SurfaceLengthMismatch {
                        x: 2,
                        y: 2,
                        z: 1,
                        ..
                    }
                )
            },
        },
        // Plotly kept the last of two values at one grid point.
        Refused {
            chart: chart(
                surface(
                    vec![dec!(1), dec!(1), dec!(2), dec!(2), dec!(1)],
                    vec![dec!(1), dec!(2), dec!(1), dec!(2), dec!(1)],
                    vec![dec!(0.1); 5],
                ),
                "twice",
            ),
            png: true,
            expected: |e| matches!(e, GraphError::DuplicateSurfacePoint { .. }),
        },
        // Plotly filled the missing grid point with zero.
        Refused {
            chart: chart(
                surface(
                    vec![dec!(1), dec!(2), dec!(1)],
                    vec![dec!(1), dec!(1), dec!(2)],
                    vec![dec!(0.1), dec!(0.2), dec!(0.3)],
                ),
                "gappy",
            ),
            png: true,
            expected: |e| {
                matches!(
                    e,
                    GraphError::IncompleteSurface {
                        expected: 4,
                        found: 3,
                        ..
                    }
                )
            },
        },
        Refused {
            chart: chart(
                surface(
                    vec![dec!(1), dec!(1)],
                    vec![dec!(1), dec!(2)],
                    vec![dec!(0.1); 2],
                ),
                "one column",
            ),
            png: true,
            expected: |e| {
                matches!(
                    e,
                    GraphError::DegenerateSurface {
                        columns: 1,
                        rows: 2,
                        ..
                    }
                )
            },
        },
        // plotly.js took this spelling; CSS does not.
        Refused {
            chart: chart(series(Some("hsl(0, 100, 50)"), None), "colour"),
            png: true,
            expected: |e| matches!(e, GraphError::InvalidColor { color, .. } if color == "hsl(0, 100, 50)"),
        },
        Refused {
            chart: chart(series(None, Some(f64::NAN)), "width"),
            png: true,
            expected: |e| matches!(e, GraphError::InvalidLineWidth { width, .. } if width.is_nan()),
        },
        Refused {
            chart: chart(line(vec![dec!(1), dec!(2), dec!(3)]), "IV 中"),
            png: true,
            expected: |e| {
                matches!(
                    e,
                    GraphError::MissingGlyph {
                        character: '中',
                        ..
                    }
                )
            },
        },
        // XML 1.0 cannot carry a NUL in the SVG's <title>.
        Refused {
            chart: chart(line(vec![dec!(1), dec!(2), dec!(3)]), "nul\0title"),
            png: false,
            expected: |e| {
                matches!(
                    e,
                    GraphError::NotXmlText {
                        character: '\0',
                        ..
                    }
                )
            },
        },
    ]
}

#[test]
fn test_typed_errors_reach_every_entry_point_unchanged() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?;
    for case in refused() {
        let Refused {
            chart,
            png,
            expected,
        } = case;
        let (png_path, svg_path) = (dir.path().join("chart.png"), dir.path().join("chart.svg"));
        let mut outcomes = vec![
            ("write_svg", chart.write_svg(&svg_path)),
            ("render svg", chart.render(OutputType::Svg(&svg_path))),
        ];
        if png {
            outcomes.push(("write_png", chart.write_png(&png_path)));
            outcomes.push(("render png", chart.render(OutputType::Png(&png_path))));
        }
        for (entry, outcome) in outcomes {
            assert!(
                outcome.as_ref().is_err_and(expected),
                "{}: {entry} gave {outcome:?}",
                chart.title
            );
        }
        // Neither an image nor a partial file is left behind.
        assert_eq!(std::fs::read_dir(dir.path())?.count(), 0, "{}", chart.title);
    }
    Ok(())
}

/// A surface from `(x, y, z)` triples, as a downstream crate builds one.
fn math_surface(points: &[(Decimal, Decimal, Decimal)]) -> Surface {
    Surface::new(
        points
            .iter()
            .map(|&(x, y, z)| Point3D::new(x, y, z))
            .collect::<BTreeSet<_>>(),
    )
}

fn math_curve() -> Curve {
    Curve::new(
        [(dec!(1), dec!(2)), (dec!(2), dec!(3)), (dec!(3), dec!(1))]
            .into_iter()
            .map(|(x, y)| Point2D::new(x, y))
            .collect(),
    )
}

#[test]
fn test_plot_builder_save_returns_the_typed_error() -> Result<(), Box<dyn std::error::Error>> {
    // `PlotBuilder` is reached through the crate's `Plottable` types: their
    // coordinate arrays always agree in length and curves carry their own
    // colours, so these are the refusals their data and the builder's
    // options can produce.
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("saved.png");
    let gappy = math_surface(&[
        (dec!(1), dec!(1), dec!(0.1)),
        (dec!(2), dec!(1), dec!(0.2)),
        (dec!(1), dec!(2), dec!(0.3)),
    ]);
    let result = gappy.plot().title("gappy").save(&path);
    assert!(
        matches!(
            result,
            Err(GraphError::IncompleteSurface {
                expected: 4,
                found: 3,
                ..
            })
        ),
        "{result:?}"
    );
    // Two heights at one grid point are two points of the set.
    let twice = math_surface(&[
        (dec!(1), dec!(1), dec!(0.1)),
        (dec!(1), dec!(1), dec!(0.2)),
        (dec!(2), dec!(1), dec!(0.1)),
        (dec!(1), dec!(2), dec!(0.1)),
        (dec!(2), dec!(2), dec!(0.1)),
    ]);
    let result = twice.plot().save(&path);
    assert!(
        matches!(result, Err(GraphError::DuplicateSurfacePoint { .. })),
        "{result:?}"
    );
    let flat = math_surface(&[(dec!(1), dec!(1), dec!(0.1)), (dec!(1), dec!(2), dec!(0.1))]);
    let result = flat.plot().save(&path);
    assert!(
        matches!(result, Err(GraphError::DegenerateSurface { .. })),
        "{result:?}"
    );
    let result = math_curve().plot().title("IV 中").save(&path);
    assert!(
        matches!(
            result,
            Err(GraphError::MissingGlyph {
                character: '中',
                ..
            })
        ),
        "{result:?}"
    );
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 0);
    math_curve().plot().title("saved").save(&path)?;
    assert_png_artifact(&path);
    Ok(())
}

/// A chart whose `to_plot` is overridden: since 0.23 the images come from
/// `graph_data()`/`graph_config()`, and the override reaches HTML only.
struct Customised;

impl Graph for Customised {
    fn graph_data(&self) -> GraphData {
        line(vec![dec!(1), dec!(3), dec!(2)])
    }

    fn graph_config(&self) -> GraphConfig {
        GraphConfig {
            title: "From graph_config".to_string(),
            width: 640,
            height: 400,
            ..GraphConfig::default()
        }
    }

    fn to_plot(&self) -> plotly::Plot {
        let mut plot = plotly::Plot::new();
        plot.set_layout(plotly::Layout::new().title("From to_plot"));
        plot
    }
}

#[test]
fn test_images_come_from_graph_data_and_html_from_to_plot() -> Result<(), Box<dyn std::error::Error>>
{
    let dir = tempfile::tempdir()?;
    let (svg, html, png) = (
        dir.path().join("chart.svg"),
        dir.path().join("chart.html"),
        dir.path().join("chart.png"),
    );
    Customised.write_svg(&svg)?;
    Customised.write_html(&html)?;
    Customised.write_png(&png)?;
    let svg = std::fs::read_to_string(&svg)?;
    assert!(svg.contains("<title>From graph_config</title>"));
    assert!(!svg.contains("From to_plot"));
    assert!(std::fs::read_to_string(&html)?.contains("From to_plot"));
    assert_png_artifact(&png);
    Ok(())
}

#[test]
fn test_concurrent_exports_are_independent() -> Result<(), Box<dyn std::error::Error>> {
    // No browser, driver or process-wide lock: exports in parallel threads
    // each write their own complete file.
    let dir = tempfile::tempdir()?;
    let handles: Vec<_> = (0..8)
        .map(|index| {
            let base = dir.path().to_path_buf();
            std::thread::spawn(move || -> Result<(PathBuf, PathBuf), GraphError> {
                let chart = Chart {
                    data: line(vec![dec!(1), Decimal::from(index), dec!(2)]),
                    title: "parallel",
                };
                let (png, svg) = (
                    base.join(format!("{index}.png")),
                    base.join(format!("{index}.svg")),
                );
                chart.write_png(&png)?;
                chart.write_svg(&svg)?;
                Ok((png, svg))
            })
        })
        .collect();
    for handle in handles {
        let (png, svg) = handle.join().map_err(|_| "export thread panicked")??;
        assert_png_artifact(&png);
        assert_svg_artifact(&svg);
    }
    Ok(())
}

/// Where `test_export_in_child_process` writes, set only by its parent.
const CHILD_DIR: &str = "OSL_NATIVE_EXPORT_CHILD_DIR";
/// Which chart the child draws.
const CHILD_INDEX: &str = "OSL_NATIVE_EXPORT_CHILD_INDEX";

/// The child half of `test_concurrent_processes_are_independent`: does
/// nothing unless that test started this process.
#[test]
fn test_export_in_child_process() -> Result<(), Box<dyn std::error::Error>> {
    let (Ok(dir), Ok(index)) = (std::env::var(CHILD_DIR), std::env::var(CHILD_INDEX)) else {
        return Ok(());
    };
    let chart = Chart {
        data: line(vec![dec!(1), Decimal::from(index.parse::<u32>()?), dec!(2)]),
        title: "process",
    };
    let dir = PathBuf::from(dir);
    chart.write_png(&dir.join(format!("{index}.png")))?;
    chart.write_svg(&dir.join(format!("{index}.svg")))?;
    Ok(())
}

#[test]
fn test_concurrent_processes_are_independent() -> Result<(), Box<dyn std::error::Error>> {
    // The old exporter shared one browser per process behind a lock; separate
    // processes now export at once, each into its own complete file.
    let dir = tempfile::tempdir()?;
    let exe = std::env::current_exe()?;
    let children = (0..4)
        .map(|index| {
            Command::new(&exe)
                .args([
                    "--exact",
                    "native_export_test::test_export_in_child_process",
                    "--test-threads=1",
                ])
                .env(CHILD_DIR, dir.path())
                .env(CHILD_INDEX, index.to_string())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
        })
        .collect::<Result<Vec<_>, _>>()?;
    for mut child in children {
        assert!(child.wait()?.success());
    }
    for index in 0..4 {
        assert_png_artifact(&dir.path().join(format!("{index}.png")));
        assert_svg_artifact(&dir.path().join(format!("{index}.svg")));
    }
    // Eight files and no partial one.
    assert_eq!(std::fs::read_dir(dir.path())?.count(), 8);
    Ok(())
}
