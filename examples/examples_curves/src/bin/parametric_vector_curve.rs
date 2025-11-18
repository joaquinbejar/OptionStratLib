use optionstratlib::prelude::*;
use rust_decimal::{Decimal, MathematicalOps};

fn main() -> Result<(), Error> {
    setup_logger();
    let params = &ConstructionParams::D2 {
        t_start: Decimal::ZERO,
        t_end: Decimal::TWO_PI * Decimal::TWO,
        steps: 100,
    };

    let curve_sin = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t: Decimal| Ok(Point2D::new(t, t.sin()))),
        params: params.clone(),
    })?;

    let curve_cos = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t: Decimal| Ok(Point2D::new(t, t.cos()))),
        params: params.clone(),
    })?;

    let vector_curve = vec![curve_sin, curve_cos];

    vector_curve
        .plot()
        .title("Parametric Curve")
        .x_label("X")
        .y_label("Sin")
        .legend(["Sin".to_string(), "Cos".to_string()].to_vec())
        .save("./Draws/Curves/vector_curve.png")?;

    Ok(())
}
