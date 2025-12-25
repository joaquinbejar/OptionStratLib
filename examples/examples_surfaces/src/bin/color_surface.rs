use positive::pos_or_panic;
use optionstratlib::prelude::*;
use std::error::Error;

fn get_option(point2d: &Point2D) -> Options {
    let strike = Positive::new_decimal(point2d.y).unwrap();
    let expiration_date = ExpirationDate::Days(Positive::new_decimal(point2d.x).unwrap());

    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        strike,
        expiration_date,
        pos_or_panic!(0.5),   // volatility
        Positive::ONE,   // quantity
        Positive::HUNDRED, // underlying price
        dec!(0.05),           // risk free rate
        OptionStyle::Call,
        Positive::ZERO, // dividend yield
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = ConstructionParams::D3 {
        x_start: dec!(1.0), // Days to maturity start
        x_end: dec!(150.0), // Days to maturity end
        y_start: dec!(1.0), // strike price start
        y_end: dec!(200.0), // strike price end
        x_steps: 250,       // Number of steps in days to maturity
        y_steps: 250,       // Number of steps in strike price
    };

    let parametric_curve = Surface::construct(ConstructionMethod::Parametric {
        f: Box::new(|t: Point2D| {
            let option = get_option(&t);
            let value = option.color().unwrap();
            let point = Point3D::new(t.x, t.y, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Color Surface")
        .x_label("Days to maturity")
        .y_label("Strike price")
        .z_label("Color")
        .dimensions(1600, 1200)
        .save("./Draws/Surfaces/color_surface.png")?;

    Ok(())
}
