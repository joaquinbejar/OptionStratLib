//! Writes the comparison artifacts and times the exports of #920.
//!
//! ```text
//! compare artifacts <dir>
//!     Every case with every renderer and format, plus the SVG without the
//!     embedded font; sizes in <dir>/artifacts.json.
//! compare export <backend> <case> <format> <file>
//!     One export and exit: the unit the cold-start measurement times from
//!     outside, process start and shutdown included.
//! compare repeat <backend> <case> <format> <iterations> <dir>
//!     <iterations> exports in one process; per-export wall times in
//!     <dir>/repeat-<backend>-<case>-<format>.json.
//! ```
//!
//! `<backend>` is `plotters` or `plotly` (the latter needs the
//! `plotly-baseline` feature and a matching chromedriver), `<case>` one of
//! `payoff`, `curves`, `surface`, and `<format>` `png`, `svg` or `svg-plain`
//! (Plotters SVG without the embedded font).

use osl_prototype_plotters_export::fixtures::{Case, Fixture};
use osl_prototype_plotters_export::{SvgText, write_png, write_svg};
use serde::Serialize;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::info;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Backend {
    Plotters,
    Plotly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Format {
    Png,
    Svg,
    SvgPlain,
}

impl Backend {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "plotters" => Ok(Backend::Plotters),
            "plotly" => Ok(Backend::Plotly),
            other => Err(format!("unknown backend {other:?}: plotters or plotly")),
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Backend::Plotters => "plotters",
            Backend::Plotly => "plotly",
        }
    }
}

impl Format {
    fn parse(text: &str) -> Result<Self, String> {
        match text {
            "png" => Ok(Format::Png),
            "svg" => Ok(Format::Svg),
            "svg-plain" => Ok(Format::SvgPlain),
            other => Err(format!("unknown format {other:?}: png, svg or svg-plain")),
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Svg => "svg",
            Format::SvgPlain => "svg-plain",
        }
    }

    const fn extension(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Svg => "svg",
            Format::SvgPlain => "plain.svg",
        }
    }
}

fn parse_case(text: &str) -> Result<Case, String> {
    Case::from_slug(text).ok_or_else(|| format!("unknown case {text:?}: payoff, curves or surface"))
}

/// One export through the requested renderer.
fn export(
    backend: Backend,
    fixture: &Fixture,
    format: Format,
    path: &Path,
) -> Result<(), Box<dyn Error>> {
    match (backend, format) {
        (Backend::Plotters, Format::Png) => write_png(&fixture.data, &fixture.config, path)?,
        (Backend::Plotters, Format::Svg) => {
            write_svg(&fixture.data, &fixture.config, path, SvgText::EmbeddedFont)?;
        }
        (Backend::Plotters, Format::SvgPlain) => {
            write_svg(&fixture.data, &fixture.config, path, SvgText::ViewerFont)?;
        }
        (Backend::Plotly, format) => plotly_export(fixture, format, path)?,
    }
    Ok(())
}

#[cfg(feature = "plotly-baseline")]
fn plotly_export(fixture: &Fixture, format: Format, path: &Path) -> Result<(), Box<dyn Error>> {
    use optionstratlib_visualization::visualization::Graph;
    match format {
        Format::Png => fixture.write_png(path)?,
        Format::Svg => fixture.write_svg(path)?,
        Format::SvgPlain => return Err("svg-plain is a Plotters-only format".into()),
    }
    Ok(())
}

#[cfg(not(feature = "plotly-baseline"))]
fn plotly_export(_: &Fixture, _: Format, _: &Path) -> Result<(), Box<dyn Error>> {
    Err("built without the plotly-baseline feature".into())
}

#[derive(Serialize)]
struct Artifact {
    backend: &'static str,
    case: &'static str,
    format: &'static str,
    file: String,
    bytes: u64,
    millis: f64,
}

fn artifacts(dir: &Path) -> Result<(), Box<dyn Error>> {
    let mut written = Vec::new();
    let mut backends = vec![Backend::Plotters];
    if cfg!(feature = "plotly-baseline") {
        backends.push(Backend::Plotly);
    }
    for case in Case::ALL {
        let fixture = case.build()?;
        for backend in &backends {
            for format in [Format::Png, Format::Svg, Format::SvgPlain] {
                if *backend == Backend::Plotly && format == Format::SvgPlain {
                    continue;
                }
                let file = format!("{}-{}.{}", case.slug(), backend.slug(), format.extension());
                let path = dir.join(&file);
                let started = Instant::now();
                export(*backend, &fixture, format, &path)?;
                let millis = started.elapsed().as_secs_f64() * 1000.0;
                let bytes = std::fs::metadata(&path)?.len();
                info!(file, bytes, millis, "artifact written");
                written.push(Artifact {
                    backend: backend.slug(),
                    case: case.slug(),
                    format: format.slug(),
                    file,
                    bytes,
                    millis,
                });
            }
        }
    }
    std::fs::write(
        dir.join("artifacts.json"),
        serde_json::to_string_pretty(&written)?,
    )?;
    Ok(())
}

#[derive(Serialize)]
struct Repeat {
    backend: &'static str,
    case: &'static str,
    format: &'static str,
    /// Wall time of each export in milliseconds, in run order; the first
    /// includes whatever the renderer sets up once per process.
    millis: Vec<f64>,
}

fn repeat(
    backend: Backend,
    case: Case,
    format: Format,
    iterations: usize,
    dir: &Path,
) -> Result<(), Box<dyn Error>> {
    let fixture = case.build()?;
    let path = dir.join(format!(
        "repeat-{}-{}.{}",
        case.slug(),
        backend.slug(),
        format.extension()
    ));
    let mut millis = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let started = Instant::now();
        export(backend, &fixture, format, &path)?;
        millis.push(started.elapsed().as_secs_f64() * 1000.0);
    }
    let report = Repeat {
        backend: backend.slug(),
        case: case.slug(),
        format: format.slug(),
        millis,
    };
    let out = dir.join(format!(
        "repeat-{}-{}-{}.json",
        backend.slug(),
        case.slug(),
        format.slug()
    ));
    std::fs::write(&out, serde_json::to_string_pretty(&report)?)?;
    info!(out = %out.display(), "repeat timings written");
    Ok(())
}

fn argument<'a>(args: &'a [String], index: usize, what: &str) -> Result<&'a str, String> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| format!("missing {what}; see the module documentation for usage"))
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let args: Vec<String> = std::env::args().skip(1).collect();
    match argument(&args, 0, "command")? {
        "artifacts" => {
            let dir = PathBuf::from(argument(&args, 1, "output directory")?);
            std::fs::create_dir_all(&dir)?;
            artifacts(&dir)
        }
        "export" => {
            let backend = Backend::parse(argument(&args, 1, "backend")?)?;
            let case = parse_case(argument(&args, 2, "case")?)?;
            let format = Format::parse(argument(&args, 3, "format")?)?;
            let path = PathBuf::from(argument(&args, 4, "output file")?);
            export(backend, &case.build()?, format, &path)
        }
        "repeat" => {
            let backend = Backend::parse(argument(&args, 1, "backend")?)?;
            let case = parse_case(argument(&args, 2, "case")?)?;
            let format = Format::parse(argument(&args, 3, "format")?)?;
            let iterations: usize = argument(&args, 4, "iterations")?.parse()?;
            let dir = PathBuf::from(argument(&args, 5, "output directory")?);
            std::fs::create_dir_all(&dir)?;
            repeat(backend, case, format, iterations, &dir)
        }
        other => Err(format!("unknown command {other:?}: artifacts, export or repeat").into()),
    }
}
