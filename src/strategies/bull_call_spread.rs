/*
Bull Call Spread Strategy

A bull call spread involves buying a call option with a lower strike price and selling a call option with a higher strike price, both with the same expiration date.
This strategy is used when a moderate rise in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential
- Limited risk
- Lower cost than buying a call option outright
*/
use super::base::{Strategies, StrategyType};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::visualization::utils::Graph;
use chrono::Utc;

const DESCRIPTION: &str = "A bull call spread involves buying a call option with a lower strike \
price and selling a call option with a higher strike price, both with the same expiration date. \
This strategy is used when a moderate rise in the underlying asset's price is expected.";

pub struct BullCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<f64>,
    long_call: Position,
    short_call: Position,
}

impl BullCallSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
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
    ) -> Self {
        let mut strategy = BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        };

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
        strategy.break_even_points.push(lower_call.break_even());

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
        strategy.break_even_points.push(higher_call.break_even());

        strategy
    }
}

impl Strategies for BullCallSpread {
    fn add_leg(&mut self, position: Position) {
        match position.option.side {
            Side::Long => self.long_call = position,
            Side::Short => self.short_call = position,
        }
    }

    fn break_even(&self) -> f64 {
        self.long_call.option.strike_price
            + (self.long_call.total_cost() - self.short_call.net_premium_received())
                / self.long_call.option.quantity as f64
    }

    fn calculate_profit_at(&self, price: f64) -> f64 {
        self.long_call.pnl_at_expiration(Some(price))
            + self.short_call.pnl_at_expiration(Some(price))
    }

    fn max_profit(&self) -> f64 {
        (self.short_call.option.strike_price - self.long_call.option.strike_price)
            * self.long_call.option.quantity as f64
            - self.total_cost()
    }

    fn max_loss(&self) -> f64 {
        self.total_cost()
    }

    fn total_cost(&self) -> f64 {
        self.long_call.net_cost() - self.short_call.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received()
    }
}

impl Graph for BullCallSpread {
    fn get_vertical_lines(&self) -> Vec<(String, f64)> {
        [("Break Even".to_string(), self.break_even())].to_vec()
    }

    fn get_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect()
    }

    fn title(&self) -> String {
        let strategy_title = format!("Strategy: {:?}", self.kind);
        let leg_titles: Vec<String> = [self.long_call.title(), self.short_call.title()]
            .iter()
            .map(|leg| leg.to_string())
            .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n{}", strategy_title, leg_titles.join("\n"))
        }
    }
}

#[cfg(test)]
mod tests_create_bull_call_spread {
    use super::*;
    use crate::model::types::ExpirationDate;
    use approx::assert_relative_eq;

    fn create_sample_bull_call_spread() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            100.0,
            90.0,
            110.0,
            ExpirationDate::Days(30.0),
            0.2,
            0.0,
            0.0,
            1,
            5.71,
            5.71,
            1.0,
            1.0,
            1.0,
            1.0,
        )
    }

    #[test]
    fn test_bull_call_spread_creation() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.name, "Bull Call Spread");
        assert_eq!(strategy.kind, StrategyType::BullCallSpread);
        assert_eq!(strategy.long_call.option.side, Side::Long);
        assert_eq!(strategy.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_bull_call_spread_legs() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.long_call.option.strike_price, 90.0);
        assert_eq!(strategy.long_call.option.side, Side::Long);
        assert_eq!(strategy.short_call.option.strike_price, 110.0);
        assert_eq!(strategy.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_bull_call_spread_max_profit() {
        let strategy = create_sample_bull_call_spread();
        assert_relative_eq!(strategy.max_profit(), 16.0, epsilon = 1e-6);
    }

    #[test]
    fn test_bull_call_spread_max_loss() {
        let strategy = create_sample_bull_call_spread();
        assert_relative_eq!(strategy.max_loss(), 4.0, epsilon = 1e-6);
    }

    #[test]
    fn test_bull_call_spread_break_even() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(strategy.break_even_points[0], 97.71, epsilon = 1e-6);
    }

    #[test]
    fn test_bull_call_spread_premiums() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.long_call.premium, 5.71);
        assert_eq!(strategy.short_call.premium, 5.71);
    }

    #[test]
    fn test_bull_call_spread_fees() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.long_call.open_fee, 1.0);
        assert_eq!(strategy.long_call.close_fee, 1.0);
        assert_eq!(strategy.short_call.open_fee, 1.0);
        assert_eq!(strategy.short_call.close_fee, 1.0);
    }

    #[test]
    fn test_bull_call_spread_trait_strategies() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.break_even(), 94.0);
        assert_eq!(strategy.calculate_profit_at(100.0), 6.0);
        assert_eq!(strategy.max_profit(), 16.0);
        assert_eq!(strategy.max_loss(), 4.0);
        assert_eq!(strategy.total_cost(), 4.0);
        assert_eq!(strategy.net_premium_received(), 3.71);
    }
}

#[cfg(test)]
mod tests_create_bull_call_spread_gold {
    use super::*;
    use crate::model::types::ExpirationDate;
    use approx::assert_relative_eq;

    fn create_sample_bull_call_spread() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            2505.8,
            2460.0,
            2515.0,
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            2,
            27.26,
            5.33,
            0.58,
            0.58,
            0.55,
            0.55,
        )
    }

    #[test]
    fn test_bull_call_spread_creation() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.name, "Bull Call Spread");
        assert_eq!(strategy.kind, StrategyType::BullCallSpread);
        assert_eq!(strategy.long_call.option.side, Side::Long);
        assert_eq!(strategy.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_bull_call_spread_legs() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.long_call.option.strike_price, 2460.0);
        assert_eq!(strategy.long_call.option.side, Side::Long);
        assert_eq!(strategy.short_call.option.strike_price, 2515.0);
        assert_eq!(strategy.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_bull_call_spread_max_profit() {
        let strategy = create_sample_bull_call_spread();
        assert_relative_eq!(strategy.max_profit(), 61.62000, epsilon = 1e-6);
    }

    #[test]
    fn test_bull_call_spread_max_loss() {
        let strategy = create_sample_bull_call_spread();
        assert_relative_eq!(strategy.max_loss(), 48.37999999, epsilon = 1e-6);
    }

    #[test]
    fn test_bull_call_spread_break_even() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.break_even_points.len(), 2);
        assert_relative_eq!(strategy.break_even_points[0], 2488.42, epsilon = 1e-6);
    }

    #[test]
    fn test_bull_call_spread_premiums() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.long_call.premium, 27.26);
        assert_eq!(strategy.short_call.premium, 5.33);
    }

    #[test]
    fn test_bull_call_spread_fees() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.long_call.open_fee, 0.58);
        assert_eq!(strategy.long_call.close_fee, 0.58);
        assert_eq!(strategy.short_call.open_fee, 0.55);
        assert_eq!(strategy.short_call.close_fee, 0.55);
    }

    #[test]
    fn test_bull_call_spread_trait_strategies() {
        let strategy = create_sample_bull_call_spread();
        assert_eq!(strategy.break_even(), 2484.19);
        assert_relative_eq!(
            strategy.calculate_profit_at(2500.0),
            31.620000,
            epsilon = 1e-6
        );
        assert_relative_eq!(strategy.max_profit(), 61.6200000, epsilon = 1e-6);
        assert_relative_eq!(strategy.max_loss(), 48.37999999, epsilon = 1e-6);
        assert_relative_eq!(strategy.total_cost(), 48.37999999, epsilon = 1e-6);
        assert_relative_eq!(strategy.net_premium_received(), 8.46, epsilon = 1e-6);
    }
}
