/*
Straddle Strategy

A straddle involves simultaneously buying a call and a put option with the same strike price and expiration date.
This strategy is used when a significant move in the underlying asset's price is expected, but the direction is uncertain.

Key characteristics:
- Unlimited profit potential
- High cost due to purchasing both a call and a put
- Profitable only with a large move in either direction
*/

use super::base::{Optimizable, Strategies, StrategyType, Validable};
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
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use std::f64;
use tracing::{debug, error, trace};

/// A Short Straddle is an options trading strategy that involves simultaneously selling
/// a put and a call option with the same strike price and expiration date. This neutral
/// strategy profits from low volatility and time decay, as the trader collects premium
/// from both options. Maximum profit is limited to the total premium collected, while
/// potential loss is unlimited. The strategy is most profitable when the underlying
/// asset stays close to the strike price through expiration.
///
/// Key characteristics:
/// - Sell 1 ATM Call
/// - Sell 1 ATM Put
/// - Same strike price
/// - Same expiration date
/// - Maximum profit: Total premium received
/// - Maximum loss: Unlimited
/// - Break-even points: Strike price +/- total premium received
/// - Ideal market forecast: Range-bound, low volatility
const SHORT_STRADDLE_DESCRIPTION: &str = "Short Straddle strategy involves simultaneously \
selling a put and a call option with identical strike prices and expiration dates. \
Profits from decreased volatility and time decay, with maximum gain limited to premium \
received and unlimited potential loss. Most effective in range-bound markets with low \
volatility expectations.";

#[derive(Clone, Debug)]
pub struct ShortStraddle {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    short_call: Position,
    short_put: Position,
}

impl ShortStraddle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        mut strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_short_call: f64,
        premium_short_put: f64,
        open_fee_short_call: f64,
        close_fee_short_call: f64,
        open_fee_short_put: f64,
        close_fee_short_put: f64,
    ) -> Self {
        if strike == PZERO {
            strike = underlying_price;
        }

        let mut strategy = ShortStraddle {
            name: "Short Straddle".to_string(),
            kind: StrategyType::Straddle,
            description: SHORT_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            strike,
            expiration.clone(),
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

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let short_put = Position::new(
            short_put_option,
            premium_short_put,
            Utc::now(),
            open_fee_short_put,
            close_fee_short_put,
        );
        strategy.add_leg(short_put.clone());

        let net_quantity = (short_call.option.quantity + short_put.option.quantity) / 2.0;
        strategy
            .break_even_points
            .push(strike - strategy.net_premium_received() / net_quantity);
        strategy
            .break_even_points
            .push(strike + strategy.net_premium_received() / net_quantity);

        strategy.break_even_points.sort();
        strategy
    }
}

impl Strategies for ShortStraddle {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.short_call.option.underlying_price
    }

    fn add_leg(&mut self, position: Position) {
        match position.option.option_style {
            OptionStyle::Call => self.short_call = position,
            OptionStyle::Put => self.short_put = position,
        }
    }

    fn get_legs(&self) -> Vec<Position> {
        vec![self.short_call.clone(), self.short_put.clone()]
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        let max_profit = self.net_premium_received();
        if max_profit < ZERO {
            Err("Max Profit is negative")
        } else {
            Ok(max_profit.into())
        }
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        Ok(f64::INFINITY.into())
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(self.short_call.net_cost() + self.short_put.net_cost())
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received() + self.short_put.net_premium_received()
    }

    fn fees(&self) -> f64 {
        self.short_call.open_fee
            + self.short_call.close_fee
            + self.short_put.open_fee
            + self.short_put.close_fee
    }

    fn profit_area(&self) -> f64 {
        let strike_diff = self.break_even_points[1] - self.break_even_points[0];
        let cat = (strike_diff / 2.0_f64.sqrt()).value();
        (cat.powf(2.0)) / (2.0 * 10.0_f64.powf(cat.log10().ceil()))
    }

    fn profit_ratio(&self) -> f64 {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        self.max_profit().unwrap_or(PZERO).value() / break_even_diff * 100.0
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        let mut break_even_points = self.break_even_points.clone();
        break_even_points.sort();
        break_even_points
    }
}

impl Validable for ShortStraddle {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.short_call.option.strike_price == self.short_put.option.strike_price
    }
}

