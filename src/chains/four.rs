/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 8/2/25
******************************************************************************/

use crate::Options;
use std::sync::Arc;

/// Represents a combination of four option positions that form a complete option strategy.
///
/// This struct encapsulates a set of four option contracts that together can create various
/// option strategies such as iron condors, iron butterflies, straddles, strangles, or custom
/// four-legged option combinations.
///
/// Each component is stored as an `Arc<Options>` to allow efficient sharing of option contract
/// data across different parts of the application without unnecessary cloning.
///
/// # Fields
///
/// * `long_call` - A call option that is purchased (long position), giving the right to buy
///   the underlying asset at the strike price.
///
/// * `short_call` - A call option that is sold (short position), creating an obligation to sell
///   the underlying asset at the strike price if the buyer exercises.
///
/// * `long_put` - A put option that is purchased (long position), giving the right to sell
///   the underlying asset at the strike price.
///
/// * `short_put` - A put option that is sold (short position), creating an obligation to buy
///   the underlying asset at the strike price if the buyer exercises.
///
/// # Usage
///
/// This structure is typically used in option strategy analysis, risk management,
/// and portfolio modeling where multiple option positions are evaluated together
/// to assess combined payoff profiles and risk characteristics.
#[derive(Debug, Clone)]
pub struct FourOptions {
    /// A purchased call option contract, giving the right to buy the underlying asset
    pub long_call: Arc<Options>,

    /// A sold call option contract, creating the obligation to sell the underlying if exercised
    pub short_call: Arc<Options>,

    /// A purchased put option contract, giving the right to sell the underlying asset
    pub long_put: Arc<Options>,

    /// A sold put option contract, creating the obligation to buy the underlying if exercised
    pub short_put: Arc<Options>,
}

impl PartialEq for FourOptions {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.long_call, &other.long_call)
            && Arc::ptr_eq(&self.short_call, &other.short_call)
            && Arc::ptr_eq(&self.long_put, &other.long_put)
            && Arc::ptr_eq(&self.short_put, &other.short_put)
    }
}

#[cfg(test)]
mod tests {
    use crate::chains::OptionData;
    use crate::chains::utils::OptionDataPriceParams;
    use crate::model::ExpirationDate;
    use crate::{OptionStyle, OptionType, Positive, Side, assert_pos_relative_eq, pos, spos};
    use rust_decimal_macros::dec;
    use std::sync::Arc;

    // Helper function to create a standard OptionDataPriceParams for testing
    fn create_test_price_params() -> OptionDataPriceParams {
        OptionDataPriceParams {
            underlying_price: pos!(100.0),
            expiration_date: ExpirationDate::Days(pos!(1.0)),
            implied_volatility: None,
            risk_free_rate: dec!(0.05),
            dividend_yield: pos!(0.02),
            underlying_symbol: Some("AAPL".to_string()),
        }
    }

    // Helper function to create a standard OptionData for testing
    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(110.0),      // strike_price
            spos!(5.0),       // call_bid
            spos!(5.5),       // call_ask
            spos!(4.0),       // put_bid
            spos!(4.5),       // put_ask
            Some(pos!(0.25)), // implied_volatility
            Some(dec!(0.6)),  // delta_call
            Some(dec!(-0.4)), // delta_put
            Some(dec!(0.05)), // gamma
            spos!(1000.0),    // volume
            Some(500),        // open_interest
        )
    }

    #[test]

    fn test_create_options() {
        let mut option_data = create_test_option_data();
        let price_params = create_test_price_params();

        // Test successful creation of options
        let result = option_data.create_options(&price_params);
        assert!(result.is_ok());
    }

    #[test]

    fn test_four_options_properties() {
        let mut option_data = create_test_option_data();
        let price_params = create_test_price_params();

        // Create the options
        let _ = option_data.create_options(&price_params);
        let options = option_data.options.as_ref().unwrap();

        // Verify long call properties
        let long_call = &options.long_call;
        assert_eq!(long_call.option_type, OptionType::European);
        assert_eq!(long_call.side, Side::Long);
        assert_eq!(long_call.expiration_date, price_params.expiration_date);
        assert_pos_relative_eq!(long_call.implied_volatility, pos!(0.25), pos!(0.0001));
        assert_pos_relative_eq!(long_call.underlying_price, pos!(100.0), pos!(0.0001));
        assert_eq!(long_call.option_style, OptionStyle::Call);
        assert_pos_relative_eq!(long_call.dividend_yield, pos!(0.02), pos!(0.0001));

        // Verify short call side
        assert_eq!(options.short_call.side, Side::Short);
        assert_eq!(options.short_call.option_style, OptionStyle::Call);

        // Verify long put style and side
        assert_eq!(options.long_put.side, Side::Long);
        assert_eq!(options.long_put.option_style, OptionStyle::Put);

        // Verify short put style and side
        assert_eq!(options.short_put.side, Side::Short);
        assert_eq!(options.short_put.option_style, OptionStyle::Put);
    }

    #[test]

    fn test_create_options_with_no_symbol() {
        let mut option_data = create_test_option_data();
        let mut price_params = create_test_price_params();
        price_params.underlying_symbol = None;

        // Test creation without underlying symbol
        let result = option_data.create_options(&price_params);
        assert!(result.is_ok());
    }

    #[test]

    fn test_create_options_with_no_iv() {
        let mut option_data = create_test_option_data();
        option_data.implied_volatility = None;
        let price_params = create_test_price_params();

        // Test creation without implied volatility
        let result = option_data.create_options(&price_params);
        assert!(result.is_ok());

        let options = option_data.options.as_ref().unwrap();
        assert_pos_relative_eq!(
            options.long_call.implied_volatility,
            Positive::ZERO,
            pos!(0.0001)
        );
    }

    #[test]

    fn test_four_options_equality() {
        // Create two identical option data objects
        let mut option_data1 = create_test_option_data();
        let mut option_data2 = create_test_option_data();
        let price_params = create_test_price_params();

        // Create options for both
        let _ = option_data1.create_options(&price_params);
        let _ = option_data2.create_options(&price_params);

        // Extract FourOptions instances
        let four_options1 = option_data1.options.as_ref().unwrap().clone();
        let four_options2 = option_data2.options.as_ref().unwrap().clone();

        // They should NOT be equal since they're different Arc pointers
        assert_ne!(four_options1, four_options2);

        // Clone should be equal to itself
        let cloned = four_options1.clone();
        assert_eq!(four_options1, cloned);
    }

    #[test]

    fn test_four_options_debug() {
        let mut option_data = create_test_option_data();
        let price_params = create_test_price_params();

        let _ = option_data.create_options(&price_params);
        let four_options = option_data.options.as_ref().unwrap();

        // Test that debug formatting works
        let debug_output = format!("{:?}", four_options);
        assert!(debug_output.contains("FourOptions"));
    }

    #[test]

    fn test_four_options_clone() {
        let mut option_data = create_test_option_data();
        let price_params = create_test_price_params();

        let _ = option_data.create_options(&price_params);
        let four_options = option_data.options.as_ref().unwrap().clone();
        let cloned = four_options.clone();

        // The cloned version should be equal to the original
        assert_eq!(four_options, cloned);

        // The Arc pointers should be the same (reference count increased)
        assert!(Arc::ptr_eq(&four_options.long_call, &cloned.long_call));
        assert!(Arc::ptr_eq(&four_options.short_call, &cloned.short_call));
        assert!(Arc::ptr_eq(&four_options.long_put, &cloned.long_put));
        assert!(Arc::ptr_eq(&four_options.short_put, &cloned.short_put));
    }
}
