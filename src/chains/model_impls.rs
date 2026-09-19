/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! Market-data conversions onto the core model types.
//!
//! Everything that turns chain data (`OptionData`) into, or refreshes, a
//! core value lives here: the `TryFrom<&OptionData>` conversion for
//! `Options` and the crate-internal [`UpdateFromOptionData`] refresh used by
//! strategy re-pricing. The impls sit in the market layer (local source
//! type, core-owned target) so the core model never references chain types.

use crate::chains::optiondata::OptionData;
use crate::error::{OptionsError, PositionError};
use crate::model::types::{OptionStyle, OptionType, Side};
use crate::model::{Options, Position};
use chrono::Utc;
use positive::Positive;
use rust_decimal::Decimal;
use tracing::trace;

/// Refreshes a core value from an [`OptionData`] row.
///
/// `Options` takes the strike and the implied volatility; `Position` also
/// re-stamps its open date and takes the bid or ask that matches its side
/// and style as the new premium.
pub(crate) trait UpdateFromOptionData {
    /// Overwrites the market-dependent fields of `self` with the values in
    /// `option_data`.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the quote the position needs (call or
    /// put, bid or ask, by side and style) is missing from `option_data`.
    /// The `Options` implementation cannot fail.
    fn update_from_option_data(&mut self, option_data: &OptionData) -> Result<(), PositionError>;
}

impl UpdateFromOptionData for Options {
    /// Updates option parameters using data from an OptionData structure.
    ///
    /// This method updates the option's strike price and implied volatility based on the
    /// values provided in the option_data parameter. If the implied volatility is not
    /// available in the option data, it defaults to zero.
    ///
    /// # Arguments
    ///
    /// * `option_data` - A reference to an OptionData structure containing updated option parameters.
    ///
    /// # Errors
    ///
    /// Never fails; the `Result` is the shared trait contract.
    fn update_from_option_data(&mut self, option_data: &OptionData) -> Result<(), PositionError> {
        self.strike_price = option_data.strike_price;
        self.implied_volatility = option_data.implied_volatility;
        trace!("Updated Option: {:#?}", self);
        Ok(())
    }
}

impl UpdateFromOptionData for Position {
    /// Updates a position with data from an `OptionData` instance, refreshing premium values
    /// and option details.
    ///
    /// This method handles the complete update of a position based on new market data,
    /// including:
    ///
    /// 1. Setting the position's timestamp to the current UTC time
    /// 2. Updating the underlying option details through the option's own update method
    /// 3. Setting the premium value based on the position's side (Long/Short) and option style (Call/Put)
    ///
    /// The premium is determined as follows:
    /// - For Long Call positions: Uses the call ask price (price to buy a call)
    /// - For Long Put positions: Uses the put ask price (price to buy a put)
    /// - For Short Call positions: Uses the call bid price (price to sell a call)
    /// - For Short Put positions: Uses the put bid price (price to sell a put)
    ///
    /// # Parameters
    ///
    /// * `option_data` - Reference to an `OptionData` struct containing current market data
    ///   for the relevant option, including bid/ask prices and option characteristics.
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] when the quote the position needs (call or
    /// put, bid or ask, by side and style) is missing from `option_data`.
    fn update_from_option_data(&mut self, option_data: &OptionData) -> Result<(), PositionError> {
        self.date = Utc::now();
        self.option.update_from_option_data(option_data)?;
        match (self.option.side, self.option.option_style) {
            (Side::Long, OptionStyle::Call) => {
                self.premium = option_data.call_ask.ok_or_else(|| {
                    PositionError::invalid_position_update(
                        "premium".to_string(),
                        "Missing call ask price for long call position".to_string(),
                    )
                })?;
            }
            (Side::Long, OptionStyle::Put) => {
                self.premium = option_data.put_ask.ok_or_else(|| {
                    PositionError::invalid_position_update(
                        "premium".to_string(),
                        "Missing put ask price for long put position".to_string(),
                    )
                })?;
            }
            (Side::Short, OptionStyle::Call) => {
                self.premium = option_data.call_bid.ok_or_else(|| {
                    PositionError::invalid_position_update(
                        "premium".to_string(),
                        "Missing call bid price for short call position".to_string(),
                    )
                })?;
            }
            (Side::Short, OptionStyle::Put) => {
                self.premium = option_data.put_bid.ok_or_else(|| {
                    PositionError::invalid_position_update(
                        "premium".to_string(),
                        "Missing put bid price for short put position".to_string(),
                    )
                })?;
            }
        }
        trace!("Updated position: {:#?}", self);
        Ok(())
    }
}

