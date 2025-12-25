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
        t_start: dec!(25.0),
        t_end: dec!(78.0),
        steps: 100,
    };
    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap());
            let value = option.vega().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Vega Curve")
        .x_label("Asset value")
        .y_label("vega")
        .save("./Draws/Curves/vega_curve.png")?;

    Ok(())
}
