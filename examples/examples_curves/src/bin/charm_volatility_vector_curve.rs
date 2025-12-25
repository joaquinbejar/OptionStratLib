use optionstratlib::prelude::*;
use std::error::Error;

fn get_option(strike: &Positive, volatility: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        *strike,
        ExpirationDate::Days(pos_or_panic!(365.0)),
        *volatility,
        Positive::ONE,  // quantity
        pos_or_panic!(50.0), // underlying price
        Decimal::ZERO,       // risk free rate
        OptionStyle::Call,
        Positive::ZERO, // dividend yield
        None,           // exotic params
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = &ConstructionParams::D2 {
        t_start: dec!(20.0),
        t_end: dec!(80.0),
        steps: 100,
    };

    let vol_20_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(0.20));
            let value = option.charm().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vol_10_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(0.10));
            let value = option.charm().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vol_5_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(0.05));
            let value = option.charm().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vector_curve = vec![vol_20_curve, vol_10_curve, vol_5_curve];

    vector_curve
        .plot()
        .title("Charm Curve")
        .x_label("Strike")
        .y_label("Charm for different Volatilities")
        .legend(vec![
            "Volatility 20%".to_string(),
            "Volatility 10%".to_string(),
            "Volatility 5%".to_string(),
        ])
        .save("./Draws/Curves/charm_volatility_vector_curve.png")?;

    Ok(())
}
