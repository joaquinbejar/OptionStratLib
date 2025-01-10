use std::error::Error;
use rust_decimal::{Decimal, MathematicalOps};
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::curves::construction::CurveConstructionMethod;
use optionstratlib::curves::visualization::Plottable;
use optionstratlib::utils::setup_logger;

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let curve_sin = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| Ok(Point2D::new(t, t.sin()))),
        t_start: Decimal::ZERO,
        t_end: Decimal::TWO_PI  * Decimal::TWO,
        steps: 100
    })?;

    let curve_cos = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| Ok(Point2D::new(t, t.cos()))),
        t_start: Decimal::ZERO ,
        t_end: Decimal::TWO_PI * Decimal::TWO,
        steps: 100
    })?;
    
    let vector_curve = vec![curve_sin, curve_cos];
    

    vector_curve
        .plot()
        .title("Parametric Curve")
        .x_label("X")
        .y_label("Sin")
        .line_width(1)
        .curve_name(["Sin".to_string(), "Cos".to_string()].to_vec())
        .save("./Draws/Curves/vector_curve.png")?;

    Ok(())
}
