/*
Bull Call Spread Strategy

A bull call spread involves buying a call option with a lower strike price and selling a call option with a higher strike price,
both with the same expiration date. This strategy is used when a moderate rise in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (difference between strikes minus net debit)
- Limited risk (net debit paid)
- Bullish strategy that profits from price increase
- Both options have same expiration date
- Lower cost than buying calls outright
- Lower risk than naked call buying
- Maximum profit achieved when price rises above higher strike
- Also known as a vertical call debit spread
*/

use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::probability::ProbabilityError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::greeks::equations::{Greek, Greeks};
use crate::model::option::Options;
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::{d2fu, f2p, Positive};
use chrono::Utc;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use tracing::{debug, error, info};

const BULL_CALL_SPREAD_DESCRIPTION: &str =
    "A bull call spread is created by buying a call option with a lower strike price \
    and simultaneously selling a call option with a higher strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate increase in the underlying \
    asset's price. The maximum profit is limited to the difference between strike prices minus \
    the net debit paid, while the maximum loss is limited to the net debit paid.";

#[derive(Clone, Debug)]
pub struct BullCallSpread {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    long_call: Position,
    short_call: Position,
}

impl BullCallSpread {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut long_strike: Positive,
        mut short_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: Positive,
        premium_long_call: f64,
        premium_short_call: f64,
        open_fee_long_call: f64,
        close_fee_long_call: f64,
        open_fee_short_call: f64,
        close_fee_short_call: f64,
    ) -> Self {
        if long_strike == Positive::ZERO {
            long_strike = underlying_price;
        }
        if short_strike == Positive::ZERO {
            short_strike = underlying_price;
        }

        let mut strategy = BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: BULL_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike,
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
        strategy
            .add_position(&long_call.clone())
            .expect("Failed to add long call");

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            short_strike,
            expiration,
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
        strategy
            .add_position(&short_call.clone())
            .expect("Failed to add short call");

        strategy.validate();

        // Calculate break-even point
        strategy
            .break_even_points
            .push(long_strike - strategy.net_premium_received() / quantity);

        strategy
    }
}

impl Positionable for BullCallSpread {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.side {
            Side::Short => {
                self.short_call = position.clone();
                Ok(())
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_call, &self.short_call])
    }
}

impl Strategies for BullCallSpread {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(self.short_call.option.strike_price);
        if profit >= ZERO {
            Ok(f2p!(profit))
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(self.long_call.option.strike_price);
        if loss <= ZERO {
            Ok(f2p!(loss.abs()))
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        }
    }

    fn total_cost(&self) -> Positive {
        f2p!(self.long_call.net_cost() + self.short_call.net_cost())
    }

    fn net_premium_received(&self) -> f64 {
        self.short_call.net_premium_received() - self.long_call.net_cost()
    }

    fn fees(&self) -> f64 {
        self.long_call.open_fee
            + self.long_call.close_fee
            + self.short_call.open_fee
            + self.short_call.close_fee
    }

    fn profit_area(&self) -> f64 {
        let high = self.max_profit().unwrap_or(Positive::ZERO);
        let base = self.short_call.option.strike_price - self.break_even_points[0];
        (high * base / 200.0).to_f64()
    }

    fn profit_ratio(&self) -> f64 {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => ZERO,
            (_, value) if value == Positive::ZERO => f64::INFINITY,
            _ => (max_profit / max_loss * 100.0).into(),
        }
    }

    fn get_break_even_points(&self) -> Vec<Positive> {
        self.break_even_points.clone()
    }
}

