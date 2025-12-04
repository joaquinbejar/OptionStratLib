use optionstratlib::prelude::*;
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
        pos!(1.0),                          // quantity
        pos!(50.0),                         // underlying price
        Decimal::ZERO,                      // risk free rate
        OptionStyle::Call,
        Positive::ZERO,                     // dividend yield
        None,
    )
}
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = ConstructionParams::D3 {
        x_start: dec!(30.0), // Underlying price start
        x_end: dec!(70.0),   // Underlying price end
        y_start: dec!(0.02), // Volatility  start
        y_end: dec!(0.5),    // Volatility price end
        x_steps: 250,        // Number of steps in underlying price
        y_steps: 250,        // Number of steps in strike price
    };

    let parametric_curve = Surface::construct(ConstructionMethod::Parametric {
        f: Box::new(|t: Point2D| {
            let option = get_option(&t);
            let value = option.vanna().unwrap();
            let point = Point3D::new(t.x, t.y, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("Vanna Surface")
        .x_label("Asset value")
        .y_label("Volatility")
        .z_label("Vanna")
        .dimensions(1600, 1200)
        .save("./Draws/Surfaces/vanna_surface.png")?;

    Ok(())
}
