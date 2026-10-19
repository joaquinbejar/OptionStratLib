//! A strategy charted to HTML, PNG and SVG with `optionstratlib-visualization`
//! (#555, #930).
//!
//! The visualization crate is the leaf of the workspace: it owns the `Graph`
//! contract and implements it for the strategies, so a program that only
//! prices or backtests never builds it. `static_export` adds `write_png` and
//! `write_svg`, drawn natively (no browser, WebDriver or installed font), and
//! implies `plotly`, which adds the interactive methods (`to_plot`,
//! `write_html`). The files are written under the system temporary
//! directory. Data the export cannot draw faithfully is a typed
//! `GraphError`, which the program handles like any other error.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_strategies::strategies::BullCallSpread;
use optionstratlib_visualization::error::GraphError;
use optionstratlib_visualization::visualization::{Graph, GraphData, Surface3D};
use rust_decimal_macros::dec;
use std::error::Error;
use std::path::{Path, PathBuf};
use tracing::info;

fn bull_call_spread() -> Result<BullCallSpread, Box<dyn Error>> {
    Ok(BullCallSpread::new(
        "SP500".to_string(),
        Positive::new(5781.88)?,
        Positive::new(5750.0)?,
        Positive::new(5820.0)?,
        ExpirationDate::Days(Positive::TWO),
        Positive::new(0.18)?,
        dec!(0.05),
        Positive::ZERO,
        Positive::TWO,
        Positive::new(85.04)?,
        Positive::new(29.85)?,
        Positive::new(0.78)?,
        Positive::new(0.78)?,
        Positive::new(0.73)?,
        Positive::new(0.73)?,
    )?)
}

/// Where this run writes its chart, with `extension`.
fn output_path(extension: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "osl-example-direct-visualization-{}.{extension}",
        std::process::id()
    ))
}

/// Number of series in the strategy's chart, and the bytes written to each
/// of `paths` (HTML, PNG, SVG).
fn chart(paths: [&Path; 3]) -> Result<(usize, [u64; 3]), Box<dyn Error>> {
    let strategy = bull_call_spread()?;
    let series = match strategy.graph_data() {
        GraphData::MultiSeries(series) => series.len(),
        GraphData::Series(_) => 1,
        GraphData::GraphSurface(_) => 0,
    };
    let [html, png, svg] = paths;
    strategy.write_html(html)?;
    strategy.write_png(png)?;
    strategy.write_svg(svg)?;
    let size = |path: &Path| std::fs::metadata(path).map(|m| m.len());
    Ok((series, [size(html)?, size(png)?, size(svg)?]))
}

/// A volatility surface with one grid point missing.
struct GappySurface;

impl Graph for GappySurface {
    fn graph_data(&self) -> GraphData {
        GraphData::GraphSurface(Surface3D {
            x: vec![dec!(90), dec!(100), dec!(90)],
            y: vec![dec!(7), dec!(7), dec!(30)],
            z: vec![dec!(0.21), dec!(0.18), dec!(0.20)],
            name: "Implied volatility".to_string(),
        })
    }
}

/// Exports `GappySurface` and reports how the export refused it. The image
/// export draws only complete grids: it never fills a missing point with
/// zero, as a browser chart would. `GraphError` is `#[non_exhaustive]`, so a
/// match keeps a wildcard arm.
fn refused_export(path: &Path) -> String {
    match GappySurface.write_png(path) {
        Ok(()) => "exported".to_string(),
        Err(GraphError::IncompleteSurface {
            expected, found, ..
        }) => format!("refused: {found} of {expected} grid points"),
        Err(other) => format!("failed: {other}"),
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let paths = [output_path("html"), output_path("png"), output_path("svg")];
    let (series, bytes) = chart([&paths[0], &paths[1], &paths[2]])?;
    for (path, bytes) in paths.iter().zip(bytes) {
        info!(
            "{series} series written to {} ({bytes} bytes)",
            path.display()
        );
    }
    info!(
        "incomplete surface: {}",
        refused_export(&output_path("gappy.png"))
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_strategy_chart_is_written_as_html_png_and_svg() -> Result<(), Box<dyn Error>> {
        let paths = [output_path("html"), output_path("png"), output_path("svg")];
        let (series, bytes) = chart([&paths[0], &paths[1], &paths[2]])?;
        assert!(series > 0);
        assert!(bytes.iter().all(|&b| b > 0));
        assert!(std::fs::read(&paths[1])?.starts_with(b"\x89PNG"));
        assert!(std::fs::read_to_string(&paths[2])?.starts_with("<svg"));
        for path in &paths {
            std::fs::remove_file(path)?;
        }
        Ok(())
    }

    #[test]
    fn test_an_incomplete_surface_is_a_typed_refusal() {
        let path = output_path("gappy-test.png");
        assert_eq!(refused_export(&path), "refused: 3 of 4 grid points");
        assert!(!path.exists());
    }
}
