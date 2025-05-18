use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Plottable,
};
use optionstratlib::greeks::{d1, d2};
use optionstratlib::utils::setup_logger;
use optionstratlib::{Positive, pos};
use rust_decimal_macros::dec;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let params = &ConstructionParams::D2 {
        t_start: dec!(1.0),
        t_end: dec!(100),
        steps: 100,
    };

    let d1_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let strike = Positive::new_decimal(t).unwrap();
            let value = d1(pos!(50.0), strike, dec!(0.0), pos!(1.0), pos!(0.1)).unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let d2_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let strike = Positive::new_decimal(t).unwrap();
            let value = d2(pos!(50.0), strike, dec!(0.0), pos!(1.0), pos!(0.1)).unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vector_curve = vec![d1_curve, d2_curve];

    vector_curve
        .plot()
        .title("d1 & d2 Curve")
        .x_label("strike")
        .y_label("d1 & d2")
        .legend(vec!["d1", "d2"])
        .save("./Draws/Curves/d1_d2_curve.png")?;
    Ok(())
}
