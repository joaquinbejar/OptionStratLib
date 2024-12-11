/*
Long Butterfly Spread Strategy

A long butterfly spread involves:
1. Buy one call at a lower strike price (ITM)
2. Sell two calls at a middle strike price (ATM)
3. Buy one call at a higher strike price (OTM)

All options have the same expiration date.

Key characteristics:
- Limited profit potential
- Limited risk
- Profit is highest when price is exactly at middle strike at expiration
- Maximum loss is limited to the net premium paid
- All options must have same expiration date
*/

use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, PositiveF64, Side, PZERO};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pos;
use crate::pricing::payoff::Profit;
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::utils::approx_equal;
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::{debug, error, info};

const LONG_BUTTERFLY_DESCRIPTION: &str =
    "A long butterfly spread is created by buying one call at a lower strike price, \
    selling two calls at a middle strike price, and buying one call at a higher strike price, \
    all with the same expiration date. This strategy profits when the underlying price stays \
    near the middle strike price at expiration.";

#[derive(Clone, Debug)]
pub struct LongButterflySpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call_low: Position,
    short_calls: Position,
    long_call_high: Position,
}

impl LongButterflySpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        low_strike: PositiveF64,
        middle_strike: PositiveF64,
        high_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_low: f64,
        premium_middle: f64,
        premium_high: f64,
        fees: f64,
    ) -> Self {
        let mut strategy = LongButterflySpread {
            name: "Long Butterfly".to_string(),
            kind: StrategyType::LongButterflySpread,
            description: LONG_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call_low: Position::default(),
            short_calls: Position::default(),
            long_call_high: Position::default(),
        };
        
        // Create two short calls at middle strike
        let short_calls = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            middle_strike,
            expiration.clone(),
            implied_volatility,
            quantity * 2.0, // Double quantity for middle strike
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_calls = Position::new(
            short_calls,
            premium_middle,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        // Create long call at lower strike
        let long_call_low = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            low_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_call_low = Position::new(
            long_call_low,
            premium_low,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        // Create long call at higher strike
        let long_call_high = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_call_high = Position::new(
            long_call_high,
            premium_high,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        strategy.validate();

        let left_profit = strategy.calculate_profit_at(low_strike) / quantity.value();
        let first_break_even = low_strike - left_profit;
        let value_at_first = strategy.calculate_profit_at(first_break_even);
        if approx_equal(value_at_first, ZERO) {
            strategy.break_even_points.push(first_break_even);
        }

        let right_profit = strategy.calculate_profit_at(high_strike) / quantity.value();
        let second_break_even = high_strike + right_profit;
        let value_at_second = strategy.calculate_profit_at(second_break_even);
        if approx_equal(value_at_second, ZERO) {
            strategy.break_even_points.push(second_break_even);
        }

        strategy
    }
}

impl Validable for LongButterflySpread {
    fn validate(&self) -> bool {
        if !self.long_call_low.validate() {
            debug!("Long call (low strike) is invalid");
            return false;
        }
        if !self.short_calls.validate() {
            debug!("Short calls (middle strike) are invalid");
            return false;
        }
        if !self.long_call_high.validate() {
            debug!("Long call (high strike) is invalid");
            return false;
        }

        if self.long_call_low.option.strike_price >= self.short_calls.option.strike_price {
            debug!("Low strike must be lower than middle strike");
            return false;
        }
        if self.short_calls.option.strike_price >= self.long_call_high.option.strike_price {
            debug!("Middle strike must be lower than high strike");
            return false;
        }

        if self.short_calls.option.quantity != self.long_call_low.option.quantity * 2.0 {
            debug!("Middle strike quantity must be double the wing quantities");
            return false;
        }
        if self.long_call_low.option.quantity != self.long_call_high.option.quantity {
            debug!("Wing quantities must be equal");
            return false;
        }

        true
    }
}

impl Positionable for LongButterflySpread {
    fn add_position(&mut self, position: &Position) -> Result<(), String> {
        match &position.option.side {
            Side::Long  => {
                // short_calls should be inserted first
                if position.option.strike_price < self.short_calls.option.strike_price {
                    self.long_call_low = position.clone();
                    Ok(())
                } else {
                    self.long_call_high = position.clone();
                    Ok(())
                }
            }
            Side::Short => {
                self.short_calls = position.clone();
                Ok(())
            },
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, String> {
        Ok(vec![&self.long_call_low, &self.short_calls, &self.long_call_high])
    }
}

impl Strategies for LongButterflySpread {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.long_call_low.option.underlying_price
    }
    
    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let profit = self.calculate_profit_at(self.short_calls.option.strike_price);
        if profit > ZERO {
            Ok(pos!(profit))
        } else {
            Err("Profit is negative")
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let left_loss = self.calculate_profit_at(self.long_call_low.option.strike_price);
        let right_loss = self.calculate_profit_at(self.long_call_high.option.strike_price);
        let max_loss = left_loss.min(right_loss);
        if max_loss > ZERO {
            error!("Loss is positive {}", max_loss);
            Err("Loss is positive")
        } else {
            Ok(pos!(max_loss.abs()))
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(
            self.long_call_low.net_cost()
                + self.short_calls.net_cost()
                + self.long_call_high.net_cost()
        )
    }

    fn net_premium_received(&self) -> f64 {
        self.short_calls.net_premium_received()
            - self.long_call_low.net_cost()
            - self.long_call_high.net_cost()
    }

    fn fees(&self) -> f64 {
        (self.long_call_low.open_fee
            + self.long_call_low.close_fee
            + self.short_calls.open_fee
            + self.short_calls.close_fee
            + self.long_call_high.open_fee
            + self.long_call_high.close_fee)
            * self.long_call_low.option.quantity.value()
    }

    fn profit_area(&self) -> f64 {
        let high = self.max_profit().unwrap_or(PZERO);
        let break_even_points = self.get_break_even_points();

        let base = if break_even_points.len() == 2 {
            self.get_break_even_points()[1] - self.get_break_even_points()[0]
        } else {
            let break_even_point = self.get_break_even_points()[0];

            if break_even_point < self.short_calls.option.strike_price {
                pos!(self.calculate_profit_at(self.long_call_high.option.strike_price))
            } else {
                pos!(self.calculate_profit_at(self.long_call_low.option.strike_price))
            }
        };
        (high * base / 200.0).value()
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit().unwrap_or(PZERO);
        let max_loss = self.max_loss().unwrap_or(PZERO);
        match (max_profit, max_loss) {
            (PZERO, _) => ZERO,
            (_, PZERO) => f64::INFINITY,
            _ => (max_profit / max_loss * 100.0).value(),
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

impl Optimizable for LongButterflySpread {
    type Strategy = LongButterflySpread;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = f64::NEG_INFINITY;
        let options: Vec<&OptionData> = option_chain.options.iter().collect();

        for (i, long_call_low) in options.iter().enumerate() {
            if !self.is_valid_long_option(long_call_low, &side) {
                continue;
            }

            for (j, short_calls) in options.iter().enumerate().skip(i + 1) {
                if !self.is_valid_short_option(short_calls, &side) {
                    continue;
                }

                for long_call_high in options.iter().skip(j + 1) {
                    if !self.is_valid_long_option(long_call_high, &side) {
                        continue;
                    }

                    if long_call_low.strike_price >= short_calls.strike_price
                        || short_calls.strike_price >= long_call_high.strike_price
                    {
                        error!("Invalid order of strikes");
                        continue;
                    }

                    if !self.are_valid_prices(&StrategyLegs::ThreeLegs {
                        first: long_call_low,
                        second: short_calls,
                        third: long_call_high,
                    }) {
                        error!("Invalid prices");
                        continue;
                    }

                    let new_strategy = self.create_strategy(
                        option_chain,
                        &StrategyLegs::ThreeLegs {
                            first: long_call_low,
                            second: short_calls,
                            third: long_call_high,
                        },
                    );

                    if !new_strategy.validate() {
                        debug!("Invalid strategy");
                        continue;
                    }

                    if new_strategy.max_profit().is_err() || new_strategy.max_loss().is_err() {
                        continue;
                    }

                    let current_value = match criteria {
                        OptimizationCriteria::Ratio => new_strategy.profit_ratio(),
                        OptimizationCriteria::Area => new_strategy.profit_area(),
                    };

                    if current_value > best_value {
                        info!("New best value: {}", current_value);
                        best_value = current_value;
                        *self = new_strategy;
                    }
                }
            }
        }
    }

    fn is_valid_short_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let is_valid_strike = match side {
            FindOptimalSide::Upper => option.strike_price >= self.get_underlying_price(),
            FindOptimalSide::Lower => option.strike_price <= self.get_underlying_price(),
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                option.strike_price >= *start && option.strike_price <= *end
            }
        };
        is_valid_strike && option.call_bid.unwrap_or(PZERO) > PZERO
    }

    fn is_valid_long_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let is_valid_strike = match side {
            FindOptimalSide::Upper => option.strike_price >= self.get_underlying_price(),
            FindOptimalSide::Lower => option.strike_price <= self.get_underlying_price(),
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                option.strike_price >= *start && option.strike_price <= *end
            }
        };
        is_valid_strike && option.call_ask.unwrap_or(PZERO) > PZERO
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => {
                low_strike.call_ask.unwrap_or(PZERO) > PZERO
                    && middle_strike.call_bid.unwrap_or(PZERO) > PZERO
                    && high_strike.call_ask.unwrap_or(PZERO) > PZERO
            }
            _ => false,
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => LongButterflySpread::new(
                chain.symbol.clone(),
                chain.underlying_price,
                low_strike.strike_price,
                middle_strike.strike_price,
                high_strike.strike_price,
                self.long_call_low.option.expiration_date.clone(),
                middle_strike.implied_volatility.unwrap().value() / 100.0,
                self.long_call_low.option.risk_free_rate,
                self.long_call_low.option.dividend_yield,
                self.long_call_low.option.quantity,
                low_strike.call_ask.unwrap().value(),
                middle_strike.call_bid.unwrap().value(),
                high_strike.call_ask.unwrap().value(),
                self.fees() / 8.0,
            ),
            _ => panic!("Invalid number of legs for Long Butterfly strategy"),
        }
    }
}

impl Profit for LongButterflySpread {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_call_low.pnl_at_expiration(&price)
            + self.short_calls.pnl_at_expiration(&price)
            + self.long_call_high.pnl_at_expiration(&price)
    }
}

