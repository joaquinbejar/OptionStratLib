use crate::model::types::OptionStyle;

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
    pub spot_prices: Option<Vec<f64>>, // Asian
    pub spot_min: Option<f64>,         // Lookback
    pub spot_max: Option<f64>,         // Lookback
}

impl PayoffInfo {
    pub fn spot_prices_len(&self) -> Option<usize> {
        self.spot_prices.as_ref().map(|vec| vec.len())
    }
}
