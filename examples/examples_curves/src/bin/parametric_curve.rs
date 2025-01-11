use optionstratlib::curves::construction::CurveConstructionMethod;
use optionstratlib::curves::visualization::Plottable;
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::utils::setup_logger;
use rust_decimal::{Decimal, MathematicalOps};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let parametric_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| Ok(Point2D::new(t, t.sin()))),
        t_start: Decimal::ZERO,
        t_end: Decimal::TWO_PI,
        steps: 100,
    })?;

    parametric_curve
        .plot()
        .title("Parametric Curve")
        .x_label("X")
        .y_label("Sin")
        .line_width(1)
        .save("./Draws/Curves/parametric_curve.png")?;

    Ok(())
}