impl Graph for LongButterflySpread {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind,
            self.long_call_low.option.underlying_symbol,
            self.long_call_low.option.quantity
        );

        let leg_titles = [format!(
                "Long Call Low Strike: ${}",
                self.long_call_low.option.strike_price
            ),
            format!(
                "Short Calls Middle Strike: ${}",
                self.short_calls.option.strike_price
            ),
            format!(
                "Long Call High Strike: ${}",
                self.long_call_high.option.strike_price
            ),
            format!("Expire: {}", self.long_call_low.option.expiration_date)];

        format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        vec![ChartVerticalLine {
            x_coordinate: self.long_call_low.option.underlying_price.value(),
            y_range: (-50000.0, 50000.0),
            label: format!(
                "Current Price: {}",
                self.long_call_low.option.underlying_price
            ),
            label_offset: (5.0, 5.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();
        let max_profit = self.max_profit().unwrap_or(PZERO);

        let left_loss = self.calculate_profit_at(self.long_call_low.option.strike_price);
        let right_loss = self.calculate_profit_at(self.long_call_high.option.strike_price);

        // Break-even points
        points.extend(
            self.break_even_points
                .iter()
                .enumerate()
                .map(|(i, &price)| ChartPoint {
                    coordinates: (price.value(), 0.0),
                    label: format!(
                        "Break Even {}: {:.2}",
                        if i == 0 { "Lower" } else { "Upper" },
                        price
                    ),
                    label_offset: LabelOffsetType::Relative(3.0, 3.0),
                    point_color: DARK_BLUE,
                    label_color: DARK_BLUE,
                    point_size: 5,
                    font_size: 18,
                }),
        );

        // Maximum profit point (at middle strike)
        points.push(ChartPoint {
            coordinates: (
                self.short_calls.option.strike_price.value(),
                max_profit.value(),
            ),
            label: format!("Max Profit {:.2}", max_profit),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let left_color = if left_loss > ZERO { DARK_GREEN } else { RED };

        // Maximum loss points (at wing strikes)
        points.push(ChartPoint {
            coordinates: (self.long_call_low.option.strike_price.value(), left_loss),
            label: format!("Left Loss {:.2}", left_loss),
            label_offset: LabelOffsetType::Relative(-30.0, -3.0),
            point_color: left_color,
            label_color: left_color,
            point_size: 5,
            font_size: 18,
        });

        let right_color = if right_loss > ZERO { DARK_GREEN } else { RED };

        points.push(ChartPoint {
            coordinates: (self.long_call_high.option.strike_price.value(), right_loss),
            label: format!("Max Loss {:.2}", right_loss),
            label_offset: LabelOffsetType::Relative(3.0, -3.0),
            point_color: right_color,
            label_color: right_color,
            point_size: 5,
            font_size: 18,
        });

        // Current price point
        points.push(self.get_point_at_price(self.long_call_low.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for LongButterflySpread {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        Ok(self.long_call_low.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.long_call_low.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_points = self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.long_call_low.option.implied_volatility),
            pos!(self.short_calls.option.implied_volatility),
            pos!(self.long_call_high.option.implied_volatility),
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            pos!(self.max_profit()?.value()),
        )?;

        profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let mut ranges = Vec::new();
        let break_even_points = self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.long_call_low.option.implied_volatility),
            pos!(self.short_calls.option.implied_volatility),
            pos!(self.long_call_high.option.implied_volatility),
        ]);

        let volatility_adjustment = Some(VolatilityAdjustment {
            base_volatility: mean_volatility,
            std_dev_adjustment: std_dev,
        });

        let mut lower_loss_range = ProfitLossRange::new(
            Some(self.long_call_low.option.strike_price),
            Some(break_even_points[0]),
            pos!(self.max_loss()?.value()),
        )?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment.clone(),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(lower_loss_range);

        let mut upper_loss_range = ProfitLossRange::new(
            Some(break_even_points[1]),
            Some(self.long_call_high.option.strike_price),
            pos!(self.max_loss()?.value()),
        )?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment,
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(upper_loss_range);

        Ok(ranges)
    }
}

/*
Short Butterfly Spread Strategy

A short butterfly spread involves:
1. Sell one call at a lower strike price (ITM)
2. Buy two calls at a middle strike price (ATM)
3. Sell one call at a higher strike price (OTM)

All options have the same expiration date.

Key characteristics:
- Limited profit potential
- Limited risk
- Loss is highest when price is exactly at middle strike at expiration
- Maximum profit is limited to the net premium received
- All options must have same expiration date
*/

const SHORT_BUTTERFLY_DESCRIPTION: &str =
    "A short butterfly spread is created by selling one call at a lower strike price, \
    buying two calls at a middle strike price, and selling one call at a higher strike price, \
    all with the same expiration date. This strategy profits when the underlying price moves \
    significantly away from the middle strike price in either direction.";

#[derive(Clone, Debug)]
pub struct ShortButterflySpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    short_call_low: Position,
    long_calls: Position,
    short_call_high: Position,
}

impl ShortButterflySpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        low_strike: PositiveF64,
        middle_strike: PositiveF64,
        high_strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_low: f64,
        premium_middle: f64,
        premium_high: f64,
        fees: f64,
    ) -> Self {
        let mut strategy = ShortButterflySpread {
            name: "Short Butterfly".to_string(),
            kind: StrategyType::ShortButterflySpread,
            description: SHORT_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call_low: Position::default(),
            long_calls: Position::default(),
            short_call_high: Position::default(),
        };

        // Create two long calls at middle strike
        let long_calls = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            middle_strike,
            expiration.clone(),
            implied_volatility,
            quantity * 2.0, // Double quantity for middle strike
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.long_calls = Position::new(
            long_calls,
            premium_middle,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        // Create short call at lower strike
        let short_call_low = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            low_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_call_low = Position::new(
            short_call_low,
            premium_low,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );



        // Create short call at higher strike
        let short_call_high = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            high_strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        strategy.short_call_high = Position::new(
            short_call_high,
            premium_high,
            Utc::now(),
            fees / 3.0,
            fees / 3.0,
        );

        strategy.validate();

        let left_profit = strategy.calculate_profit_at(low_strike) / quantity.value();
        let first_break_even = low_strike + left_profit;
        let value_at_first = strategy.calculate_profit_at(first_break_even);
        if approx_equal(value_at_first, ZERO) {
            strategy.break_even_points.push(first_break_even);
        }

        let right_profit = strategy.calculate_profit_at(high_strike) / quantity.value();
        let second_break_even = high_strike - right_profit;
        let value_at_second = strategy.calculate_profit_at(second_break_even);
        if approx_equal(value_at_second, ZERO) {
            strategy.break_even_points.push(second_break_even);
        }

        strategy
    }
}

