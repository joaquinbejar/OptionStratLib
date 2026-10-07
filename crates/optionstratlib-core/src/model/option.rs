use crate::constants::ZERO;
use crate::error::{OptionsError, OptionsResult};
use crate::model::ExpirationDate;
use crate::model::decimal::finite_decimal;
use crate::model::payoff::{Payoff, PayoffInfo};
use crate::model::types::{OptionStyle, OptionType, Side};
use positive::Positive;
#[cfg(test)]
use positive::pos_or_panic;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::error;

/// Projects a quantity-scaled `f64` payoff back onto `Decimal`, reporting the
/// rejection instead of substituting zero.
///
/// The three payoff entry points below (`payoff`, `payoff_at_price`,
/// `intrinsic_value`) evaluate `OptionType::payoff` in `f64` and scale it by
/// the position quantity. Both factors can be as large as `Positive::MAX`
/// (`≈ 7.92e28`), so the product routinely leaves the `Decimal` range — and a
/// deep in-the-money call on an underlying at `Positive::MAX` leaves it even
/// at quantity one, because the `f64` nearest to `Decimal::MAX` rounds above
/// it. Those cases used to come back as `Ok(Decimal::ZERO)`: a worthless
/// payoff for an option that is worth more than the type can express.
///
/// The returned error is [`OptionsError::PayoffError`], carrying the entry
/// point and the offending value.
#[cold]
#[inline(never)]
fn payoff_out_of_range(context: &'static str, value: f64) -> OptionsError {
    OptionsError::PayoffError {
        reason: format!(
            "{context}: payoff {value} is not representable as a Decimal (non-finite or out of range)"
        ),
    }
}

/// Parameters for exotic option pricing models.
///
/// This structure holds specific data required by various exotic option types
/// such as Asian options (which depend on average prices) and Lookback options
/// (which depend on minimum/maximum prices during the option's lifetime).
///
/// Each field is optional since different exotic option types require different parameters.
#[derive(Clone, Default, PartialEq, Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct ExoticParams {
    /// Historical spot prices, primarily used for Asian options which
    /// depend on the average price of the underlying asset.
    pub spot_prices: Option<Vec<Positive>>, // Asian

    /// Minimum observed spot price during the option's lifetime,
    /// used for lookback option pricing.
    pub spot_min: Option<Decimal>, // Lookback

    /// Maximum observed spot price during the option's lifetime,
    /// used for lookback option pricing.
    pub spot_max: Option<Decimal>, // Lookback

    /// Local cap for Cliquet options, limiting the periodic return.
    pub cliquet_local_cap: Option<Decimal>, // Cliquet

    /// Local floor for Cliquet options, limiting the periodic return.
    pub cliquet_local_floor: Option<Decimal>, // Cliquet

    /// Global cap for Cliquet options, limiting the total return.
    pub cliquet_global_cap: Option<Decimal>, // Cliquet

    /// Global floor for Cliquet options, limiting the total return.
    pub cliquet_global_floor: Option<Decimal>, // Cliquet

    /// Price of the second underlying asset for Rainbow options.
    pub rainbow_second_asset_price: Option<Positive>, // Rainbow

    /// Volatility of the second underlying asset for Rainbow options.
    pub rainbow_second_asset_volatility: Option<Positive>, // Rainbow

    /// Dividend yield of the second underlying asset for Rainbow options.
    pub rainbow_second_asset_dividend: Option<Positive>, // Rainbow

    /// Correlation between the two underlying assets for Rainbow options.
    /// Must be between -1.0 and 1.0.
    pub rainbow_correlation: Option<Decimal>, // Rainbow

    /// Volatility of the second underlying asset for Spread options.
    pub spread_second_asset_volatility: Option<Positive>, // Spread

    /// Dividend yield of the second underlying asset for Spread options.
    pub spread_second_asset_dividend: Option<Positive>, // Spread

    /// Correlation between the two underlying assets for Spread options.
    /// Must be between -1.0 and 1.0.
    pub spread_correlation: Option<Decimal>, // Spread

    /// Volatility of the exchange rate for Quanto options.
    pub quanto_fx_volatility: Option<Positive>, // Quanto

    /// Correlation between the underlying asset and the exchange rate for Quanto options.
    /// Must be between -1.0 and 1.0.
    pub quanto_fx_correlation: Option<Decimal>, // Quanto

    /// Foreign risk-free interest rate for Quanto options.
    pub quanto_foreign_rate: Option<Decimal>, // Quanto

    /// Volatility of the second underlying asset for Exchange options.
    pub exchange_second_asset_volatility: Option<Positive>, // Exchange

    /// Dividend yield of the second underlying asset for Exchange options.
    pub exchange_second_asset_dividend: Option<Positive>, // Exchange

    /// Correlation between the two underlying assets for Exchange options.
    /// Must be between -1.0 and 1.0.
    pub exchange_correlation: Option<Decimal>, // Exchange
}

