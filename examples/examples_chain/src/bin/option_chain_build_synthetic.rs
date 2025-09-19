/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 29/1/25
******************************************************************************/
use optionstratlib::prelude::*;
use tracing::info;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    setup_logger();
    let mut option_chain_base =
        OptionChain::load_from_json("examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    // option_chain_base.update_expiration_date(get_today_formatted());
    option_chain_base.update_expiration_date(get_x_days_formatted(2));
    let chain_params = option_chain_base.to_build_params()?;
    info!("Chain params: {}", chain_params);
    let mut option_chain = OptionChain::build_chain(&chain_params);
    option_chain.update_greeks();
    // info!("{}", option_chain);
    option_chain.show();
    let curve = option_chain.curve(&BasicAxisTypes::Volatility, &OptionStyle::Call, &Side::Long)?;

    curve
        .plot()
        .title("Volatility Curve")
        .x_label("strike")
        .y_label("Volatility")
        .save("Draws/Curves/option_chain_curve.png")?;

    info!("Curve saved");

    Ok(())
}
