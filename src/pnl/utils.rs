/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/8/24
******************************************************************************/
use crate::model::option::Options;

/// Calculates the profit and loss (P&L) for a given option based on the current price of the underlying asset.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` instance, which contains details about the option, such as strike price and premium.
/// * `current_price` - The current price of the underlying asset.
///
/// # Returns
///
/// The calculated profit and loss (P&L) as a floating-point number (`f64`). This is determined by subtracting the option's
/// premium from its intrinsic value, based on the current price.
///
pub fn pnl(option: &Options, current_price: f64) -> f64 {
    option.intrinsic_value(current_price) - option.premium
}

/// Computes the profit and loss (PnL) at expiration for a given options contract.
///
/// # Arguments
///
/// * `option` - A reference to an `Options` struct representing the options contract.
///
/// # Returns
///
/// A `f64` value representing the profit and loss at expiration.
///
/// This function calls the `pnl` function with the options contract and its underlying price
/// to compute the profit and loss.
pub fn pnl_at_expiration(option: &Options) -> f64 {
    pnl(option, option.underlying_price)
}
