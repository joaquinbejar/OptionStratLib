/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use super::base::{Strategies, StrategyType};
use crate::constants::DARK_GREEN;
use crate::constants::{
    DARK_BLUE, STRIKE_PRICE_LOWER_BOUND_MULTIPLIER, STRIKE_PRICE_UPPER_BOUND_MULTIPLIER, ZERO,
};
use crate::model::chain::{OptionChain, OptionData};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::strategies::utils::{calculate_price_range, FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::ChartPoint;
use crate::visualization::utils::Graph;
use chrono::Utc;
use tracing::{debug, error};

const RATIO_CALL_SPREAD_DESCRIPTION: &str =
    "A Ratio Call Spread involves buying one call option and selling multiple call options \
    at a higher strike price. This strategy is used when a moderate rise in the underlying \
    asset's price is expected, but with limited upside potential.";

#[derive(Clone, Debug)]
pub struct CallButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<f64>,
    long_call_itm: Position,
    long_call_otm: Position,
    short_call: Position,
    underlying_price: f64,
}

impl CallButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: f64,
        long_strike_itm: f64,
        long_strike_otm: f64,
        short_strike: f64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        long_quantity: u32,
        short_quantity: u32,
        premium_long_itm: f64,
        premium_long_otm: f64,
        premium_short: f64,
        open_fee_long: f64,
        close_fee_long: f64,
        open_fee_short: f64,
        close_fee_short: f64,
    ) -> Self {
        let mut strategy = CallButterfly {
            name: underlying_symbol.to_string(),
            kind: StrategyType::CallButterfly,
            description: RATIO_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call_itm: Position::default(),
            long_call_otm: Position::default(),
            short_call: Position::default(),
            underlying_price,
        };
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike,
            expiration.clone(),
            implied_volatility,
            short_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short,
            Utc::now(),
            open_fee_short,
            close_fee_short,
        );
        strategy.add_leg(short_call.clone());
        strategy.short_call = short_call;

        let long_call_itm_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike_itm,
            expiration.clone(),
            implied_volatility,
            long_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call_itm = Position::new(
            long_call_itm_option,
            premium_long_itm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy.add_leg(long_call_itm.clone());
        strategy.long_call_itm = long_call_itm;

        let long_call_otm_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike_otm,
            expiration.clone(),
            implied_volatility,
            long_quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call_otm = Position::new(
            long_call_otm_option,
            premium_long_otm,
            Utc::now(),
            open_fee_long,
            close_fee_long,
        );
        strategy.add_leg(long_call_otm.clone());
        strategy.long_call_otm = long_call_otm;

        // Calculate break-even points
        let loss_at_itm_strike =
            strategy.calculate_profit_at(strategy.long_call_itm.option.strike_price);
        let loss_at_otm_strike =
            strategy.calculate_profit_at(strategy.long_call_otm.option.strike_price);

        let first_bep = strategy.long_call_itm.option.strike_price
            - (loss_at_itm_strike / long_quantity as f64);
        strategy.break_even_points.push(first_bep);

        let second_bep = strategy.long_call_otm.option.strike_price
            + (loss_at_otm_strike / long_quantity as f64);
        strategy.break_even_points.push(second_bep);

        // TODO: fix break_even_points when legs have same loss
        strategy
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
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

                let long_itm = &options[long_itm_index];
                let long_otm = &options[long_otm_index];

                if !self.are_valid_prices(long_itm, long_otm, short_option) {
                    continue;
                }

                let strategy = self.create_strategy(option_chain, long_itm, long_otm, short_option);

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
            FindOptimalSide::Upper => short_option.strike_price >= self.underlying_price,
            FindOptimalSide::Lower => short_option.strike_price <= self.underlying_price,
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                short_option.strike_price >= *start && short_option.strike_price <= *end
            }
        }
    }

    fn are_valid_prices(
        &self,
        long_itm: &OptionData,
        long_otm: &OptionData,
        short_option: &OptionData,
    ) -> bool {
        long_itm.call_ask > ZERO && long_otm.call_ask > ZERO && short_option.call_bid > ZERO
    }

    fn create_strategy(
        &self,
        option_chain: &OptionChain,
        long_itm: &OptionData,
        long_otm: &OptionData,
        short_option: &OptionData,
    ) -> CallButterfly {
        CallButterfly::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_itm.strike_price,
            long_otm.strike_price,
            short_option.strike_price,
            self.short_call.option.expiration_date.clone(),
            short_option.implied_volatility,
            self.long_call_itm.option.risk_free_rate,
            self.long_call_itm.option.dividend_yield,
            self.long_call_itm.option.quantity,
            self.short_call.option.quantity,
            long_itm.call_ask,
            long_otm.call_ask,
            short_option.call_bid,
            self.long_call_itm.open_fee,
            self.long_call_itm.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
        )
    }
}

impl Default for CallButterfly {
    fn default() -> Self {
        CallButterfly::new(
            "".to_string(),
            0.0,
            0.0,
            0.0,
            0.0,
            ExpirationDate::Days(0.0),
            0.0,
            0.0,
            0.0,
            1,
            2,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
        )
    }
}

impl Strategies for CallButterfly {
    fn add_leg(&mut self, position: Position) {
        match position.option.side {
            Side::Long => {
                if position.option.strike_price >= self.short_call.option.strike_price {
                    self.long_call_otm = position
                } else {
                    self.long_call_itm = position
                }
            }
            Side::Short => self.short_call = position,
        }
    }

