/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! [`Greeks`] implementations for the core model types.
//!
//! `Options` and `Position` expose the Greeks by handing the trait a view of
//! their single underlying contract. The implementations live next to the
//! trait (local trait, core-owned type) so the core model never references
//! the Greeks kernels.

use crate::error::GreeksError;
use crate::greeks::Greeks;
use optionstratlib_core::model::{Options, Position};

impl Greeks for Options {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![self])
    }
}

/// Implementation of the `Greeks` trait for the `Position` struct.
///
/// This implementation allows a `Position` to calculate option Greeks (delta, gamma,
/// theta, vega, rho, etc.) by accessing its underlying option contract. The implementation
/// provides a way to expose the position's option for use in Greek calculations.
///
impl Greeks for Position {
    /// Returns a vector containing a reference to the option contract associated with this position.
    ///
    /// This method satisfies the `Greeks` trait requirement by providing access to the
    /// option contract that will be used for calculating various Greek values.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<&Options>)` - A vector containing a reference to the position's underlying option
    /// - `Err(GreeksError)` - If there is an error accessing the option data
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.option])
    }
}

#[cfg(test)]
mod tests_greeks {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::types::{OptionStyle, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest;
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_delta() {
        let delta = create_sample_option_simplest(OptionStyle::Call, Side::Long)
            .delta()
            .unwrap();
        assert_decimal_eq!(delta, dec!(0.5338307582207135564475476937), EPSILON);
    }

    #[test]
    fn test_delta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.delta().unwrap(),
            dec!(1.0676615164414271128950953874),
            EPSILON
        );
    }

    #[test]
    fn test_gamma() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.gamma().unwrap(),
            dec!(0.0692632117482215620683508231),
            EPSILON
        );
    }

    #[test]
    fn test_gamma_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.gamma().unwrap(),
            dec!(0.1385264234964431241367016462),
            EPSILON
        );
    }

    #[test]
    fn test_theta() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.theta().unwrap(),
            dec!(-0.0434671314177636287945041349),
            EPSILON
        );
    }

    #[test]
    fn test_theta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.theta().unwrap(),
            dec!(-0.0869342628355272575890082697),
            EPSILON
        );
    }

    #[test]
    fn test_vega() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.vega().unwrap(),
            dec!(0.1138573343806381728362131205),
            EPSILON
        );
    }

    #[test]
    fn test_vega_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.vega().unwrap(),
            dec!(0.2277146687612763456724262410),
            EPSILON
        );
    }

    #[test]
    fn test_rho() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.rho().unwrap(),
            dec!(0.041863419880440417503050762),
            EPSILON
        );
    }

    #[test]
    fn test_rho_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.rho().unwrap(),
            dec!(0.0837268397608808350061015239),
            EPSILON
        );
    }

    #[test]
    fn test_rho_d() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.rho_d().unwrap(),
            dec!(-0.0438765006756750824436552223),
            EPSILON
        );
    }

    #[test]
    fn test_rho_d_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.rho_d().unwrap(),
            dec!(-0.0877530013513501648873104446),
            EPSILON
        );
    }

    #[test]
    fn test_vanna() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.vanna().unwrap(),
            dec!(-0.0569286671903190864181065602),
            EPSILON
        );
    }

    #[test]
    fn test_vanna_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.vanna().unwrap(),
            dec!(-0.1138573343806381728362131204),
            EPSILON
        );
    }

    #[test]
    fn test_vomma() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.vomma().unwrap(),
            dec!(0.0014037205608571828124031742),
            EPSILON
        );
    }

    #[test]
    fn test_vomma_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.vomma().unwrap(),
            dec!(0.0028074411217143656248063484),
            EPSILON
        );
    }

    #[test]
    fn test_veta() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(option.veta().unwrap(), dec!(0.000027206189), EPSILON);
    }

    #[test]
    fn test_veta_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(option.veta().unwrap(), dec!(0.000054412378), EPSILON);
    }

    #[test]
    fn test_vomma_and_veta_scale_linearly_in_quantity() {
        // `vega` already carries the position size, so `vomma` and `veta` must
        // not apply it a second time. Both are first derivatives of vega and
        // are therefore linear, not quadratic, in quantity.
        let single = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let vomma_per_lot = single.vomma().unwrap();
        let veta_per_lot = single.veta().unwrap();

        for lots in [2u64, 5, 10] {
            let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
            option.quantity = pos_or_panic!(lots as f64);
            let scale = Decimal::from(lots);
            assert_decimal_eq!(option.vomma().unwrap(), vomma_per_lot * scale, EPSILON);
            assert_decimal_eq!(option.veta().unwrap(), veta_per_lot * scale, EPSILON);
        }
    }

    #[test]
    fn test_charm() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.charm().unwrap(),
            dec!(-0.0005546611716779658921659644),
            EPSILON
        );
    }

    #[test]
    fn test_charm_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.charm().unwrap(),
            dec!(-0.0011093223433559317843319287),
            EPSILON
        );
    }

    #[test]
    fn test_color() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_decimal_eq!(
            option.color().unwrap(),
            dec!(-0.0011648237847885846501315451),
            EPSILON
        );
    }

    #[test]
    fn test_color_size() {
        let mut option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        option.quantity = Positive::TWO;
        assert_decimal_eq!(
            option.color().unwrap(),
            dec!(-0.0023296475695771693002630901),
            EPSILON
        );
    }
}

