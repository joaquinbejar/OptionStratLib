use positive::pos_or_panic;
use optionstratlib::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

fn get_option(underlying_asset: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        pos_or_panic!(50.0),
        ExpirationDate::Days(pos_or_panic!(365.0)),
        pos_or_panic!(0.1),
        Positive::ONE,
        *underlying_asset,
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Error> {
    setup_logger();
    let params = ConstructionParams::D2 {
        t_start: dec!(10.0),
        t_end: dec!(90),
        steps: 100,
    };

    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap());
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Delta Curve")
        .x_label("Asset value")
        .y_label("delta")
        .save("./Draws/Curves/delta_curve.png")?;

    Ok(())
}
