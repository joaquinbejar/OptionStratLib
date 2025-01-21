use optionstratlib::curves::visualization::Plottable;
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::greeks::{d1, d2};
use optionstratlib::utils::setup_logger;
use optionstratlib::{pos, Positive};
use rust_decimal_macros::dec;
use std::error::Error;
use optionstratlib::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
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
        .line_width(1)
        .curve_name(["d1".to_string(), "d2".to_string()].to_vec())
        .save("./Draws/Curves/d1_d2_curve.png")?;

    Ok(())
}