#[cfg(test)]
mod tests_greek_trait {
    use super::*;
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest;
    use optionstratlib_core::{model::Positive, pos_or_panic};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    const EPSILON: Decimal = dec!(1e-6);

    #[test]
    fn test_greeks_implementation() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let greeks = option.greeks().unwrap();

        assert_decimal_eq!(greeks.delta, option.delta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.gamma, option.gamma().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.theta, option.theta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.vega, option.vega().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.rho, option.rho().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.rho_d, option.rho_d().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.vanna, option.vanna().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.vomma, option.vomma().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.veta, option.veta().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.charm, option.charm().unwrap(), EPSILON);
        assert_decimal_eq!(greeks.color, option.color().unwrap(), EPSILON);
    }

    #[test]
    fn test_greeks_consistency() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let greeks = option.greeks().unwrap();

        assert!(
            greeks.delta >= Decimal::NEGATIVE_ONE && greeks.delta <= Decimal::ONE,
            "Delta should be between -1 and 1"
        );
        assert!(
            greeks.gamma >= Decimal::ZERO,
            "Gamma should be non-negative"
        );
        assert!(greeks.vega >= Decimal::ZERO, "Vega should be non-negative");
    }

    #[test]
    fn test_greeks_for_different_options() {
        let call_option = Options::new(
            OptionType::European,
            Side::Long,
            "TEST".to_string(),
            pos_or_panic!(5790.0), // strike
            ExpirationDate::Days(pos_or_panic!(18.0)),
            pos_or_panic!(0.1),     // initial iv
            Positive::ONE,          // qty
            pos_or_panic!(5781.88), // underlying
            dec!(0.05),             // rate
            OptionStyle::Call,
            Positive::ZERO, // div
            None,
        );
        let mut put_option = call_option.clone();
        put_option.option_style = OptionStyle::Put;

        let call_greeks = call_option.greeks().unwrap();
        let put_greeks = put_option.greeks().unwrap();

        assert_decimal_eq!(
            call_greeks.delta + put_greeks.delta.abs(),
            Decimal::ONE,
            EPSILON
        );
        assert_decimal_eq!(call_greeks.gamma, put_greeks.gamma, EPSILON);
        assert_decimal_eq!(call_greeks.vega, put_greeks.vega, EPSILON);
    }
}

#[cfg(test)]
mod tests_greeks_contract_size {
    use super::*;
    use optionstratlib_core::model::types::{BinaryType, OptionStyle, OptionType, Side};
    use optionstratlib_core::model::utils::create_sample_option_simplest;
    use optionstratlib_core::model::{ExpirationDate, Positive};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    type Calculator = fn(&Options) -> Result<Decimal, GreeksError>;

    /// Every per-position greek, which must scale by `quantity × contract_size`.
    const SCALED: [(&str, Calculator); 11] = [
        ("delta", |o| o.delta()),
        ("gamma", |o| o.gamma()),
        ("theta", |o| o.theta()),
        ("vega", |o| o.vega()),
        ("rho", |o| o.rho()),
        ("rho_d", |o| o.rho_d()),
        ("vanna", |o| o.vanna()),
        ("vomma", |o| o.vomma()),
        ("veta", |o| o.veta()),
        ("charm", |o| o.charm()),
        ("color", |o| o.color()),
    ];