impl Validable for ShortButterflySpread {
    fn validate(&self) -> bool {
        if !self.short_call_low.validate() {
            debug!("Short call (low strike) is invalid");
            return false;
        }
        if !self.long_calls.validate() {
            debug!("Long calls (middle strike) are invalid");
            return false;
        }
        if !self.short_call_high.validate() {
            debug!("Short call (high strike) is invalid");
            return false;
        }

        if self.short_call_low.option.strike_price >= self.long_calls.option.strike_price {
            debug!("Low strike must be lower than middle strike");
            return false;
        }
        if self.long_calls.option.strike_price >= self.short_call_high.option.strike_price {
            debug!("Middle strike must be lower than high strike");
            return false;
        }

        if self.long_calls.option.quantity != self.short_call_low.option.quantity * 2.0 {
            debug!("Middle strike quantity must be double the wing quantities");
            return false;
        }
        if self.short_call_low.option.quantity != self.short_call_high.option.quantity {
            debug!("Wing quantities must be equal");
            return false;
        }

        true
    }
}

impl Positionable for ShortButterflySpread {
    fn add_position(&mut self, position: &Position) -> Result<(), String> {
        match &position.option.side {
            Side::Short  => {
                // long_calls should be inserted first
                if position.option.strike_price < self.long_calls.option.strike_price {
                    self.short_call_low = position.clone();
                    Ok(())
                } else {
                    self.short_call_high = position.clone();
                    Ok(())
                }
            }
            Side::Long => {
                self.long_calls = position.clone();
                Ok(())
            },
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, String> {
        Ok(vec![&self.short_call_low, &self.long_calls, &self.short_call_high])
    }
}

impl Strategies for ShortButterflySpread {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.short_call_low.option.underlying_price
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let left_profit = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_call_high.option.strike_price);
        let max_profit = left_profit.max(right_profit);
        if max_profit > ZERO {
            Ok(pos!(max_profit))
        } else {
            Err("Profit is negative")
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        let loss = self.calculate_profit_at(self.long_calls.option.strike_price);
        if loss > ZERO {
            error!("Loss is positive {}", loss);
            Err("Loss is positive")
        } else {
            Ok(pos!(loss.abs()))
        }
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(
            self.short_call_low.net_cost()
                + self.long_calls.net_cost()
                + self.short_call_high.net_cost()
        )
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call_low.net_premium_received() + self.short_call_high.net_premium_received()
            - self.long_calls.net_cost()
    }

    fn fees(&self) -> f64 {
        (self.short_call_low.open_fee
            + self.short_call_low.close_fee
            + self.long_calls.open_fee
            + self.long_calls.close_fee
            + self.short_call_high.open_fee
            + self.short_call_high.close_fee)
            * self.short_call_low.option.quantity.value()
    }

    fn profit_area(&self) -> f64 {
        let break_even_points = self.get_break_even_points();
        let left_profit = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_call_high.option.strike_price);

        if break_even_points.len() == 2 {
            left_profit + right_profit
        } else {
            left_profit.max(right_profit)
        }
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit().unwrap_or(PZERO);
        let max_loss = self.max_loss().unwrap_or(PZERO);
        match (max_profit, max_loss) {
            (PZERO, _) => ZERO,
            (_, PZERO) => f64::INFINITY,
            _ => (max_profit / max_loss * 100.0).value(),
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        self.break_even_points.clone()
    }
}

impl Optimizable for ShortButterflySpread {
    type Strategy = ShortButterflySpread;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = f64::NEG_INFINITY;
        let options: Vec<&OptionData> = option_chain.options.iter().collect();

        for (i, short_call_low) in options.iter().enumerate() {
            if !self.is_valid_short_option(short_call_low, &side) {
                continue;
            }

            for (j, long_calls) in options.iter().enumerate().skip(i + 1) {
                if !self.is_valid_long_option(long_calls, &side) {
                    continue;
                }

                for short_call_high in options.iter().skip(j + 1) {
                    if !self.is_valid_short_option(short_call_high, &side) {
                        continue;
                    }

                    if short_call_low.strike_price >= long_calls.strike_price
                        || long_calls.strike_price >= short_call_high.strike_price
                    {
                        error!("Invalid order of strikes");
                        continue;
                    }

                    if !self.are_valid_prices(&StrategyLegs::ThreeLegs {
                        first: short_call_low,
                        second: long_calls,
                        third: short_call_high,
                    }) {
                        error!("Invalid prices");
                        continue;
                    }

                    let new_strategy = self.create_strategy(
                        option_chain,
                        &StrategyLegs::ThreeLegs {
                            first: short_call_low,
                            second: long_calls,
                            third: short_call_high,
                        },
                    );

                    if !new_strategy.validate() {
                        debug!("Invalid strategy");
                        continue;
                    }

                    if new_strategy.max_profit().is_err() || new_strategy.max_loss().is_err() {
                        continue;
                    }

                    let current_value = match criteria {
                        OptimizationCriteria::Ratio => new_strategy.profit_ratio(),
                        OptimizationCriteria::Area => new_strategy.profit_area(),
                    };

                    if current_value > best_value {
                        info!("New best value: {}", current_value);
                        best_value = current_value;
                        *self = new_strategy;
                    }
                }
            }
        }
    }

    fn is_valid_short_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let is_valid_strike = match side {
            FindOptimalSide::Upper => option.strike_price >= self.get_underlying_price(),
            FindOptimalSide::Lower => option.strike_price <= self.get_underlying_price(),
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                option.strike_price >= *start && option.strike_price <= *end
            }
        };
        is_valid_strike && option.call_bid.unwrap_or(PZERO) > PZERO
    }

    fn is_valid_long_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let is_valid_strike = match side {
            FindOptimalSide::Upper => option.strike_price >= self.get_underlying_price(),
            FindOptimalSide::Lower => option.strike_price <= self.get_underlying_price(),
            FindOptimalSide::All => true,
            FindOptimalSide::Range(start, end) => {
                option.strike_price >= *start && option.strike_price <= *end
            }
        };
        is_valid_strike && option.call_ask.unwrap_or(PZERO) > PZERO
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => {
                low_strike.call_bid.unwrap_or(PZERO) > PZERO
                    && middle_strike.call_ask.unwrap_or(PZERO) > PZERO
                    && high_strike.call_bid.unwrap_or(PZERO) > PZERO
            }
            _ => false,
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        match legs {
            StrategyLegs::ThreeLegs {
                first: low_strike,
                second: middle_strike,
                third: high_strike,
            } => ShortButterflySpread::new(
                chain.symbol.clone(),
                chain.underlying_price,
                low_strike.strike_price,
                middle_strike.strike_price,
                high_strike.strike_price,
                self.short_call_low.option.expiration_date.clone(),
                middle_strike.implied_volatility.unwrap().value() / 100.0,
                self.short_call_low.option.risk_free_rate,
                self.short_call_low.option.dividend_yield,
                self.short_call_low.option.quantity,
                low_strike.call_bid.unwrap().value(),
                middle_strike.call_ask.unwrap().value(),
                high_strike.call_bid.unwrap().value(),
                self.fees() / 8.0,
            ),
            _ => panic!("Invalid number of legs for Short Butterfly strategy"),
        }
    }
}

