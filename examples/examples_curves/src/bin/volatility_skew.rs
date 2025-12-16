use optionstratlib::prelude::*;

fn main() -> Result<(), Error> {
    setup_logger();
    let option_chain =
        OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")?;
    let skew_curve = option_chain.volatility_skew();
    skew_curve
        .plot()
        .title("Volatility Skew")
        .x_label("Moneyness")
        .y_label("Implied Volatility")
        .save("./Draws/Curves/volatility_skew.png")?;

    Ok(())
}

// #[cfg(test)]
// mod tests_volatility_skew_curve {
//     use super::*;

//     #[test]
//     fn test_skew_curve_volatility_consistency() {
//         let option_chain =
//             OptionChain::load_from_json(
//                 "../../examples/Chains/SP500-18-oct-2024-5781.88.json").unwrap();
//         let skew_curve = option_chain.volatility_skew();
//         for point in skew_curve.points.iter() {
//             println!("x: {}, y: {}", point.x, point.y);
//         }
//     }
// }
