use optionstratlib::prelude::*;
use positive::pos_or_panic;

fn get_option(underlying_price: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        pos_or_panic!(50.0),
        ExpirationDate::Days(pos_or_panic!(365.0)),
        pos_or_panic!(0.10),
        Positive::ONE,
        *underlying_price,
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
            let value = option.rho().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Rho Curve")
        .x_label("Asset value")
        .y_label("rho")
        .save("./Draws/Curves/rho_curve.png")?;

    Ok(())
}
