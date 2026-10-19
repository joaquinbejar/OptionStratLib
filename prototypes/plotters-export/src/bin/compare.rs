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
//! compare outlines <dir>
//!     The native artifacts of #923 for every case and the text fixture:
//!     PNG and the three SVG text modes, each named and listed with its
//!     mode in <dir>/manifest.json, plus each outlines SVG's label boxes in
//!     <dir>/<case>.labels.json.
//! ```
//!
//! `<backend>` is `plotters`; `plotly` is refused since #927 (see
//! [`PLOTLY_RETIRED`]). `<case>` is one of
//! `payoff`, `curves`, `surface` or `text` (the #923 text fixture), and
//! `<format>` `png`, `svg` (embedded font), `svg-plain` (Plotters SVG
//! without the embedded font) or `svg-outlines` (Plotters SVG with text as
//! outlines).

use osl_prototype_plotters_export::fixtures::{Case, Fixture};
use osl_prototype_plotters_export::{SvgText, render_svg_outlined, write_png, write_svg};
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
    SvgOutlines,
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
            "svg-outlines" => Ok(Format::SvgOutlines),
            other => Err(format!(
                "unknown format {other:?}: png, svg, svg-plain or svg-outlines"
            )),
        }
    }

    const fn slug(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Svg => "svg",
            Format::SvgPlain => "svg-plain",
            Format::SvgOutlines => "svg-outlines",
        }
    }

    /// How the format carries text: `None` for a raster.
    const fn text_mode(self) -> Option<&'static str> {
        match self {
            Format::Png => None,
            Format::Svg => Some("embedded-font"),
            Format::SvgPlain => Some("viewer-font"),
            Format::SvgOutlines => Some("outlines"),
        }
    }

    const fn extension(self) -> &'static str {
        match self {
            Format::Png => "png",
            Format::Svg => "svg",
            Format::SvgPlain => "plain.svg",
            Format::SvgOutlines => "outlines.svg",
        }
    }
}

fn parse_case(text: &str) -> Result<Case, String> {
    Case::from_slug(text)
        .ok_or_else(|| format!("unknown case {text:?}: payoff, curves, surface or text"))
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
        (Backend::Plotters, Format::SvgOutlines) => {
            write_svg(&fixture.data, &fixture.config, path, SvgText::Outlines)?;
        }
        (Backend::Plotly, format) => plotly_export(fixture, format, path)?,
    }
    Ok(())
}

/// Why the `plotly` backend is refused. It drove `Graph::write_png` and
/// `write_svg` of `optionstratlib-visualization` with `static_export`, which
/// were the `plotly_static` browser export when #920, #923 and #931 measured
/// them. Since #927 those methods draw natively, so the same call would
/// label this renderer's own output as Plotly's.
const PLOTLY_RETIRED: &str = "the Plotly baseline is retired: since #927 Graph::write_png/write_svg \
     draw natively, so they no longer measure plotly_static. Build `compare` with \
     `--features plotly-baseline` at 8484b42f, the last commit with the browser export, \
     to reproduce the baseline; the recorded results are under results/ and artifacts/";

fn plotly_export(_: &Fixture, _: Format, _: &Path) -> Result<(), Box<dyn Error>> {
    Err(PLOTLY_RETIRED.into())
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
    let backends = [Backend::Plotters];
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
struct NativeArtifact {
    case: &'static str,
    format: &'static str,
    /// `embedded-font`, `viewer-font` or `outlines`; absent for PNG.
    text: Option<&'static str>,
    file: String,
    width: u32,
    height: u32,
    bytes: u64,
}

/// The native artifacts of #923, with their manifest and label boxes.
fn outlines(dir: &Path) -> Result<(), Box<dyn Error>> {
    let mut written = Vec::new();
    for case in Case::EVERY {
        let fixture = case.build()?;
        for format in [
            Format::Png,
            Format::Svg,
            Format::SvgPlain,
            Format::SvgOutlines,
        ] {
            let file = format!("{}.{}", case.slug(), format.extension());
            let path = dir.join(&file);
            export(Backend::Plotters, &fixture, format, &path)?;
            let bytes = std::fs::metadata(&path)?.len();
            info!(file, bytes, "artifact written");
            written.push(NativeArtifact {
                case: case.slug(),
                format: format.slug(),
                text: format.text_mode(),
                file,
                width: fixture.config.width,
                height: fixture.config.height,
                bytes,
            });
        }
        let outlined = render_svg_outlined(&fixture.data, &fixture.config)?;
        std::fs::write(
            dir.join(format!("{}.labels.json", case.slug())),
            serde_json::to_string_pretty(&outlined.labels)?,
        )?;
    }
    std::fs::write(
        dir.join("manifest.json"),
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
        "outlines" => {
            let dir = PathBuf::from(argument(&args, 1, "output directory")?);
            std::fs::create_dir_all(&dir)?;
            outlines(&dir)
        }
        other => {
            Err(format!("unknown command {other:?}: artifacts, export, repeat or outlines").into())
        }
    }
}
