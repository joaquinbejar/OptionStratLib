use optionstratlib::curves::Point2D;
use optionstratlib::error::SurfaceError;
use optionstratlib::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Plottable,
};
use optionstratlib::greeks::delta;
use optionstratlib::model::Options;
use optionstratlib::model::types::{OptionStyle, Side};
use optionstratlib::surfaces::{Point3D, Surface};
use optionstratlib::utils::setup_logger;
use optionstratlib::{ExpirationDate, OptionType, Positive, pos};
use rust_decimal_macros::dec;

fn main() -> Result<(), SurfaceError> {
    setup_logger();

    // Define construction parameters for the surface
    let params = ConstructionParams::D3 {
        x_start: dec!(50.0), // Underlying price start
        x_end: dec!(150.0),  // Underlying price end
        y_start: dec!(50.0), // Strike price start
        y_end: dec!(150.0),  // Strike price end
        x_steps: 250,        // Number of steps in underlying price
        y_steps: 250,        // Number of steps in strike price
    };

    // Create a surface representing delta values
    let delta_surface = Surface::construct(ConstructionMethod::Parametric {
        f: Box::new(move |t: Point2D| {
            // Create option with dynamic underlying and strike prices
            let strike = Positive::new_decimal(t.y).unwrap();
            let underlying = Positive::new_decimal(t.x).unwrap();
            let option = Options::new(
                OptionType::European,
                Side::Long,
                "Example".to_string(),
                strike, // Strike price
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),         // Implied volatility
                Positive::ONE,     // Quantity
                underlying,        // Underlying price
                dec!(0.05),        // Risk-free rate
                OptionStyle::Call, // Option style
                Positive::ZERO,    // Dividend yield
                None,              // Exotic params
            );

            // Calculate delta
            let delta_value = delta(&option)?;

            // Create a 3D point with underlying price (x), strike price (y), and delta (z)
            Ok(Point3D::new(t.x, t.y, delta_value))
        }),
        params,
    })?;

    // Plot the surface
    delta_surface
        .plot()
        .title("Option Delta Surface")
        .x_label("Underlying Price")
        .y_label("Strike Price")
        .z_label("Delta")
        .dimensions(1600, 1200)
        .save("./Draws/Surfaces/delta_surface.png")?;

    Ok(())
}