impl Profit for ShortButterflySpread {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.short_call_low.pnl_at_expiration(&price)
            + self.long_calls.pnl_at_expiration(&price)
            + self.short_call_high.pnl_at_expiration(&price)
    }
}

impl Graph for ShortButterflySpread {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind,
            self.short_call_low.option.underlying_symbol,
            self.short_call_low.option.quantity
        );

        let leg_titles = [format!(
                "Short Call Low Strike: ${}",
                self.short_call_low.option.strike_price
            ),
            format!(
                "Long Calls Middle Strike: ${}",
                self.long_calls.option.strike_price
            ),
            format!(
                "Short Call High Strike: ${}",
                self.short_call_high.option.strike_price
            ),
            format!("Expire: {}", self.short_call_low.option.expiration_date)];

        format!("{}\n\t{}", strategy_title, leg_titles.join("\n\t"))
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        vec![ChartVerticalLine {
            x_coordinate: self.short_call_low.option.underlying_price.value(),
            y_range: (-50000.0, 50000.0),
            label: format!(
                "Current Price: {}",
                self.short_call_low.option.underlying_price
            ),
            label_offset: (5.0, 5.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();
        let max_loss = self.max_loss().unwrap_or(PZERO);

        let left_profit = self.calculate_profit_at(self.short_call_low.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_call_high.option.strike_price);

        // Break-even points
        points.extend(
            self.break_even_points
                .iter()
                .enumerate()
                .map(|(i, &price)| ChartPoint {
                    coordinates: (price.value(), 0.0),
                    label: format!(
                        "Break Even {}: {:.2}",
                        if i == 0 { "Lower" } else { "Upper" },
                        price
                    ),
                    label_offset: LabelOffsetType::Relative(3.0, 3.0),
                    point_color: DARK_BLUE,
                    label_color: DARK_BLUE,
                    point_size: 5,
                    font_size: 18,
                }),
        );

        // Maximum loss point (at middle strike)
        points.push(ChartPoint {
            coordinates: (
                self.long_calls.option.strike_price.value(),
                -max_loss.value(),
            ),
            label: format!("Max Loss {:.2}", -max_loss.value()),
            label_offset: LabelOffsetType::Relative(3.0, -3.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let left_color = if left_profit > ZERO { DARK_GREEN } else { RED };

        // Maximum profit points (at wing strikes)
        points.push(ChartPoint {
            coordinates: (self.short_call_low.option.strike_price.value(), left_profit),
            label: format!("Left Profit {:.2}", left_profit),
            label_offset: LabelOffsetType::Relative(-30.0, 3.0),
            point_color: left_color,
            label_color: left_color,
            point_size: 5,
            font_size: 18,
        });

        let right_color = if right_profit > ZERO { DARK_GREEN } else { RED };

        points.push(ChartPoint {
            coordinates: (
                self.short_call_high.option.strike_price.value(),
                right_profit,
            ),
            label: format!("Right Profit {:.2}", right_profit),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: right_color,
            label_color: right_color,
            point_size: 5,
            font_size: 18,
        });

        // Current price point
        points.push(self.get_point_at_price(self.short_call_low.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for ShortButterflySpread {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        Ok(self.short_call_low.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.short_call_low.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let mut ranges = Vec::new();
        let break_even_points = self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.short_call_low.option.implied_volatility),
            pos!(self.long_calls.option.implied_volatility),
            pos!(self.short_call_high.option.implied_volatility),
        ]);

        let volatility_adjustment = Some(VolatilityAdjustment {
            base_volatility: mean_volatility,
            std_dev_adjustment: std_dev,
        });

        let mut lower_profit_range = ProfitLossRange::new(
            Some(self.short_call_low.option.strike_price),
            Some(break_even_points[0]),
            pos!(self.max_profit()?.value()),
        )?;

        lower_profit_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment.clone(),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(lower_profit_range);

        let mut upper_profit_range = ProfitLossRange::new(
            Some(break_even_points[1]),
            Some(self.short_call_high.option.strike_price),
            pos!(self.max_profit()?.value()),
        )?;

        upper_profit_range.calculate_probability(
            self.get_underlying_price(),
            volatility_adjustment,
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        ranges.push(upper_profit_range);

        Ok(ranges)
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let break_even_points = self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(self.short_call_low.option.implied_volatility),
            pos!(self.long_calls.option.implied_volatility),
            pos!(self.short_call_high.option.implied_volatility),
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            pos!(self.max_loss()?.value()),
        )?;

        loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![loss_range])
    }
}

#[cfg(test)]
mod tests_long_butterfly_spread {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(90.0),                 // low_strike
            pos!(100.0),                // middle_strike
            pos!(110.0),                // high_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            3.0,                        // premium_low
            2.0,                        // premium_middle
            1.0,                        // premium_high
            0.05,                       // fees
        )
    }

    #[test]
    fn test_new_butterfly_basic_properties() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.name, "Long Butterfly");
        assert_eq!(butterfly.kind, StrategyType::LongButterflySpread);
        assert!(!butterfly.description.is_empty());
        assert!(butterfly.description.contains("long butterfly spread"));
    }

    #[test]
    fn test_butterfly_strikes() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.long_call_low.option.strike_price, pos!(90.0));
        assert_eq!(butterfly.short_calls.option.strike_price, pos!(100.0));
        assert_eq!(butterfly.long_call_high.option.strike_price, pos!(110.0));
    }

    #[test]
    fn test_butterfly_quantities() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.long_call_low.option.quantity, pos!(1.0));
        assert_eq!(butterfly.short_calls.option.quantity, pos!(2.0)); // Double quantity
        assert_eq!(butterfly.long_call_high.option.quantity, pos!(1.0));
    }

    #[test]
    fn test_butterfly_sides() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.long_call_low.option.side, Side::Long);
        assert_eq!(butterfly.short_calls.option.side, Side::Short);
        assert_eq!(butterfly.long_call_high.option.side, Side::Long);
    }

    #[test]
    fn test_butterfly_option_styles() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.long_call_low.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(butterfly.short_calls.option.option_style, OptionStyle::Call);
        assert_eq!(
            butterfly.long_call_high.option.option_style,
            OptionStyle::Call
        );
    }

    #[test]
    fn test_butterfly_expiration_consistency() {
        let butterfly = create_test_butterfly();
        let expiration = ExpirationDate::Days(30.0);

        assert_eq!(
            format!("{:?}", butterfly.long_call_low.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.short_calls.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.long_call_high.option.expiration_date),
            format!("{:?}", expiration)
        );
    }

    #[test]
    fn test_butterfly_fees_distribution() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            3.0, // total fees = 3.0
        );

        assert_eq!(butterfly.long_call_low.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.short_calls.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.long_call_high.open_fee, 1.0); // fees / 3
    }

    #[test]
    fn test_butterfly_break_even_points() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.break_even_points;

        assert_eq!(break_even_points.len(), 2);
        assert!(break_even_points[0] > butterfly.long_call_low.option.strike_price);
        assert!(break_even_points[0] < butterfly.short_calls.option.strike_price);
        assert!(break_even_points[1] > butterfly.short_calls.option.strike_price);
        assert!(break_even_points[1] < butterfly.long_call_high.option.strike_price);
    }

    #[test]
    fn test_butterfly_with_different_quantities() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            3.0,
            2.0,
            1.0,
            0.0,
        );

        assert_eq!(butterfly.long_call_low.option.quantity, pos!(2.0));
        assert_eq!(butterfly.short_calls.option.quantity, pos!(4.0)); // 2 * 2
        assert_eq!(butterfly.long_call_high.option.quantity, pos!(2.0));
    }

    #[test]
    fn test_butterfly_with_symmetric_strikes() {
        let butterfly = create_test_butterfly();

        let lower_width =
            butterfly.short_calls.option.strike_price - butterfly.long_call_low.option.strike_price;
        let upper_width = butterfly.long_call_high.option.strike_price
            - butterfly.short_calls.option.strike_price;

        assert_eq!(lower_width, upper_width);
    }

    #[test]
    fn test_butterfly_with_equal_implied_volatility() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.long_call_low.option.implied_volatility,
            butterfly.short_calls.option.implied_volatility
        );
        assert_eq!(
            butterfly.short_calls.option.implied_volatility,
            butterfly.long_call_high.option.implied_volatility
        );
    }

    #[test]
    fn test_butterfly_with_invalid_premiums() {
        let check_profit = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            1.0,
            1.0,
            4.0,
        );
        assert!(check_profit.max_profit().is_err());
    }
}

