use optionstratlib::chains::chain::OptionChain;
use optionstratlib::utils::setup_logger;
use optionstratlib::volatility::VolatilitySmile;
use std::error::Error;
use optionstratlib::geometrics::Plottable;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let smile_curve = option_chain.smile();
    smile_curve
        .plot()
        .title("Volatility Smile")
        .x_label("Strike Price")
        .y_label("Implied Volatility")
        .line_width(1)
        .save("./Draws/Curves/volatility_smile.png")?;

    Ok(())
}
