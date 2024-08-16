use crate::model::types::{OptionStyle, Side};

pub trait Payoff {
    fn payoff(&self, info: &PayoffInfo) -> f64;
}

/// `PayoffInfo` is a struct that holds information about an option's payoff variables.
///
/// # Fields
///
/// - `spot` (f64): The current spot price of the underlying asset.
/// - `strike` (f64): The strike price of the option.
/// - `style` (OptionStyle): The style of the option (e.g., European, American).
/// - `spot_prices` (Option<Vec<f64>>): A vector of spot prices for Asian options, where the payoff depends on the average price of the underlying asset.
/// - `spot_min` (Option<f64>): The minimum spot price for Lookback options, where the payoff depends on the minimum price of the underlying asset.
/// - `spot_max` (Option<f64>): The maximum spot price for Lookback options, where the payoff depends on the maximum price of the underlying asset.
///
pub struct PayoffInfo {
    pub spot: f64,
    pub strike: f64,
    pub style: OptionStyle,
    pub side: Side,
    pub spot_prices: Option<Vec<f64>>, // Asian
    pub spot_min: Option<f64>,         // Lookback
    pub spot_max: Option<f64>,         // Lookback
}

impl Default for PayoffInfo {
    fn default() -> Self {
        PayoffInfo {
            spot: 0.0,
            strike: 0.0,
            style: OptionStyle::Call,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        }
    }
}

impl PayoffInfo {
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
    let payoff = match info.style {
        OptionStyle::Call => (info.spot - info.strike).max(0.0),
        OptionStyle::Put => (info.strike - info.spot).max(0.0),
    };
    match info.side {
        Side::Long => payoff,
        Side::Short => -payoff,
    }
}