#[cfg(test)]
mod tests_short_butterfly_spread {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(90.0),                 // low_strike
            pos!(100.0),                // middle_strike
            pos!(110.0),                // high_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            10.0,                       // premium_low
            1.0,                        // premium_middle
            0.5,                        // premium_high
            0.05,                       // fees
        )
    }

    #[test]
    fn test_new_butterfly_basic_properties() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.name, "Short Butterfly");
        assert_eq!(butterfly.kind, StrategyType::ShortButterflySpread);
        assert!(!butterfly.description.is_empty());
        assert!(butterfly.description.contains("short butterfly spread"));
    }

    #[test]
    fn test_butterfly_strikes() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.short_call_low.option.strike_price, pos!(90.0));
        assert_eq!(butterfly.long_calls.option.strike_price, pos!(100.0));
        assert_eq!(butterfly.short_call_high.option.strike_price, pos!(110.0));
    }

    #[test]
    fn test_butterfly_quantities() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.short_call_low.option.quantity, pos!(1.0));
        assert_eq!(butterfly.long_calls.option.quantity, pos!(2.0)); // Double quantity
        assert_eq!(butterfly.short_call_high.option.quantity, pos!(1.0));
    }

    #[test]
    fn test_butterfly_sides() {
        let butterfly = create_test_butterfly();

        assert_eq!(butterfly.short_call_low.option.side, Side::Short);
        assert_eq!(butterfly.long_calls.option.side, Side::Long);
        assert_eq!(butterfly.short_call_high.option.side, Side::Short);
    }

    #[test]
    fn test_butterfly_option_styles() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.short_call_low.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(butterfly.long_calls.option.option_style, OptionStyle::Call);
        assert_eq!(
            butterfly.short_call_high.option.option_style,
            OptionStyle::Call
        );
    }

    #[test]
    fn test_butterfly_expiration_consistency() {
        let butterfly = create_test_butterfly();
        let expiration = ExpirationDate::Days(30.0);

        assert_eq!(
            format!("{:?}", butterfly.short_call_low.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.long_calls.option.expiration_date),
            format!("{:?}", expiration)
        );
        assert_eq!(
            format!("{:?}", butterfly.short_call_high.option.expiration_date),
            format!("{:?}", expiration)
        );
    }

    #[test]
    fn test_butterfly_fees_distribution() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            3.0, // total fees = 3.0
        );

        assert_eq!(butterfly.short_call_low.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.long_calls.open_fee, 1.0); // fees / 3
        assert_eq!(butterfly.short_call_high.open_fee, 1.0); // fees / 3
    }

    #[test]
    fn test_butterfly_break_even_points() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.break_even_points;

        assert_eq!(break_even_points.len(), 2);
        assert!(break_even_points[0] > butterfly.short_call_low.option.strike_price);
        assert!(break_even_points[0] < butterfly.long_calls.option.strike_price);
        assert!(break_even_points[1] > butterfly.long_calls.option.strike_price);
        assert!(break_even_points[1] < butterfly.short_call_high.option.strike_price);
    }

    #[test]
    fn test_butterfly_with_different_quantities() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0),
            3.0,
            2.0,
            1.0,
            0.0,
        );

        assert_eq!(butterfly.short_call_low.option.quantity, pos!(2.0));
        assert_eq!(butterfly.long_calls.option.quantity, pos!(4.0)); // 2 * 2
        assert_eq!(butterfly.short_call_high.option.quantity, pos!(2.0));
    }

    #[test]
    fn test_butterfly_with_symmetric_strikes() {
        let butterfly = create_test_butterfly();

        let lower_width =
            butterfly.long_calls.option.strike_price - butterfly.short_call_low.option.strike_price;
        let upper_width = butterfly.short_call_high.option.strike_price
            - butterfly.long_calls.option.strike_price;

        assert_eq!(lower_width, upper_width);
    }

    #[test]
    fn test_butterfly_with_equal_implied_volatility() {
        let butterfly = create_test_butterfly();

        assert_eq!(
            butterfly.short_call_low.option.implied_volatility,
            butterfly.long_calls.option.implied_volatility
        );
        assert_eq!(
            butterfly.long_calls.option.implied_volatility,
            butterfly.short_call_high.option.implied_volatility
        );
    }

    #[test]
    fn test_butterfly_underlying_price_consistency() {
        let butterfly = create_test_butterfly();
        let underlying_price = pos!(100.0);

        assert_eq!(
            butterfly.short_call_low.option.underlying_price,
            underlying_price
        );
        assert_eq!(
            butterfly.long_calls.option.underlying_price,
            underlying_price
        );
        assert_eq!(
            butterfly.short_call_high.option.underlying_price,
            underlying_price
        );
    }

    #[test]

    fn test_butterfly_with_invalid_premiums() {
        let max_loss = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            10.0,
            1.0,
            10.0,
            0.05,
        );
        assert!(max_loss.max_loss().is_err());
    }

    #[test]
    fn test_butterfly_risk_free_rate_consistency() {
        let butterfly = create_test_butterfly();
        let risk_free_rate = 0.05;

        assert_eq!(
            butterfly.short_call_low.option.risk_free_rate,
            risk_free_rate
        );
        assert_eq!(butterfly.long_calls.option.risk_free_rate, risk_free_rate);
        assert_eq!(
            butterfly.short_call_high.option.risk_free_rate,
            risk_free_rate
        );
    }
}

#[cfg(test)]
mod tests_long_butterfly_validation {
    use super::*;

    fn create_valid_position(
        side: Side,
        strike_price: PositiveF64,
        quantity: PositiveF64,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                ExpirationDate::Days(30.0),
                0.20,
                quantity,
                pos!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_valid_long_butterfly() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        assert!(butterfly.validate());
    }

