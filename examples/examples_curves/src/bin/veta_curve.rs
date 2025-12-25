use optionstratlib::prelude::*;
use std::error::Error;
use positive::pos_or_panic;

fn get_option(strike: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        *strike,
        ExpirationDate::Days(pos_or_panic!(365.0)),
        pos_or_panic!(0.10),
        Positive::ONE,
        pos_or_panic!(50.0),
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = ConstructionParams::D2 {
        t_start: dec!(25.0),
        t_end: dec!(78.0),
        steps: 100,
    };
    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap());
            let value = option.veta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Veta Curve")
        .x_label("Asset value")
        .y_label("Veta")
        .save("./Draws/Curves/veta_curve.png")?;

    Ok(())
}