/// Represents a financial option contract with its essential parameters and characteristics.
///
/// This structure contains all the necessary information to define an options contract,
/// including its type (call/put), market position (long/short), pricing parameters,
/// and contract specifications. It serves as the core data model for option pricing,
/// risk analysis, and strategy development.
///
/// The `Options` struct supports both standard option types and exotic options through
/// the optional `exotic_params` field, making it versatile for various financial modeling
/// scenarios.
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
pub struct Options {
    /// Specifies whether this is a European or American option
    pub option_type: OptionType,

    /// Indicates whether the position is Long (purchased) or Short (sold/written),
    /// which determines the profit/loss direction and risk profile.
    pub side: Side,

    /// The ticker symbol or identifier of the underlying asset (e.g., "AAPL" for Apple stock).
    pub underlying_symbol: String,

    /// The price at which the option holder can exercise their right to buy (for calls)
    /// or sell (for puts) the underlying asset.
    pub strike_price: Positive,

    /// When the option contract expires, either as days from now or as a specific date.
    pub expiration_date: ExpirationDate,

    /// The market's expectation for future volatility of the underlying asset,
    /// a key parameter for option pricing models.
    pub implied_volatility: Positive,

    /// The number of contracts in this position.
    pub quantity: Positive,

    /// The current market price of the underlying asset.
    pub underlying_price: Positive,

    /// The current risk-free interest rate used in option pricing models,
    /// typically based on treasury yields of similar duration. Annual, as a
    /// fraction; may be negative (#709).
    pub risk_free_rate: Decimal,

    /// The option is a Call or Put option, determining the fundamental right
    /// option can be exercised.
    pub option_style: OptionStyle,

    /// The annualized dividend yield of the underlying asset, affecting option pricing
    /// particularly for longer-dated contracts.
    pub dividend_yield: Positive,

    /// Additional parameters required for exotic option types like Asian or Lookback options.
    /// This field is None for standard (vanilla) options.
    pub exotic_params: Option<ExoticParams>,
}