    #[test]
    fn test_invalid_long_call_low() {
        let mut butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        butterfly.long_call_low = create_valid_position(Side::Long, pos!(90.0), PZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_strike_order_low() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(100.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_quantities() {
        let mut butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        butterfly.short_calls = create_valid_position(Side::Short, pos!(100.0), pos!(1.0));
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_unequal_wing_quantities() {
        let mut butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        butterfly.long_call_high = create_valid_position(Side::Long, pos!(110.0), pos!(2.0));
        assert!(!butterfly.validate());
    }
}

#[cfg(test)]
mod tests_short_butterfly_validation {
    use super::*;

    fn create_valid_position(
        side: Side,
        strike_price: PositiveF64,
        quantity: PositiveF64,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                ExpirationDate::Days(30.0),
                0.20,
                quantity,
                pos!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_valid_short_butterfly() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        assert!(butterfly.validate());
    }

    #[test]
    fn test_invalid_short_call_low() {
        let mut butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        butterfly.short_call_low = create_valid_position(Side::Short, pos!(90.0), PZERO);
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_strike_order_high() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(90.0),
            pos!(100.0),
            pos!(100.0),
            pos!(100.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_invalid_middle_quantities() {
        let mut butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        butterfly.long_calls = create_valid_position(Side::Long, pos!(100.0), pos!(1.0));
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_unequal_wing_quantities_short() {
        let mut butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            1.0,
            2.0,
            1.0,
            0.0,
        );
        butterfly.short_call_high = create_valid_position(Side::Short, pos!(110.0), pos!(2.0));
        assert!(!butterfly.validate());
    }
}

#[cfg(test)]
mod tests_butterfly_strategies {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_long_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // quantity
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.0,       // fees
        )
    }

    fn create_test_short_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // quantity
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.0,       // fees
        )
    }

    #[test]
    fn test_underlying_price() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        assert_eq!(long_butterfly.get_underlying_price(), pos!(100.0));
        assert_eq!(short_butterfly.get_underlying_price(), pos!(100.0));
    }

    #[test]
    fn test_add_leg_long_butterfly() {
        let mut butterfly = create_test_long_butterfly();
        let new_long = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos!(85.0),
                ExpirationDate::Days(30.0),
                0.20,
                pos!(1.0),
                pos!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        );

        butterfly.add_position(&new_long.clone()).expect("Failed to add position");
        assert_eq!(butterfly.long_call_low.option.strike_price, pos!(85.0));
    }

    #[test]
    fn test_add_leg_short_butterfly() {
        let mut butterfly = create_test_short_butterfly();
        let new_short = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                "TEST".to_string(),
                pos!(85.0),
                ExpirationDate::Days(30.0),
                0.20,
                pos!(1.0),
                pos!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        );

        butterfly.add_position(&new_short.clone()).expect("Failed to add position");
        assert_eq!(butterfly.short_call_low.option.strike_price, pos!(85.0));
    }

    #[test]
    fn test_get_legs() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        assert_eq!(long_butterfly.get_positions().unwrap().len(), 3);
        assert_eq!(short_butterfly.get_positions().unwrap().len(), 3);
    }

    #[test]
    fn test_max_profit_long_butterfly() {
        let butterfly = create_test_long_butterfly();
        let max_profit = butterfly.max_profit().unwrap();
        // Max profit at middle strike
        let expected_profit = butterfly.calculate_profit_at(pos!(100.0));
        assert_eq!(max_profit.value(), expected_profit);
    }

    #[test]
    fn test_max_loss_long_butterfly() {
        let butterfly = create_test_long_butterfly();
        let max_loss = butterfly.max_loss().unwrap();
        // Max loss at wings
        let left_loss = butterfly.calculate_profit_at(pos!(90.0));
        let right_loss = butterfly.calculate_profit_at(pos!(110.0));
        assert_eq!(max_loss.value(), left_loss.min(right_loss).abs());
    }

    #[test]
    fn test_max_loss_short_butterfly() {
        let butterfly = create_test_short_butterfly();
        let max_loss = butterfly.max_loss().unwrap();
        // Max loss at middle strike
        let expected_loss = butterfly.calculate_profit_at(pos!(100.0));
        assert_eq!(max_loss.value(), expected_loss.abs());
    }

    #[test]
    fn test_total_cost() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        assert!(long_butterfly.total_cost() > PZERO);
        assert!(short_butterfly.total_cost() > PZERO);
    }

    #[test]
    fn test_fees() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            3.0, // total fees
        );
        assert_eq!(butterfly.fees(), 6.0);
    }

    #[test]
    fn test_fees_bis() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0),
            3.0,
            2.0,
            1.0,
            3.0, // total fees
        );

        assert_eq!(butterfly.fees(), 12.0);
    }

    #[test]
    fn test_profit_area_long_butterfly() {
        let butterfly = create_test_long_butterfly();
        let area = butterfly.profit_area();
        assert!(area > ZERO);
    }

    #[test]
    fn test_profit_area_short_butterfly() {
        let butterfly = create_test_short_butterfly();
        let area = butterfly.profit_area();
        assert!(area >= ZERO);
    }

    #[test]
    fn test_profit_ratio() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        assert!(long_butterfly.profit_ratio() > ZERO);
        assert!(short_butterfly.profit_ratio() >= ZERO);
    }

    #[test]
    fn test_break_even_points() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        assert_eq!(long_butterfly.get_break_even_points().len(), 2);
        assert_eq!(short_butterfly.get_break_even_points().len(), 2);
    }

    #[test]
    fn test_profits_with_quantities() {
        let long_butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            3.0,
            2.0,
            1.0,
            0.0,
        );

        let base_butterfly = create_test_long_butterfly();
        assert_eq!(
            long_butterfly.max_profit().unwrap().value(),
            base_butterfly.max_profit().unwrap().value() * 2.0
        );
    }
}

