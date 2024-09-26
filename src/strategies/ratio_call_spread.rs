/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/

use super::base::{Strategies, StrategyType};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::visualization::utils::Graph;
use chrono::Utc;
use tracing::info;

const RATIO_CALL_SPREAD_DESCRIPTION: &str =
    "A Ratio Call Spread involves buying one call option and selling multiple call options \
    at a higher strike price. This strategy is used when a moderate rise in the underlying \
    asset's price is expected, but with limited upside potential.";

pub struct RatioCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<f64>,
    long_call_itm: Position,
    long_call_otm: Position,
    short_call: Position,
    underlying_price: f64,
}

impl RatioCallSpread {
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
        let mut strategy = RatioCallSpread {
            name: "Ratio Call Spread".to_string(),
            kind: StrategyType::RatioCallSpread,
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

        // Calculate break-even points
        let net_debit = strategy.total_cost() / long_quantity as f64;

        strategy
            .break_even_points
            .push(long_call_itm.option.strike_price + net_debit);

        strategy
            .break_even_points
            .push(long_call_otm.option.strike_price - net_debit);

        strategy
    }
}

impl Strategies for RatioCallSpread {
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
        // info!("{:.2} {:.2} {:.2} {:.2} {:.2}", price, long_call_itm_profit , long_call_otm_profit , short_call_profit, long_call_itm_profit + long_call_otm_profit + short_call_profit);
        long_call_itm_profit + long_call_otm_profit + short_call_profit
    }

    fn max_profit(&self) -> f64 {
        let premium_paid = self.long_call_itm.premium + self.long_call_otm.premium;
        let premium_received = self.short_call.premium * self.short_call.option.quantity as f64;
        let range = self.short_call.option.strike_price - self.long_call_itm.option.strike_price;
        let fees = self.fees();
        let max_profit = range - (premium_paid - premium_received + fees);
        max_profit
    }

    fn max_loss(&self) -> f64 {
        self.total_cost()
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

    fn area(&self) -> f64 {
        let range = self.short_call.option.strike_price - self.long_call_itm.option.strike_price;
        let max_profit = self.max_profit();
        (range * max_profit / 2.0) / self.underlying_price * 100.0
    }
}

impl Graph for RatioCallSpread {
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
        self.break_even_points
            .iter()
            .enumerate()
            .map(|(i, &point)| (format!("Break Even {}", i + 1), point))
            .collect()
    }
}

// #[cfg(test)]
// mod tests_ratio_call_spread {
//     use super::*;
//
//     fn setup() -> RatioCallSpread {
//         RatioCallSpread::new(
//             "AAPL".to_string(),
//             150.0,
//             155.0,
//             160.0,
//             ExpirationDate::Days(30.0),
//             0.2,
//             0.01,
//             0.02,
//             1,
//             2,
//             3.0,
//             1.5,
//             0.1,
//             0.1,
//             0.1,
//             0.1,
//         )
//     }
//
//     #[test]
//     fn test_new() {
//         let strategy = setup();
//         assert_eq!(strategy.name, "Ratio Call Spread");
//         assert_eq!(strategy.kind, StrategyType::RatioCallSpread);
//         assert!(strategy.description.contains("A Ratio Call Spread involves"));
//     }
//
//     #[test]
//     fn test_break_even() {
//         let strategy = setup();
//         assert_eq!(strategy.break_even(), 155.0);
//     }
//
//     #[test]
//     fn test_calculate_profit_at() {
//         let strategy = setup();
//         let price = 157.0;
//         assert!(strategy.calculate_profit_at(price) > 0.0);
//     }
//
//     #[test]
//     fn test_max_profit() {
//         let strategy = setup();
//         assert!(strategy.max_profit() > 0.0);
//     }
//
//     #[test]
//     fn test_max_loss() {
//         let strategy = setup();
//         assert_eq!(strategy.max_loss(), strategy.total_cost());
//     }
//
//     #[test]
//     fn test_total_cost() {
//         let strategy = setup();
//         assert!(strategy.total_cost() > 0.0);
//     }
//
//     #[test]
//     fn test_net_premium_received() {
//         let strategy = setup();
//         assert!(strategy.net_premium_received() < 0.0);
//     }
//
//     #[test]
//     fn test_fees() {
//         let strategy = setup();
//         assert_eq!(strategy.fees(), 0.4);
//     }
//
//     #[test]
//     fn test_graph_methods() {
//         let strategy = setup();
//
//         let vertical_lines = strategy.get_vertical_lines();
//         assert_eq!(vertical_lines.len(), 2);
//         assert_eq!(vertical_lines[0].0, "Break Even 1");
//         assert_eq!(vertical_lines[1].0, "Break Even 2");
//
//         let data = vec![150.0, 155.0, 160.0, 165.0, 170.0];
//         let values = strategy.get_values(&data);
//         for (i, &price) in data.iter().enumerate() {
//             assert_eq!(values[i], strategy.calculate_profit_at(price));
//         }
//
//         let title = strategy.title();
//         assert!(title.contains("Ratio Call Spread Strategy"));
//         assert!(title.contains("Call"));
//     }
// }
