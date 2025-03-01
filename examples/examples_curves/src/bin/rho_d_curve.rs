use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Plottable,
};
use optionstratlib::greeks::Greeks;
use optionstratlib::utils::setup_logger;
use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side, pos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

fn get_option(underlying_price: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        pos!(50.0),
        ExpirationDate::Days(pos!(365.0)),
        pos!(0.10),
        pos!(1.0),
        *underlying_price,
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = ConstructionParams::D2 {
        t_start: dec!(20.0),
        t_end: dec!(80),
        steps: 100,
    };
    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap());
            let value = option.rho_d().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Rho_d Curve")
        .x_label("Asset value")
        .y_label("rho_d")
        .line_width(1)
        .save("./Draws/Curves/rho_d_curve.png")?;

    Ok(())
}