impl Options {
    /// Creates a new options contract with the specified parameters.
    ///
    /// This constructor creates an instance of `Options` with all the required parameters
    /// for defining and pricing an option contract. It supports both standard (vanilla)
    /// options and exotic options through the optional `exotic_params` parameter.
    ///
    /// # Parameters
    ///
    /// * `option_type` - Specifies whether this is a Call or Put option, determining the fundamental
    ///   right the option contract provides.
    /// * `side` - Indicates whether the position is Long (purchased) or Short (sold/written),
    ///   which determines the profit/loss direction.
    /// * `underlying_symbol` - The ticker symbol or identifier of the underlying asset (e.g., "AAPL").
    /// * `strike_price` - The price at which the option can be exercised, represented as a `Positive` value.
    /// * `expiration_date` - When the option contract expires, either as days from now or as a specific date.
    /// * `implied_volatility` - The market's expectation for future volatility of the underlying asset,
    ///   a key parameter for option pricing.
    /// * `quantity` - The number of contracts in this position, represented as a `Positive` value.
    /// * `underlying_price` - The current market price of the underlying asset.
    /// * `risk_free_rate` - The current risk-free interest rate used in option pricing models.
    /// * `option_style` - The option exercise style (European or American), determining when the
    ///   option can be exercised.
    /// * `dividend_yield` - The annualized dividend yield of the underlying asset, affecting option pricing.
    /// * `exotic_params` - Additional parameters required for exotic option types. Set to `None` for
    ///   standard (vanilla) options.
    ///
    /// # Returns
    ///
    /// A fully configured `Options` instance with all the specified parameters.
    ///
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        option_type: OptionType,
        side: Side,
        underlying_symbol: String,
        strike_price: Positive,
        expiration_date: ExpirationDate,
        implied_volatility: Positive,
        quantity: Positive,
        underlying_price: Positive,
        risk_free_rate: Decimal,
        option_style: OptionStyle,
        dividend_yield: Positive,
        exotic_params: Option<ExoticParams>,
    ) -> Self {
        Options {
            option_type,
            side,
            underlying_symbol,
            strike_price,
            expiration_date,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            option_style,
            dividend_yield,
            exotic_params,
        }
    }

    /// Calculates the time to expiration of the option in years.
    ///
    /// This function computes the time remaining until the option's expiration date,
    /// expressed as a positive decimal value representing years. This is a key parameter
    /// used in option pricing models.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Positive>` - A result containing the time to expiration in years
    ///   as a Positive value, or an error if the calculation failed.
    ///
    /// # Errors
    ///
    /// Propagates any [`expiration_date::error::ExpirationDateError`] returned by
    /// [`ExpirationDate::get_years`] (wrapped as [`OptionsError::ExpirationDate`])
    /// when the stored expiration cannot be converted to a positive
    /// year fraction (e.g. past expiration or invalid date).
    pub fn time_to_expiration(&self) -> OptionsResult<Positive> {
        Ok(self.expiration_date.get_years()?)
    }

    /// Determines if the option position is long (purchased).
    ///
    /// A long position indicates that the option has been bought, meaning the holder
    /// has the right to exercise the option according to its terms.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the option is held as a long position, false otherwise.
    ///
    #[must_use]
    pub fn is_long(&self) -> bool {
        matches!(self.side, Side::Long)
    }

    /// Determines if the option position is short (written/sold).
    ///
    /// A short position indicates that the option has been sold or written, meaning
    /// the holder has the obligation to fulfill the contract terms if the option is exercised.
    ///
    /// # Returns
    ///
    /// * `bool` - Returns true if the option is held as a short position, false otherwise.
    ///
    #[must_use]
    pub fn is_short(&self) -> bool {
        matches!(self.side, Side::Short)
    }

    /// Calculates the intrinsic value (payoff) of the option at the current underlying price.
    ///
    /// The payoff represents what the option would be worth if exercised immediately,
    /// based on the current market conditions. For out-of-the-money options, the payoff
    /// will be zero.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - A result containing the calculated payoff as a
    ///   Decimal value, adjusted for the quantity of contracts held, or an error if
    ///   the calculation failed.
    ///
    /// This method is useful for determining the exercise value of an option and for
    /// analyzing whether an option has intrinsic value.
    ///
    /// # Errors
    ///
    /// Returns [`OptionsError::PayoffError`] when the quantity-scaled payoff is
    /// not representable as a `Decimal` (non-finite, or beyond the `Decimal`
    /// range). It previously returned `Ok(Decimal::ZERO)` for those inputs,
    /// which reported a deep in-the-money position as worthless.
    pub fn payoff(&self) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: self.underlying_price,
            strike: self.strike_price,
            style: self.option_style,
            side: self.side,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let payoff = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        finite_decimal(payoff).ok_or_else(|| payoff_out_of_range("Options::payoff", payoff))
    }

    /// Calculates the financial payoff value of the option at a specific underlying price.
    ///
    /// This method determines the option's payoff based on its type, strike price, style,
    /// and side (long/short) at the given underlying price. The result represents the
    /// total profit or loss for the option position at that price, adjusted by the position quantity.
    ///
    /// # Parameters
    ///
    /// * `price` - A `Positive` value representing the hypothetical price of the underlying asset.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - The calculated payoff value as a `Decimal`, wrapped in a `Result` type.
    ///   Returns an `Err` if the payoff calculation encounters an error.
    ///
    /// # Errors
    ///
    /// Returns [`OptionsError::PayoffError`] when the quantity-scaled payoff at
    /// `price` is not representable as a `Decimal` (non-finite, or beyond the
    /// `Decimal` range). It previously returned `Ok(Decimal::ZERO)` for those
    /// inputs, which reported a deep in-the-money position as worthless.
    pub fn payoff_at_price(&self, price: &Positive) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: *price,
            strike: self.strike_price,
            style: self.option_style,
            side: self.side,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let price = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        finite_decimal(price).ok_or_else(|| payoff_out_of_range("Options::payoff_at_price", price))
    }

    /// Calculates the intrinsic value of the option.
    ///
    /// The intrinsic value is the difference between the underlying asset's price and the option's strike price.
    /// For call options, the intrinsic value is the maximum of zero and the difference between the underlying price and the strike price.
    /// For put options, the intrinsic value is the maximum of zero and the difference between the strike price and the underlying price.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The current price of the underlying asset.
    ///
    /// # Returns
    ///
    /// * `OptionsResult<Decimal>` - The intrinsic value of the option, or an error if the calculation fails.
    ///
    /// # Errors
    ///
    /// Returns [`OptionsError::PayoffError`] when the quantity-scaled intrinsic
    /// value is not representable as a `Decimal` (non-finite, or beyond the
    /// `Decimal` range). It previously returned `Ok(Decimal::ZERO)` for those
    /// inputs, which reported a deep in-the-money position as worthless.
    pub fn intrinsic_value(&self, underlying_price: Positive) -> OptionsResult<Decimal> {
        let payoff_info = PayoffInfo {
            spot: underlying_price,
            strike: self.strike_price,
            style: self.option_style,
            side: self.side,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        let iv = self.option_type.payoff(&payoff_info) * self.quantity.to_f64();
        finite_decimal(iv).ok_or_else(|| payoff_out_of_range("Options::intrinsic_value", iv))
    }

    /// Determines whether an option is "in-the-money" based on its current price relative to strike price.
    ///
    /// An option is considered in-the-money when:
    /// - For Call options: the underlying asset price is greater than or equal to the strike price
    /// - For Put options: the underlying asset price is less than or equal to the strike price
    ///
    /// This status is important for evaluating the option's current value and potential profitability.
    ///
    /// # Returns
    /// `true` if the option is in-the-money, `false` otherwise
    #[must_use]
    pub fn is_in_the_money(&self) -> bool {
        match self.option_style {
            OptionStyle::Call => self.underlying_price >= self.strike_price,
            OptionStyle::Put => self.underlying_price <= self.strike_price,
        }
    }

    /// Validates that the option parameters are in a valid state for calculations.
    ///
    /// This function performs comprehensive validation of the option's critical parameters
    /// to ensure they meet basic requirements for meaningful financial calculations.
    /// It logs detailed error messages when validation fails.
    ///
    /// Validation checks include:
    /// - Underlying symbol is not empty
    /// - Implied volatility is non-negative
    /// - Quantity is non-zero
    /// - Strike price is positive and non-zero
    /// - Underlying price is positive and non-zero
    ///
    /// The risk-free rate is not checked: a negative rate is a legitimate
    /// market input (EUR, CHF and JPY rates were negative for years) and the
    /// pricers handle it (#709).
    ///
    /// # Returns
    /// `true` if all parameters are valid, `false` if any validation fails
    pub(crate) fn validate(&self) -> bool {
        if self.underlying_symbol == *"" {
            error!("Underlying symbol is empty");
            return false;
        }
        if self.implied_volatility < ZERO {
            error!("Implied volatility is less than zero");
            return false;
        }
        if self.quantity == ZERO {
            error!("Quantity is equal to zero");
            return false;
        }
        if self.strike_price == Positive::ZERO {
            error!("Strike is zero");
            return false;
        }
        if self.underlying_price == Positive::ZERO {
            error!("Underlying price is zero");
            return false;
        }
        true
    }
}

