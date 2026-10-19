//! PNG and SVG export through the `optionstratlib` facade with only its
//! `static_export` feature (#927, #930).
//!
//! The images are drawn natively from `graph_data()` and `graph_config()`:
//! no browser, WebDriver, network or installed font. SVG text is drawn as
//! glyph outlines of the bundled Open Sans, so the file looks the same in
//! every viewer; each label keeps its text as an accessible name. Data the
//! export cannot draw faithfully is a typed `GraphError` rather than a
//! silently repaired chart. The files are written under the system
//! temporary directory.
use optionstratlib::error::{Error, GraphError};
use optionstratlib::prelude::*;
use optionstratlib::visualization::{GraphData, Series2D, TraceMode};
use positive::pos_or_panic;
use std::path::PathBuf;

/// A path for this run under the system temporary directory.
fn output(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("osl-native-export-{}-{name}", std::process::id()))
}

/// A long call's payoff, as the prelude charts it.
fn long_call() -> Result<LongCall, Error> {
    Ok(LongCall::new(
        "SP500".to_string(),
        pos_or_panic!(5800.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.18),
        Positive::ONE,
        pos_or_panic!(5781.88),
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(85.0),
        pos_or_panic!(0.78),
        pos_or_panic!(0.73),
    )?)
}

/// A downstream chart whose x and y arrays disagree in length: a browser
/// chart drew the shorter one, the image export refuses it.
struct Lopsided;

impl Graph for Lopsided {
    fn graph_data(&self) -> GraphData {
        GraphData::Series(Series2D {
            x: vec![dec!(1), dec!(2), dec!(3)],
            y: vec![dec!(1), dec!(2)],
            name: "lopsided".to_string(),
            mode: TraceMode::Lines,
            line_color: None,
            line_width: None,
        })
    }
}

fn main() -> Result<(), Error> {
    osl_example_support::setup_logger();
    let strategy = long_call()?;
    let (png, svg) = (output("long-call.png"), output("long-call.svg"));
    strategy.write_png(&png)?;
    strategy.write_svg(&svg)?;
    info!("payoff written to {} and {}", png.display(), svg.display());

    // `GraphError` is `#[non_exhaustive]`: match what you handle and keep a
    // wildcard arm.
    match Lopsided.write_png(&output("lopsided.png")) {
        Err(GraphError::LengthMismatch { name, x, y }) => {
            info!("{name}: refused, {x} x values for {y} y values");
        }
        Err(other) => return Err(other.into()),
        Ok(()) => info!("lopsided series exported"),
    }
    Ok(())
}
