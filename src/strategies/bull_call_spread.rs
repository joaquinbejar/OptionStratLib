/*
Bull Call Spread Strategy

A bull call spread involves buying a call option with a lower strike price and selling a call option with a higher strike price, both with the same expiration date.
This strategy is used when a moderate rise in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential
- Limited risk
- Lower cost than buying a call option outright
*/

// src/strategies/bull_call_spread.rs

use super::base::{Strategy, StrategyType};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use chrono::Utc;

pub fn create_bull_call_spread(
    underlying_symbol: String,
    underlying_price: f64,
    lower_strike: f64,
    higher_strike: f64,
    expiration: ExpirationDate,
    implied_volatility: f64,
    risk_free_rate: f64,
    dividend_yield: f64,
    quantity: u32,
    premium_long_call: f64,
    premium_short_call: f64,
    open_fee_long_call: f64,
    close_fee_long_call: f64,
    open_fee_short_call: f64,
    close_fee_short_call: f64,
) -> Strategy {
    let mut strategy = Strategy::new(
        "Bull Call Spread".to_string(),
        StrategyType::BullCallSpread,
        "A strategy involving buying a call option with a lower strike price and selling a call option with a higher strike price.".to_string(),
    );

    // Add the long call option with lower strike
    let lower_call_option = Options::new(
        OptionType::European,
        Side::Long,
        underlying_symbol.clone(),
        lower_strike,
        expiration.clone(),
        implied_volatility,
        quantity,
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let lower_call = Position::new(
        lower_call_option,
        premium_long_call,
        Utc::now(),
        open_fee_long_call,
        close_fee_long_call,
    );

    strategy.add_leg(lower_call.clone());

    // Add the short call option with higher strike
    let higher_call_option = Options::new(
        OptionType::European,
        Side::Short,
        underlying_symbol,
        higher_strike,
        expiration.clone(),
        implied_volatility,
        quantity,
        underlying_price,
        risk_free_rate,
        OptionStyle::Call,
        dividend_yield,
        None,
    );
    let higher_call = Position::new(
        higher_call_option,
        premium_short_call,
        Utc::now(),
        open_fee_short_call,
        close_fee_short_call,
    );
    strategy.add_leg(higher_call.clone());

    // Calculate and set max profit, max loss, and break-even points
    let lower_premium = lower_call.max_loss();
    let higher_premium = higher_call.max_profit();
    let net_premium = lower_premium - higher_premium;

    let max_profit = higher_strike - lower_strike - net_premium;
    strategy.set_max_profit(max_profit);

    let max_loss = net_premium;
    strategy.set_max_loss(max_loss);

    let break_even = lower_strike + net_premium;
    strategy.add_break_even_point(break_even);

    strategy
}