#[cfg(test)]
mod tests_butterfly_optimizable {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::spos;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string());

        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                pos!(strike),
                spos!(5.0),   // call_bid
                spos!(5.2),   // call_ask
                spos!(5.0),   // put_bid
                spos!(5.2),   // put_ask
                spos!(0.2),   // implied_volatility
                Some(0.5),    // delta
                spos!(100.0), // volume
                Some(50),     // open_interest
            );
        }
        chain
    }

    fn create_test_long_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            0.0,
        )
    }

    fn create_test_short_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            0.0,
        )
    }

    #[test]
    fn test_long_butterfly_find_optimal_area() {
        let mut butterfly = create_test_long_butterfly();
        let chain = create_test_option_chain();
        let initial_area = butterfly.profit_area();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        assert!(butterfly.profit_area() >= initial_area);
    }

    #[test]
    fn test_long_butterfly_valid_strike_order() {
        let mut butterfly = create_test_long_butterfly();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(
            butterfly.long_call_low.option.strike_price < butterfly.short_calls.option.strike_price
        );
        assert!(
            butterfly.short_calls.option.strike_price
                < butterfly.long_call_high.option.strike_price
        );
    }

    #[test]
    fn test_long_butterfly_is_valid_long_option() {
        let butterfly = create_test_long_butterfly();
        let option = OptionData::new(
            pos!(95.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        assert!(butterfly.is_valid_long_option(&option, &FindOptimalSide::All));
        assert!(butterfly.is_valid_long_option(&option, &FindOptimalSide::Lower));
        assert!(!butterfly.is_valid_long_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_long_butterfly_are_valid_prices() {
        let butterfly = create_test_long_butterfly();
        let chain = create_test_option_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::ThreeLegs {
            first: options[1],  // 90.0 strike
            second: options[3], // 100.0 strike
            third: options[5],  // 110.0 strike
        };

        assert!(butterfly.are_valid_prices(&legs));
    }

    #[test]
    fn test_short_butterfly_find_optimal_ratio() {
        let mut butterfly = create_test_short_butterfly();
        let chain = create_test_option_chain();
        let initial_ratio = butterfly.profit_ratio();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(butterfly.validate());
        assert!(butterfly.profit_ratio() >= initial_ratio);
    }

    #[test]
    fn test_short_butterfly_find_optimal_area() {
        let mut butterfly = create_test_short_butterfly();
        let chain = create_test_option_chain();
        let initial_area = butterfly.profit_area();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        assert!(butterfly.profit_area() >= initial_area);
    }

    #[test]
    fn test_short_butterfly_valid_strike_order() {
        let mut butterfly = create_test_short_butterfly();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(
            butterfly.short_call_low.option.strike_price < butterfly.long_calls.option.strike_price
        );
        assert!(
            butterfly.long_calls.option.strike_price
                < butterfly.short_call_high.option.strike_price
        );
    }

    #[test]
    fn test_short_butterfly_is_valid_short_option() {
        let butterfly = create_test_short_butterfly();
        let option = OptionData::new(
            pos!(105.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        assert!(butterfly.is_valid_short_option(&option, &FindOptimalSide::All));
        assert!(!butterfly.is_valid_short_option(&option, &FindOptimalSide::Lower));
        assert!(butterfly.is_valid_short_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_short_butterfly_are_valid_prices() {
        let butterfly = create_test_short_butterfly();
        let chain = create_test_option_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::ThreeLegs {
            first: options[1],  // 90.0 strike
            second: options[3], // 100.0 strike
            third: options[5],  // 110.0 strike
        };

        assert!(butterfly.are_valid_prices(&legs));
    }

    #[test]
    fn test_find_optimal_with_range() {
        let mut long_butterfly = create_test_long_butterfly();
        let mut short_butterfly = create_test_short_butterfly();
        let chain = create_test_option_chain();

        long_butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
            OptimizationCriteria::Ratio,
        );
        short_butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(long_butterfly.short_calls.option.strike_price >= pos!(95.0));
        assert!(long_butterfly.short_calls.option.strike_price <= pos!(105.0));
        assert!(short_butterfly.long_calls.option.strike_price >= pos!(95.0));
        assert!(short_butterfly.long_calls.option.strike_price <= pos!(105.0));
    }

    #[test]
    fn test_invalid_prices() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        let invalid_option = OptionData::new(
            pos!(100.0),
            None, // missing call_bid
            None, // missing call_ask
            None,
            None,
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::ThreeLegs {
            first: &invalid_option,
            second: &invalid_option,
            third: &invalid_option,
        };

        assert!(!long_butterfly.are_valid_prices(&legs));
        assert!(!short_butterfly.are_valid_prices(&legs));
    }
}

#[cfg(test)]
mod tests_long_butterfly_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;

    fn create_test_long_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // quantity
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.05,      // fees
        )
    }

    #[test]
    fn test_profit_at_middle_strike() {
        let butterfly = create_test_long_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(100.0));
        assert!(profit > 0.0);
        assert_eq!(profit, butterfly.max_profit().unwrap().value());
    }

    #[test]
    fn test_profit_below_lowest_strike() {
        let butterfly = create_test_long_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(85.0));
        assert!(profit < 0.0);
        assert_eq!(profit, -butterfly.max_loss().unwrap().value());
    }

    #[test]
    fn test_profit_above_highest_strike() {
        let butterfly = create_test_long_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(115.0));
        assert!(profit < 0.0);
        assert_relative_eq!(
            profit,
            -butterfly.max_loss().unwrap().value(),
            epsilon = 0.0001
        );
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let butterfly = create_test_long_butterfly();
        let break_even_points = butterfly.get_break_even_points();

        for &point in &break_even_points {
            let profit = butterfly.calculate_profit_at(point);
            assert_relative_eq!(profit, 0.0, epsilon = 0.01);
        }
    }

    #[test]
    fn test_profit_with_different_quantities() {
        let butterfly = LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(2.0), // quantity = 2
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.0,       // fees
        );

        let scaled_profit = butterfly.calculate_profit_at(pos!(100.0));
        assert_relative_eq!(scaled_profit, 20.0, epsilon = 0.0001);
    }
}

#[cfg(test)]
mod tests_short_butterfly_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;

    fn create_test_short_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            pos!(1.0), // quantity
            3.0,       // premium_low
            2.0,       // premium_middle
            1.0,       // premium_high
            0.05,      // fees
        )
    }

    #[test]
    fn test_profit_at_middle_strike() {
        let butterfly = create_test_short_butterfly();
        let profit = butterfly.calculate_profit_at(pos!(100.0));
        assert!(profit < 0.0);
        assert_eq!(profit, -butterfly.max_loss().unwrap().value());
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let butterfly = create_test_short_butterfly();
        let break_even_points = butterfly.get_break_even_points();

        for &point in &break_even_points {
            let profit = butterfly.calculate_profit_at(point);
            assert_relative_eq!(profit, 0.0, epsilon = 0.01);
        }
    }

    #[test]
    fn test_profit_with_different_quantities() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(2.0), // quantity = 2
            3.0,
            2.0,
            1.0,
            0.05,
        );
        let scaled_profit = butterfly.calculate_profit_at(pos!(85.0));
        assert_relative_eq!(scaled_profit, -0.2666666, epsilon = 0.0001);
    }

    #[test]
    fn test_profit_symmetry() {
        let butterfly = create_test_short_butterfly();
        let low_extreme_profit = butterfly.calculate_profit_at(pos!(85.0));
        let high_extreme_profit = butterfly.calculate_profit_at(pos!(115.0));

        assert_relative_eq!(low_extreme_profit, high_extreme_profit, epsilon = 0.01);
    }

    #[test]
    fn test_profit_with_fees() {
        let butterfly = ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            3.0, // fees = 3.0
        );

        let base_butterfly = create_test_short_butterfly();
        let profit_with_fees = butterfly.calculate_profit_at(pos!(85.0));
        let profit_without_fees = base_butterfly.calculate_profit_at(pos!(85.0));

        assert!(profit_with_fees < profit_without_fees);
    }
}

#[cfg(test)]
mod tests_long_butterfly_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            0.0,
        )
    }

    #[test]
    fn test_title_format() {
        let butterfly = create_test_butterfly();
        let title = butterfly.title();

        assert!(title.contains("LongButterflySpread Strategy"));
        assert!(title.contains("TEST"));
        assert!(title.contains("Size 1"));
        assert!(title.contains("Long Call Low Strike: $90"));
        assert!(title.contains("Short Calls Middle Strike: $100"));
        assert!(title.contains("Long Call High Strike: $110"));
        assert!(title.contains("Expire"));
    }

    #[test]
    fn test_vertical_lines() {
        let butterfly = create_test_butterfly();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        assert_eq!(line.x_coordinate, 100.0);
        assert_eq!(line.y_range, (-50000.0, 50000.0));
        assert!(line.label.contains("Current Price: 100"));
        assert_eq!(line.line_color, ORANGE);
    }

    #[test]
    fn test_get_points() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        assert_eq!(points.len(), 6);

        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();
        assert_eq!(break_even_points.len(), 2);
        for point in break_even_points {
            assert_eq!(point.coordinates.1, 0.0);
            assert_eq!(point.point_color, DARK_BLUE);
        }

        let max_profit_point = points
            .iter()
            .find(|p| p.label.contains("Max Profit"))
            .unwrap();
        assert_eq!(max_profit_point.coordinates.0, 100.0);
        assert_eq!(max_profit_point.point_color, DARK_GREEN);

        let loss_points: Vec<&ChartPoint<(f64, f64)>> =
            points.iter().filter(|p| p.label.contains("Loss")).collect();
        for point in loss_points {
            assert!(point.coordinates.1 <= 0.0);
            assert_eq!(point.point_color, RED);
        }
    }

    #[test]
    fn test_point_colors() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        for point in points {
            match point.label.as_str() {
                label if label.contains("Break Even") => {
                    assert_eq!(point.point_color, DARK_BLUE);
                    assert_eq!(point.label_color, DARK_BLUE);
                }
                label if label.contains("Max Profit") => {
                    assert_eq!(point.point_color, DARK_GREEN);
                    assert_eq!(point.label_color, DARK_GREEN);
                }
                label if label.contains("Loss") && point.coordinates.1 < 0.0 => {
                    assert_eq!(point.point_color, RED);
                    assert_eq!(point.label_color, RED);
                }
                _ => {}
            }
        }
    }
}

