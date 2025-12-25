use optionstratlib::prelude::*;
use std::error::Error;

fn get_option(strike: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        *strike,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.10), // implied volatility
        Positive::ONE,  // quantity
        pos_or_panic!(50.0), // underlying price
        Decimal::ZERO,       // risk free rate
        OptionStyle::Call,
        Positive::ZERO, // dividend yield
        None,           // exotic params
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = ConstructionParams::D2 {
        t_start: dec!(20.0),
        t_end: dec!(80.0),
        steps: 100,
    };
    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap());
            let value = option.charm().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Charm Curve")
        .x_label("Asset value")
        .y_label("Charm")
        .save("./Draws/Curves/charm_curve.png")?;

    Ok(())
}
