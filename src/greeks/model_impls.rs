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
use crate::model::{Options, Position};

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
    use crate::assert_decimal_eq;
    use crate::model::types::{OptionStyle, Side};
    use crate::model::utils::create_sample_option_simplest;
    use positive::{Positive, pos_or_panic};
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
    use crate::assert_decimal_eq;
    use crate::model::ExpirationDate;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::model::utils::create_sample_option_simplest;
    use positive::{Positive, pos_or_panic};
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
