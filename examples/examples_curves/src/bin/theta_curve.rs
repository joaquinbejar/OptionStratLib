use optionstratlib::prelude::*;

fn get_option(strike: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        *strike,
        ExpirationDate::Days(pos_or_panic!(365.0)),
        pos_or_panic!(0.10),
        pos_or_panic!(1.0),
        pos_or_panic!(50.0),
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Error> {
    setup_logger();
    let params = ConstructionParams::D2 {
        t_start: dec!(20.0),
        t_end: dec!(80),
        steps: 100,
    };
    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap());
            let value = option.theta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Theta Curve")
        .x_label("Asset value")
        .y_label("theta")
        .save("./Draws/Curves/theta_curve.png")?;

    Ok(())
}
