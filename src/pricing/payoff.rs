use crate::error::PricingError;
use crate::model::types::{OptionStyle, Side};
use num_traits::ToPrimitive;
use positive::Positive;
use rust_decimal::Decimal;
use tracing::trace;

/// Defines a contract for calculating the payoff value of an option.
///
/// The `Payoff` trait establishes a standard interface for implementing different
/// option payoff calculations. Classes that implement this trait can define specific
/// payoff formulas for various option types (standard calls/puts, exotic options, etc.).
///
/// # Examples
///
/// Implementing the trait for a standard call option:
///
/// ```rust
/// use num_traits::ToPrimitive;
/// use optionstratlib::pricing::{Payoff, PayoffInfo};
/// use optionstratlib::Side;
/// struct CallOption;
///
/// impl Payoff for CallOption {
///     fn payoff(&self, info: &PayoffInfo) -> f64 {
///         let spot = info.spot.value().to_f64().unwrap();
///         let strike = info.strike.value().to_f64().unwrap();
///         match info.side {
///             Side::Long => (spot - strike).max(0.0),
///             Side::Short => -1.0 * (spot - strike).max(0.0),
///         }
///     }
/// }
/// ```
///
/// # Usage
///
/// This trait is typically used within the options pricing module to:
/// - Create standardized payoff calculations for different option types
/// - Enable polymorphic handling of various option payoff strategies
/// - Support both standard and exotic option payoffs through a unified interface
pub trait Payoff {
    /// Calculates the payoff value of an option based on the provided information.
    ///
    /// # Parameters
    ///
    /// * `info` - A reference to a `PayoffInfo` struct containing all necessary data
    ///   for calculating the option's payoff, including spot price, strike price,
    ///   option style, position side, and additional parameters for exotic options.
    ///
    /// # Returns
    ///
    /// Returns the calculated payoff value as a `f64`.
    fn payoff(&self, info: &PayoffInfo) -> f64;
}
/// `PayoffInfo` is a struct that holds information about an option's payoff calculation parameters.
///
/// This structure encapsulates all the necessary variables to calculate the payoff of different
/// option types, including standard options (calls and puts) as well as exotic options like
/// Asian and Lookback options.
///
/// # Usage
///
/// This structure is typically used within the options pricing module to calculate
/// the final payoff value of different option types at expiration or exercise.
///
#[derive(Debug)]
pub struct PayoffInfo {
    /// * `spot` - The current market price of the underlying asset.
    ///   This value is used as the reference price for calculating option payoffs.
    pub spot: Positive,
    /// * `strike` - The strike price specified in the option contract.
    ///   This is the price at which the option holder can buy (for calls) or sell (for puts)
    ///   the underlying asset.
    pub strike: Positive,
    /// * `style` - Defines whether the option is a Call or Put.
    ///   Call options give the right to buy, while put options give the right to sell.
    pub style: OptionStyle,
    /// * `side` - Indicates whether the position is Long (bought) or Short (sold).
    ///   This affects the direction of the payoff calculation.
    pub side: Side,
    /// * `spot_prices` - A collection of historical spot prices used specifically for Asian options.
    ///   Asian options base their payoff on the average price of the underlying asset over a specified period.
    pub spot_prices: Option<Vec<f64>>, // Asian
    /// * `spot_min` - The minimum observed price of the underlying asset during the option's life.
    ///   This field is used specifically for Lookback options where the payoff depends on the
    ///   minimum price reached.
    pub spot_min: Option<f64>, // Lookback
    /// * `spot_max` - The maximum observed price of the underlying asset during the option's life.
    ///   This field is used specifically for Lookback options where the payoff depends on the
    ///   maximum price reached.
    pub spot_max: Option<f64>, // Lookback
}

impl Default for PayoffInfo {
    fn default() -> Self {
        PayoffInfo {
            spot: Positive::ZERO,
            strike: Positive::ZERO,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        }
    }
}