    fn value(label: &str, result: Result<Decimal, GreeksError>) -> Decimal {
        match result {
            Ok(value) => value,
            Err(e) => panic!("{label} failed: {e}"),
        }
    }

    fn assert_scales(one: &Options) {
        let sized = one.clone().with_contract_size(Positive::HUNDRED);
        let mut hundred_contracts = one.clone();
        hundred_contracts.quantity = Positive::HUNDRED;
        for (label, calculate) in SCALED {
            let one_value = value(label, calculate(one));
            let sized_value = value(label, calculate(&sized));
            // One contract of 100 units is the same position as 100 of one.
            assert_eq!(
                sized_value,
                value(label, calculate(&hundred_contracts)),
                "{label}: contract size and quantity disagree"
            );
            let expected = one_value * Decimal::ONE_HUNDRED;
            assert!(
                (sized_value - expected).abs() <= dec!(1e-12),
                "{label}: {sized_value} is not 100 × {one_value}"
            );
        }
    }

    #[test]
    fn test_greeks_contract_size_scales_every_greek_long_and_short() {
        for side in [Side::Long, Side::Short] {
            for style in [OptionStyle::Call, OptionStyle::Put] {
                assert_scales(&create_sample_option_simplest(style, side));
            }
        }
    }

    #[test]
    fn test_greeks_contract_size_scales_degenerate_branches() {
        // At expiry and at zero volatility each greek runs its own branch.
        let mut at_expiry = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        at_expiry.underlying_price = Positive::new(110.0).unwrap_or(Positive::HUNDRED);
        at_expiry.expiration_date = ExpirationDate::Days(Positive::ZERO);
        let sized = at_expiry.clone().with_contract_size(Positive::HUNDRED);
        assert_eq!(value("delta", sized.delta()), dec!(100));

        let mut zero_vol = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        zero_vol.implied_volatility = Positive::ZERO;
        let sized = zero_vol.clone().with_contract_size(Positive::HUNDRED);
        assert_eq!(
            value("delta", sized.delta()),
            value("delta", zero_vol.delta()) * Decimal::ONE_HUNDRED
        );
    }

    #[test]
    fn test_greeks_contract_size_scales_numerical_fallback() {
        // A non-European contract prices its delta and gamma numerically.
        let mut binary = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        binary.option_type = OptionType::Binary {
            binary_type: BinaryType::CashOrNothing,
        };
        let sized = binary.clone().with_contract_size(Positive::HUNDRED);
        let mut hundred_contracts = binary.clone();
        hundred_contracts.quantity = Positive::HUNDRED;
        for (label, calculate) in [("delta", SCALED[0].1), ("gamma", SCALED[1].1)] {
            let sized_value = value(label, calculate(&sized));
            assert_eq!(sized_value, value(label, calculate(&hundred_contracts)));
            assert_ne!(sized_value, value(label, calculate(&binary)));
        }
    }

    #[test]
    fn test_greeks_contract_size_aggregate_and_alpha() {
        let one = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let sized = one.clone().with_contract_size(Positive::HUNDRED);
        let (Ok(one_greeks), Ok(sized_greeks)) = (one.greeks(), sized.greeks()) else {
            panic!("greeks compute");
        };
        assert_eq!(sized_greeks.delta, value("delta", sized.delta()));
        assert_eq!(sized_greeks.vega, value("vega", sized.vega()));
        assert!(
            (sized_greeks.gamma - one_greeks.gamma * Decimal::ONE_HUNDRED).abs() <= dec!(1e-12)
        );
        // Alpha is a gamma / theta ratio: the size cancels.
        assert!((sized_greeks.alpha - one_greeks.alpha).abs() <= dec!(1e-12));
    }

    #[test]
    fn test_greeks_contract_size_on_position_matches_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long)
            .with_contract_size(Positive::HUNDRED);
        let position = Position::new(
            option.clone(),
            Positive::ONE,
            chrono::Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );
        assert_eq!(
            value("delta", position.delta()),
            value("delta", option.delta())
        );
    }

    #[test]
    fn test_greeks_contract_size_overflow_is_an_error() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let mut huge = option.with_contract_size(Positive::MAX);
        huge.quantity = Positive::MAX;
        assert!(matches!(huge.delta(), Err(GreeksError::PositiveError(_))));
    }
}