    fn break_even(&self) -> f64 {
        self.break_even_points[0]
    }

    fn calculate_profit_at(&self, price: f64) -> f64 {
        let long_call_itm_profit = self.long_call_itm.pnl_at_expiration(Some(price));
        let long_call_otm_profit = self.long_call_otm.pnl_at_expiration(Some(price));
        let short_call_profit = self.short_call.pnl_at_expiration(Some(price));
        long_call_itm_profit + long_call_otm_profit + short_call_profit
    }

    fn max_profit(&self) -> f64 {
        self.calculate_profit_at(self.short_call.option.strike_price)
    }

    fn max_loss(&self) -> f64 {
        let lower_loss = self.calculate_profit_at(self.long_call_itm.option.strike_price);
        let upper_loss = self.calculate_profit_at(self.long_call_otm.option.strike_price);

        lower_loss.min(upper_loss)
    }

    fn total_cost(&self) -> f64 {
        self.long_call_itm.net_cost() + self.long_call_otm.net_cost() - self.short_call.net_cost()
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.long_call_itm.open_fee
            + self.long_call_itm.close_fee
            + self.long_call_otm.open_fee
            + self.long_call_otm.close_fee
            + self.short_call.open_fee * self.short_call.option.quantity as f64
            + self.short_call.close_fee * self.short_call.option.quantity as f64
    }

    fn profit_area(&self) -> f64 {
        let range = self.short_call.option.strike_price - self.long_call_itm.option.strike_price;
        let max_profit = self.max_profit();
        (range * max_profit / 2.0) / self.underlying_price * 100.0
    }

    fn profit_ratio(&self) -> f64 {
        self.max_profit() / self.max_loss().abs()
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
        if !self.long_call_itm.validate() {
            return false;
        }
        if !self.long_call_otm.validate() {
            return false;
        }
        if !self.short_call.validate() {
            return false;
        }
        if self.underlying_price <= 0.0 {
            error!("Underlying price must be greater than zero");
            return false;
        }
        if self.short_call.option.quantity != self.long_call_itm.option.quantity * 2 {
            error!("Short call quantity must be twice the long call quantity and currently is short: {} and long: {}",
                self.short_call.option.quantity, self.long_call_itm.option.quantity);
            return false;
        }
        true
    }

    fn best_range_to_show(&self, step: f64) -> Option<Vec<f64>> {
        let (first_option, last_option) = (
            self.long_call_itm.option.clone(),
            self.long_call_otm.option.clone(),
        );
        let start_price = first_option.strike_price * STRIKE_PRICE_LOWER_BOUND_MULTIPLIER;
        let end_price = last_option.strike_price * STRIKE_PRICE_UPPER_BOUND_MULTIPLIER;
        Some(calculate_price_range(start_price, end_price, step))
    }
}

impl Graph for CallButterfly {
    fn title(&self) -> String {
        let strategy_title = format!("Ratio Call Spread Strategy: {:?}", self.kind);
        let long_call_itm_title = self.long_call_itm.title();
        let long_call_otm_title = self.long_call_otm.title();
        let short_call_title = self.short_call.title();

        format!(
            "{}\n\t{}\n\t{}\n\t{}",
            strategy_title, long_call_itm_title, long_call_otm_title, short_call_title
        )
    }

    fn get_values(&self, data: &[f64]) -> Vec<f64> {
        data.iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect()
    }

    fn get_vertical_lines(&self) -> Vec<(String, f64)> {
        vec![("Current Price".to_string(), self.short_call.option.underlying_price,)]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0], 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: (-10.0, -1.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 3,
            font_size: 15,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1], 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: (-10.0, -1.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 3,
            font_size: 15,
        });

        points.push(ChartPoint {
            coordinates: (self.short_call.option.strike_price, self.max_profit()),
            label: format!("Max Profit\n\n{:.2}", self.max_profit()),
            label_offset: (2.0, 1.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 3,
            font_size: 15,
        });
        points
    }
}

#[cfg(test)]
mod tests_call_butterfly {
    use super::*;
    use approx::assert_relative_eq;

    fn setup() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            150.0,
            155.0,
            160.0,
            157.5,
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            1,
            2,
            3.0,
            1.5,
            2.0,
            0.1,
            0.1,
            0.1,
            0.0,
        )
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "AAPL");
        assert_eq!(strategy.kind, StrategyType::CallButterfly);
        assert!(strategy
            .description
            .contains("A Ratio Call Spread involves"));
    }

    #[test]
    fn test_break_even() {
        let strategy = setup();
        assert_eq!(strategy.break_even(), 156.1);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 157.0;
        assert!(strategy.calculate_profit_at(price) > 0.0);
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert!(strategy.max_profit() > 0.0);
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(strategy.max_loss().abs(), strategy.total_cost());
    }

    #[test]
    fn test_total_cost() {
        let strategy = setup();
        assert!(strategy.total_cost() > 0.0);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(strategy.net_premium_received(), 3.8);
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        assert_relative_eq!(strategy.fees(), 0.6, epsilon = f64::EPSILON);
    }

    #[test]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].0, "Current Price");

        let data = vec![150.0, 155.0, 160.0, 165.0, 170.0];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(values[i], strategy.calculate_profit_at(price));
        }

        let title = strategy.title();
        assert!(title.contains("Ratio Call Spread Strategy"));
        assert!(title.contains("Call"));
    }
}
