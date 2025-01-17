use optionstratlib::curves::construction::CurveConstructionMethod;
use optionstratlib::curves::visualization::Plottable;
use optionstratlib::curves::{Curve, Point2D};
use optionstratlib::greeks::Greeks;
use optionstratlib::utils::setup_logger;
use optionstratlib::{pos, ExpirationDate, OptionStyle, OptionType, Options, Positive, Side};
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

    let one_month_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(30.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        t_start,
        t_end,
        steps,
    })?;

    let three_month_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(90.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        t_start,
        t_end,
        steps,
    })?;

    let six_month_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(180.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        t_start,
        t_end,
        steps,
    })?;

    let nine_month_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(270.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        t_start,
        t_end,
        steps,
    })?;

    let twelve_month_curve = Curve::construct(CurveConstructionMethod::Parametric {
        f: Box::new(|t| {
            let option = get_option(&Positive::new_decimal(t).unwrap(), &pos!(365.0));
            let value = option.delta().unwrap();
            let point = Point2D::new(t, value);
            Ok(point)
        }),
        t_start,
        t_end,
        steps,
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
        .line_width(1)
        .curve_name(vec![
            "1 month".to_string(),
            "3 months".to_string(),
            "6 months".to_string(),
            "9 months".to_string(),
            "12 months".to_string(),
        ])
        .save("./Draws/Curves/delta_maturity_vector_curve.png")?;

    Ok(())
}
