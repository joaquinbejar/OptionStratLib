/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::prelude::*;
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
        .save("Draws/Surfaces/option_chain_surface.png")?;

    info!("Surface saved");
    Ok(())
}
