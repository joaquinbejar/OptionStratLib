/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::model::BasicAxisTypes;
use optionstratlib::prelude::*;
use osl_example_support::setup_logger;
use tracing::info;

fn main() -> Result<(), optionstratlib::error::Error> {
    setup_logger();
    let mut option_chain =
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    info!("Chain loaded");
    option_chain.update_greeks();

    let curve = option_chain.curve(&BasicAxisTypes::Volatility, &OptionStyle::Call, &Side::Long)?;

    curve
        .plot()
        .title("Volatility Curve")
        .x_label("strike")
        .y_label("Volatility")
        .save("Draws/Curves/option_chain_curve.png")?;

    info!("Curve saved");

    // Delta of every strike at each of these volatilities: a complete
    // strike x volatility grid. Without the list each strike gives one point
    // at its own implied volatility, a scatter that is not a surface, and the
    // image export refuses it as `GraphError::IncompleteSurface`.
    let volatilities = [0.10, 0.15, 0.20, 0.25, 0.30, 0.35, 0.40]
        .into_iter()
        .map(|volatility| pos_or_panic!(volatility))
        .collect();
    let surface = option_chain.surface(
        &BasicAxisTypes::Delta,
        &OptionStyle::Call,
        Some(volatilities),
        &Side::Long,
    )?;

    surface
        .plot()
        .title("Delta Surface")
        .x_label("strike")
        .y_label("Volatility")
        .z_label("Delta")
        .save("Draws/Surfaces/option_chain_surface.png")?;

    info!("Surface saved");
    Ok(())
}
