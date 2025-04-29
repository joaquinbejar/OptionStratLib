/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/12/24
******************************************************************************/
use crate::Options;
use crate::greeks::Greeks;
use rust_decimal::Decimal;
use std::error::Error;

/// Represents a collection of option positions at the same strike price.
///
/// This structure groups four option positions - long and short calls, along with
/// long and short puts - that share the same strike price. It's commonly used for
/// analyzing complex option strategies like straddles, strangles, iron condors,
/// and other multi-leg option combinations that involve positions at the same strike.
///
/// # Fields
///
/// * `long_call` - A long (bought) call option position at the strike price.
///   Represents the right to buy the underlying asset at the strike price.
///
/// * `short_call` - A short (sold/written) call option position at the strike price.
///   Represents the obligation to sell the underlying asset at the strike price
///   if the option is exercised by its holder.
///
/// * `long_put` - A long (bought) put option position at the strike price.
///   Represents the right to sell the underlying asset at the strike price.
///
/// * `short_put` - A short (sold/written) put option position at the strike price.
///   Represents the obligation to buy the underlying asset at the strike price
///   if the option is exercised by its holder.
///
/// # Usage
///
/// This struct is typically used in option strategy analysis, risk assessment,
/// and for calculating combined payoff profiles of multiple option positions
/// at the same strike price.
#[derive(Debug, Clone)]
pub struct OptionsInStrike {
    /// A long (bought) call option position at this strike price
    pub long_call: Options,

    /// A short (sold/written) call option position at this strike price
    pub short_call: Options,

    /// A long (bought) put option position at this strike price
    pub long_put: Options,

    /// A short (sold/written) put option position at this strike price
    pub short_put: Options,
}

impl OptionsInStrike {
    /// Creates a new `OptionsInStrike` instance with the four basic option positions.
    ///
    /// This constructor creates a collection of option positions at the same strike price,
    /// containing both call and put options in long and short positions.
    ///
    /// # Parameters
    ///
    /// * `long_call` - A long (bought) call option position at this strike price.
    /// * `short_call` - A short (sold/written) call option position at this strike price.
    /// * `long_put` - A long (bought) put option position at this strike price.
    /// * `short_put` - A short (sold/written) put option position at this strike price.
    ///
    /// # Returns
    ///
    /// A new `OptionsInStrike` instance with the specified option positions.
    ///
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

    /// Calculates delta values for all four option positions at this strike price.
    ///
    /// Delta measures the rate of change of the option's price with respect to changes
    /// in the underlying asset's price. This method computes delta for all four basic
    /// option positions and returns them in a structured format.
    ///
    /// # Returns
    ///
    /// * `Result<DeltasInStrike, Box<dyn Error>>` - A Result containing delta values for all
    ///   four option positions if successful, or an error if any delta calculation fails.
    ///
    /// # Errors
    ///
    /// This method will return an error if any of the underlying delta calculations fail,
    /// which may occur due to invalid option parameters or computation errors.
    ///
    pub fn deltas(&self) -> Result<DeltasInStrike, Box<dyn Error>> {
        Ok(DeltasInStrike {
            long_call: self.long_call.delta()?,
            short_call: self.short_call.delta()?,
            long_put: self.long_put.delta()?,
            short_put: self.short_put.delta()?,
        })
    }
}

/// Represents option delta values for all four basic option positions at a specific strike price.
///
/// This structure contains delta values for the four fundamental option positions:
/// long call, short call, long put, and short put. Delta measures the rate of change
/// of an option's price with respect to changes in the underlying asset's price.
///
/// # Fields
///
/// * `long_call` - Delta value for a long call position. Typically ranges between 0 and 1,
///   where values closer to 1 indicate the option behaves more like the underlying asset.
///
/// * `short_call` - Delta value for a short call position. This is the negative of the long
///   call delta, typically ranging from -1 to 0.
///
/// * `long_put` - Delta value for a long put position. Typically ranges between -1 and 0,
///   where values closer to -1 indicate stronger inverse correlation with the underlying.
///
/// * `short_put` - Delta value for a short put position. This is the negative of the long
///   put delta, typically ranging from 0 to 1.
///
/// # Usage
///
/// This struct is typically used in options analysis, risk management, and strategy development
/// to assess position sensitivity to underlying price movements at specific strike prices.
///
/// Delta values are essential for understanding directional exposure and for implementing
/// delta-neutral strategies in options trading.
#[derive(Debug, Clone)]
pub struct DeltasInStrike {
    /// Delta value for a long call option position
    pub long_call: Decimal,

    /// Delta value for a short call option position
    pub short_call: Decimal,

    /// Delta value for a long put option position
    pub long_put: Decimal,

    /// Delta value for a short put option position
    pub short_put: Decimal,
}

#[cfg(test)]
mod tests_options_in_strike {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::{ExpirationDate, pos};
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