impl Validable for BullCallSpread {
    fn validate(&self) -> bool {
        if !self.long_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        if !self.short_call.validate() {
            debug!("Short call is invalid");
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call.option.strike_price {
            error!(
                "Long call strike price {} must be lower than short call strike price {}",
                self.long_call.option.strike_price, self.short_call.option.strike_price
            );
            return false;
        }

        true
    }
}

impl Optimizable for BullCallSpread {
    type Strategy = BullCallSpread;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_double_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |&option| {
                option.0.is_valid_optimal_side(underlying_price, &side)
                    && option.1.is_valid_optimal_side(underlying_price, &side)
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long, short)| {
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long, short)| {
                let legs = StrategyLegs::TwoLegs {
                    first: long,
                    second: short,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long, short)| OptionDataGroup::Two(long, short))
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = f64::NEG_INFINITY;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (long, short) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: long,
                second: short,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio(),
                OptimizationCriteria::Area => strategy.profit_area(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        BullCallSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_call.option.expiration_date.clone(),
            long.implied_volatility.unwrap().to_f64() / 100.0,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long.call_ask.unwrap().to_f64(),
            short.call_bid.unwrap().to_f64(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
        )
    }
}

impl Profit for BullCallSpread {
    fn calculate_profit_at(&self, price: Positive) -> f64 {
        let price = Some(price);
        self.long_call.pnl_at_expiration(&price) + self.short_call.pnl_at_expiration(&price)
    }
}

impl Graph for BullCallSpread {
    fn title(&self) -> String {
        format!(
            "{} Strategy:\n\t{}\n\t{}",
            self.name,
            self.long_call.title(),
            self.short_call.title()
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let underlying_price = self.long_call.option.underlying_price.to_f64();
        vec![ChartVerticalLine {
            x_coordinate: underlying_price,
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!("Current Price: {:.2}", underlying_price),
            label_offset: (4.0, 0.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }]
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points = Vec::new();

        // Break Even Point
        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Break Even {:.2}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(10.0, -10.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Profit Point (at higher strike price)
        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.to_f64(),
                self.max_profit().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!("Max Profit {:.2}", self.max_profit().unwrap_or(Positive::ZERO)),
            label_offset: LabelOffsetType::Relative(10.0, 10.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        // Maximum Loss Point (at lower strike price)
        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -self.max_loss().unwrap_or(Positive::ZERO).to_f64(),
            ),
            label: format!("Max Loss -{:.2}", self.max_loss().unwrap_or(Positive::ZERO)),
            label_offset: LabelOffsetType::Relative(-120.0, -10.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        // Current Price Point
        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for BullCallSpread {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.long_call.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<f64> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            f2p!(self.long_call.option.implied_volatility),
            f2p!(self.short_call.option.implied_volatility),
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_point),
            Some(self.short_call.option.strike_price),
            f2p!(self.max_profit()?.to_f64()),
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

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            f2p!(self.long_call.option.implied_volatility),
            f2p!(self.short_call.option.implied_volatility),
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(self.long_call.option.strike_price),
            Some(break_even_point),
            f2p!(self.max_loss()?.to_f64()),
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

impl Greeks for BullCallSpread {
    fn greeks(&self) -> Greek {
        let long_call_greek = self.long_call.greeks();
        let short_call_greek = self.short_call.greeks();

        Greek {
            delta: long_call_greek.delta + short_call_greek.delta,
            gamma: long_call_greek.gamma + short_call_greek.gamma,
            theta: long_call_greek.theta + short_call_greek.theta,
            vega: long_call_greek.vega + short_call_greek.vega,
            rho: long_call_greek.rho + short_call_greek.rho,
            rho_d: long_call_greek.rho_d + short_call_greek.rho_d,
        }
    }
}

impl DeltaNeutrality for BullCallSpread {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_delta = self.long_call.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let threshold = DELTA_THRESHOLD;

        let s_c_delta = d2fu!(short_call_delta.unwrap()).unwrap();
        let l_c_delta = d2fu!(long_call_delta.unwrap()).unwrap();
        DeltaInfo {
            net_delta: l_c_delta + s_c_delta,
            individual_deltas: vec![l_c_delta, s_c_delta],
            is_neutral: (l_c_delta + s_c_delta).abs() < threshold,
            underlying_price: self.long_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta = d2fu!(self.short_call.option.delta().unwrap()).unwrap();
        vec![DeltaAdjustment::SellOptions {
            quantity: f2p!((net_delta.abs() / delta).abs()) * self.short_call.option.quantity,
            strike: self.short_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta = d2fu!(self.long_call.option.delta().unwrap()).unwrap();

        vec![DeltaAdjustment::BuyOptions {
            quantity: f2p!((net_delta.abs() / delta).abs()) * self.long_call.option.quantity,
            strike: self.long_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }
}

#[cfg(test)]
mod tests_bull_call_spread_strategy {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::f2p;

    fn create_test_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),                // underlying_price
            f2p!(95.0),                 // long_strike
            f2p!(100.0),                // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            f2p!(1.0),                  // quantity
            2.0,                        // premium_long_call
            1.0,                        // premium_short_call
            0.0,                        // fees
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_new_bull_call_spread() {
        let spread = create_test_spread();

        assert_eq!(spread.name, "Bull Call Spread");
        assert_eq!(spread.kind, StrategyType::BullCallSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(spread.get_underlying_price(), f2p!(100.0));
        assert_eq!(spread.long_call.option.strike_price, f2p!(95.0));
        assert_eq!(spread.short_call.option.strike_price, f2p!(100.0));
    }

    #[test]
    fn test_add_leg() {
        let mut spread = create_test_spread();
        let new_long_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                f2p!(90.0),
                ExpirationDate::Days(30.0),
                0.20,
                f2p!(1.0),
                f2p!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            1.5,
            Utc::now(),
            0.0,
            0.0,
        );

        spread
            .add_position(&new_long_call.clone())
            .expect("Failed to add long call");
        assert_eq!(spread.long_call.option.strike_price, f2p!(90.0));
    }

    #[test]
    fn test_get_legs() {
        let spread = create_test_spread();
        let legs = spread.get_positions().expect("Failed to get positions");

        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].option.side, Side::Long);
        assert_eq!(legs[1].option.side, Side::Short);
        assert_eq!(legs[0].option.option_style, OptionStyle::Call);
        assert_eq!(legs[1].option.option_style, OptionStyle::Call);
    }

    #[test]
    fn test_max_profit() {
        let spread = create_test_spread();
        let max_profit = spread.max_profit().unwrap();
        assert_eq!(max_profit, f2p!(4.0));
    }

    #[test]
    fn test_max_loss() {
        let spread = create_test_spread();
        let max_loss = spread.max_loss().unwrap();
        assert_eq!(max_loss, f2p!(1.0));
    }

    #[test]
    fn test_total_cost() {
        let spread = create_test_spread();
        assert_eq!(spread.total_cost(), f2p!(3.0));
    }

    #[test]
    fn test_fees() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            2.0,
            1.0,
            0.5, // open_fee_long_call
            0.5, // close_fee_long_call
            0.5, // open_fee_short_call
            0.5, // close_fee_short_call
        );

        assert_eq!(spread.fees(), 2.0);
    }

    #[test]
    fn test_break_even_points() {
        let spread = create_test_spread();
        let break_even_points = spread.get_break_even_points();

        assert_eq!(break_even_points.len(), 1);
        // Break-even = long strike + net debit = 95 + 1 = 96
        assert_eq!(break_even_points[0], f2p!(96.0));
    }

    #[test]
    fn test_profit_area() {
        let spread = create_test_spread();
        let area = spread.profit_area();
        assert_eq!(area, 0.08);
    }

    #[test]
    fn test_profit_ratio() {
        let spread = create_test_spread();
        let ratio = spread.profit_ratio();

        // Ratio = (max_profit / max_loss) * 100
        // = (4.0 / 1.0) * 100 = 400
        assert_eq!(ratio, 400.0);
    }

    #[test]
    fn test_default_strikes() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            Positive::ZERO, // long_strike = default
            Positive::ZERO, // short_strike = default
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        assert_eq!(spread.long_call.option.strike_price, f2p!(100.0));
        assert_eq!(spread.short_call.option.strike_price, f2p!(100.0));
    }

    #[test]
    fn test_invalid_strikes() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(100.0),
            f2p!(95.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            2.0,
            1.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        assert!(!spread.validate());
    }
}

#[cfg(test)]
mod tests_bull_call_spread_validation {
    use super::*;
    use crate::model::types::ExpirationDate;
    use chrono::Utc;

    fn create_valid_position(
        side: Side,
        strike_price: Positive,
        expiration: ExpirationDate,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                expiration,
                0.20,
                f2p!(1.0),
                f2p!(100.0),
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
    fn test_valid_bull_call_spread() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, f2p!(95.0), ExpirationDate::Days(30.0)),
            short_call: create_valid_position(Side::Short, f2p!(100.0), ExpirationDate::Days(30.0)),
        };

        assert!(spread.validate(), "Valid spread should pass validation");
    }

    #[test]
    fn test_invalid_long_call() {
        let mut invalid_long =
            create_valid_position(Side::Long, f2p!(95.0), ExpirationDate::Days(30.0));
        invalid_long.option.quantity = Positive::ZERO;

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: invalid_long,
            short_call: create_valid_position(Side::Short, f2p!(100.0), ExpirationDate::Days(30.0)),
        };

        assert!(
            !spread.validate(),
            "Spread with invalid long call should fail validation"
        );
    }

    #[test]
    fn test_invalid_short_call() {
        let mut invalid_short =
            create_valid_position(Side::Short, f2p!(100.0), ExpirationDate::Days(30.0));
        invalid_short.option.quantity = Positive::ZERO;

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, f2p!(95.0), ExpirationDate::Days(30.0)),
            short_call: invalid_short,
        };

