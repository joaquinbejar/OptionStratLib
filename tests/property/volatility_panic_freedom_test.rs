//! Property-based tests for panic freedom across the volatility module.
//!
//! The library is embedded in long-running services, where a panic kills the
//! worker thread and takes the in-flight request with it. Every failure must
//! therefore come back as a `Result`, including for inputs that are extreme
//! but structurally valid.
//!
//! Volatility fails along three axes, and all three are driven here. The
//! numeric axis is the usual `Decimal` domain: returns at the smallest
//! representable scale, at `Decimal::MAX` and `Decimal::MIN`, and the
//! ordinary values between. The structural axis is the shape of the sample:
//! an empty slice (the GARCH recursion has no seed), a single observation
//! (the sample variance divides by `n - 1`), and a window wider than the
//! sample or of width zero (`slice::windows` has no meaning at zero and
//! panics rather than yielding nothing). The third axis is the timeframe:
//! `TimeFrame::Custom` carries an arbitrary `Positive`, so the periods per
//! year that every square-root-of-time rescaling divides by can be zero,
//! `1e-27`, or `Positive::MAX`.
//!
//! The assertion is deliberately weak: whatever comes back, it must come
//! back.

use optionstratlib::utils::time::TimeFrame;
use optionstratlib::volatility::{
    adjust_volatility, annualized_volatility, calculate_iv, constant_volatility,
    de_annualized_volatility, ewma_volatility, garch_volatility, generate_ou_process,
    historical_volatility, implied_volatility, simulate_heston_volatility,
    uncertain_volatility_bounds, volatility_for_dt,
};
use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Side};
use positive::Positive;
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`. As a number of periods per year it
/// makes the annualisation ratio overflow; as a return it makes the squared
/// deviation underflow to zero.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// Returns that reach the arithmetic failures: the edges of the `Decimal`
/// range, the smallest representable scale, and the ordinary values between.
fn extreme_return() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(TINY),
        Just(-TINY),
        Just(dec!(0.01)),
        Just(dec!(-0.02)),
        Just(dec!(1)),
        Just(dec!(1000000)),
        Just(dec!(-1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

/// `Positive` values at the edges of the domain. `Positive` is `>= 0`, so
/// `Positive::ZERO` is a legitimate divisor to hand to a rescaling.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(Positive::new_decimal(TINY).unwrap_or(Positive::ZERO)),
        Just(Positive::new_decimal(dec!(0.2)).unwrap_or(Positive::ZERO)),
        Just(Positive::new_decimal(dec!(100000000000000)).unwrap_or(Positive::ZERO)),
        Just(Positive::MAX),
    ]
}

/// Every named timeframe plus the `Custom` ones whose periods per year are
/// the extremes the square-root-of-time rule divides by.
fn extreme_timeframe() -> impl Strategy<Value = TimeFrame> {
    prop_oneof![
        Just(TimeFrame::Microsecond),
        Just(TimeFrame::Millisecond),
        Just(TimeFrame::Second),
        Just(TimeFrame::Minute),
        Just(TimeFrame::Hour),
        Just(TimeFrame::Day),
        Just(TimeFrame::Week),
        Just(TimeFrame::Month),
        Just(TimeFrame::Quarter),
        Just(TimeFrame::Year),
        extreme_positive().prop_map(TimeFrame::Custom),
    ]
}

/// The structural shapes a return sample can take, each paired with the
/// numeric extreme the strategy draws.
fn degenerate_returns() -> impl Strategy<Value = Vec<Decimal>> {
    (extreme_return(), extreme_return(), 0usize..7).prop_map(|(a, b, shape)| match shape {
        // Empty: the GARCH seed has no first element, the EWMA no initial
        // variance, and the sample mean divides by zero.
        0 => vec![],
        // A single observation: the sample variance divides by `n - 1`.
        1 => vec![a],
        // Two observations, the minimum the sample variance accepts.
        2 => vec![a, b],
        // Flat: zero variance, zero standard deviation.
        3 => vec![a; 4],
        // Alternating extremes: the centred deviations span the whole range.
        4 => vec![a, b, a, b],
        // Ordinary returns with one extreme spliced in.
        5 => vec![dec!(0.01), a, dec!(-0.02), b, dec!(0.015)],
        // Longer sample, so the recursions run for several steps.
        _ => vec![a, b, dec!(0.01), a, dec!(-0.01), b, dec!(0.02), a],
    })
}

fn option_with(
    strike: Positive,
    underlying: Positive,
    volatility: Positive,
    days: Positive,
    rate: Decimal,
    style: OptionStyle,
) -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "PROBE".to_string(),
        strike,
        ExpirationDate::Days(days),
        volatility,
        Positive::ONE,
        underlying,
        rate,
        style,
        Positive::ZERO,
        None,
    )
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The sample estimators return for every degenerate sample shape and
    /// every extreme decay factor. `historical_volatility` is driven at
    /// window zero, which `slice::windows` rejects by aborting, and at a
    /// window wider than the sample.
    #[test]
    fn test_sample_estimators_never_panic(
        returns in degenerate_returns(),
        lambda in extreme_return(),
        window in 0usize..6,
    ) {
        let _ = constant_volatility(&returns);
        let _ = historical_volatility(&returns, window);
        let _ = historical_volatility(&returns, usize::MAX);
        let _ = ewma_volatility(&returns, lambda);
        let _ = garch_volatility(&returns, lambda, lambda, lambda);
    }

    /// The GARCH recursion returns for every combination of extreme
    /// parameters, including the ones that drive the variance out of the
    /// representable range on the first step.
    #[test]
    fn test_garch_volatility_never_panics(
        returns in degenerate_returns(),
        omega in extreme_return(),
        alpha in extreme_return(),
        beta in extreme_return(),
    ) {
        let _ = garch_volatility(&returns, omega, alpha, beta);
    }

    /// The Heston Euler step returns for every extreme parameter set. Each
    /// factor of `v + κ(θ − v)dt + ξ√v dW` can overflow on its own, so the
    /// same draw is applied to all of them and to `dt` independently.
    #[test]
    fn test_simulate_heston_volatility_never_panics(
        param in extreme_return(),
        dt in extreme_return(),
        steps in 0usize..6,
    ) {
        let _ = simulate_heston_volatility(param, param, param, param, dt, steps);
        let _ = simulate_heston_volatility(param, dec!(0.04), dec!(0.3), dec!(0.04), dt, steps);
    }

    /// The Ornstein-Uhlenbeck path returns for every extreme parameter,
    /// including a zero time step and a level at `Positive::MAX`.
    #[test]
    fn test_generate_ou_process_never_panics(
        level in extreme_positive(),
        dt in extreme_positive(),
        steps in 0usize..6,
    ) {
        let _ = generate_ou_process(level, level, level, level, dt, steps);
        let _ = generate_ou_process(
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            Positive::ONE,
            dt,
            steps,
        );
    }

    /// Every square-root-of-time rescaling returns for every timeframe pair,
    /// including the `Custom` timeframes whose periods per year are zero
    /// (the divisor of the de-annualisation) and `1e-27` (which makes the
    /// period ratio overflow).
    #[test]
    fn test_timeframe_rescaling_never_panics(
        volatility in extreme_positive(),
        from_frame in extreme_timeframe(),
        to_frame in extreme_timeframe(),
    ) {
        let _ = annualized_volatility(volatility, from_frame);
        let _ = de_annualized_volatility(volatility, from_frame);
        let _ = adjust_volatility(volatility, from_frame, to_frame);
        let _ = volatility_for_dt(volatility, Positive::ONE, from_frame, to_frame);
    }

    /// The implied-volatility grid search returns for every extreme option
    /// geometry and for the iteration counts at which the grid size
    /// `100 * max_iterations` overflows `i64`.
    #[test]
    fn test_implied_volatility_never_panics(
        strike in extreme_positive(),
        underlying in extreme_positive(),
        days in extreme_positive(),
        market_price in extreme_positive(),
    ) {
        let mut option = option_with(
            strike,
            underlying,
            Positive::new_decimal(dec!(0.2)).unwrap_or(Positive::ZERO),
            days,
            dec!(0.05),
            OptionStyle::Call,
        );
        let _ = implied_volatility(market_price, &mut option, 2);
        for max_iterations in [i64::MIN, -1, 0, 1, i64::MAX] {
            let _ = implied_volatility(market_price, &mut option, max_iterations);
        }
        let _ = calculate_iv(
            market_price,
            strike,
            OptionStyle::Put,
            underlying,
            days,
            "PROBE".to_string(),
        );
    }

    /// The uncertain-volatility bounds return for every extreme option
    /// geometry and every volatility band, including one whose ends are
    /// `Positive::ZERO` and `Positive::MAX`.
    #[test]
    fn test_uncertain_volatility_bounds_never_panics(
        strike in extreme_positive(),
        underlying in extreme_positive(),
        days in extreme_positive(),
        min_volatility in extreme_positive(),
        max_volatility in extreme_positive(),
    ) {
        let option = option_with(
            strike,
            underlying,
            Positive::new_decimal(dec!(0.2)).unwrap_or(Positive::ZERO),
            days,
            dec!(0.05),
            OptionStyle::Call,
        );
        let _ = uncertain_volatility_bounds(&option, min_volatility, max_volatility);
        let _ = uncertain_volatility_bounds(&option, Positive::ZERO, Positive::MAX);
    }
}