impl Optimizable for ShortStraddle {
    type Strategy = ShortStraddle;

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for call_index in 0..options.len() {
            let call_option = &options[call_index];

            for put_option in &options[..call_index] {
                if call_option.strike_price <= put_option.strike_price {
                    error!(
                        "Invalid strike prices CALL: {:#?} PUT: {:#?}",
                        call_option.strike_price, put_option.strike_price
                    );
                    continue;
                }

                if !self.is_valid_short_option(put_option, &side)
                    || !self.is_valid_short_option(call_option, &side)
                {
                    continue;
                }

                let legs = StrategyLegs::TwoLegs {
                    first: call_option,
                    second: put_option,
                };

                if !self.are_valid_prices(&legs) {
                    error!(
                        "Invalid Bid prices  Put({}): {:?} Call({}): {:?} ",
                        put_option.strike_price,
                        put_option.put_bid.unwrap_or(PZERO),
                        call_option.strike_price,
                        call_option.call_bid.unwrap_or(PZERO)
                    );
                    continue;
                }

                debug!("Creating Strategy");
                let strategy: ShortStraddle = self.create_strategy(option_chain, &legs);

                if !strategy.validate() {
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
        debug!("Best Value: {}", best_value);
    }

    fn is_valid_short_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let underlying_price = match (
            self.short_put.option.underlying_price,
            self.short_call.option.underlying_price,
        ) {
            (PZERO, PZERO) => PZERO,
            (PZERO, call) => call,
            (put, _) => put,
        };
        if underlying_price == PZERO {
            error!("Invalid underlying_price option");
            return false;
        }

        match side {
            FindOptimalSide::Upper => {
                let valid = option.strike_price >= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} <= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::Lower => {
                let valid = option.strike_price <= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} >= {}",
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
                        "Option is out of range: {} >= {} && {} <= {}",
                        option.strike_price, *start, option.strike_price, *end
                    );
                }
                valid
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        let (call, put) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        call.call_bid.unwrap() > PZERO && put.put_bid.unwrap() > PZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (call, put) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        if !call.validate() || !put.validate() {
            panic!("Invalid options");
        }
        ShortStraddle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            self.short_call.option.expiration_date.clone(),
            call.implied_volatility.unwrap().value() / 100.0,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            call.call_bid.unwrap().value(),
            put.put_bid.unwrap().value(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for ShortStraddle {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        trace!(
            "Price: {:?} Strike: {} Call: {:.2} Strike: {} Put: {:.2} Profit: {:.2}",
            price,
            self.short_call.option.strike_price,
            self.short_call.pnl_at_expiration(&price),
            self.short_put.option.strike_price,
            self.short_put.pnl_at_expiration(&price),
            self.short_call.pnl_at_expiration(&price) + self.short_put.pnl_at_expiration(&price)
        );
        self.short_call.pnl_at_expiration(&price) + self.short_put.pnl_at_expiration(&price)
    }
}

impl Graph for ShortStraddle {
    fn title(&self) -> String {
        let strategy_title = format!("Short {:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.title(), self.short_put.title()]
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
        let max_value = f64::INFINITY;
        let min_value = f64::NEG_INFINITY;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.short_call.option.underlying_price.value(),
            y_range: (min_value, max_value),
            label: format!(
                "Current Price: {:.2}",
                self.short_call.option.underlying_price
            ),
            label_offset: (4.0, -1.0),
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

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(0.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(-230.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordinates: (f64, f64) = (
            -self.short_put.option.strike_price.value() / 30.0,
            max_profit.value() / 15.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.value(),
                max_profit.value(),
            ),
            label: format!(
                "Max Profit {:.2} at {:.0}",
                max_profit, self.short_put.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });
        points.push(self.get_point_at_price(self.short_put.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for ShortStraddle {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        let option = &self.short_call.option;
        Ok(option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.short_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let option = &self.short_call.option;
        let break_even_points = &self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(option.implied_volatility),
            pos!(self.short_put.option.implied_volatility),
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            PZERO,
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
        let option = &self.short_call.option;
        let break_even_points = &self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(option.implied_volatility),
            pos!(self.short_put.option.implied_volatility),
        ]);

        let mut lower_loss_range = ProfitLossRange::new(None, Some(break_even_points[0]), PZERO)?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        let mut upper_loss_range = ProfitLossRange::new(Some(break_even_points[1]), None, PZERO)?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![lower_loss_range, upper_loss_range])
    }
}

/// A Long Straddle is an options trading strategy that involves simultaneously buying
/// a put and a call option with the same strike price and expiration date. This strategy
/// profits from high volatility, as it makes money when the underlying asset moves
/// significantly in either direction. While the maximum loss is limited to the total
/// premium paid, potential profit is unlimited. The strategy is most effective when
/// expecting a large price movement but uncertain about the direction.
///
/// Key characteristics:
/// - Buy 1 ATM Call
/// - Buy 1 ATM Put
/// - Same strike price
/// - Same expiration date
/// - Maximum loss: Total premium paid
/// - Maximum profit: Unlimited
/// - Break-even points: Strike price +/- total premium paid
/// - Ideal market forecast: High volatility, large price movement
///
const LONG_STRADDLE_DESCRIPTION: &str = "Long Straddle strategy involves simultaneously \
buying a put and a call option with identical strike prices and expiration dates. \
Profits from increased volatility and significant price movements in either direction. \
Maximum loss limited to premium paid with unlimited profit potential. Most effective \
when expecting large price movements but uncertain about direction.";

#[derive(Clone, Debug)]
pub struct LongStraddle {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<PositiveF64>,
    long_call: Position,
    long_put: Position,
}

impl LongStraddle {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: PositiveF64,
        mut strike: PositiveF64,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: PositiveF64,
        premium_long_call: f64,
        premium_long_put: f64,
        open_fee_long_call: f64,
        close_fee_long_call: f64,
        open_fee_long_put: f64,
        close_fee_long_put: f64,
    ) -> Self {
        if strike == PZERO {
            strike = underlying_price;
        }

        let mut strategy = LongStraddle {
            name: "Long Straddle".to_string(),
            kind: StrategyType::Straddle,
            description: LONG_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            strike,
            expiration.clone(),
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

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            strike,
            expiration,
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Put,
            dividend_yield,
            None,
        );
        let long_put = Position::new(
            long_put_option,
            premium_long_put,
            Utc::now(),
            open_fee_long_put,
            close_fee_long_put,
        );
        strategy.add_leg(long_put.clone());

        let net_quantity = (long_call.option.quantity + long_put.option.quantity) / pos!(2.0);

        strategy
            .break_even_points
            .push(strike - strategy.total_cost() / net_quantity);

        strategy
            .break_even_points
            .push(strike + strategy.total_cost() / net_quantity);

        strategy.break_even_points.sort();
        strategy
    }
}

impl Strategies for LongStraddle {
    fn get_underlying_price(&self) -> PositiveF64 {
        self.long_call.option.underlying_price
    }

    fn add_leg(&mut self, position: Position) {
        match position.option.option_style {
            OptionStyle::Call => self.long_call = position,
            OptionStyle::Put => self.long_put = position,
        }
    }

    fn get_legs(&self) -> Vec<Position> {
        vec![self.long_call.clone(), self.long_put.clone()]
    }

    fn max_profit(&self) -> Result<PositiveF64, &str> {
        Ok(f64::INFINITY.into()) // Theoretically unlimited
    }

    fn max_loss(&self) -> Result<PositiveF64, &str> {
        Ok(self.total_cost())
    }

    fn total_cost(&self) -> PositiveF64 {
        pos!(self.long_call.net_cost() + self.long_put.net_cost())
    }

    fn net_premium_received(&self) -> f64 {
        0.0 // Long Straddle doesn't receive premium
    }

    fn fees(&self) -> f64 {
        self.long_call.open_fee
            + self.long_call.close_fee
            + self.long_put.open_fee
            + self.long_put.close_fee
    }

    fn profit_area(&self) -> f64 {
        let strike_diff = self.break_even_points[1] - self.break_even_points[0];
        let cat = (strike_diff / 2.0_f64.sqrt()).value();
        let loss_area = (cat.powf(2.0)) / (2.0 * 10.0_f64.powf(cat.log10().ceil()));
        (1.0 / loss_area) * 10000.0 // Invert the value to get the profit area: the lower, the better
    }

    fn profit_ratio(&self) -> f64 {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        match self.max_loss() {
            Ok(max_loss) => ((break_even_diff / max_loss) * 100.0).value(),
            Err(_) => ZERO,
        }
    }

    fn get_break_even_points(&self) -> Vec<PositiveF64> {
        let mut break_even_points = self.break_even_points.clone();
        break_even_points.sort();
        break_even_points
    }
}

impl Validable for LongStraddle {
    fn validate(&self) -> bool {
        self.long_call.validate()
            && self.long_put.validate()
            && self.long_call.option.strike_price == self.long_put.option.strike_price
    }
}

impl Optimizable for LongStraddle {
    type Strategy = LongStraddle;
    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let options: Vec<&OptionData> = option_chain.options.iter().collect();
        let mut best_value = f64::NEG_INFINITY;

        for call_index in 0..options.len() {
            let call_option = &options[call_index];

            for put_option in &options[..call_index] {
                trace!(
                    "Call: {:#?} Put: {:#?}",
                    call_option.strike_price,
                    put_option.strike_price
                );
                if call_option.strike_price <= put_option.strike_price {
                    error!(
                        "Invalid strike prices Put: {:#?} Call: {:#?} ",
                        put_option.strike_price, call_option.strike_price
                    );
                    continue;
                }

                if !self.is_valid_long_option(put_option, &side)
                    || !self.is_valid_long_option(call_option, &side)
                {
                    error!("Invalid option");
                    continue;
                }

                let legs = StrategyLegs::TwoLegs {
                    first: call_option,
                    second: put_option,
                };

                if !self.are_valid_prices(&legs) {
                    error!(
                        "Invalid Ask prices Put: {:#?} Call: {:#?} ",
                        put_option.put_ask, call_option.call_ask
                    );
                    continue;
                }

                let strategy: LongStraddle = self.create_strategy(option_chain, &legs);

                if !strategy.validate() {
                    error!("Invalid strategy");
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

    fn is_valid_long_option(&self, option: &OptionData, side: &FindOptimalSide) -> bool {
        let underlying_price = match (
            self.long_put.option.underlying_price,
            self.long_call.option.underlying_price,
        ) {
            (PZERO, PZERO) => PZERO,
            (PZERO, call) => call,
            (put, _) => put,
        };
        if underlying_price == PZERO {
            error!("Invalid underlying_price option");
            return false;
        }

        match side {
            FindOptimalSide::Upper => {
                let valid = option.strike_price >= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} <= {}",
                        option.strike_price, underlying_price
                    );
                }
                valid
            }
            FindOptimalSide::Lower => {
                let valid = option.strike_price <= underlying_price;
                if !valid {
                    debug!(
                        "Option is out of range: {} >= {}",
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
                        "Option is out of range: {} >= {} && {} <= {}",
                        option.strike_price, *start, option.strike_price, *end
                    );
                }
                valid
            }
        }
    }

    fn are_valid_prices(&self, legs: &StrategyLegs) -> bool {
        let (call, put) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        call.call_ask.unwrap() > PZERO && put.put_ask.unwrap() > PZERO
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (call, put) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        LongStraddle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            self.long_call.option.expiration_date.clone(),
            call.implied_volatility.unwrap().value() / 100.0,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            call.call_ask.unwrap().value(),
            put.put_ask.unwrap().value(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.long_put.open_fee,
            self.long_put.close_fee,
        )
    }
}

impl Profit for LongStraddle {
    fn calculate_profit_at(&self, price: PositiveF64) -> f64 {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price) + self.long_put.pnl_at_expiration(&price)
    }
}

impl Graph for LongStraddle {
    fn title(&self) -> String {
        let strategy_title = format!("Long {:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.long_call.title(), self.long_put.title()]
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
        let max_value = f64::INFINITY;
        let min_value = f64::NEG_INFINITY;

        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.value(),
            y_range: (min_value, max_value),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (4.0, -50.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();
        let max_loss = self.max_loss().unwrap_or(PZERO);

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].value(), 0.0),
            label: format!("Low Break Even {}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].value(), 0.0),
            label: format!("High Break Even {}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(-60.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.value(),
                -max_loss.value(),
            ),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                max_loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(0.0, -20.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for LongStraddle {
    fn get_expiration(&self) -> Result<ExpirationDate, String> {
        let option = &self.long_call.option;
        Ok(option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let option = &self.long_call.option;
        let break_even_points = &self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(option.implied_volatility),
            pos!(self.long_put.option.implied_volatility),
        ]);

        let mut lower_profit_range = ProfitLossRange::new(None, Some(break_even_points[0]), PZERO)?;

        lower_profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        let mut upper_profit_range = ProfitLossRange::new(Some(break_even_points[1]), None, PZERO)?;

        upper_profit_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![lower_profit_range, upper_profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, String> {
        let option = &self.long_call.option;
        let break_even_points = &self.get_break_even_points();

        let (mean_volatility, std_dev) = mean_and_std(vec![
            pos!(option.implied_volatility),
            pos!(self.long_call.option.implied_volatility),
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            PZERO,
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
mod tests_short_straddle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};

    fn setup() -> ShortStraddle {
        ShortStraddle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(150.0),
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(100.0),
            2.0,
            1.5,
            0.1,
            0.1,
            0.1,
            0.1,
        )
    }

    #[test]
    fn test_atm_strike_initialization() {
        let underlying_price = pos!(150.0);
        let strategy = ShortStraddle::new(
            "AAPL".to_string(),
            underlying_price,
            PZERO,
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(100.0),
            2.0,
            1.5,
            0.1,
            0.1,
            0.1,
            0.1,
        );

        assert_eq!(
            strategy.short_call.option.strike_price, underlying_price,
            "Strike should default to underlying price when PZERO is provided"
        );
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "Short Straddle");
        assert_eq!(strategy.kind, StrategyType::Straddle);
        assert_eq!(
            strategy.description,
            "Short Straddle strategy involves simultaneously selling a put and a call option with \
            identical strike prices and expiration dates. Profits from decreased volatility and \
            time decay, with maximum gain limited to premium received and unlimited potential \
            loss. Most effective in range-bound markets with low volatility expectations."
        );
    }

    #[test]
    fn test_strikes_are_equal() {
        let strategy = setup();
        assert_eq!(
            strategy.short_call.option.strike_price, strategy.short_put.option.strike_price,
            "Call and Put strikes should be equal in a Straddle"
        );
    }

    #[test]
    fn test_validate() {
        let strategy = setup();
        assert!(
            strategy.validate(),
            "Strategy should be valid with equal strikes"
        );

        let valid_strategy = ShortStraddle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0), // Diferente strike
            ExpirationDate::Days(30.0),
            0.2,
            0.01,
            0.02,
            pos!(100.0),
            2.0,
            1.5,
            0.1,
            0.1,
            0.1,
            0.1,
        );
        assert!(valid_strategy.validate());
        assert_eq!(
            valid_strategy.short_call.option.strike_price,
            valid_strategy.short_put.option.strike_price
        );
    }

    #[test]
    fn test_break_even() {
        let strategy = setup();
        assert_eq!(strategy.break_even()[0], 146.9);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 150.0;
        assert_eq!(strategy.calculate_profit_at(pos!(price)), 310.0);
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert_eq!(
            strategy.max_profit().unwrap_or(PZERO),
            strategy.net_premium_received()
        );
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(strategy.max_loss().unwrap_or(PZERO), f64::INFINITY);
    }

    #[test]
    fn test_total_cost() {
        let strategy = setup();
        assert_eq!(
            strategy.total_cost(),
            strategy.short_call.net_cost() + strategy.short_put.net_cost()
        );
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(
            strategy.net_premium_received(),
            strategy.short_call.net_premium_received() + strategy.short_put.net_premium_received()
        );
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        let expected_fees = 0.4;
        assert_eq!(strategy.fees(), expected_fees);
    }

    #[test]
    fn test_area() {
        let strategy = setup();
        assert_eq!(strategy.profit_area(), 0.9609999999999962);
    }

    #[test]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        let data = vec![
            pos!(140.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(values[i], strategy.calculate_profit_at(price));
        }

        let title = strategy.title();
        assert!(title.contains("Short Straddle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]
    fn test_add_leg() {
        let mut strategy = setup();
        let original_call = strategy.short_call.clone();
        let original_put = strategy.short_put.clone();

        // Test adding a new call leg
        strategy.add_leg(original_call.clone());
        assert_eq!(strategy.short_call, original_call);

        // Test adding a new put leg
        strategy.add_leg(original_put.clone());
        assert_eq!(strategy.short_put, original_put);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio = strategy.max_profit().unwrap_or(PZERO) / break_even_diff * 100.0;
        assert_eq!(strategy.profit_ratio(), expected_ratio.value());
    }

    #[test]
    fn test_best_ratio() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_area() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_range_to_show() {
        let strategy = setup();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]
    fn test_is_valid_short_option() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.first().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        // Test FindOptimalSide::Upper
        assert!(strategy.is_valid_short_option(option_data, &FindOptimalSide::Upper));

        // Test FindOptimalSide::Lower
        assert!(!strategy.is_valid_short_option(option_data, &FindOptimalSide::Lower));

        // Test FindOptimalSide::All
        assert!(strategy.is_valid_short_option(option_data, &FindOptimalSide::All));

        // Test FindOptimalSide::Range
        assert!(strategy
            .is_valid_short_option(option_data, &FindOptimalSide::Range(min_strike, max_strike)));
    }

    #[test]
    fn test_are_valid_prices() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.options.last().unwrap();
        let put_option = option_chain.options.first().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        assert!(strategy.are_valid_prices(&legs));

        let mut invalid_call = call_option.clone();
        invalid_call.call_bid = Some(pos!(0.0));

        let legs = StrategyLegs::TwoLegs {
            first: &invalid_call,
            second: put_option,
        };
        assert!(!strategy.are_valid_prices(&legs));
    }

    #[test]
    fn test_create_strategy() {
        let strategy = setup();
        let chain = create_test_option_chain();
        let call_option = chain.options.first().unwrap();
        let put_option = chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());

        let call_option = chain.options.last().unwrap();
        let put_option = chain.options.first().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
    }

    #[test]
    fn test_get_points() {
        let strategy = setup();
        let points = strategy.get_points();

        assert_eq!(points.len(), 4);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].value()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].value()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(1150.0),
            ExpirationDate::Days(30.0),
            spos!(0.2),
            0.01,
            0.02,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(10.0),
            0.00001,
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_long_straddle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};

    fn setup_long_straddle() -> LongStraddle {
        LongStraddle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(150.0),
            ExpirationDate::Days(30.0),
            0.25,
            0.01,
            0.02,
            pos!(10.0),
            5.0,
            5.0,
            0.5,
            0.5,
            0.5,
            0.5,
        )
    }

    #[test]
    fn test_long_straddle_new() {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let call_strike = pos!(160.0);
        let expiration = ExpirationDate::default();
        let implied_volatility = 0.25;
        let risk_free_rate = 0.01;
        let dividend_yield = 0.02;
        let quantity = pos!(10.0);
        let premium_long_call = 5.0;
        let premium_long_put = 5.0;
        let open_fee_long_call = 0.5;
        let close_fee_long_call = 0.5;
        let open_fee_long_put = 0.5;
        let close_fee_long_put = 0.5;

        let strategy = LongStraddle::new(
            underlying_symbol.clone(),
            underlying_price,
            call_strike,
            expiration.clone(),
            implied_volatility,
            risk_free_rate,
            dividend_yield,
            quantity,
            premium_long_call,
            premium_long_put,
            open_fee_long_call,
            close_fee_long_call,
            open_fee_long_put,
            close_fee_long_put,
        );

        assert_eq!(strategy.name, "Long Straddle");
        assert_eq!(strategy.kind, StrategyType::Straddle);
        assert_eq!(strategy.description, LONG_STRADDLE_DESCRIPTION);

        let break_even_points = vec![148.0, 172.0];
        assert_eq!(strategy.break_even_points, break_even_points);
    }

    #[test]
    fn test_break_even() {
        let long_straddle = setup_long_straddle();
        assert_eq!(long_straddle.break_even()[0], 138.0);
    }

    #[test]
    fn test_total_cost() {
        let long_straddle = setup_long_straddle();
        assert_eq!(
            long_straddle.total_cost(),
            long_straddle.long_call.net_cost() + long_straddle.long_put.net_cost()
        );
    }

    #[test]
    fn test_calculate_profit_at() {
        let long_straddle = setup_long_straddle();
        let price = pos!(150.0);
        let expected_profit = long_straddle.long_call.pnl_at_expiration(&Some(price))
            + long_straddle.long_put.pnl_at_expiration(&Some(price));
        assert_eq!(long_straddle.calculate_profit_at(price), expected_profit);
    }

    #[test]
    fn test_new() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.name, "Long Straddle");
        assert_eq!(strategy.kind, StrategyType::Straddle);
        assert_eq!(strategy.description, LONG_STRADDLE_DESCRIPTION);
    }

