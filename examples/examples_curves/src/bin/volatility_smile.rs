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

// #[cfg(test)]
// mod tests_volatility_smile_curve {
//     use super::*;

//     #[test]
//     fn test_smile_curve_volatility_consistency() {
//         let option_chain =
//             OptionChain::load_from_json(
//                 "../../examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
//         let smile_curve = option_chain.smile();
//         for point in smile_curve.points.iter() {
//             info!("x: {}, y: {}", point.x, point.y);
//         }
//     }
// }