impl PayoffInfo {
    /// Returns the length of the spot prices collection if it exists.
    ///
    /// This method provides a safe way to check the length of the historical spot prices
    /// vector without direct access to the optional field. It's particularly useful when
    /// working with Asian options, which use a collection of historical prices to calculate
    /// their payoff based on average price.
    ///
    /// # Returns
    ///
    /// * `Some(usize)` - The number of spot prices in the collection if it exists
    /// * `None` - If no spot prices are available (the vector is None)
    ///
    /// # Example
    ///
    /// ```
    /// use optionstratlib::pricing::PayoffInfo;
    /// use positive::Positive;
    /// use optionstratlib::model::types::{OptionStyle, Side};
    ///
    /// let payoff_info = PayoffInfo {
    ///     spot: Positive::new(100.0).unwrap(),
    ///     strike: Positive::new(105.0).unwrap(),
    ///     style: OptionStyle::Call,
    ///     side: Side::Long,
    ///     spot_prices: Some(vec![98.0, 99.0, 101.0, 102.0]),
    ///     spot_min: None,
    ///     spot_max: None,
    /// };
    ///
    /// assert_eq!(payoff_info.spot_prices_len(), Some(4));
    /// ```
    pub fn spot_prices_len(&self) -> Option<usize> {
        self.spot_prices.as_ref().map(|vec| vec.len())
    }
}

/// Calculates the standard payoff for an option given its information.
///
/// # Arguments
///
/// * `info` - A reference to a `PayoffInfo` struct that contains the essential details of the option such as its style, spot price, and strike price.
///
/// # Returns
///
/// * `f64` - The payoff value based on the type of the option (call or put).
///
/// This function evaluates the payoff based on the option style:
/// - For a call option: Max(spot price - strike price, 0)
/// - For a put option: Max(strike price - spot price, 0)
pub(crate) fn standard_payoff(info: &PayoffInfo) -> f64 {
    trace!("standard_payoff - spot: {}", info.spot);
    trace!("standard_payoff - info.strike: {}", info.strike);
    trace!(
        "standard_payoff - (info.spot - info.strike): {}",
        info.spot - info.strike
    );

    let spot: Decimal = info.spot.into();
    let strike: Decimal = info.strike.into();

    let payoff = match info.style {
        OptionStyle::Call => (spot - strike).max(Decimal::ZERO).to_f64().unwrap(),
        OptionStyle::Put => (strike - spot).max(Decimal::ZERO).to_f64().unwrap(),
    };

    match info.side {
        Side::Long => payoff,
        Side::Short => -payoff,
    }
}

/// Defines the profit calculation behavior for financial instruments.
///
/// This trait is used to calculate and visualize profit values at different price points
/// for various financial instruments and strategies. It provides:
/// 1. A required method to calculate the actual profit value at a given price
/// 2. A default implementation to convert the profit calculation into a visualization point
///
/// # Usage
///
/// Implement this trait for any type that can calculate profit at a specific price point,
/// such as options contracts, spreads, or complex trading strategies.
///
pub trait Profit {
    /// Calculates the profit at a specified price.
    ///
    /// # Parameters
    ///
    /// * `price` - A positive value representing the price at which to calculate profit
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, PricingError>` - The calculated profit as a Decimal value,
    ///   or an error if the calculation fails
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError>;

    /// Creates a chart point representation of the profit at the given price.
    ///
    /// This method automatically determines the appropriate visualization properties based
    /// on the profit value, such as color (green for positive profit, red for negative).
    ///
    /// # Parameters
    ///
    /// * `price` - A positive value representing the price for which to create a chart point
    ///
    /// # Returns
    ///
    /// * `ChartPoint<(f64, f64)>` - A formatted chart point with coordinates (price, profit),
    ///   styling, and a formatted profit label
    fn get_point_at_price(&self, _price: &Positive) -> Result<(Decimal, Decimal), PricingError> {
        let profit = self.calculate_profit_at(_price)?;
        let price: Decimal = _price.into();
        let point = (price, profit);
        trace!("get_point_at_price - point: {:?}", point);
        Ok(point)
    }
}

#[cfg(test)]
mod tests_standard_payoff {
    use super::*;
    use crate::model::types::OptionType;
    use positive::pos_or_panic;

    #[test]
    fn test_call_option_in_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 10.0);
    }

    #[test]
    fn test_call_option_at_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 0.0);
    }

    #[test]
    fn test_call_option_out_of_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 0.0);
    }

    #[test]
    fn test_put_option_in_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(90.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 10.0);
    }

    #[test]
    fn test_put_option_at_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 0.0);
    }

    #[test]
    fn test_put_option_out_of_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos_or_panic!(110.0),
            strike: Positive::HUNDRED,
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 0.0);
    }
}