        assert!(
            !spread.validate(),
            "Spread with invalid short call should fail validation"
        );
    }

    #[test]
    fn test_invalid_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, f2p!(100.0), ExpirationDate::Days(30.0)),
            short_call: create_valid_position(Side::Short, f2p!(95.0), ExpirationDate::Days(30.0)),
        };

        assert!(
            !spread.validate(),
            "Spread with long strike price >= short strike price should fail validation"
        );
    }

    #[test]
    fn test_equal_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, f2p!(100.0), ExpirationDate::Days(30.0)),
            short_call: create_valid_position(Side::Short, f2p!(100.0), ExpirationDate::Days(30.0)),
        };

        assert!(
            !spread.validate(),
            "Spread with equal strike prices should fail validation"
        );
    }

    #[test]
    fn test_different_expiration_dates_same_day() {
        let date1 = ExpirationDate::DateTime(Utc::now() + chrono::Duration::days(30));
        let date2 = ExpirationDate::Days(30.0);

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, f2p!(95.0), date1),
            short_call: create_valid_position(Side::Short, f2p!(100.0), date2),
        };

        assert!(
            spread.validate(),
            "Spread with different ExpirationDate types but same date should pass validation"
        );
    }

    #[test]
    fn test_boundary_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, f2p!(94.99), ExpirationDate::Days(30.0)),
            short_call: create_valid_position(Side::Short, f2p!(95.0), ExpirationDate::Days(30.0)),
        };

        assert!(
            spread.validate(),
            "Spread with very close but valid strike prices should pass validation"
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_optimization {
    use super::*;
    use crate::chains::chain::OptionData;
    use crate::model::types::ExpirationDate;
    use crate::spos;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-12-31".to_string(), None, None);

        chain.add_option(
            f2p!(85.0),   // strike
            spos!(16.0),  // call_bid
            spos!(16.2),  // call_ask
            None,         // put_bid
            None,         // put_ask
            spos!(0.2),   // implied_volatility
            Some(0.8),    // delta
            spos!(100.0), // volume
            Some(50),     // open_interest
        );

        chain.add_option(
            f2p!(90.0),
            spos!(11.5),
            spos!(11.7),
            None,
            None,
            spos!(0.2),
            Some(0.7),
            spos!(150.0),
            Some(75),
        );

        chain.add_option(
            f2p!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            spos!(0.2),
            Some(0.6),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            f2p!(100.0),
            spos!(3.5),
            spos!(3.7),
            None,
            None,
            spos!(0.2),
            Some(0.5),
            spos!(250.0),
            Some(125),
        );

        chain.add_option(
            f2p!(105.0),
            spos!(1.0),
            spos!(1.2),
            None,
            None,
            spos!(0.2),
            Some(0.4),
            spos!(300.0),
            Some(150),
        );

        chain
    }

    fn create_base_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            7.2, // premium_long_call
            3.5, // premium_short_call
            0.0,
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.profit_ratio() > 0.0,
            "Profit ratio should be positive"
        );
    }

    #[test]
    fn test_find_optimal_area() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(spread.profit_area() > 0.0, "Profit area should be positive");
    }

    #[test]
    fn test_find_optimal_upper_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        assert!(spread.short_call.option.strike_price >= chain.underlying_price);
        assert!(spread.long_call.option.strike_price >= chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_lower_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Lower, OptimizationCriteria::Ratio);

        assert!(spread.short_call.option.strike_price <= chain.underlying_price);
        assert!(spread.long_call.option.strike_price <= chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_range() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(
            &chain,
            FindOptimalSide::Range(f2p!(90.0), f2p!(100.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(spread.short_call.option.strike_price <= f2p!(100.0));
        assert!(spread.short_call.option.strike_price >= f2p!(90.0));
        assert!(spread.long_call.option.strike_price <= f2p!(100.0));
        assert!(spread.long_call.option.strike_price >= f2p!(90.0));
    }

    #[test]
    fn test_is_valid_long_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            f2p!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            spos!(0.2),
            Some(0.6),
            spos!(100.0),
            Some(50),
        );

        assert!(spread.is_valid_long_option(&option, &FindOptimalSide::All));
        assert!(spread.is_valid_long_option(&option, &FindOptimalSide::Lower));
        assert!(!spread.is_valid_long_option(&option, &FindOptimalSide::Upper));
        assert!(
            spread.is_valid_long_option(&option, &FindOptimalSide::Range(f2p!(90.0), f2p!(100.0)))
        );
    }

    #[test]
    fn test_is_valid_short_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            f2p!(105.0),
            spos!(1.0),
            spos!(1.2),
            None,
            None,
            spos!(0.2),
            Some(0.4),
            spos!(100.0),
            Some(50),
        );

        assert!(spread.is_valid_short_option(&option, &FindOptimalSide::All));
        assert!(!spread.is_valid_short_option(&option, &FindOptimalSide::Lower));
        assert!(spread.is_valid_short_option(&option, &FindOptimalSide::Upper));
        assert!(!spread
            .is_valid_short_option(&option, &FindOptimalSide::Range(f2p!(90.0), f2p!(100.0))));
    }

    #[test]
    fn test_are_valid_prices() {
        let spread = create_base_spread();
        let long_option = OptionData::new(
            f2p!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            spos!(0.2),
            Some(0.6),
            spos!(100.0),
            Some(50),
        );
        let short_option = OptionData::new(
            f2p!(100.0),
            spos!(3.5),
            spos!(3.7),
            None,
            None,
            spos!(0.2),
            Some(0.5),
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::TwoLegs {
            first: &long_option,
            second: &short_option,
        };
        assert!(spread.are_valid_prices(&legs));
    }

    #[test]
    fn test_invalid_prices() {
        let spread = create_base_spread();
        let long_option = OptionData::new(
            f2p!(95.0),
            spos!(7.2),
            spos!(0.0),
            None,
            None,
            spos!(0.2),
            Some(0.6),
            spos!(100.0),
            Some(50),
        );
        let short_option = OptionData::new(
            f2p!(100.0),
            spos!(0.0),
            spos!(3.5),
            None,
            None,
            spos!(0.2),
            Some(0.5),
            spos!(100.0),
            Some(50),
        );

        let legs = StrategyLegs::TwoLegs {
            first: &long_option,
            second: &short_option,
        };
        assert!(!spread.are_valid_prices(&legs));
    }

    #[test]
    fn test_create_strategy() {
        let spread = create_base_spread();
        let chain = create_test_chain();
        let long_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == f2p!(95.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == f2p!(100.0))
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs);

        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_call.option.strike_price, f2p!(95.0));
        assert_eq!(new_strategy.short_call.option.strike_price, f2p!(100.0));
        assert_eq!(
            new_strategy.long_call.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(
            new_strategy.short_call.option.option_style,
            OptionStyle::Call
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_profit {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::f2p;

    fn create_test_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),                // underlying_price
            f2p!(95.0),                 // long_strike
            f2p!(100.0),                // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            f2p!(1.0),                  // quantity
            4.0,                        // premium_long_call
            2.0,                        // premium_short_call
            0.0,                        // fees
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_profit_below_long_strike() {
        let spread = create_test_spread();
        let price = f2p!(90.0);
        assert_eq!(spread.calculate_profit_at(price), -2.0);
    }

    #[test]
    fn test_profit_at_long_strike() {
        let spread = create_test_spread();
        let price = f2p!(95.0);
        assert_eq!(spread.calculate_profit_at(price), -2.0);
    }

    #[test]
    fn test_profit_between_strikes() {
        let spread = create_test_spread();
        let price = f2p!(97.5);
        assert_eq!(spread.calculate_profit_at(price), 0.5);
    }

    #[test]
    fn test_profit_at_break_even() {
        let spread = create_test_spread();
        let price = f2p!(97.0);
        assert!(spread.calculate_profit_at(price).abs() < 0.001);
    }

    #[test]
    fn test_profit_at_short_strike() {
        let spread = create_test_spread();
        let price = f2p!(100.0);
        assert_eq!(spread.calculate_profit_at(price), 3.0);
    }

    #[test]
    fn test_profit_above_short_strike() {
        let spread = create_test_spread();
        let price = f2p!(105.0);
        assert_eq!(spread.calculate_profit_at(price), 3.0);
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(2.0),
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let price = f2p!(105.0);
        assert_eq!(spread.calculate_profit_at(price), 6.0);
    }

    #[test]
    fn test_profit_with_fees() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            4.0,
            2.0,
            0.5, // open_fee_long_call
            0.5, // close_fee_long_call
            0.5, // open_fee_short_call
            0.5, // close_fee_short_call
        );

        let price = f2p!(105.0);
        assert_eq!(spread.calculate_profit_at(price), 1.0);
    }

    #[test]
    fn test_maximum_profit() {
        let spread = create_test_spread();
        let price = f2p!(150.0);
        assert_eq!(spread.calculate_profit_at(price), 3.0);
    }

    #[test]
    fn test_maximum_loss() {
        let spread = create_test_spread();
        let price = f2p!(50.0);
        assert_eq!(spread.calculate_profit_at(price), -2.0);
    }
}

