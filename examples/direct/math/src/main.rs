//! Curves and interpolation with `optionstratlib-math` alone (#555).
//!
//! A volatility smile (strike against implied volatility) is a `Curve` of
//! decimal points; the crate reads values between its points with linear,
//! cubic and spline interpolation. Nothing else is compiled: the crate knows
//! about no option, chain, Greek, simulation or chart, only about
//! `optionstratlib-core` and general numeric crates. Prices and Greeks
//! belong to `optionstratlib-pricing`, plotting to
//! `optionstratlib-visualization`.

use optionstratlib_math::curves::{Curve, Point2D};
use optionstratlib_math::geometrics::{Interpolate, InterpolationType};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::collections::BTreeSet;
use std::error::Error;
use tracing::info;

/// Five (strike, volatility) points of a smile, lowest at the money.
fn smile() -> Curve {
    let points: BTreeSet<Point2D> = [
        (dec!(90), dec!(0.28)),
        (dec!(95), dec!(0.24)),
        (dec!(100), dec!(0.20)),
        (dec!(105), dec!(0.22)),
        (dec!(110), dec!(0.26)),
    ]
    .into_iter()
    .map(|(strike, volatility)| Point2D::new(strike, volatility))
    .collect();
    Curve::new(points)
}

/// The smile read at `strike` with one interpolation method.
fn volatility_at(
    curve: &Curve,
    strike: Decimal,
    method: InterpolationType,
) -> Result<Decimal, Box<dyn Error>> {
    Ok(curve.interpolate(strike, method)?.y)
}

fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt().init();
    let curve = smile();
    for (name, method) in [
        ("linear", InterpolationType::Linear),
        ("cubic", InterpolationType::Cubic),
        ("spline", InterpolationType::Spline),
    ] {
        info!(
            "{name} volatility at strike 102.5: {:.4}",
            volatility_at(&curve, dec!(102.5), method)?
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interpolating_the_smile_between_two_strikes() -> Result<(), Box<dyn Error>> {
        let curve = smile();
        // Linear: halfway between 0.20 at 100 and 0.22 at 105.
        assert_eq!(
            volatility_at(&curve, dec!(102.5), InterpolationType::Linear)?,
            dec!(0.21)
        );
        // The smoother methods stay near it, and at a node every method
        // returns the sample.
        for method in [InterpolationType::Cubic, InterpolationType::Spline] {
            let value = volatility_at(&curve, dec!(102.5), method)?;
            assert!(
                value > dec!(0.19) && value < dec!(0.23),
                "{method:?}: {value}"
            );
            assert_eq!(volatility_at(&curve, dec!(100), method)?, dec!(0.20));
        }
        Ok(())
    }
}
