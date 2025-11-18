use optionstratlib::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};

fn main() -> Result<(), Error> {
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
        .save("./Draws/Curves/parametric_curve.png")?;

    Ok(())
}
