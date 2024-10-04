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
use crate::constants::{
    STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER, ZERO,
};
use crate::model::chain::{OptionChain, OptionData};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::{ShapeStyle, BLACK};
use tracing::{debug, error};

const DESCRIPTION: &str = "A bull call spread involves buying a call option with a lower strike \
price and selling a call option with a higher strike price, both with the same expiration date. \
This strategy is used when a moderate rise in the underlying asset's price is expected.";

#[derive(Clone, Debug)]
pub struct BullCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call: Position,
    short_call: Position,
}

impl BullCallSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        lower_strike: PositiveF64,
        higher_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
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

        // TODO: push the break even points
        // strategy.break_even_points.push(lower_call.break_even());

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

        // TODO: push the break even points
        // strategy.break_even_points.push(higher_call.break_even());

        strategy.validate();
        strategy
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        // TODO: Check this
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for short_index in 1..options.len() - 1 {
            let short_option = &options[short_index];
            if !self.is_valid_short_option(short_option, &side) {
                debug!("Skipping short option: {}", short_option.strike_price);
                continue;
            }

            for long_itm_index in 0..short_index {
                let long_otm_index = short_index + (short_index - long_itm_index);

                if long_otm_index >= options.len() {
                    continue;
                }

                let long_option = &options[long_itm_index];

                if !self.are_valid_prices(long_option, short_option) {
                    continue;
                }

                let strategy = self.create_strategy(option_chain, long_option, short_option);

                if !strategy.validate() {
                    panic!("Invalid strategy");
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio(),
                    OptimizationCriteria::Area => strategy.profit_area(),
                };

                debug!(
                    "{}: {:.2}%",
                    if matches!(criteria, OptimizationCriteria::Ratio) {
                        "Ratio"
                    } else {
                        "Area"
                    },
                    current_value
                );

                if current_value > best_value {
                    best_value = current_value;
                    self.clone_from(&strategy);
                }
            }
        }
    }

    fn is_valid_short_option(&self, short_option: &OptionData, side: &FindOptimalSide) -> bool {
        match side {
            FindOptimalSide::Upper => {
                short_option.strike_price >= self.short_call.option.underlying_price
            }
            FindOptimalSide::Lower => {
                short_option.strike_price <= self.short_call.option.underlying_price
            }
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                short_option.strike_price >= *start && short_option.strike_price <= *end
            }
        }
    }

    fn are_valid_prices(&self, long_option: &OptionData, short_option: &OptionData) -> bool {
        long_option.call_ask > ZERO && short_option.call_bid > ZERO
    }

    fn create_strategy(
        &self,
        option_chain: &OptionChain,
        long_option: &OptionData,
        short_option: &OptionData,
    ) -> BullCallSpread {
        BullCallSpread::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_option.strike_price,
            short_option.strike_price,
            self.short_call.option.expiration_date.clone(),
            short_option.implied_volatility,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long_option.call_ask,
            short_option.call_bid,
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
        )
    }
}

impl Strategies for BullCallSpread {
    fn add_leg(&mut self, position: Position) {
        match position.option.side {
            Side::Long => self.long_call = position,
            Side::Short => self.short_call = position,
        }
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        vec![
            self.short_call.option.strike_price
                - pos!(self.calculate_profit_at(self.short_call.option.strike_price))
                    / self.long_call.option.quantity,
        ]
    }

    fn max_profit(&self) -> f64 {
        self.calculate_profit_at(self.short_call.option.strike_price)
    }

    fn max_loss(&self) -> f64 {
        self.calculate_profit_at(self.long_call.option.strike_price)
            .abs()
    }

    fn total_cost(&self) -> f64 {
        self.long_call.net_cost() - self.short_call.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.short_call.open_fee
            + self.short_call.close_fee
            + self.long_call.open_fee
            + self.long_call.close_fee
    }

    fn profit_area(&self) -> f64 {
        // TODO: Implement this
        // (self.short_call.option.strike_price - self.break_even()) * self.max_profit() / 100.0
        ZERO
    }

    fn profit_ratio(&self) -> f64 {
        (self.max_profit() / self.max_loss()).abs() * 100.0
    }

    fn best_ratio(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Ratio);
    }

    fn best_area(&mut self, option_chain: &OptionChain, side: FindOptimalSide) {
        self.find_optimal(option_chain, side, OptimizationCriteria::Area);
    }

    fn validate(&self) -> bool {
        if self.name.is_empty() {
            error!("Symbol is required");
            return false;
        }
        if !self.long_call.validate() {
            return false;
        }
        if !self.short_call.validate() {
            return false;
        }

        if self.long_call.option.underlying_price <= PZERO {
            error!("Underlying price must be greater than zero");
            return false;
        }
        if self.short_call.option.strike_price <= self.long_call.option.strike_price {
            error!("Long call strike price must be less than short call strike price");
            return false;
        }
        true
    }

    fn best_range_to_show(&self, step: PositiveF64) -> Option<Vec<PositiveF64>> {
        let (first_option, last_option) = (
            self.long_call.option.clone(),
            self.short_call.option.clone(),
        );
        let start_price = first_option.strike_price * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = last_option.strike_price * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Profit for BullCallSpread {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price) + self.long_call.pnl_at_expiration(&price)
    }
}

impl Graph for BullCallSpread {
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

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: ZERO, // TODO current price
            y_range: (-50000.0, 50000.0),
            label: "Break Even".to_string(),
            label_offset: (5.0, 5.0),
            line_color: BLACK,
            label_color: BLACK,
            line_style: ShapeStyle::from(&BLACK).stroke_width(1),
            font_size: 18,
        }];

        vertical_lines
    }
}

#[cfg(test)]
mod tests_create_bull_call_spread {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;

    fn create_sample_bull_call_spread() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.0,
            0.0,
            pos!(1.0),
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
        assert_relative_eq!(strategy.break_even_points[0].value(), 97.71, epsilon = 1e-6);
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
        assert_eq!(strategy.break_even()[0], 94.0);
        assert_eq!(strategy.calculate_profit_at(pos!(100.0)), 6.0);
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
    use crate::pos;
    use approx::assert_relative_eq;

    fn create_sample_bull_call_spread() -> BullCallSpread {
        BullCallSpread::new(
            "GOLD".to_string(),
            pos!(2505.8),
            pos!(2460.0),
            pos!(2515.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.05,
            0.0,
            pos!(2.0),
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
        assert_relative_eq!(
            strategy.break_even_points[0].value(),
            2488.42,
            epsilon = 1e-6
        );
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
        assert_eq!(strategy.break_even()[0], 2484.19);
        assert_relative_eq!(
            strategy.calculate_profit_at(pos!(2500.0)),
            31.620000,
            epsilon = 1e-6
        );
        assert_relative_eq!(strategy.max_profit(), 61.6200000, epsilon = 1e-6);
        assert_relative_eq!(strategy.max_loss(), 48.37999999, epsilon = 1e-6);
        assert_relative_eq!(strategy.total_cost(), 48.37999999, epsilon = 1e-6);
        assert_relative_eq!(strategy.net_premium_received(), 8.46, epsilon = 1e-6);
    }
}