impl Default for Options {
    fn default() -> Self {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "".to_string(),
            strike_price: Positive::ZERO,
            expiration_date: ExpirationDate::Days(Positive::ZERO),
            implied_volatility: Positive::ZERO,
            quantity: Positive::ZERO,
            underlying_price: Positive::ZERO,
            risk_free_rate: Decimal::ZERO,
            option_style: OptionStyle::Call,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }
}

#[cfg(test)]
mod tests_options {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;

    use approx::assert_relative_eq;
    use chrono::{Duration, Utc};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_option() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_eq!(option.underlying_symbol, "AAPL");
        assert_eq!(option.strike_price, 100.0);
        assert_eq!(option.implied_volatility, 0.2);
    }

    #[test]
    fn test_time_to_expiration() {
        let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert_relative_eq!(
            option.time_to_expiration().unwrap().to_f64(),
            30.0 / 365.0,
            epsilon = 0.0001
        );

        let future_date = Utc::now() + Duration::days(60);
        let option_with_datetime = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::DateTime(future_date),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.01),
            None,
        );
        assert!(option_with_datetime.time_to_expiration().unwrap() >= 59.0 / 365.0);
        assert!(option_with_datetime.time_to_expiration().unwrap() < 61.0 / 365.0);
    }

    #[test]
    fn test_is_long_and_short() {
        let long_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        assert!(long_option.is_long());
        assert!(!long_option.is_short());

        let short_option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(105.0),
            dec!(0.05),
            OptionStyle::Call,
            pos_or_panic!(0.01),
            None,
        );
        assert!(!short_option.is_long());
        assert!(short_option.is_short());
    }

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, Decimal::ZERO); // max(100 - 100, 0) = 0

        let put_option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            Positive::ONE,
            pos_or_panic!(95.0),
            dec!(0.05),
            OptionStyle::Put,
            pos_or_panic!(0.01),
            None,
        );
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff.to_f64().unwrap(), 5.0); // max(100 - 95, 0) = 5
    }
}

