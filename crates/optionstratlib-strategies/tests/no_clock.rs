//! Building strategies with and without the wall clock (#944).

use chrono::{DateTime, Duration, TimeZone, Utc};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_strategies::error::StrategyError;
use optionstratlib_strategies::strategies::base::Positionable;
use optionstratlib_strategies::strategies::{BullCallSpread, IronCondor, LongCall, LongStraddle};
use rust_decimal_macros::dec;

fn date() -> DateTime<Utc> {
    Utc.with_ymd_and_hms(2030, 1, 1, 9, 30, 0).unwrap()
}

fn spread(date: DateTime<Utc>) -> Result<BullCallSpread, StrategyError> {
    BullCallSpread::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(95.0),
        pos_or_panic!(105.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(8.0),
        pos_or_panic!(3.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        date,
    )
}

#[test]
fn test_new_at_opens_every_leg_at_the_given_date() {
    let strategy = spread(date()).unwrap();
    assert_eq!(strategy.long_call.date, date());
    assert_eq!(strategy.short_call.date, date());
}

#[test]
fn test_new_at_is_deterministic() {
    let later = date() + Duration::days(1);
    let (first, second) = (spread(later).unwrap(), spread(later).unwrap());
    assert_eq!(first.break_even_points, second.break_even_points);
    assert_eq!(first.long_call, second.long_call);
}

#[test]
fn test_new_at_builds_single_leg_multi_leg_and_condor_strategies() {
    let expiration = ExpirationDate::Days(pos_or_panic!(30.0));
    let long_call = LongCall::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        expiration,
        pos_or_panic!(0.2),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        Positive::ZERO,
        Positive::ZERO,
        date(),
    )
    .unwrap();
    let legs = long_call.get_positions().unwrap();
    assert_eq!(legs.len(), 1);
    assert_eq!(legs[0].date, date());

    let straddle = LongStraddle::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        Positive::HUNDRED,
        expiration,
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(5.0),
        pos_or_panic!(5.0),
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        Positive::ZERO,
        date(),
    )
    .unwrap();
    assert_eq!(straddle.long_put.date, date());

    let condor = IronCondor::new_at(
        "TEST".to_string(),
        Positive::HUNDRED,
        pos_or_panic!(110.0),
        pos_or_panic!(90.0),
        pos_or_panic!(120.0),
        pos_or_panic!(80.0),
        expiration,
        pos_or_panic!(0.2),
        dec!(0.05),
        Positive::ZERO,
        Positive::ONE,
        pos_or_panic!(2.0),
        pos_or_panic!(2.0),
        pos_or_panic!(1.0),
        pos_or_panic!(1.0),
        Positive::ZERO,
        Positive::ZERO,
        date(),
    )
    .unwrap();
    assert_eq!(condor.long_put.date, date());
}

#[cfg(not(feature = "clock"))]
mod without_clock {
    use super::*;
    use optionstratlib_core::error::ClockError;
    use optionstratlib_core::model::Options;
    use optionstratlib_core::model::Position;
    use optionstratlib_core::model::{OptionStyle, OptionType, Side};
    use optionstratlib_core::spos;
    use optionstratlib_core::utils::time::now;
    use optionstratlib_market::chains::OptionChain;
    use optionstratlib_market::chains::StrategyLegs;
    use optionstratlib_market::chains::utils::{
        FindOptimalSide, OptionChainBuildParams, OptionDataPriceParams,
    };
    use optionstratlib_strategies::error::AdjustmentError;
    use optionstratlib_strategies::strategies::StrategyConstructor;
    use optionstratlib_strategies::strategies::base::Optimizable;
    use optionstratlib_strategies::strategies::delta_neutral::{
        AdjustmentConfig, AdjustmentOptimizer, AdjustmentTarget,
    };
    use optionstratlib_strategies::strategies::utils::OptimizationCriteria;
    use rust_decimal::Decimal;

    /// A 21-strike chain, built from the reference datetime, so the search is parallel.
    fn chain() -> OptionChain {
        chain_expiring(ExpirationDate::Days(pos_or_panic!(30.0)))
    }

