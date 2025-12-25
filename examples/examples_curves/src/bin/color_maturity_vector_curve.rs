use optionstratlib::prelude::*;
use std::error::Error;

fn get_option(underlying_asset: &Positive, maturity: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        pos_or_panic!(50.0), // strike price
        ExpirationDate::Days(*maturity),
        pos_or_panic!(0.2), // implied volatility
        Positive::ONE, // quantity
        *underlying_asset,  // underlying price
        Decimal::ZERO,      // risk free rate
        OptionStyle::Call,
        Positive::ZERO, // dividend yield
        None,           // exotic params
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let params = &ConstructionParams::D2 {
        t_start: dec!(35.0),
        t_end: dec!(65.0),
        steps: 120,
    };

    let dte_60_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(60.0));
            let value = option.color().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let dte_30_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(30.0));
            let value = option.color().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let dte_10_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(10.0));
            let value = option.color().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let dte_5_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(5.0));
            let value = option.color().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let dte_1_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &Positive::ONE);
            let value = option.color().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let dte_05_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos_or_panic!(0.5));
            let value = option.color().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vector_curve = vec![
        dte_60_curve,
        dte_30_curve,
        dte_10_curve,
        dte_5_curve,
        dte_1_curve,
        dte_05_curve,
    ];

    vector_curve
        .plot()
        .title("Color Curve")
        .x_label("Asset value")
        .y_label("Color for different maturities")
        .legend(vec![
            "60 DTE".to_string(),
            "30 DTE".to_string(),
            "10 DTE".to_string(),
            "5 DTE".to_string(),
            "1 DTE".to_string(),
            "0.5 DTE".to_string(),
        ])
        .save("./Draws/Curves/color_maturity_vector_curve.png")?;

    Ok(())
}