#[cfg(test)]
mod tests_valid_option {
    use super::*;

    use rust_decimal_macros::dec;

    fn create_valid_option() -> Options {
        Options {
            option_type: OptionType::European,
            side: Side::Long,
            underlying_symbol: "AAPL".to_string(),
            strike_price: Positive::HUNDRED,
            expiration_date: ExpirationDate::Days(pos_or_panic!(30.0)),
            implied_volatility: pos_or_panic!(0.2),
            quantity: Positive::ONE,
            underlying_price: pos_or_panic!(105.0),
            risk_free_rate: dec!(0.05),
            option_style: OptionStyle::Call,
            dividend_yield: pos_or_panic!(0.01),
            exotic_params: None,
        }
    }

    #[test]
    fn test_valid_option() {
        let option = create_valid_option();
        assert!(option.validate());
    }

    #[test]
    fn test_empty_underlying_symbol() {
        let mut option = create_valid_option();
        option.underlying_symbol = "".to_string();
        assert!(!option.validate());
    }

    #[test]
    fn test_zero_strike_price() {
        let mut option = create_valid_option();
        option.strike_price = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    fn test_zero_quantity() {
        let mut option = create_valid_option();
        option.quantity = Positive::ZERO;
        assert!(!option.validate());
    }

    #[test]
    fn test_zero_underlying_price() {
        let mut option = create_valid_option();
        option.underlying_price = Positive::ZERO;
        assert!(!option.validate());
    }

    /// A negative risk-free rate is a market input, not an invalid option
    /// (#709): r = -1 % validates, and so do a zero rate and a deeply
    /// negative one.
    #[test]
    fn test_options_validate_negative_risk_free_rate_is_valid() {
        for rate in [dec!(-0.01), Decimal::ZERO, dec!(-0.0075), dec!(-0.25)] {
            let mut option = create_valid_option();
            option.risk_free_rate = rate;
            assert!(option.validate(), "r = {rate}");
        }
    }

    /// The other checks still reject at a negative rate.
    #[test]
    fn test_options_validate_negative_rate_keeps_other_checks() {
        let mut option = create_valid_option();
        option.risk_free_rate = dec!(-0.01);
        option.quantity = Positive::ZERO;
        assert!(!option.validate());

        let mut option = create_valid_option();
        option.risk_free_rate = dec!(-0.01);
        option.underlying_symbol = String::new();
        assert!(!option.validate());

        let mut option = create_valid_option();
        option.risk_free_rate = dec!(-0.01);
        option.underlying_price = Positive::ZERO;
        assert!(!option.validate());
    }
}

#[cfg(test)]
mod tests_options_payoffs {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;