impl TryFrom<&OptionData> for Options {
    type Error = OptionsError;

    fn try_from(option_data: &OptionData) -> Result<Self, Self::Error> {
        let underlying_symbol =
            option_data
                .symbol
                .clone()
                .ok_or_else(|| OptionsError::ValidationError {
                    field: "symbol".to_string(),
                    reason: "OptionData must have a valid symbol".to_string(),
                })?;

        let expiration_date =
            option_data
                .expiration_date
                .ok_or_else(|| OptionsError::ValidationError {
                    field: "expiration_date".to_string(),
                    reason: "OptionData must have a valid expiration date".to_string(),
                })?;

        let underlying_price = option_data
            .underlying_price
            .as_ref()
            .map(|p| **p)
            .ok_or_else(|| OptionsError::ValidationError {
                field: "underlying_price".to_string(),
                reason: "OptionData must have a valid underlying price".to_string(),
            })?;

        Ok(Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol,
            strike_price: option_data.strike_price,
            expiration_date,
            implied_volatility: option_data.implied_volatility,
            quantity: Positive::ONE,
            underlying_price,
            risk_free_rate: option_data.risk_free_rate.unwrap_or(Decimal::ZERO),
            option_style: OptionStyle::Call,
            dividend_yield: option_data.dividend_yield.unwrap_or(Positive::ZERO),
            exotic_params: None,
        })
    }
}

#[cfg(test)]
mod tests_update_from_option_data {
    use super::*;
    use positive::{pos_or_panic, spos};

    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos_or_panic!(110.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            pos_or_panic!(0.25),
            Some(dec!(-0.3)),
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    fn create_wrong_call_ask_test_option_data() -> OptionData {
        OptionData::new(
            pos_or_panic!(110.0),
            spos!(9.5),          // call_bid
            None,                // call_ask missing value
            spos!(8.5),          // put_bid
            spos!(9.0),          // put_ask
            pos_or_panic!(0.25), // iv
            Some(dec!(-0.3)),    // delta
            Some(dec!(0.3)),     // gamma
            Some(dec!(0.3)),     // vega
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    fn create_wrong_put_ask_test_option_data() -> OptionData {
        OptionData::new(
            pos_or_panic!(110.0),
            spos!(9.5),          // call_bid
            spos!(10.0),         // call_ask
            spos!(8.5),          // put_bid
            None,                // put_ask missing value
            pos_or_panic!(0.25), // iv
            Some(dec!(-0.3)),    // delta
            Some(dec!(0.3)),     // gamma
            Some(dec!(0.3)),     // vega
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    fn create_wrong_call_bid_test_option_data() -> OptionData {
        OptionData::new(
            pos_or_panic!(110.0),
            None,                // call_bid missing value
            spos!(10.0),         // call_ask
            spos!(8.5),          // put_bid
            spos!(9.0),          // put_ask
            pos_or_panic!(0.25), // iv
            Some(dec!(-0.3)),    // delta
            Some(dec!(0.3)),     // gamma
            Some(dec!(0.3)),     // vega
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    fn create_wrong_put_bid_test_option_data() -> OptionData {
        OptionData::new(
            pos_or_panic!(110.0),
            spos!(9.5),          // call_bid
            spos!(10.0),         // call_ask
            None,                // put_bid missing value
            spos!(9.0),          // put_ask
            pos_or_panic!(0.25), // iv
            Some(dec!(-0.3)),    // delta
            Some(dec!(0.3)),     // gamma
            Some(dec!(0.3)),     // vega
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        )
    }

    #[test]
    fn test_update_long_call() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        let _ = position.update_from_option_data(&option_data);

        assert_eq!(position.option.strike_price, pos_or_panic!(110.0));
        assert_eq!(position.option.implied_volatility, 0.25);
        assert_eq!(position.premium, 10.0); // call_ask
    }

    #[test]
    fn test_update_short_call() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        let _ = position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.5); // call_bid
    }

    #[test]
    fn test_update_long_put() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        let _ = position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.0); // put_ask
    }

    #[test]
    fn test_update_short_put() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        let _ = position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 8.5); // put_bid
    }

    #[test]
    fn test_update_wrong_long_call() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_wrong_call_ask_test_option_data();
        let result = position.update_from_option_data(&option_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_wrong_long_put() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_wrong_put_ask_test_option_data();
        let result = position.update_from_option_data(&option_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_wrong_short_call() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_wrong_call_bid_test_option_data();
        let result = position.update_from_option_data(&option_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_update_wrong_short_put() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_wrong_put_bid_test_option_data();
        let result = position.update_from_option_data(&option_data);

        assert!(result.is_err());
    }
}
