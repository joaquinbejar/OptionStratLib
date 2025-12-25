use optionstratlib::prelude::*;

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

            // Calculate delta
            let delta_value = d1(
                underlying,
                strike,
                dec!(0.05),
                pos_or_panic!(30.0),
                pos_or_panic!(0.2),
            )?;

            // Create a 3D point with underlying price (x), strike price (y), and delta (z)
            Ok(Point3D::new(t.x, t.y, delta_value))
        }),
        params,
    })?;

    // Plot the surface
    delta_surface
        .plot()
        .title("D1 Surface")
        .x_label("Underlying Price")
        .y_label("Strike Price")
        .z_label("d1")
        .dimensions(1600, 900);
    // .save("Draws/Surfaces/d1_surface.png")?;

    delta_surface.write_html("Draws/Surfaces/d1_surface.html".as_ref())?;
    delta_surface.write_png("Draws/Surfaces/d1_surface.png".as_ref())?;
    delta_surface.write_svg("Draws/Surfaces/d1_surface.svg".as_ref())?;
    Ok(())
}