    use rust_decimal_macros::dec;

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(95.0),
        );
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, dec!(5.0)); // max(100 - 95, 0) = 5

        let call_option_otm = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(105.0),
        );
        let call_payoff_otm = call_option_otm.payoff().unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option_simplest_strike(
            Side::Short,
            OptionStyle::Call,
            pos_or_panic!(95.0),
        );
        let call_payoff = call_option.payoff().unwrap();
        assert_eq!(call_payoff, dec!(-5.0)); // -max(100 - 95, 0) = -5

        let call_option_otm = create_sample_option_simplest_strike(
            Side::Short,
            OptionStyle::Call,
            pos_or_panic!(105.0),
        );
        let call_payoff_otm = call_option_otm.payoff().unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Put,
            pos_or_panic!(105.0),
        );
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff, dec!(5.0)); // max(105 - 100, 0) = 5

        let put_option_otm =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos_or_panic!(95.0));
        let put_payoff_otm = put_option_otm.payoff().unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option_simplest_strike(
            Side::Short,
            OptionStyle::Put,
            pos_or_panic!(105.0),
        );
        let put_payoff = put_option.payoff().unwrap();
        assert_eq!(put_payoff, dec!(-5.0)); // -max(105 - 100, 0) = -5

        let put_option_otm = create_sample_option_simplest_strike(
            Side::Short,
            OptionStyle::Put,
            pos_or_panic!(95.0),
        );
        let put_payoff_otm = put_option_otm.payoff().unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    /// The three payoff entry points reported `Ok(0)` for a payoff too large
    /// to represent, which is a deep in-the-money position priced at nothing.
    #[test]
    fn test_payoff_out_of_range_reports_an_error_instead_of_zero() {
        let mut option = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(95.0),
        );
        option.strike_price = Positive::ZERO;
        option.underlying_price = Positive::MAX;
        option.quantity = Positive::ONE;

        // `Positive::MAX` is representable, but the nearest `f64` to it
        // rounds above `Decimal::MAX`, so the round trip fails at quantity
        // one — no extreme quantity is needed.
        assert!(matches!(
            option.payoff(),
            Err(OptionsError::PayoffError { .. })
        ));
        assert!(matches!(
            option.payoff_at_price(&Positive::MAX),
            Err(OptionsError::PayoffError { .. })
        ));
        assert!(matches!(
            option.intrinsic_value(Positive::MAX),
            Err(OptionsError::PayoffError { .. })
        ));
    }

    /// A payoff that is genuinely zero still comes back as `Ok(0)`: the fix
    /// distinguishes "worth nothing" from "cannot be represented".
    #[test]
    fn test_payoff_out_of_the_money_still_returns_zero() {
        let mut option =
            create_sample_option_simplest_strike(Side::Long, OptionStyle::Put, pos_or_panic!(95.0));
        option.strike_price = Positive::ZERO;
        option.underlying_price = Positive::MAX;
        option.quantity = Positive::ONE;

        assert_eq!(option.payoff().unwrap(), Decimal::ZERO);
        assert_eq!(
            option.payoff_at_price(&Positive::MAX).unwrap(),
            Decimal::ZERO
        );
        assert_eq!(
            option.intrinsic_value(Positive::MAX).unwrap(),
            Decimal::ZERO
        );
    }
}

#[cfg(test)]
mod tests_options_payoff_at_price {
    use super::*;
    use crate::model::utils::create_sample_option_simplest;

    use rust_decimal_macros::dec;

    #[test]
    fn test_payoff_european_call_long() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff = call_option.payoff_at_price(&pos_or_panic!(105.0)).unwrap();
        assert_eq!(call_payoff, dec!(5.0)); // max(105 - 100, 0) = 5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Long);
        let call_payoff_otm = call_option_otm
            .payoff_at_price(&pos_or_panic!(95.0))
            .unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_call_short() {
        let call_option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff = call_option.payoff_at_price(&pos_or_panic!(105.0)).unwrap();
        assert_eq!(call_payoff, dec!(-5.0)); // -max(105 - 100, 0) = -5

        let call_option_otm = create_sample_option_simplest(OptionStyle::Call, Side::Short);
        let call_payoff_otm = call_option_otm
            .payoff_at_price(&pos_or_panic!(95.0))
            .unwrap();
        assert_eq!(call_payoff_otm, Decimal::ZERO); // -max(95 - 100, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_long() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff = put_option.payoff_at_price(&pos_or_panic!(95.0)).unwrap();
        assert_eq!(put_payoff, dec!(5.0)); // max(100 - 95, 0) = 5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Long);
        let put_payoff_otm = put_option_otm
            .payoff_at_price(&pos_or_panic!(105.0))
            .unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // max(100 - 105, 0) = 0
    }