#[cfg(test)]
mod tests_bull_call_spread_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::f2p;

    fn create_test_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),                // underlying_price
            f2p!(95.0),                 // long_strike
            f2p!(100.0),                // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            f2p!(1.0),                  // quantity
            4.0,                        // premium_long_call
            2.0,                        // premium_short_call
            0.0,                        // fees
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_title_format() {
        let spread = create_test_spread();
        let title = spread.title();

        assert!(title.contains("Bull Call Spread Strategy"));
        assert!(title.contains("Long"));
        assert!(title.contains("Short"));
        assert!(title.contains("TEST")); // symbol
        assert!(title.contains("95")); // long strike
        assert!(title.contains("100")); // short strike
    }

    #[test]
    fn test_get_vertical_lines() {
        let spread = create_test_spread();
        let lines = spread.get_vertical_lines();

        assert_eq!(lines.len(), 1);

        let line = &lines[0];
        assert_eq!(line.x_coordinate, 100.0);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price"));
        assert!(line.label.contains("100.00"));
        assert_eq!(line.label_offset, (4.0, 0.0));
        assert_eq!(line.line_color, ORANGE);
        assert_eq!(line.label_color, ORANGE);
        assert_eq!(line.font_size, 18);
    }

    #[test]
    fn test_get_points() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert_eq!(points.len(), 4);

        let break_even = &points[0];
        assert_eq!(break_even.coordinates.1, 0.0);
        assert_eq!(break_even.coordinates.0, 97.0);
        assert!(break_even.label.contains("Break Even"));
        assert_eq!(break_even.point_color, DARK_BLUE);
        assert_eq!(break_even.label_color, DARK_BLUE);
        assert_eq!(break_even.point_size, 5);
        assert_eq!(break_even.font_size, 18);

        let max_profit = &points[1];
        assert_eq!(max_profit.coordinates.0, 100.0);
        assert_eq!(max_profit.coordinates.1, 3.0);
        assert!(max_profit.label.contains("Max Profit"));
        assert_eq!(max_profit.point_color, DARK_GREEN);
        assert_eq!(max_profit.label_color, DARK_GREEN);

        let max_loss = &points[2];
        assert_eq!(max_loss.coordinates.0, 95.0);
        assert_eq!(max_loss.coordinates.1, -2.0);
        assert!(max_loss.label.contains("Max Loss"));
        assert_eq!(max_loss.point_color, RED);
        assert_eq!(max_loss.label_color, RED);
    }

    #[test]
    fn test_points_coordinates() {
        let spread = create_test_spread();
        let points = spread.get_points();

        // Break-even point
        assert_eq!(points[0].coordinates.0, 97.0);
        assert_eq!(points[0].coordinates.1, 0.0);

        // Maximum profit point
        assert_eq!(points[1].coordinates.0, 100.0);
        assert_eq!(points[1].coordinates.1, 3.0);

        // Maximum loss point
        assert_eq!(points[2].coordinates.0, 95.0);
        assert_eq!(points[2].coordinates.1, -2.0);

        // Current price point
        assert_eq!(points[3].coordinates.0, 100.0);
        let current_profit = spread.calculate_profit_at(f2p!(100.0));
        assert_eq!(points[3].coordinates.1, current_profit);
    }

    #[test]
    fn test_point_labels() {
        let spread = create_test_spread();
        let points = spread.get_points();

        assert!(points[0].label.contains("97.00")); // Break-even
        assert!(points[1].label.contains("3.00")); // Max profit
        assert!(points[2].label.contains("2.00")); // Max loss
        assert!(points[3].label.contains("3.00")); // Current price
    }

    #[test]
    fn test_points_style() {
        let spread = create_test_spread();
        let points = spread.get_points();

        for point in points.iter() {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert!(matches!(point.point_color, DARK_BLUE | DARK_GREEN | RED));
            assert_eq!(point.point_color, point.label_color);
        }
    }

    #[test]
    fn test_graph_with_zero_profits() {
        let mut spread = create_test_spread();
        spread.short_call.premium = 1.0;
        spread.long_call.premium = 6.0;

        let points = spread.get_points();
        let max_profit_point = &points[1];

        assert_eq!(max_profit_point.coordinates.1, 0.0);
        assert!(max_profit_point.label.contains("0.00"));
    }

    #[test]
    fn test_graph_with_different_quantities() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(2.0), // quantity = 2
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let points = spread.get_points();
        let max_profit_point = &points[1];
        let max_loss_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 6.0); // 2 * 3.0
        assert_eq!(max_loss_point.coordinates.1, -4.0); // 2 * -2.0
    }

    #[test]
    fn test_graph_at_extremes() {
        let spread = create_test_spread();
        let profit_at_zero = spread.calculate_profit_at(f2p!(0.0));
        let profit_at_high = spread.calculate_profit_at(f2p!(1000.0));

        assert_eq!(profit_at_zero, -2.0);
        assert_eq!(profit_at_high, 3.0);
    }
}

