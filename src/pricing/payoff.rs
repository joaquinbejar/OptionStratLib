use crate::constants::{DARK_GREEN, ZERO};
use crate::model::types::{OptionStyle, Side};
use crate::visualization::model::{ChartPoint, LabelOffsetType};
use crate::Positive;
use plotters::prelude::RED;
use tracing::trace;

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
/// - `spot_prices` (`Option<Vec<f64>>`): A vector of spot prices for Asian options, where the payoff depends on the average price of the underlying asset.
/// - `spot_min` (`Option<f64>`): The minimum spot price for Lookback options, where the payoff depends on the minimum price of the underlying asset.
/// - `spot_max` (`Option<f64>`): The maximum spot price for Lookback options, where the payoff depends on the maximum price of the underlying asset.
///
#[derive(Debug)]
pub struct PayoffInfo {
    pub spot: Positive,
    pub strike: Positive,
    pub style: OptionStyle,
    pub side: Side,
    pub spot_prices: Option<Vec<f64>>, // Asian
    pub spot_min: Option<f64>,         // Lookback
    pub spot_max: Option<f64>,         // Lookback
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
    let payoff = match info.style {
        OptionStyle::Call => (info.spot - info.strike).max(Positive::ZERO).into(),
        OptionStyle::Put => (info.strike - info.spot).max(Positive::ZERO).into(),
    };

    match info.side {
        Side::Long => payoff,
        Side::Short => -payoff,
    }
}

pub trait Profit {
    fn calculate_profit_at(&self, price: Positive) -> f64;

    fn get_point_at_price(&self, price: Positive) -> ChartPoint<(f64, f64)> {
        let value_at_current_price = self.calculate_profit_at(price);
        let color = if value_at_current_price >= ZERO {
            DARK_GREEN
        } else {
            RED
        };
        ChartPoint {
            coordinates: (price.into(), value_at_current_price),
            label: format!("{:.2}", value_at_current_price),
            label_offset: LabelOffsetType::Relative(4.0, 1.0),
            point_color: color,
            label_color: color,
            point_size: 5,
            font_size: 18,
        }
    }
}

#[cfg(test)]
mod tests_standard_payoff {
    use super::*;
    use crate::model::types::OptionType;
    use crate::pos;

    #[test]
    fn test_call_option_in_the_money() {
        let option_type = OptionType::European;
        let info = PayoffInfo {
            spot: pos!(110.0),
            strike: pos!(100.0),
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
            spot: pos!(100.0),
            strike: pos!(100.0),
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
            spot: pos!(90.0),
            strike: pos!(100.0),
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
            spot: pos!(90.0),
            strike: pos!(100.0),
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
            spot: pos!(100.0),
            strike: pos!(100.0),
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
            spot: pos!(110.0),
            strike: pos!(100.0),
            style: OptionStyle::Put,
            side: Side::Long,
            spot_prices: None,
            spot_min: None,
            spot_max: None,
        };
        assert_eq!(option_type.payoff(&info), 0.0);
    }
}
