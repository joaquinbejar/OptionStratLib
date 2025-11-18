use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let smile_curve = option_chain.smile();
    smile_curve
        .plot()
        .title("Volatility Smile")
        .x_label("Strike Price")
        .y_label("Implied Volatility")
        .save("./Draws/Curves/volatility_smile.png")?;

    Ok(())
}