#[cfg(test)]
mod tests_bull_call_spread_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;

    fn create_test_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),                // underlying_price
            f2p!(95.0),                 // long_strike
            f2p!(100.0),                // short_strike
            ExpirationDate::Days(30.0), // expiration
            0.20,                       // implied_volatility
            0.05,                       // risk_free_rate
            0.0,                        // dividend_yield
            f2p!(1.0),                  // quantity
            4.0,                        // premium_long_call
            2.0,                        // premium_short_call
            0.0,                        // fees
            0.0,
            0.0,
            0.0,
        )
    }

    #[test]
    fn test_get_expiration() {
        let spread = create_test_spread();
        let result = spread.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 30.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    fn test_get_risk_free_rate() {
        let spread = create_test_spread();
        assert_eq!(spread.get_risk_free_rate(), Some(0.05));
    }

    #[test]
    fn test_get_profit_ranges() {
        let spread = create_test_spread();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.lower_bound.unwrap(), f2p!(97.0)); // Break-even
        assert_eq!(range.upper_bound.unwrap(), f2p!(100.0)); // Short strike
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_get_loss_ranges() {
        let spread = create_test_spread();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.lower_bound.unwrap(), f2p!(95.0)); // Long strike
        assert_eq!(range.upper_bound.unwrap(), f2p!(97.0)); // Break-even
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_probability_of_profit() {
        let spread = create_test_spread();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= f2p!(1.0));
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let spread = create_test_spread();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: f2p!(0.25),
            std_dev_adjustment: f2p!(0.05),
        });

        let result = spread.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= f2p!(1.0));
    }

    #[test]
    fn test_probability_with_uptrend() {
        let spread = create_test_spread();
        let trend = Some(PriceTrend {
            drift_rate: 0.8,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob < f2p!(0.5));
        assert!(prob <= f2p!(1.0));
    }

    #[test]
    fn test_probability_with_downtrend() {
        let spread = create_test_spread();
        let trend = Some(PriceTrend {
            drift_rate: -0.1,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob < f2p!(0.5));
        assert!(prob > Positive::ZERO);
    }

    #[test]
    fn test_analyze_probabilities() {
        let spread = create_test_spread();
        let result = spread.analyze_probabilities(None, None);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert!(analysis.expected_value > Positive::ZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let spread = create_test_spread();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= f2p!(1.0));
    }

    #[test]
    fn test_probability_near_expiration() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(1.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());
        let prob = result.unwrap();

        assert!(prob < f2p!(0.5));
    }

    #[test]
    fn test_probability_with_high_volatility() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(95.0),
            f2p!(100.0),
            ExpirationDate::Days(30.0),
            0.50, // Alta volatilidad
            0.05,
            0.0,
            f2p!(1.0),
            4.0,
            2.0,
            0.0,
            0.0,
            0.0,
            0.0,
        );

        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());
        let prob = result.unwrap();

        assert!(prob < f2p!(0.3));
        assert!(prob < f2p!(0.7));
    }
}