    fn chain_expiring(expiration: ExpirationDate) -> OptionChain {
        ExpirationDate::set_reference_datetime(Some(date()));
        let built = OptionChain::build_chain(&OptionChainBuildParams::new(
            "TEST".to_string(),
            None,
            10,
            spos!(1.0),
            dec!(-0.3),
            Decimal::ZERO,
            pos_or_panic!(0.02),
            2,
            OptionDataPriceParams::new(
                Some(Box::new(Positive::HUNDRED)),
                Some(expiration),
                Some(dec!(0.05)),
                spos!(0.02),
                Some("TEST".to_string()),
            ),
            pos_or_panic!(0.17),
        ));
        ExpirationDate::set_reference_datetime(None);
        built.unwrap()
    }

    /// The optimiser builds each candidate with `create_strategy` and drops what fails, so
    /// a constructor that needed the clock used to leave it with nothing, silently.
    #[test]
    fn test_create_strategy_opens_at_the_date_of_the_strategy() {
        let chain = chain();
        let strategy = spread(date()).unwrap();
        let options: Vec<_> = chain.options.iter().collect();
        let legs = StrategyLegs::TwoLegs {
            first: options[5],
            second: options[15],
        };
        let created = strategy.create_strategy(&chain, &legs).unwrap();
        assert_eq!(created.long_call.date, date());
    }

    #[test]
    fn test_filter_combinations_and_find_optimal_find_candidates() {
        let chain = chain();
        let mut strategy = spread(date()).unwrap();
        assert!(
            strategy
                .filter_combinations(&chain, FindOptimalSide::All)
                .count()
                > 0
        );
        strategy
            .find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio)
            .unwrap();
        assert_eq!(strategy.long_call.date, date());
    }

    fn long_call_leg() -> Vec<Position> {
        vec![Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos_or_panic!(5.0),
            date(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )]
    }

    /// Candidates for a new leg used to vanish: with no date, `get_position` asks the clock.
    #[test]
    fn test_adjustment_optimizer_finds_new_leg_candidates() {
        let chain = chain();
        let positions = long_call_leg();
        let config = AdjustmentConfig {
            prefer_existing_legs: false,
            ..AdjustmentConfig::default()
        };
        let plan = AdjustmentOptimizer::with_chain(
            &positions,
            &chain,
            config,
            AdjustmentTarget::delta_neutral(),
        )
        .optimize()
        .unwrap();
        assert!(!plan.actions.is_empty());
    }

    /// With no time to measure the chain from, every candidate would be dropped and the
    /// plan come back empty: an unhedged delta reported as a success.
    #[test]
    fn test_adjustment_optimizer_on_a_datetime_chain_without_a_time_is_an_error() {
        let chain = chain_expiring(ExpirationDate::DateTime(date() + Duration::days(30)));
        let positions = long_call_leg();
        let optimizer = AdjustmentOptimizer::with_chain(
            &positions,
            &chain,
            AdjustmentConfig::default(),
            AdjustmentTarget::delta_neutral(),
        );
        assert!(matches!(
            optimizer.optimize(),
            Err(AdjustmentError::GreeksError(_))
        ));

        ExpirationDate::set_reference_datetime(Some(date()));
        let plan = optimizer.optimize();
        ExpirationDate::set_reference_datetime(None);
        assert!(!plan.unwrap().actions.is_empty());
    }

    /// Fails if anything in this configuration turned the clock on.
    #[test]
    fn test_no_dependency_turns_the_clock_on() {
        assert_eq!(now(), Err(ClockError::Unavailable));
    }

    #[test]
    fn test_new_without_a_clock_is_a_typed_error() {
        let result = LongCall::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(5.0),
            Positive::ZERO,
            Positive::ZERO,
        );
        assert!(matches!(
            result,
            Err(StrategyError::Clock(ClockError::Unavailable))
        ));
    }

    /// Rebuilding from positions keeps the date each leg already carries.
    #[test]
    fn test_get_strategy_keeps_the_date_of_the_legs() {
        let strategy = spread(date()).unwrap();
        let positions: Vec<_> = strategy
            .get_positions()
            .unwrap()
            .into_iter()
            .cloned()
            .collect();
        let rebuilt = BullCallSpread::get_strategy(&positions).unwrap();
        assert_eq!(rebuilt.long_call.date, date());
        assert_eq!(rebuilt.short_call.date, date());
    }
}
