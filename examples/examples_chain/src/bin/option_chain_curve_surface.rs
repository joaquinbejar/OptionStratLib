/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::curves::BasicCurves;
use optionstratlib::geometrics::Plottable;
use optionstratlib::model::BasicAxisTypes;
use optionstratlib::surfaces::BasicSurfaces;
use optionstratlib::utils::setup_logger;
use optionstratlib::{OptionStyle, Side};
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
        .line_width(1)
        .save("Draws/Curves/option_chain_curve.png")?;

    info!("Curve saved");

    let surface = option_chain.surface(
        &BasicAxisTypes::Delta,
        &OptionStyle::Call,
        None,
        &Side::Long,
    )?;

    surface
        .plot()
        .title("Volatility Surface")
        .x_label("strike")
        .y_label("Volatility")
        .z_label("Delta")
        .line_width(1)
        .save("Draws/Surfaces/option_chain_surface.png")?;

    info!("Surface saved");
    Ok(())
}