    #[test]
    fn test_validate() {
        let strategy = setup_long_straddle();
        assert!(strategy.validate());
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.max_profit().unwrap_or(PZERO), f64::INFINITY);
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup_long_straddle();
        assert_eq!(
            strategy.max_loss().unwrap_or(PZERO),
            strategy.total_cost().value()
        );
    }

    #[test]
    fn test_fees() {
        let strategy = setup_long_straddle();
        let expected_fees = 2.0; // 0.5 * 4 fees
        assert_eq!(strategy.fees(), expected_fees);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.net_premium_received(), 0.0);
    }

    #[test]
    fn test_profit_area() {
        let strategy = setup_long_straddle();
        let area = strategy.profit_area();
        assert!(area > 0.0);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup_long_straddle();
        assert_eq!(strategy.profit_ratio(), 20.0);
    }

    #[test]
    fn test_add_leg() {
        let mut strategy = setup_long_straddle();
        let original_call = strategy.long_call.clone();
        let original_put = strategy.long_put.clone();

        strategy.add_leg(original_call.clone());
        assert_eq!(strategy.long_call, original_call);

        strategy.add_leg(original_put.clone());
        assert_eq!(strategy.long_put, original_put);
    }

    #[test]
    fn test_graph_methods() {
        let strategy = setup_long_straddle();

        // Test vertical lines
        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        let data = vec![
            pos!(130.0),
            pos!(140.0),
            pos!(150.0),
            pos!(160.0),
            pos!(170.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(values[i], strategy.calculate_profit_at(price));
        }

        // Test title
        let title = strategy.title();
        assert!(title.contains("Long Straddle Strategy"));
        assert!(title.contains("Call"));
        assert!(title.contains("Put"));
    }

    #[test]
    fn test_best_ratio() {
        let mut strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();

        strategy.best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_area() {
        let mut strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();

        strategy.best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_range_to_show() {
        let strategy = setup_long_straddle();
        let step = pos!(1.0);

        let range = strategy.best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]
    fn test_is_valid_long_option() {
        let strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.last().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::Upper));
        assert!(!strategy.is_valid_long_option(option_data, &FindOptimalSide::Lower));
        assert!(strategy.is_valid_long_option(option_data, &FindOptimalSide::All));
        assert!(strategy
            .is_valid_long_option(option_data, &FindOptimalSide::Range(min_strike, max_strike)));
    }

    #[test]
    fn test_are_valid_prices() {
        let strategy = setup_long_straddle();
        let option_chain = create_test_option_chain();
        let call_option = option_chain.options.first().unwrap();
        let put_option = option_chain.options.last().unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        assert!(strategy.are_valid_prices(&legs));

        let mut invalid_call = call_option.clone();
        invalid_call.call_ask = Some(pos!(0.0));

        let legs = StrategyLegs::TwoLegs {
            first: &invalid_call,
            second: put_option,
        };
        assert!(!strategy.are_valid_prices(&legs));
    }

    #[test]
    fn test_create_strategy() {
        let strategy = setup_long_straddle();
        let chain = create_test_option_chain();
        let call_option = chain.options.first().unwrap();
        let put_option = chain.options.last().unwrap();
        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
    }

    #[test]
    fn test_get_points() {
        let strategy = setup_long_straddle();
        let points = strategy.get_points();

        // Should have 4 points: 2 break-even, 1 max loss, 1 current price
        assert_eq!(points.len(), 4);

        let break_even_points: Vec<f64> = points[0..2].iter().map(|p| p.coordinates.0).collect();
        assert!(break_even_points.contains(&strategy.break_even_points[0].value()));
        assert!(break_even_points.contains(&strategy.break_even_points[1].value()));
    }

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(150.0),
            ExpirationDate::Days(30.0),
            spos!(0.65),
            0.01,
            0.02,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            pos!(5.0),
            0.00001,
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_short_straddle_probability {
    use super::*;
    use crate::model::types::{ExpirationDate, PositiveF64};
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;

    /// Helper function that creates a basic short Straddle for testing purposes
    /// Returns a ShortStraddle instance with predefined test values
    fn create_test_short_straddle() -> ShortStraddle {
        ShortStraddle::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(110.0),                // strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            2.0,                        // premium_short_call
            2.0,                        // premium_short_put
            0.0,                        // open_fee_short_call
            0.0,                        // close_fee_short_call
            0.0,                        // open_fee_short_put
            0.0,                        // close_fee_short_put
        )
    }

    #[test]
    fn test_probability_of_profit_basic() {
        let straddle = create_test_short_straddle();
        let result = straddle.probability_of_profit(None, None);

        assert!(result.is_ok(), "Probability calculation should succeed");
        let prob = result.unwrap();
        assert!(prob > PZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_volatility_adjustment() {
        let straddle = create_test_short_straddle();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        };

        let result = straddle.probability_of_profit(Some(vol_adj), None);

        assert!(
            result.is_ok(),
            "Probability calculation with volatility adjustment should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > PZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_trend() {
        let straddle = create_test_short_straddle();
        let trend = PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        };

        let result = straddle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > PZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_downward_trend() {
        let straddle = create_test_short_straddle();
        let trend = PriceTrend {
            drift_rate: -0.1,
            confidence: 0.90,
        };

        let result = straddle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with downward trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > PZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_get_reference_price() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_underlying_price();

        assert_eq!(
            result,
            pos!(100.0),
            "Reference price should match underlying price"
        );
    }

    #[test]
    fn test_get_expiration() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_expiration();

        assert!(result.is_ok(), "Expiration retrieval should succeed");
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    fn test_get_profit_ranges() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_profit_ranges();

        assert!(result.is_ok(), "Profit ranges calculation should succeed");
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1, "Should have exactly one profit range");

        let range = &ranges[0];
        assert!(range.lower_bound.is_some(), "Lower bound should be defined");
        assert!(range.upper_bound.is_some(), "Upper bound should be defined");
        assert!(range.probability > PZERO, "Probability should be positive");
    }
}

