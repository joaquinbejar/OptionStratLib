/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! Compile fixtures for the capability split of the core model (#499).
//!
//! Each test module imports exactly what it needs and nothing else, so a
//! regression that makes a core method depend on a trait import, or that
//! drops a trait from its owning module, fails here first.

use optionstratlib::model::Position;
use optionstratlib::{ExpirationDate, OptionStyle, OptionType, Options, Side};
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;

fn sample_option() -> Options {
    Options::new(
        OptionType::European,
        Side::Long,
        "TEST".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.2),
        Positive::ONE,
        pos_or_panic!(100.0),
        dec!(0.05),
        OptionStyle::Call,
        Positive::ZERO,
        None,
    )
}

/// What core keeps after the pricing wrappers are gone: `Position`'s own
/// cost and P&L arithmetic still resolves with no trait in scope, because it
/// is core-owned. `Options`' pricing methods no longer appear here; they
/// require the trait, which `trait_form_from_owning_module` pins.
mod inherent_without_trait_import {
    use super::*;

    #[test]
    fn test_position_inherent_pnl_helpers_compile_without_trait_import() {
        let position = Position::new(
            sample_option(),
            pos_or_panic!(5.0),
            chrono::Utc::now(),
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
            None,
            None,
        );
        assert!(position.total_cost().is_ok());
        assert!(position.premium_received().is_ok());
        assert!(position.net_premium_received().is_ok());
        assert!(position.net_cost().is_ok());
        assert!(position.fees().is_ok());
        assert!(position.break_even().is_some());
        assert!(position.unrealized_pnl(pos_or_panic!(6.0)).is_ok());
        assert!(position.pnl_at_expiration(&None).is_ok());
    }
}

/// The 0.22 canonical form: pricing reaches `Options` only through the
/// pricing-owned extension trait, imported from its owning module. There is
/// no inherent form to compare against any more, which is what lets `model`
/// leave the facade without depending on `pricing` (#499).
mod trait_form_from_owning_module {
    use super::*;
    use optionstratlib::pricing::OptionPricing;
    use optionstratlib::utils::{DETERMINISTIC_RNG_DEFAULT_SEED, deterministic_rng};
    use std::num::NonZeroUsize;

    #[test]
    fn test_every_pricing_method_resolves_through_the_trait() {
        let option = sample_option();
        let steps = NonZeroUsize::new(50).expect("non-zero step count");
        assert!(option.calculate_price_black_scholes().is_ok());
        assert!(option.calculate_price_binomial(steps).is_ok());
        assert!(option.calculate_price_binomial_tree(steps).is_ok());
        assert!(
            option
                .calculate_price_telegraph(
                    steps,
                    &mut deterministic_rng(DETERMINISTIC_RNG_DEFAULT_SEED)
                )
                .is_ok()
        );
        assert!(option.time_value().is_ok());
        assert!(option.calculate_implied_volatility(dec!(3.0)).is_ok());
    }

    #[test]
    fn test_fully_qualified_and_method_syntax_agree() {
        let option = sample_option();
        let steps = NonZeroUsize::new(50).expect("non-zero step count");
        assert_eq!(
            OptionPricing::calculate_price_black_scholes(&option).ok(),
            option.calculate_price_black_scholes().ok()
        );
        assert_eq!(
            OptionPricing::calculate_price_binomial(&option, steps).ok(),
            option.calculate_price_binomial(steps).ok()
        );
        assert_eq!(
            OptionPricing::time_value(&option).ok(),
            option.time_value().ok()
        );
    }

    #[test]
    fn test_option_pricing_trait_is_usable_on_a_generic_bound() {
        fn price<T: OptionPricing>(contract: &T) -> Option<rust_decimal::Decimal> {
            contract.calculate_price_black_scholes().ok()
        }
        let option = sample_option();
        assert_eq!(price(&option), option.calculate_price_black_scholes().ok());
    }
}

/// The prelude keeps carrying the trait, so prelude users see no change.
mod trait_form_from_prelude {
    use optionstratlib::prelude::*;
    use positive::{Positive, pos_or_panic};
    use rust_decimal_macros::dec;

    #[test]
    fn test_option_pricing_trait_is_in_the_prelude() {
        fn price<T: OptionPricing>(contract: &T) -> Option<rust_decimal::Decimal> {
            contract.calculate_price_black_scholes().ok()
        }
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(100.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(100.0),
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );
        assert!(price(&option).is_some());
    }
}

/// Trait impls that moved to their owning layer keep resolving for callers
/// that import the trait, exactly as in 0.21.
mod moved_trait_impls_still_resolve {
    use super::*;
    use optionstratlib::greeks::Greeks;
    use optionstratlib::pnl::PnLCalculator;
    use optionstratlib::strategies::base::BasicAble;
    use optionstratlib::visualization::Graph;

    #[test]
    fn test_greeks_pnl_basic_able_and_graph_resolve_on_options_and_position() {
        let option = sample_option();
        let position = Position::new(
            option.clone(),
            pos_or_panic!(5.0),
            chrono::Utc::now(),
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
            None,
            None,
        );
        assert!(option.delta().is_ok());
        assert!(position.delta().is_ok());
        assert!(
            option
                .calculate_pnl_at_expiration(&pos_or_panic!(110.0))
                .is_ok()
        );
        assert!(
            position
                .calculate_pnl_at_expiration(&pos_or_panic!(110.0))
                .is_ok()
        );
        assert_eq!(option.get_title(), position.get_title());
        let _ = option.graph_data();
        let _ = position.graph_config();
    }
}

/// The leg Greeks and a trade's P&L reach the core types through their owning
/// layers (#498): `greeks::LegGreeks` for the legs, `PnL::from(&Trade)` for a
/// trade. Core keeps the data; nothing here needs `model` to import an upper
/// layer.
mod leg_greeks_and_trade_pnl_from_owning_layers {
    use super::*;
    use optionstratlib::greeks::LegGreeks;
    use optionstratlib::model::leg::{Leg, SpotPosition};
    use optionstratlib::model::types::Action;
    use optionstratlib::model::{Trade, TradeStatus};
    use optionstratlib::pnl::PnL;

    #[test]
    fn test_leg_greeks_resolve_through_the_pricing_trait() {
        let spot = SpotPosition::long("AAPL".to_string(), Positive::HUNDRED, pos_or_panic!(150.0));
        let leg = Leg::spot(spot.clone());
        assert_eq!(spot.delta().ok(), Some(dec!(100)));
        assert_eq!(leg.delta().ok(), Some(dec!(100)));
        assert_eq!(leg.gamma().ok(), Some(rust_decimal::Decimal::ZERO));
    }

    #[test]
    fn test_trade_pnl_comes_from_the_pnl_layer() {
        let trade = Trade::new(
            uuid::Uuid::new_v4(),
            Action::Sell,
            Side::Short,
            OptionStyle::Put,
            pos_or_panic!(0.25),
            None,
            pos_or_panic!(200.0),
            chrono::Utc::now(),
            Positive::ONE,
            pos_or_panic!(3.0),
            pos_or_panic!(190.0),
            None,
            TradeStatus::Open,
        );
        let pnl = PnL::from(&trade);
        assert_eq!(pnl.realized, Some(trade.net()));
    }
}
