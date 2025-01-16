use optionstratlib::curves::construction::CurveConstructionMethod;
use optionstratlib::curves::visualization::Plottable;
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::utils::setup_logger;
use std::error::Error;
use rust_decimal_macros::dec;
use optionstratlib::greeks::d2;
use optionstratlib::{pos, Positive};

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let parametric_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| {
            let strike = Positive::new_decimal(t).unwrap();
            let value = d2(
                pos!(50.0),
                strike,
                dec!(0.0),
                pos!(1.0),
                pos!(0.1),
            ).unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        t_start: dec!(1.0),
        t_end: dec!(100),
        steps: 100,
    })?;

    parametric_curve
        .plot()
        .title("d2 Curve")
        .x_label("strike")
        .y_label("d1")
        .line_width(1)
        .save("./Draws/Curves/d2_curve.png")?;

    Ok(())
}
