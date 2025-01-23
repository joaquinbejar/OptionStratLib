use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::{
    ConstructionMethod, ConstructionParams, GeometricObject, Plottable,
};
use optionstratlib::utils::setup_logger;
use rust_decimal::{Decimal, MathematicalOps};
use rust_decimal_macros::dec;
use std::error::Error;
fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = ConstructionParams::D2 {
        t_start: Decimal::ZERO,
        t_end: Decimal::TWO_PI,
        steps: 100,
    };

    let parametric_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t: Decimal| Ok(Point2D::new(t, t.sin()))),
        params: params.clone(),
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
