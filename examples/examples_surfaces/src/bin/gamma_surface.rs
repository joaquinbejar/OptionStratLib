use optionstratlib::curves::Point2D;
use optionstratlib::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Plottable,
};
use optionstratlib::greeks::Greeks;
use optionstratlib::surfaces::{Point3D, Surface};
use optionstratlib::utils::setup_logger;
use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Positive, Side, pos};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

fn get_option(point2d: &Point2D) -> Options {
    let strike = Positive::new_decimal(point2d.x).unwrap();
    let volatilitity = Positive::new_decimal(point2d.y).unwrap();

    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        strike,
        ExpirationDate::Days(pos!(365.0)),
        volatilitity,
        pos!(1.0),
        pos!(50.0),
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    let params = ConstructionParams::D3 {
        x_start: dec!(10.0), // Underlying price start
        x_end: dec!(90.0),   // Underlying price end
        y_start: dec!(0.02), // Volatility  start
        y_end: dec!(0.5),    // Volatility price end
        x_steps: 250,        // Number of steps in underlying price
        y_steps: 250,        // Number of steps in strike price
    };

    let parametric_curve = Surface::construct(ConstructionMethod::Parametric {
        f: Box::new(|t: Point2D| {
            let option = get_option(&t);
            let value = option.gamma().unwrap();
            let point = Point3D::new(t.x, t.y, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Gamma Surface")
        .x_label("Asset value")
        .y_label("Volatility")
        .z_label("Gamma")
        .dimensions(1600, 1200)
        .save("./Draws/Surfaces/gamma_surface.png")?;

    Ok(())
}
