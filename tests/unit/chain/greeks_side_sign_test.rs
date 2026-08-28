/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 28/8/26
******************************************************************************/

//! The acceptance criterion of issue #428: for every strategy implementing
//! `Greeks`, the aggregate must equal the manually signed per-leg sum, for all
//! twelve values. Before the fix only `delta` respected `Side`, so any strategy
//! with a short leg reported the other eleven with the sign of the equivalent
//! long position.

use optionstratlib::greeks::{
    Greeks, alpha, charm, color, delta, gamma, rho, rho_d, theta, vanna, vega, veta, vomma,
};
use optionstratlib::strategies::{BullPutSpread, IronCondor, LongCall, ShortStraddle};
use optionstratlib::{ExpirationDate, Options, assert_decimal_eq};
use positive::{Positive, pos_or_panic};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use std::error::Error;

const EPSILON: Decimal = dec!(1e-20);

/// `Σ greek(leg)`, since each per-leg function already carries its own `Side`.
///
/// This checks the aggregate against the individual functions rather than
/// against an outside reference, so it catches the aggregate drifting from the
/// per-leg path. The genuinely independent assertion in this file is
/// `test_short_premium_strategy_collects_decay`.
fn signed_sum(
    legs: &[&Options],
    greek: fn(&Options) -> Result<Decimal, optionstratlib::error::greeks::GreeksError>,
    name: &str,
) -> Decimal {
    let mut total = Decimal::ZERO;
    for leg in legs {
        let value = match greek(leg) {
            Ok(value) => value,
            Err(e) => panic!("{name} should compute for a leg: {e}"),
        };
        total += value;
    }
    total
}

fn assert_aggregate_matches_legs<T: Greeks>(strategy: &T, label: &str) {
    let legs = match strategy.get_options() {
        Ok(legs) => legs,
        Err(e) => panic!("{label} should expose its legs: {e}"),
    };
    let aggregate = match strategy.greeks() {
        Ok(aggregate) => aggregate,
        Err(e) => panic!("{label} aggregate should compute: {e}"),
    };

    // The per-leg functions are already signed, so summing them unmodified is
    // the signed sum the issue asks for.
    assert_decimal_eq!(aggregate.delta, signed_sum(&legs, delta, "delta"), EPSILON);
    assert_decimal_eq!(aggregate.gamma, signed_sum(&legs, gamma, "gamma"), EPSILON);
    assert_decimal_eq!(aggregate.theta, signed_sum(&legs, theta, "theta"), EPSILON);
    assert_decimal_eq!(aggregate.vega, signed_sum(&legs, vega, "vega"), EPSILON);
    assert_decimal_eq!(aggregate.rho, signed_sum(&legs, rho, "rho"), EPSILON);
    assert_decimal_eq!(aggregate.rho_d, signed_sum(&legs, rho_d, "rho_d"), EPSILON);
    assert_decimal_eq!(aggregate.alpha, signed_sum(&legs, alpha, "alpha"), EPSILON);
    assert_decimal_eq!(aggregate.vanna, signed_sum(&legs, vanna, "vanna"), EPSILON);
    assert_decimal_eq!(aggregate.vomma, signed_sum(&legs, vomma, "vomma"), EPSILON);
    assert_decimal_eq!(aggregate.veta, signed_sum(&legs, veta, "veta"), EPSILON);
    assert_decimal_eq!(aggregate.charm, signed_sum(&legs, charm, "charm"), EPSILON);
    assert_decimal_eq!(aggregate.color, signed_sum(&legs, color, "color"), EPSILON);
}

#[test]
fn test_single_long_leg_aggregate_matches_signed_legs() -> Result<(), Box<dyn Error>> {
    let strategy = LongCall::new(
        "TEST".to_string(),
        pos_or_panic!(105.0),                      // long_call_strike
        ExpirationDate::Days(pos_or_panic!(30.0)), // long_call_expiration
        pos_or_panic!(0.2),                        // implied_volatility
        Positive::TWO,                             // quantity
        Positive::HUNDRED,                         // underlying_price
        dec!(0.05),                                // risk_free_rate
        pos_or_panic!(0.02),                       // dividend_yield
        pos_or_panic!(3.5),                        // premium_long_call
        Positive::ZERO,                            // open_fee
        Positive::ZERO,                            // close_fee
    )?;
    assert_aggregate_matches_legs(&strategy, "long call");
    Ok(())
}

#[test]
fn test_vertical_spread_aggregate_matches_signed_legs() -> Result<(), Box<dyn Error>> {
    let strategy = BullPutSpread::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        pos_or_panic!(0.02),
        Positive::TWO,
        pos_or_panic!(2.0),
        pos_or_panic!(5.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
    )?;
    assert_aggregate_matches_legs(&strategy, "bull put spread");
    Ok(())
}

#[test]
fn test_condor_aggregate_matches_signed_legs() -> Result<(), Box<dyn Error>> {
    let strategy = IronCondor::new(
        "GOLD".to_string(),
        pos_or_panic!(2646.9),
        pos_or_panic!(2725.0),
        pos_or_panic!(2560.0),
        pos_or_panic!(2800.0),
        pos_or_panic!(2500.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.1548),
        dec!(0.05),
        Positive::ZERO,
        Positive::TWO,
        pos_or_panic!(38.8),
        pos_or_panic!(30.4),
        pos_or_panic!(23.3),
        pos_or_panic!(16.8),
        pos_or_panic!(0.96),
        pos_or_panic!(0.96),
    )?;
    assert_aggregate_matches_legs(&strategy, "iron condor");
    Ok(())
}

#[test]
fn test_straddle_aggregate_matches_signed_legs() -> Result<(), Box<dyn Error>> {
    let strategy = ShortStraddle::new(
        "CL".to_string(),
        pos_or_panic!(7138.5),
        pos_or_panic!(7140.0),
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(84.2),
        pos_or_panic!(353.2),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
    )?;
    assert_aggregate_matches_legs(&strategy, "short straddle");
    Ok(())
}

/// A short-premium strategy collects decay, so its aggregate theta is positive.
#[test]
fn test_short_premium_strategy_collects_decay() -> Result<(), Box<dyn Error>> {
    let strategy = ShortStraddle::new(
        "CL".to_string(),
        pos_or_panic!(7138.5),
        pos_or_panic!(7140.0),
        ExpirationDate::Days(pos_or_panic!(45.0)),
        pos_or_panic!(0.3745),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(84.2),
        pos_or_panic!(353.2),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
        pos_or_panic!(7.01),
    )?;
    let greeks = strategy.greeks()?;
    assert!(
        greeks.theta.is_sign_positive(),
        "a short straddle collects decay, so theta must be positive, got {}",
        greeks.theta
    );
    Ok(())
}
