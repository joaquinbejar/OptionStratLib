/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
/*
    The "Poor Man's Covered Call" is an options strategy designed to simulate a traditional covered call,
    but with a lower capital requirement. In a standard covered call, an investor holds a long position
    in the underlying asset (e.g., a stock) and sells a call option against it to generate premium income.
    This strategy works well for neutral to slightly bullish market outlooks.
    However, instead of purchasing the underlying asset (which can be capital-intensive), the "Poor Man's
    Covered Call" involves buying a deep-in-the-money LEAP (Long-term Equity Anticipation Security) call
    option with a long expiration date and selling a short-term out-of-the-money call option against it.
    By using a LEAP, the investor still benefits from the movement of the underlying asset while avoiding
    the need to purchase it outright. The premium collected from selling the short-term call generates income
    and helps offset the cost of the LEAP.
    The strategy has two main components:
    1. **Long LEAP Call**: This serves as a substitute for holding the underlying asset. The deep-in-the-money
       LEAP behaves similarly to the underlying asset's price movement but costs a fraction of its price.
       The LEAP should have a delta close to 1, meaning it moves nearly dollar-for-dollar with the underlying asset.
    2. **Short Call**: A short-term out-of-the-money call is sold against the long LEAP. This generates premium
       income, and if the underlying asset's price rises above the strike price of the short call, the investor may
       need to sell the asset (or close the position), locking in potential gains.
    The goal is to capture some upside potential of the underlying asset while reducing risk through a lower capital
    commitment. The key risks involve the loss of the premium collected if the underlying asset does not move favorably
    and potential limitations on profits if the underlying asset's price rises sharply, triggering the short call.
    This strategy is often used by investors who are moderately bullish on an asset but wish to reduce the cost
    and risk associated with traditional covered call strategies.
*/

use super::base::{Optimizable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::{debug, error};

const PMCC_DESCRIPTION: &str =
    "A Poor Man's Covered Call (PMCC) is an options strategy that simulates a covered call \
    using long-term equity anticipation securities (LEAPS) instead of the underlying stock. \
    It involves buying a long-term in-the-money call option and selling a short-term out-of-the-money call option. \
    This strategy aims to generate income while reducing the capital required compared to a traditional covered call.";

#[derive(Clone, Debug)]
pub struct PoorMansCoveredCall {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call: Position,
    short_call: Position,
}

impl PoorMansCoveredCall {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        long_call_strike: PositiveF64,
        short_call_strike: PositiveF64,
        long_call_expiration: ExpirationDate,
        short_call_expiration: ExpirationDate,
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
        let mut strategy = PoorMansCoveredCall {
            name: "Poor Man's Covered Call".to_string(),
            kind: StrategyType::PoorMansCoveredCall,
            description: PMCC_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        };

        // Long Call (LEAPS)
        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_call_strike,
            long_call_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let long_call = Position::new(
            long_call_option,
            premium_long_call,
            Utc::now(),
            open_fee_long_call,
            close_fee_long_call,
        );
        strategy.add_leg(long_call.clone());

        // Short Call
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_call_strike,
            short_call_expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call = Position::new(
            short_call_option,
            premium_short_call,
            Utc::now(),
            open_fee_short_call,
            close_fee_short_call,
        );
        strategy.add_leg(short_call.clone());

        // Calculate break-even point
        let net_debit =
            (strategy.long_call.max_loss() - strategy.short_call.max_profit()) / quantity;
        strategy
            .break_even_points
            .push(long_call_strike + net_debit);
        strategy
    }
}

impl Validable for PoorMansCoveredCall {
    fn validate(&self) -> bool {
        self.short_call.validate() && self.long_call.validate()
    }
}

