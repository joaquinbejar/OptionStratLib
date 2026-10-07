//! A strategy charted to HTML with `optionstratlib-visualization` (#555).
//!
//! The visualization crate is the leaf of the workspace: it owns the `Graph`
//! contract and implements it for the strategies, so a program that only
//! prices or backtests never builds it. The `plotly` feature adds the
//! interactive methods (`to_plot`, `write_html`); it resolves Plotly and none
//! of the image-export packages, which only `static_export` brings. The HTML
//! is written under the system temporary directory.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_strategies::strategies::BullCallSpread;
use optionstratlib_visualization::visualization::{Graph, GraphData};
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

/// Where this run writes its chart.
fn output_path() -> PathBuf {
    std::env::temp_dir().join(format!(
        "osl-example-direct-visualization-{}.html",
        std::process::id()
    ))
}

/// Number of series in the strategy's chart, and the bytes of HTML written.
fn chart(path: &Path) -> Result<(usize, u64), Box<dyn Error>> {
    let strategy = bull_call_spread()?;
    let series = match strategy.graph_data() {
        GraphData::MultiSeries(series) => series.len(),
        GraphData::Series(_) => 1,
        GraphData::GraphSurface(_) => 0,
    };
    strategy.write_html(path)?;
    Ok((series, std::fs::metadata(path)?.len()))
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let path = output_path();
    let (series, bytes) = chart(&path)?;
    info!(
        "{series} series written to {} ({bytes} bytes)",
        path.display()
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_the_strategy_chart_is_written_as_html() -> Result<(), Box<dyn Error>> {
        let path = output_path();
        let (series, bytes) = chart(&path)?;
        std::fs::remove_file(&path)?;
        assert!(series > 0);
        assert!(bytes > 0);
        Ok(())
    }
}
