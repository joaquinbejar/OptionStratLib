use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::geometrics::{ConstructionMethod, ConstructionParams, GeometricObject, Plottable};
use optionstratlib::greeks::Greeks;
use optionstratlib::utils::setup_logger;
use optionstratlib::{pos, ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

fn get_option(strike: &Positive, volatility: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        *strike,
        ExpirationDate::Days(pos!(365.0)),
        *volatility,
        pos!(1.0),
        pos!(50.0),
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();

    let params = &ConstructionParams::D2 {
        t_start: dec!(25.0),
        t_end: dec!(78.0),
        steps: 100,
    };

    let vol_20_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(0.20));
            let value = option.vega().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vol_10_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(0.10));
            let value = option.vega().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vol_5_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(0.05));
            let value = option.vega().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vector_curve = vec![vol_20_curve, vol_10_curve, vol_5_curve];

    vector_curve
        .plot()
        .title("Vega Curve")
        .x_label("Strike")
        .y_label("vega for different Volatilities")
        .line_width(1)
        .curve_name(vec![
            "Volatility 20%".to_string(),
            "Volatility 10%".to_string(),
            "Volatility 5%".to_string(),
        ])
        .save("./Draws/Curves/vega_volatility_vector_curve.png")?;

    Ok(())
}