impl Strategies for PoorMansCoveredCall {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.long_call.option.underlying_price
    }

    fn add_leg(&mut self, position: Position) {
        match (
            position.option.option_style.clone(),
            position.option.side.clone(),
        ) {
            (OptionStyle::Call, Side::Long) => self.long_call = position,
            (OptionStyle::Call, Side::Short) => self.short_call = position,
            _ => panic!("Invalid option type for Poor Man's Covered Call strategy"),
        }
    }

    fn get_legs(&self) -> Vec<Position> {
        vec![self.long_call.clone(), self.short_call.clone()]
    }

    fn break_even(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let profit = self.calculate_profit_at(self.short_call.option.strike_price);
        if profit <= ZERO {
            Ok(PZERO)
        } else {
            Ok(profit.into())
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let loss = self.calculate_profit_at(self.long_call.option.strike_price);
        if loss >= ZERO {
            Ok(PZERO)
        } else {
            Ok(loss.abs().into())
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(self.long_call.net_cost() + self.short_call.net_cost())
    }

    fn net_premium_received(&self) -> f64 {
        self.long_call.net_premium_received() + self.short_call.net_premium_received()
    }

    fn fees(&self) -> f64 {
        (self.long_call.open_fee + self.long_call.close_fee) * self.long_call.option.quantity
            + (self.short_call.open_fee + self.short_call.close_fee)
                * self.short_call.option.quantity
    }

    fn profit_area(&self) -> f64 {
        let base = (self.short_call.option.strike_price
            - (self.short_call.option.strike_price - self.max_profit().unwrap_or(PZERO)))
        .value();
        let high = self.max_profit().unwrap_or(PZERO).value();
        base * high / 200.0
    }

    fn profit_ratio(&self) -> f64 {
        match (self.max_profit(), self.max_loss()) {
            (Ok(profit), Ok(loss)) => (profit / loss).value() * 100.0,
            _ => ZERO,
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

impl Optimizable for PoorMansCoveredCall {
    type Strategy = PoorMansCoveredCall;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for long_call_index in 0..options.len() {
            let long_call_option = &options[long_call_index];
            for short_call_option in &options[(long_call_index + 1)..] {
                debug!(
                    "Long: {:#?} Short: {:#?}",
                    long_call_option.strike_price, short_call_option.strike_price
                );
                if long_call_option.strike_price >= short_call_option.strike_price {
                    debug!(
                        "Invalid strike prices long call option: {:#?} short call option: {:#?} ",
                        long_call_option.strike_price, short_call_option.strike_price
                    );
                    continue;
                }

                if !self.is_valid_short_option(short_call_option, &side)
                    || !self.is_valid_long_option(long_call_option, &side)
                {
                    debug!("Invalid option");
                    continue;
                }

                let strategy: PoorMansCoveredCall =
                    self.create_strategy(option_chain, long_call_option, short_call_option);

                if !strategy.validate() {
                    debug!("Invalid strategy");
                    continue;
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio(),
                    OptimizationCriteria::Area => strategy.profit_area(),
                };

                if current_value > best_value {
                    best_value = current_value;
                    *self = strategy.clone();
                }
            }
        }
    }

    fn is_valid_short_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let underlying_price = self.short_call.option.underlying_price;
        if underlying_price == PZERO {
            error!("Invalid underlying_price option");
            return false;
        }

        match side {
            FindOptimalSide::Upper => {
                let valid = option.strike_price >= underlying_price;
                if !valid {
                    debug!(
                        "Short Option is out of range: {} <= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::Lower => {
                let valid = option.strike_price <= underlying_price;
                if !valid {
                    debug!(
                        "Short Option is out of range: {} >= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                let valid = option.strike_price >= *start && option.strike_price <= *end;
                if !valid {
                    debug!(
                        " Short Option is out of range: {} >= {} && {} <= {}",
                        option.strike_price, *start, option.strike_price, *end
                    );
                }
                valid
            }
        }
    }

    fn is_valid_long_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let underlying_price = self.long_call.option.underlying_price;
        if underlying_price == PZERO {
            error!("Invalid underlying_price option");
            return false;
        }

        match side {
            FindOptimalSide::Upper => {
                let valid = option.strike_price >= underlying_price;
                if !valid {
                    debug!(
                        "Long Option is out of range: {} <= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::Lower => {
                let valid = option.strike_price <= underlying_price;
                if !valid {
                    debug!(
                        "Long Option is out of range: {} >= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                let valid = option.strike_price >= *start && option.strike_price <= *end;
                if !valid {
                    debug!(
                        "Long Option is out of range: {} >= {} && {} <= {}",
                        option.strike_price, *start, option.strike_price, *end
                    );
                }
                valid
            }
        }
    }

    fn create_strategy(
        &self,
        _chain: &OptionChain,
        long: &OptionData,
        short: &OptionData,
    ) -> Self::Strategy {
        let mut long_call_option = self.long_call.option.clone();
        long_call_option.update_from_option_data(long);

        let mut short_call_option = self.short_call.option.clone();
        short_call_option.update_from_option_data(short);

        let mut strategy = self.clone();
        strategy.long_call.update_from_option_data(long);
        strategy.short_call.update_from_option_data(short);

        // Calculate break-even point
        let net_debit = (strategy.long_call.max_loss() - strategy.short_call.max_profit())
            / strategy.long_call.option.quantity;

        if let Some(primer_elemento) = strategy.break_even_points.get_mut(0) {
            *primer_elemento = long_call_option.strike_price + net_debit;
        }

        strategy
    }
}

impl Profit for PoorMansCoveredCall {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price) + self.short_call.pnl_at_expiration(&price)
    }
}

impl Graph for PoorMansCoveredCall {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind, self.long_call.option.underlying_symbol, self.long_call.option.quantity
        );
        let leg_titles: Vec<String> = [
            format!("Long Call (LEAPS): ${}", self.long_call.option.strike_price),
            format!("Short Call: ${}", self.short_call.option.strike_price),
            format!(
                "Long Call Expiry: {}",
                self.long_call.option.expiration_date
            ),
            format!(
                "Short Call Expiry: {}",
                self.short_call.option.expiration_date
            ),
        ]
        .iter()
        .map(|leg| leg.to_string())
        .collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
        }
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.value(),
            y_range: (-50000.0, 50000.0),
            label: format!("Current Price: {}", self.short_call.option.underlying_price),
            label_offset: (5.0, 5.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_profit = self.max_profit().unwrap_or(PZERO);
        let max_loss = self.max_loss().unwrap_or(PZERO);

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Break Even\n\n{}", self.break_even_points[0]),
            label_offset: (-30.0, 15.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.short_call.option.strike_price.value() / 2000.0,
            max_profit.value() / 10.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.value(),
                max_profit.value(),
            ),
            label: format!(
                "Max Profit {:.2} at {:.0}",
                max_profit, self.short_call.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.long_call.option.strike_price.value() / 2000.0,
            -max_loss.value() / 50.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.value(),
                -max_loss.value(),
            ),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                max_loss, self.long_call.option.strike_price
            ),
            label_offset: coordiantes,
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::types::{ExpirationDate, PZERO};
    use crate::pos;

    fn create_pmcc_strategy() -> PoorMansCoveredCall {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let long_call_strike = pos!(140.0);
        let short_call_strike = pos!(160.0);
        let long_call_expiration = ExpirationDate::Days(365.0);
        let short_call_expiration = ExpirationDate::Days(30.0);
        let implied_volatility = 0.20;
        let risk_free_rate = 0.01;
        let dividend_yield = 0.005;
        let quantity = pos!(1.0);
        let premium_long_call = 15.0;
        let premium_short_call = 5.0;
        let open_fee_long_call = 1.0;
        let close_fee_long_call = 1.0;
        let open_fee_short_call = 0.5;
        let close_fee_short_call = 0.5;

        PoorMansCoveredCall::new(
            underlying_symbol,
            underlying_price,
            long_call_strike,
            short_call_strike,
            long_call_expiration,
            short_call_expiration,
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            quantity,
            premium_long_call,
            premium_short_call,
            open_fee_long_call,
            close_fee_long_call,
            open_fee_short_call,
            close_fee_short_call,
        )
    }

    #[test]
    fn test_create_pmcc_strategy() {
        let pmcc = create_pmcc_strategy();
        assert_eq!(pmcc.name, "Poor Man's Covered Call");
        assert_eq!(pmcc.long_call.option.strike_price, pos!(140.0));
        assert_eq!(pmcc.short_call.option.strike_price, pos!(160.0));
    }

    #[test]
    fn test_max_profit() {
        let pmcc = create_pmcc_strategy();
        let max_profit = pmcc.max_profit().unwrap_or(PZERO);
        assert!(max_profit > PZERO);
    }

    #[test]
    fn test_max_loss() {
        let pmcc = create_pmcc_strategy();
        let max_loss = pmcc.max_loss().unwrap_or(PZERO);
        assert!(max_loss > PZERO);
    }

    #[test]
    fn test_break_even() {
        let pmcc = create_pmcc_strategy();
        let break_even = pmcc.break_even();
        assert_eq!(break_even.len(), 1);
        assert!(break_even[0].value() > 0.0);
    }

    #[test]
    fn test_total_cost() {
        let pmcc = create_pmcc_strategy();
        let total_cost = pmcc.total_cost();
        assert!(total_cost > PZERO);
    }

    #[test]
    fn test_fees() {
        let pmcc = create_pmcc_strategy();
        let fees = pmcc.fees();
        assert!(fees > 0.0);
    }

    #[test]
    fn test_profit_area() {
        let pmcc = create_pmcc_strategy();
        let profit_area = pmcc.profit_area();
        assert!(profit_area > 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let pmcc = create_pmcc_strategy();
        let profit_ratio = pmcc.profit_ratio();
        assert!(profit_ratio > 0.0);
    }

    #[test]
    fn test_best_range_to_show() {
        let pmcc = create_pmcc_strategy();
        let step = pos!(1.0);
        let range = pmcc.best_range_to_show(step);
        assert!(range.is_some());
        let range_values = range.unwrap();
        assert!(!range_values.is_empty());
    }

    #[test]
    fn test_calculate_profit_at() {
        let pmcc = create_pmcc_strategy();
        let profit = pmcc.calculate_profit_at(pos!(150.0));
        assert!(
            profit >= -pmcc.max_loss().unwrap_or(PZERO).value()
                && profit <= pmcc.max_profit().unwrap_or(PZERO).value()
        );
    }

    #[test]
    fn test_graph_title() {
        let pmcc = create_pmcc_strategy();
        let title = pmcc.title();
        assert!(title.contains("PoorMansCoveredCall Strategy"));
    }

    #[test]
    fn test_vertical_lines() {
        let pmcc = create_pmcc_strategy();
        let vertical_lines = pmcc.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].x_coordinate, 150.0);
    }

    #[test]
    fn test_graph_points() {
        let pmcc = create_pmcc_strategy();
        let points = pmcc.get_points();
        assert!(!points.is_empty());
    }
}

#[cfg(test)]
mod tests_pmcc_validation {
    use super::*;

    fn create_basic_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(365.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.01,
            0.005,
            pos!(1.0),
            15.0,
            5.0,
            1.0,
            1.0,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_validate_valid_strategy() {
        let strategy = create_basic_strategy();
        assert!(strategy.validate());
    }

    #[test]
    fn test_add_leg_long_call() {
        let mut strategy = create_basic_strategy();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(140.0),
            ExpirationDate::Days(365.0),
            0.2,
            pos!(1.0),
            pos!(150.0),
            0.01,
            OptionStyle::Call,
            0.005,
            None,
        );
        let position = Position::new(option, 15.0, Utc::now(), 1.0, 1.0);
        strategy.add_leg(position.clone());
        assert_eq!(strategy.long_call, position);
    }

    #[test]
    fn test_add_leg_short_call() {
        let mut strategy = create_basic_strategy();
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            pos!(160.0),
            ExpirationDate::Days(30.0),
            0.2,
            pos!(1.0),
            pos!(150.0),
            0.01,
            OptionStyle::Call,
            0.005,
            None,
        );
        let position = Position::new(option, 5.0, Utc::now(), 0.5, 0.5);
        strategy.add_leg(position.clone());
        assert_eq!(strategy.short_call, position);
    }

    #[test]
    #[should_panic(expected = "Invalid option type for Poor Man's Covered Call strategy")]
    fn test_add_leg_invalid_option() {
        let mut strategy = create_basic_strategy();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(140.0),
            ExpirationDate::Days(365.0),
            0.2,
            pos!(1.0),
            pos!(150.0),
            0.01,
            OptionStyle::Put,
            0.005,
            None,
        );
        let position = Position::new(option, 15.0, Utc::now(), 1.0, 1.0);
        strategy.add_leg(position);
    }
}

#[cfg(test)]
mod tests_pmcc_optimization {
    use super::*;
    use crate::spos;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("AAPL", pos!(150.0), "2024-01-01".to_string());

        // Add options at various strikes
        for strike in [140.0, 145.0, 150.0, 155.0, 160.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(5.0),
                spos!(5.2),
                spos!(4.8),
                spos!(5.0),
                spos!(0.2),
                Some(0.5),
                spos!(100.0),
                Some(50),
            );
        }
        chain
    }

    fn create_base_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(365.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.01,
            0.005,
            pos!(1.0),
            15.0,
            5.0,
            1.0,
            1.0,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_is_valid_short_option() {
        let strategy = create_base_strategy();
        let option = OptionData::new(
            pos!(160.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(strategy.is_valid_short_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_is_valid_long_option() {
        let strategy = create_base_strategy();
        let option = OptionData::new(
            pos!(140.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(strategy.is_valid_long_option(&option, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut strategy = create_base_strategy();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(strategy.validate());
    }

    #[test]
    fn test_find_optimal_area() {
        let mut strategy = create_base_strategy();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);
        assert!(strategy.validate());
    }

    #[test]
    fn test_invalid_short_option_zero_underlying() {
        let mut strategy = create_base_strategy();
        strategy.short_call.option.underlying_price = PZERO;
        let option = OptionData::new(
            pos!(160.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(!strategy.is_valid_short_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_invalid_long_option_zero_underlying() {
        let mut strategy = create_base_strategy();
        strategy.long_call.option.underlying_price = PZERO;
        let option = OptionData::new(
            pos!(140.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            spos!(0.2),
            None,
            None,
            None,
        );
        assert!(!strategy.is_valid_long_option(&option, &FindOptimalSide::Lower));
    }
}

#[cfg(test)]
mod tests_pmcc_pnl {
    use super::*;

    fn create_test_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(365.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.01,
            0.005,
            pos!(1.0),
            15.0,
            5.0,
            1.0,
            1.0,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_calculate_profit_at_various_prices() {
        let strategy = create_test_strategy();

        // Below long strike
        let profit_below = strategy.calculate_profit_at(pos!(130.0));
        assert!(profit_below < 0.0);

        // Between strikes
        let profit_middle = strategy.calculate_profit_at(pos!(150.0));
        assert!(profit_middle > profit_below);

        // At short strike
        let profit_short = strategy.calculate_profit_at(strategy.short_call.option.strike_price);
        assert_eq!(profit_short, strategy.max_profit().unwrap_or(PZERO).value());

        // Above short strike
        let profit_above = strategy.calculate_profit_at(pos!(170.0));
        assert_eq!(profit_above, profit_above);
    }

    #[test]
    fn test_break_even_point() {
        let strategy = create_test_strategy();
        assert_eq!(strategy.break_even_points.len(), 1);
        let break_even = strategy.break_even_points[0];
        let profit_at_be = strategy.calculate_profit_at(break_even);
        assert!(profit_at_be.abs() < 0.01);
    }

    #[test]
    fn test_net_premium() {
        let strategy = create_test_strategy();
        let net_premium = strategy.net_premium_received();
        assert_eq!(net_premium, strategy.short_call.net_premium_received());
    }

    #[test]
    fn test_max_profit_max_loss_relationship() {
        let strategy = create_test_strategy();
        assert!(strategy.max_profit().unwrap_or(PZERO) > PZERO);
        assert!(strategy.max_loss().unwrap_or(PZERO) > PZERO);
        assert!(strategy.max_loss().unwrap_or(PZERO) > strategy.max_profit().unwrap_or(PZERO));
    }
}

#[cfg(test)]
mod tests_pmcc_graph {
    use super::*;

    fn create_test_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(365.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.01,
            0.005,
            pos!(1.0),
            15.0,
            5.0,
            1.0,
            1.0,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_title_format() {
        let strategy = create_test_strategy();
        let title = strategy.title();
        assert!(title.contains("PoorMansCoveredCall Strategy"));
        assert!(title.contains("AAPL"));
        assert!(title.contains("Long Call (LEAPS)"));
        assert!(title.contains("Short Call"));
    }

    #[test]
    fn test_vertical_lines_format() {
        let strategy = create_test_strategy();
        let lines = strategy.get_vertical_lines();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].x_coordinate, 150.0);
        assert!(lines[0].label.contains("Current Price"));
    }

    #[test]
    fn test_points_format() {
        let strategy = create_test_strategy();
        let points = strategy.get_points();

        assert!(points.iter().any(|p| p.label.contains("Break Even")));
        assert!(points.iter().any(|p| p.label.contains("Max Profit")));
        assert!(points.iter().any(|p| p.label.contains("Max Loss")));

        let break_even_point = points
            .iter()
            .find(|p| p.label.contains("Break Even"))
            .unwrap();
        assert_eq!(break_even_point.coordinates.1, 0.0);
    }

    #[test]
    fn test_point_colors() {
        let strategy = create_test_strategy();
        let points = strategy.get_points();

        let max_profit_point = points
            .iter()
            .find(|p| p.label.contains("Max Profit"))
            .unwrap();
        assert_eq!(max_profit_point.point_color, DARK_GREEN);

        let max_loss_point = points
            .iter()
            .find(|p| p.label.contains("Max Loss"))
            .unwrap();
        assert_eq!(max_loss_point.point_color, RED);
    }
}

#[cfg(test)]
mod tests_pmcc_best_area {
    use super::*;
    use crate::utils::logger::setup_logger;

    fn set_up() -> Result<(PoorMansCoveredCall, OptionChain), String> {
        setup_logger();
        let option_chain =
            OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
                .unwrap();
        let underlying_price = option_chain.underlying_price;

        let strategy = PoorMansCoveredCall::new(
            "SP500".to_string(),
            underlying_price,
            pos!(5700.0), // long strike ITM
            pos!(5900.0), // short strike OTM
            ExpirationDate::Days(365.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.01,
            0.005,
            pos!(1.0),
            15.0,
            5.0,
            1.0,
            1.0,
            0.5,
            0.5,
        );

        Ok((strategy, option_chain))
    }

    #[test]
    fn test_best_area_all() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_area(&option_chain, FindOptimalSide::All);

        // Verifica las propiedades clave de la estrategia optimizada
        assert!(strategy.profit_area() > 0.0);
        assert!(strategy.profit_ratio() > 0.0);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert!(strategy.total_cost() > PZERO);
        assert!(strategy.fees() > 0.0);

        // Verifica que las posiciones tienen strikes válidos
        assert!(strategy.long_call.option.strike_price < strategy.short_call.option.strike_price);
    }

    #[test]
    fn test_best_area_upper() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_area(&option_chain, FindOptimalSide::Upper);

        // Verifica que los strikes están por encima del precio subyacente
        assert!(strategy.long_call.option.strike_price >= strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        // Verifica otras propiedades
        assert!(strategy.profit_area() > 0.0);
        assert!(strategy.max_profit().unwrap_or(PZERO) > PZERO);
    }

    #[test]
    fn test_best_area_lower() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_area(&option_chain, FindOptimalSide::Lower);

        // Verifica que los strikes están por debajo del precio subyacente
        assert!(strategy.long_call.option.strike_price <= strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.profit_area() > 0.0);
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_pmcc_best_ratio {
    use super::*;
    use crate::utils::logger::setup_logger;

    fn set_up() -> Result<(PoorMansCoveredCall, OptionChain), String> {
        setup_logger();
        let option_chain =
            OptionChain::load_from_json("./examples/Chains/SP500-18-oct-2024-5781.88.json")
                .unwrap();
        let underlying_price = option_chain.underlying_price;

        let strategy = PoorMansCoveredCall::new(
            "SP500".to_string(),
            underlying_price,
            pos!(5700.0),
            pos!(5900.0),
            ExpirationDate::Days(365.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.01,
            0.005,
            pos!(1.0),
            15.0,
            5.0,
            1.0,
            1.0,
            0.5,
            0.5,
        );

        Ok((strategy, option_chain))
    }

    #[test]
    fn test_best_ratio_all() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_ratio(&option_chain, FindOptimalSide::All);

        assert!(strategy.profit_ratio() > 0.0);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert!(strategy.max_profit().unwrap_or(PZERO) > PZERO);
        assert!(strategy.max_loss().unwrap_or(PZERO) > PZERO);
        assert!(strategy.fees() > 0.0);
    }

    #[test]
    fn test_best_ratio_upper() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_ratio(&option_chain, FindOptimalSide::Upper);

        // Verifica que los strikes están por encima del precio subyacente
        assert!(strategy.long_call.option.strike_price >= strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.profit_ratio() > 0.0);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_ratio_with_range() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_ratio(
            &option_chain,
            FindOptimalSide::Range(pos!(5750.0), pos!(5850.0)),
        );

        // Verifica que los strikes están dentro del rango especificado
        assert!(strategy.long_call.option.strike_price >= pos!(5750.0));
        assert!(strategy.short_call.option.strike_price <= pos!(5850.0));

        assert!(strategy.profit_ratio() > 0.0);
        assert!(strategy.validate());
    }
}
