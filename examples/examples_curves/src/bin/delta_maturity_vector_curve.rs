use optionstratlib::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

fn get_option(underlying_asset: &Positive, maturity: &Positive) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "XYZ".parse().unwrap(),
        pos!(50.0),
        ExpirationDate::Days(*maturity),
        pos!(0.1),
        pos!(1.0),
        *underlying_asset,
        Decimal::ZERO,
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    let t_start = dec!(35.0);
    let t_end = dec!(68.0);
    let steps = 100;

    let params = &ConstructionParams::D2 {
        t_start,
        t_end,
        steps,
    };

    let one_month_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(30.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let three_month_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(90.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let six_month_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(180.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let nine_month_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(270.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let twelve_month_curve = Curve::construct(ConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(365.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        params: params.clone(),
    })?;

    let vector_curve = vec![
        one_month_curve,
        three_month_curve,
        six_month_curve,
        nine_month_curve,
        twelve_month_curve,
    ];

    vector_curve
        .plot()
        .title("Deltas Curve")
        .x_label("Asset value")
        .y_label("Deltas for different maturities")
        .legend(vec![
            "1 month",
            "3 months",
            "6 months",
            "9 months",
            "12 months",
        ])
        .save("./Draws/Curves/delta_maturity_vector_curve.png")?;

    Ok(())
}