#[cfg(test)]
mod tests_short_butterfly_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(90.0),  // low_strike
            pos!(100.0), // middle_strike
            pos!(110.0), // high_strike
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            3.0,
            2.0,
            1.0,
            0.0,
        )
    }

    #[test]
    fn test_title_format() {
        let butterfly = create_test_butterfly();
        let title = butterfly.title();

        assert!(title.contains("ShortButterflySpread Strategy"));
        assert!(title.contains("TEST"));
        assert!(title.contains("Size 1"));
        assert!(title.contains("Short Call Low Strike: $90"));
        assert!(title.contains("Long Calls Middle Strike: $100"));
        assert!(title.contains("Short Call High Strike: $110"));
        assert!(title.contains("Expire"));
    }

    #[test]
    fn test_vertical_lines() {
        let butterfly = create_test_butterfly();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1);
        let line = &lines[0];
        assert_eq!(line.x_coordinate, 100.0);
        assert_eq!(line.y_range, (-50000.0, 50000.0));
        assert!(line.label.contains("Current Price: 100"));
        assert_eq!(line.line_color, ORANGE);
    }

    #[test]
    fn test_get_points() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        assert_eq!(points.len(), 6);

        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();
        assert_eq!(break_even_points.len(), 2);
        for point in break_even_points {
            assert_eq!(point.coordinates.1, 0.0);
            assert_eq!(point.point_color, DARK_BLUE);
        }

        let max_loss_point = points
            .iter()
            .find(|p| p.label.contains("Max Loss"))
            .unwrap();
        assert_eq!(max_loss_point.coordinates.0, 100.0);
        assert_eq!(max_loss_point.point_color, RED);

        let profit_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Profit"))
            .collect();
        for point in profit_points {
            assert!(point.coordinates.1 >= 0.0);
        }
    }

    #[test]
    fn test_point_symmetry() {
        let butterfly = create_test_butterfly();
        let points = butterfly.get_points();

        let profit_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Profit"))
            .collect();

        assert_eq!(
            profit_points[0].coordinates.1,
            profit_points[1].coordinates.1
        );
    }
}

#[cfg(test)]
mod tests_butterfly_probability {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;

    fn create_test_long_butterfly() -> LongButterflySpread {
        LongButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            10.0,
            2.0,
            1.0,
            0.05,
        )
    }

    fn create_test_short_butterfly() -> ShortButterflySpread {
        ShortButterflySpread::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(90.0),
            pos!(100.0),
            pos!(110.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            pos!(1.0),
            10.0,
            2.0,
            1.0,
            0.05,
        )
    }

    mod long_butterfly_tests {
        use super::*;

        #[test]
        fn test_get_expiration() {
            let butterfly = create_test_long_butterfly();
            let expiration = butterfly.get_expiration().unwrap();
            match expiration {
                ExpirationDate::Days(days) => assert_eq!(days, 30.0),
                _ => panic!("Expected ExpirationDate::Days"),
            }
        }

        #[test]
        fn test_get_risk_free_rate() {
            let butterfly = create_test_long_butterfly();
            assert_eq!(butterfly.get_risk_free_rate(), Some(0.05));
        }

        #[test]
        fn test_get_profit_ranges() {
            let butterfly = create_test_long_butterfly();
            let ranges = butterfly.get_profit_ranges().unwrap();

            assert_eq!(ranges.len(), 1);
            let range = &ranges[0];

            let break_even_points = butterfly.get_break_even_points();
            assert_eq!(range.lower_bound.unwrap(), break_even_points[0]);
            assert_eq!(range.upper_bound.unwrap(), break_even_points[1]);
            assert!(range.probability > PZERO);
        }

        #[test]
        fn test_get_loss_ranges() {
            let butterfly = create_test_long_butterfly();
            let ranges = butterfly.get_loss_ranges().unwrap();

            assert_eq!(ranges.len(), 2);

            let lower_range = &ranges[0];
            assert_eq!(
                lower_range.lower_bound.unwrap(),
                butterfly.long_call_low.option.strike_price
            );
            assert_eq!(
                lower_range.upper_bound.unwrap(),
                butterfly.get_break_even_points()[0]
            );
            assert!(lower_range.probability > PZERO);

            let upper_range = &ranges[1];
            assert_eq!(
                upper_range.lower_bound.unwrap(),
                butterfly.get_break_even_points()[1]
            );
            assert_eq!(
                upper_range.upper_bound.unwrap(),
                butterfly.long_call_high.option.strike_price
            );
            assert!(upper_range.probability > PZERO);
        }
    }

    mod short_butterfly_tests {
        use super::*;

        #[test]
        fn test_get_expiration() {
            let butterfly = create_test_short_butterfly();
            let expiration = butterfly.get_expiration().unwrap();
            match expiration {
                ExpirationDate::Days(days) => assert_eq!(days, 30.0),
                _ => panic!("Expected ExpirationDate::Days"),
            }
        }

        #[test]
        fn test_get_risk_free_rate() {
            let butterfly = create_test_short_butterfly();
            assert_eq!(butterfly.get_risk_free_rate(), Some(0.05));
        }

        #[test]
        fn test_get_profit_ranges() {
            let butterfly = create_test_short_butterfly();
            let ranges = butterfly.get_profit_ranges().unwrap();

            assert_eq!(ranges.len(), 2);

            let lower_range = &ranges[0];
            assert_eq!(
                lower_range.lower_bound.unwrap(),
                butterfly.short_call_low.option.strike_price
            );
            assert_eq!(
                lower_range.upper_bound.unwrap(),
                butterfly.get_break_even_points()[0]
            );
            assert!(lower_range.probability > PZERO);

            let upper_range = &ranges[1];
            assert_eq!(
                upper_range.lower_bound.unwrap(),
                butterfly.get_break_even_points()[1]
            );
            assert_eq!(
                upper_range.upper_bound.unwrap(),
                butterfly.short_call_high.option.strike_price
            );
            assert!(upper_range.probability > PZERO);
        }

        #[test]
        fn test_get_loss_ranges() {
            let butterfly = create_test_short_butterfly();
            let ranges = butterfly.get_loss_ranges().unwrap();

            assert_eq!(ranges.len(), 1);
            let range = &ranges[0];

            let break_even_points = butterfly.get_break_even_points();
            assert_eq!(range.lower_bound.unwrap(), break_even_points[0]);
            assert_eq!(range.upper_bound.unwrap(), break_even_points[1]);
            assert!(range.probability > PZERO);
        }
    }

    #[test]
    fn test_volatility_calculations() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        let long_ranges = long_butterfly.get_profit_ranges().unwrap();
        let short_ranges = short_butterfly.get_profit_ranges().unwrap();

        assert!(!long_ranges.is_empty());
        assert!(!short_ranges.is_empty());
        assert!(long_ranges[0].probability > PZERO);
        assert!(short_ranges[0].probability > PZERO);
    }

    #[test]
    fn test_probability_sum() {
        let long_butterfly = create_test_long_butterfly();
        let short_butterfly = create_test_short_butterfly();

        let long_profit_ranges = long_butterfly.get_profit_ranges().unwrap();
        let long_loss_ranges = long_butterfly.get_loss_ranges().unwrap();
        let long_total_prob = long_profit_ranges
            .iter()
            .map(|r| r.probability.value())
            .sum::<f64>()
            + long_loss_ranges
                .iter()
                .map(|r| r.probability.value())
                .sum::<f64>();
        assert!((long_total_prob - 1.0).abs() < 0.1);

        let short_profit_ranges = short_butterfly.get_profit_ranges().unwrap();
        let short_loss_ranges = short_butterfly.get_loss_ranges().unwrap();
        let short_total_prob = short_profit_ranges
            .iter()
            .map(|r| r.probability.value())
            .sum::<f64>()
            + short_loss_ranges
                .iter()
                .map(|r| r.probability.value())
                .sum::<f64>();
        assert!((short_total_prob - 1.0).abs() < 0.1);
    }
}
