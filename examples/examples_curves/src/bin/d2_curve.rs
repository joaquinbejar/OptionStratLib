use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Plottable,
};
use optionstratlib::greeks::d2;
use optionstratlib::utils::setup_logger;
use optionstratlib::{Positive, pos};
use rust_decimal_macros::dec;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let params = ConstructionParams::D2 {
        t_start: dec!(1.0),
        t_end: dec!(100),
        steps: 100,
    };

    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let strike = Positive::new_decimal(t).unwrap();
            let value = d2(pos!(50.0), strike, dec!(0.0), pos!(1.0), pos!(0.1)).unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    parametric_curve
        .plot()
        .title("d2 Curve")
        .x_label("strike")
        .y_label("d2")
        .save("./Draws/Curves/d2_curve.png")?;

    Ok(())
}
