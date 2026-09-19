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

/// The 0.21 surface: inherent pricing methods on `Options` resolve with no
/// trait in scope. These are the forwarding wrappers over `OptionPricing`.
mod inherent_without_trait_import {
    use super::*;
    use std::num::NonZeroUsize;

    #[test]
    fn test_options_inherent_pricing_methods_compile_without_trait_import() {
        let option = sample_option();
        let steps = NonZeroUsize::new(50).expect("non-zero step count");
        let bs = option.calculate_price_black_scholes();
        let binomial = option.calculate_price_binomial(steps);
        let tree = option.calculate_price_binomial_tree(steps);
        let telegraph = option.calculate_price_telegraph(steps);
        let time_value = option.time_value();
        let iv = option.calculate_implied_volatility(dec!(3.0));
        assert!(bs.is_ok());
        assert!(binomial.is_ok());
        assert!(tree.is_ok());
        assert!(telegraph.is_ok());
        assert!(time_value.is_ok());
        assert!(iv.is_ok());
    }

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

/// The 0.22 canonical form: the same methods through the pricing-owned
/// extension trait, imported from its owning module.
mod trait_form_from_owning_module {
    use super::*;
    use optionstratlib::pricing::OptionPricing;
    use std::num::NonZeroUsize;

    #[test]
    fn test_option_pricing_trait_form_matches_inherent_form() {
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
        // The telegraph kernel samples a random process, so two runs do not
        // compare; the trait form only has to resolve and succeed.
        assert!(OptionPricing::calculate_price_telegraph(&option, steps).is_ok());
        assert_eq!(
            OptionPricing::time_value(&option).ok(),
            option.time_value().ok()
        );
        assert_eq!(
            OptionPricing::calculate_implied_volatility(&option, dec!(3.0)).ok(),
            option.calculate_implied_volatility(dec!(3.0)).ok()
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
