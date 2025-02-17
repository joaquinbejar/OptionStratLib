/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/12/24
******************************************************************************/
use crate::greeks::Greeks;
use crate::Options;
use rust_decimal::Decimal;
use std::error::Error;

#[derive(Debug, Clone)]
pub struct OptionsInStrike {
    pub long_call: Options,
    pub short_call: Options,
    pub long_put: Options,
    pub short_put: Options,
}

impl OptionsInStrike {
    pub fn new(
        long_call: Options,
        short_call: Options,
        long_put: Options,
        short_put: Options,
    ) -> OptionsInStrike {
        OptionsInStrike {
            long_call,
            short_call,
            long_put,
            short_put,
        }
    }

    pub fn deltas(&self) -> Result<DeltasInStrike, Box<dyn Error>> {
        Ok(DeltasInStrike {
            long_call: self.long_call.delta()?,
            short_call: self.short_call.delta()?,
            long_put: self.long_put.delta()?,
            short_put: self.short_put.delta()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct DeltasInStrike {
    pub long_call: Decimal,
    pub short_call: Decimal,
    pub long_put: Decimal,
    pub short_put: Decimal,
}

#[cfg(test)]
mod tests_options_in_strike {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a sample option for testing
    fn create_test_option(side: Side, style: OptionStyle) -> Options {
        Options::new(
            OptionType::European,
            side,
            "TEST".to_string(),
            pos!(100.0), // strike_price
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),   // implied_volatility
            pos!(1.0),   // quantity
            pos!(100.0), // underlying_price
            dec!(0.05),  // risk_free_rate
            style,
            pos!(0.01), // dividend_yield
            None,
        )
    }

    #[test]
    fn test_new_options_in_strike() {
        let long_call = create_test_option(Side::Long, OptionStyle::Call);
        let short_call = create_test_option(Side::Short, OptionStyle::Call);
        let long_put = create_test_option(Side::Long, OptionStyle::Put);
        let short_put = create_test_option(Side::Short, OptionStyle::Put);

        let options_in_strike = OptionsInStrike::new(
            long_call.clone(),
            short_call.clone(),
            long_put.clone(),
            short_put.clone(),
        );

        assert_eq!(options_in_strike.long_call.side, Side::Long);
        assert_eq!(options_in_strike.long_call.option_style, OptionStyle::Call);
        assert_eq!(options_in_strike.short_call.side, Side::Short);
        assert_eq!(options_in_strike.short_call.option_style, OptionStyle::Call);
        assert_eq!(options_in_strike.long_put.side, Side::Long);
        assert_eq!(options_in_strike.long_put.option_style, OptionStyle::Put);
        assert_eq!(options_in_strike.short_put.side, Side::Short);
        assert_eq!(options_in_strike.short_put.option_style, OptionStyle::Put);
    }

    #[test]
    fn test_deltas_calculation() {
        let long_call = create_test_option(Side::Long, OptionStyle::Call);
        let short_call = create_test_option(Side::Short, OptionStyle::Call);
        let long_put = create_test_option(Side::Long, OptionStyle::Put);
        let short_put = create_test_option(Side::Short, OptionStyle::Put);

        let options_in_strike = OptionsInStrike::new(long_call, short_call, long_put, short_put);

        let deltas = options_in_strike.deltas().unwrap();

        // Long call delta should be positive
        assert!(deltas.long_call > Decimal::ZERO);
        // Short call delta should be negative
        assert!(deltas.short_call < Decimal::ZERO);
        // Long put delta should be negative
        assert!(deltas.long_put < Decimal::ZERO);
        // Short put delta should be positive
        assert!(deltas.short_put > Decimal::ZERO);
    }

    #[test]
    fn test_clone() {
        let long_call = create_test_option(Side::Long, OptionStyle::Call);
        let short_call = create_test_option(Side::Short, OptionStyle::Call);
        let long_put = create_test_option(Side::Long, OptionStyle::Put);
        let short_put = create_test_option(Side::Short, OptionStyle::Put);

        let options_in_strike = OptionsInStrike::new(long_call, short_call, long_put, short_put);

        let cloned = options_in_strike.clone();

        assert_eq!(
            cloned.long_call.strike_price,
            options_in_strike.long_call.strike_price
        );
        assert_eq!(
            cloned.short_call.strike_price,
            options_in_strike.short_call.strike_price
        );
        assert_eq!(
            cloned.long_put.strike_price,
            options_in_strike.long_put.strike_price
        );
        assert_eq!(
            cloned.short_put.strike_price,
            options_in_strike.short_put.strike_price
        );
    }
}

#[cfg(test)]
mod tests_deltas_in_strike {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn test_deltas_in_strike_creation() {
        let deltas = DeltasInStrike {
            long_call: dec!(0.6),
            short_call: dec!(-0.6),
            long_put: dec!(-0.4),
            short_put: dec!(0.4),
        };

        assert_eq!(deltas.long_call, dec!(0.6));
        assert_eq!(deltas.short_call, dec!(-0.6));
        assert_eq!(deltas.long_put, dec!(-0.4));
        assert_eq!(deltas.short_put, dec!(0.4));
    }

    #[test]
    fn test_deltas_in_strike_clone() {
        let deltas = DeltasInStrike {
            long_call: dec!(0.6),
            short_call: dec!(-0.6),
            long_put: dec!(-0.4),
            short_put: dec!(0.4),
        };

        let cloned = deltas.clone();

        assert_eq!(cloned.long_call, deltas.long_call);
        assert_eq!(cloned.short_call, deltas.short_call);
        assert_eq!(cloned.long_put, deltas.long_put);
        assert_eq!(cloned.short_put, deltas.short_put);
    }

    #[test]
    fn test_deltas_in_strike_debug() {
        let deltas = DeltasInStrike {
            long_call: dec!(0.6),
            short_call: dec!(-0.6),
            long_put: dec!(-0.4),
            short_put: dec!(0.4),
        };

        let debug_output = format!("{:?}", deltas);
        assert!(debug_output.contains("0.6"));
        assert!(debug_output.contains("-0.6"));
        assert!(debug_output.contains("-0.4"));
        assert!(debug_output.contains("0.4"));
    }
}