    #[test]
    fn test_payoff_european_put_short() {
        let put_option = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff = put_option.payoff_at_price(&pos_or_panic!(95.0)).unwrap();
        assert_eq!(put_payoff, dec!(-5.0)); // -max(100 - 95, 0) = -5

        let put_option_otm = create_sample_option_simplest(OptionStyle::Put, Side::Short);
        let put_payoff_otm = put_option_otm
            .payoff_at_price(&pos_or_panic!(105.0))
            .unwrap();
        assert_eq!(put_payoff_otm, Decimal::ZERO); // -max(100 - 105, 0) = 0
    }
}

#[cfg(test)]
mod tests_options_payoffs_with_quantity {
    use super::*;
    use crate::model::utils::create_sample_option;

    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_payoff_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 50.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(95.0),
            pos_or_panic!(4.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_payoff_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(105.0),
            pos_or_panic!(3.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(95.0),
            pos_or_panic!(7.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_payoff_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(95.0),
            Positive::TWO,
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 10.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(105.0),
            pos_or_panic!(7.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_payoff_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(95.0),
            pos_or_panic!(3.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), -15.0);

        let option_otm = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(105.0),
            pos_or_panic!(3.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option_otm.payoff().unwrap(), Decimal::ZERO);
    }

    #[test]
    fn test_payoff_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(110.0),
            pos_or_panic!(3.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(option.payoff().unwrap().to_f64().unwrap(), 30.0); // (110 - 100) * 3
    }

    #[test]
    fn test_intrinsic_value_call_long() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            pos_or_panic!(11.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(105.0)).unwrap(),
            dec!(55.0)
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(95.0)).unwrap(),
            Decimal::ZERO
        );
    }

    #[test]
    fn test_intrinsic_value_call_short() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Short,
            Positive::HUNDRED,
            pos_or_panic!(13.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(105.0)).unwrap(),
            dec!(-65.0)
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(95.0)).unwrap(),
            Decimal::ZERO
        );
    }

    #[test]
    fn test_intrinsic_value_put_long() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            pos_or_panic!(17.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(95.0)).unwrap(),
            dec!(85.0)
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(105.0)).unwrap(),
            Decimal::ZERO
        );
    }

    #[test]
    fn test_intrinsic_value_put_short() {
        let option = create_sample_option(
            OptionStyle::Put,
            Side::Short,
            Positive::HUNDRED,
            pos_or_panic!(19.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(95.0)).unwrap(),
            dec!(-95.0)
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(105.0)).unwrap(),
            Decimal::ZERO
        );
    }

    #[test]
    fn test_intrinsic_value_with_quantity() {
        let option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            pos_or_panic!(23.0),
            Positive::HUNDRED,
            pos_or_panic!(0.02),
        );
        assert_eq!(
            option.intrinsic_value(pos_or_panic!(110.0)).unwrap(),
            dec!(230.0)
        ); // (110 - 100) * 23
    }
}

#[cfg(test)]
mod tests_in_the_money {
    use super::*;
    use crate::model::utils::create_sample_option;

    #[test]
    fn test_call_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(110.0),
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.02),
        );
        option.strike_price = Positive::HUNDRED;
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_call_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.02),
        );
        option.strike_price = Positive::HUNDRED;
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_call_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.02),
        );
        option.strike_price = Positive::HUNDRED;
        assert!(!option.is_in_the_money());
    }

    #[test]
    fn test_put_in_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.02),
        );
        option.strike_price = Positive::HUNDRED;
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_put_at_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            Positive::HUNDRED,
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.02),
        );
        option.strike_price = Positive::HUNDRED;
        assert!(option.is_in_the_money());
    }

    #[test]
    fn test_put_out_of_the_money() {
        let mut option = create_sample_option(
            OptionStyle::Put,
            Side::Long,
            pos_or_panic!(110.0),
            Positive::ONE,
            pos_or_panic!(110.0),
            pos_or_panic!(0.02),
        );
        option.strike_price = Positive::HUNDRED;
        assert!(!option.is_in_the_money());
    }
}

#[cfg(test)]
mod tests_serialize_deserialize {
    use super::*;
    use crate::model::utils::create_sample_option_simplest_strike;

    #[test]
    fn test_serialize_deserialize_options() {
        let options = create_sample_option_simplest_strike(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(95.0),
        );
        let serialized = serde_json::to_string(&options).expect("Failed to serialize");
        let deserialized: Options =
            serde_json::from_str(&serialized).expect("Failed to deserialize");
        assert_eq!(options, deserialized);
    }
}