#[cfg(test)]
mod tests_delta {
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bull_call_spread::BullCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{d2fu, f2p, Positive};
    use approx::assert_relative_eq;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullCallSpread {
        let underlying_price = f2p!(5781.88);
        BullCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(2.0),
            0.18,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // long quantity
            85.04,     // premium_long
            29.85,     // premium_short
            0.78,      // open_fee_long
            0.78,      // open_fee_long
            0.73,      // close_fee_long
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(f2p!(5750.0), f2p!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.3502030,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(1.0922693934308985),
                strike: f2p!(5820.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = f2p!(1.0922693934308985);
        let delta = d2fu!(option.delta().unwrap()).unwrap();

        assert_relative_eq!(delta, -0.35020, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(f2p!(5850.0), f2p!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.1234671,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.626251716937553),
                strike: f2p!(5850.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = f2p!(0.626251716937553);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.123467, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(f2p!(5820.0), f2p!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_delta_size {
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::bull_call_spread::BullCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{d2fu, f2p, Positive};
    use approx::assert_relative_eq;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullCallSpread {
        let underlying_price = f2p!(5781.88);
        BullCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(2.0),
            0.18,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(2.0), // long quantity
            85.04,     // premium_long
            29.85,     // premium_short
            0.78,      // open_fee_long
            0.78,      // open_fee_long
            0.73,      // close_fee_long
            0.73,      // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(f2p!(5750.0), f2p!(5820.9));
        
        let size = 0.7086;
        let delta = f2p!(2.239306943523854);
        
        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: delta,
                strike: f2p!(5820.9),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, -size, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(f2p!(5850.0), f2p!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.246934,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(1.252503433875106),
                strike: f2p!(5850.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = f2p!(1.252503433875106);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.24693, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(f2p!(5820.0), f2p!(5820.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}