#[cfg(test)]
mod tests_short_straddle_probability_bis {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;

    fn create_test_short_straddle() -> ShortStraddle {
        ShortStraddle::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(110.0),                // strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            2.0,                        // premium_short_call
            2.0,                        // premium_short_put
            0.0,                        // open_fee_short_call
            0.0,                        // close_fee_short_call
            0.0,                        // open_fee_short_put
            0.0,                        // close_fee_short_put
        )
    }

    #[test]
    fn test_get_expiration() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    fn test_get_risk_free_rate() {
        let straddle = create_test_short_straddle();
        assert_eq!(straddle.get_risk_free_rate(), Some(0.05));
    }

    #[test]
    fn test_get_profit_ranges() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Short Straddle has one profit range

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
    }

    #[test]
    fn test_get_loss_ranges() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Short Straddle has two loss ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].lower_bound.is_none()); // First loss range extends to negative infinity
        assert!(ranges[1].upper_bound.is_none()); // Second loss range extends to positive infinity
    }

    #[test]
    fn test_probability_of_profit() {
        let straddle = create_test_short_straddle();
        let result = straddle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let straddle = create_test_short_straddle();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = straddle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_trend() {
        let straddle = create_test_short_straddle();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = straddle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_analyze_probabilities() {
        let straddle = create_test_short_straddle();
        let result = straddle.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > PZERO);
        assert!(analysis.probability_of_max_profit >= PZERO);
        assert!(analysis.probability_of_max_loss >= PZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > PZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let straddle = create_test_short_straddle();
        let result = straddle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= PZERO);
        assert!(max_loss_prob >= PZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_long_straddle_probability {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;

    fn create_test_long_straddle() -> LongStraddle {
        LongStraddle::new(
            "TEST".to_string(),
            pos!(100.0),                // underlying_price
            pos!(110.0),                // strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            pos!(1.0),                  // quantity
            2.0,                        // premium_long_call
            2.0,                        // premium_long_put
            0.0,                        // open_fee_long_call
            0.0,                        // close_fee_long_call
            0.0,                        // open_fee_long_put
            0.0,                        // close_fee_long_put
        )
    }

    #[test]
    fn test_get_expiration() {
        let straddle = create_test_long_straddle();
        let result = straddle.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    fn test_get_risk_free_rate() {
        let straddle = create_test_long_straddle();
        assert_eq!(straddle.get_risk_free_rate(), Some(0.05));
    }

    #[test]
    fn test_get_profit_ranges() {
        let straddle = create_test_long_straddle();
        let result = straddle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Long Straddle has two profit ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].upper_bound.is_some());
        assert!(ranges[1].lower_bound.is_some());
    }

    #[test]
    fn test_get_loss_ranges() {
        let straddle = create_test_long_straddle();
        let result = straddle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Long Straddle has one loss range
        assert!(ranges[0].lower_bound.is_some());
        assert!(ranges[0].upper_bound.is_some());
    }

    #[test]
    fn test_probability_of_profit() {
        let straddle = create_test_long_straddle();
        let result = straddle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let straddle = create_test_long_straddle();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = straddle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_trend() {
        let straddle = create_test_long_straddle();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = straddle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > PZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_expected_value_calculation() {
        let straddle = create_test_long_straddle();
        let result = straddle.expected_value(None, None);

        assert!(result.is_ok());
        let ev = result.unwrap();
        assert!(ev >= PZERO, "Expected value should be non-negative");

        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });
        let result_with_vol = straddle.expected_value(vol_adj, None);
        assert!(result_with_vol.is_ok());
        assert!(result_with_vol.unwrap() >= PZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let straddle = create_test_long_straddle();
        let result = straddle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= PZERO);
        assert!(max_loss_prob >= PZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}
