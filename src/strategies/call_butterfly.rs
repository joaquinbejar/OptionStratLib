/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/
use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN};
use crate::error::position::PositionError;
use crate::error::strategies::{BreakEvenErrorKind, ProfitLossErrorKind, StrategyError};
use crate::error::{GreeksError, ProbabilityError};
use crate::greeks::Greeks;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::model::{Position, ProfitLossRange};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::Options;
use crate::{spos, Positive};
use chrono::Utc;
use num_traits::ToPrimitive;
use plotters::prelude::{ShapeStyle, RED};
use plotters::style::full_palette::ORANGE;
use rust_decimal::Decimal;
use std::error::Error;
use tracing::{error, info};

const RATIO_CALL_SPREAD_DESCRIPTION: &str =
    "A Ratio Call Spread involves buying one call option and selling multiple call options \
    at a higher strike price. This strategy is used when a moderate rise in the underlying \
    asset's price is expected, but with limited upside potential.";

#[derive(Clone, Debug)]
pub struct CallButterfly {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    long_call: Position,
    short_call_low: Position,
    short_call_high: Position,
    underlying_price: Positive,
}

impl CallButterfly {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        long_call_strike: Positive,
        short_call_low_strike: Positive,
        short_call_high_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_short_call_low: Positive,
        premium_short_call_high: Positive,
        open_fee_long: Positive,
        close_fee_long: Positive,
        open_fee_short_low: Positive,
        close_fee_short_low: Positive,
        open_fee_short_high: Positive,
        close_fee_short_high: Positive,
    ) -> Self {
        let mut strategy = CallButterfly {
            name: underlying_symbol.to_string(),
            kind: StrategyType::CallButterfly,
            description: RATIO_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call_low: Position::default(),
            short_call_high: Position::default(),
            underlying_price,
        };
        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_call_strike,
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
            open_fee_long,
            close_fee_long,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid short call");
        strategy.long_call = long_call;

        let short_call_low_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_low_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_low = Position::new(
            short_call_low_option,
            premium_short_call_low,
            Utc::now(),
            open_fee_short_low,
            close_fee_short_low,
        );
        strategy
            .add_position(&short_call_low.clone())
            .expect("Invalid long call itm");
        strategy.short_call_low = short_call_low;

        let short_call_high_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_high_strike,
            expiration.clone(),
            implied_volatility,
            quantity,
            underlying_price,
            risk_free_rate,
            OptionStyle::Call,
            dividend_yield,
            None,
        );
        let short_call_high = Position::new(
            short_call_high_option,
            premium_short_call_high,
            Utc::now(),
            open_fee_short_high,
            close_fee_short_high,
        );
        strategy
            .add_position(&short_call_high.clone())
            .expect("Invalid long call otm");
        strategy.short_call_high = short_call_high;

        // Calculate break-even points
        strategy.break_even_points.push(
            (strategy.long_call.option.strike_price
                - strategy
                    .calculate_profit_at(strategy.long_call.option.strike_price)
                    .unwrap()
                    / quantity)
                .round_to(2),
        );

        strategy.break_even_points.push(
            (strategy.short_call_high.option.strike_price
                + strategy
                    .calculate_profit_at(strategy.short_call_high.option.strike_price)
                    .unwrap()
                    / quantity)
                .round_to(2),
        );

        strategy
    }
}

impl Default for CallButterfly {
    fn default() -> Self {
        CallButterfly::new(
            "".to_string(),
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            ExpirationDate::Days(Positive::ZERO),
            Positive::ZERO,
            Decimal::ZERO,
            Positive::ZERO,
            Positive::ONE,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }
}

impl Positionable for CallButterfly {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.side {
            Side::Short => {
                if position.option.strike_price >= self.long_call.option.strike_price {
                    self.short_call_high = position.clone();
                    Ok(())
                } else {
                    self.short_call_low = position.clone();
                    Ok(())
                }
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.long_call,
            &self.short_call_low,
            &self.short_call_high,
        ])
    }
}

impl Strategies for CallButterfly {
    fn get_underlying_price(&self) -> Positive {
        self.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.calculate_profit_at(self.short_call_high.option.strike_price)?;
        if max_profit > Decimal::ZERO {
            Ok(max_profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY)
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let break_even = self.get_break_even_points()?;
        if break_even.len() != 2 {
            return Err(StrategyError::BreakEvenError(
                BreakEvenErrorKind::NoBreakEvenPoints,
            ));
        }
        let base_low = break_even[1] - break_even[0];
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let base_high =
            self.short_call_high.option.strike_price - self.short_call_low.option.strike_price;
        Ok(((base_low + base_high) * max_profit / 2.0).into())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_loss = match self.max_loss().unwrap() {
            value if value == Positive::ZERO => spos!(1.0),
            value if value == Positive::INFINITY => spos!(1.0),
            value => Some(value),
        };

        match (self.max_profit(), max_loss) {
            (Ok(max_profit), Some(ml)) => Ok((max_profit / ml * 100.0).into()),
            _ => Ok(Decimal::ZERO),
        }
    }

    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }
}

impl Validable for CallButterfly {
    fn validate(&self) -> bool {
        if self.name.is_empty() {
            error!("Symbol is required");
            return false;
        }
        if !self.long_call.validate() {
            return false;
        }
        if !self.short_call_low.validate() {
            return false;
        }
        if !self.short_call_high.validate() {
            return false;
        }
        if self.underlying_price <= Positive::ZERO {
            error!("Underlying price must be greater than zero");
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call_low.option.strike_price {
            error!("Long call strike price must be less than short call strike price");
            return false;
        }
        if self.short_call_low.option.strike_price >= self.short_call_high.option.strike_price {
            error!("Short call low strike price must be less than short call high strike price");
            return false;
        }
        true
    }
}

impl Optimizable for CallButterfly {
    type Strategy = CallButterfly;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_triple_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(long, short_low, short_high)| {
                long.is_valid_optimal_side(underlying_price, &side)
                    && short_low.is_valid_optimal_side(underlying_price, &side)
                    && short_high.is_valid_optimal_side(underlying_price, &side)
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long, short_low, short_high)| {
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_low.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_high.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long, short_low, short_high)| {
                let legs = StrategyLegs::ThreeLegs {
                    first: long,
                    second: short_low,
                    third: short_high,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long, short_low, short_high)| {
                OptionDataGroup::Three(long, short_low, short_high)
            })
    }

    fn find_optimal(
        &mut self,
        option_chain: &OptionChain,
        side: FindOptimalSide,
        criteria: OptimizationCriteria,
    ) {
        let mut best_value = Decimal::MIN;
        let strategy_clone = self.clone();
        let options_iter = strategy_clone.filter_combinations(option_chain, side);

        for option_data_group in options_iter {
            // Unpack the OptionDataGroup into individual options
            let (long, short_low, short_high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::ThreeLegs {
                first: long,
                second: short_low,
                third: short_high,
            };
            let strategy = self.create_strategy(option_chain, &legs);
            // Calculate the current value based on the optimization criteria
            let current_value = match criteria {
                OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.profit_area().unwrap(),
            };

            if current_value > best_value {
                // Update the best value and replace the current strategy
                info!("Found better value: {}", current_value);
                best_value = current_value;
                *self = strategy.clone();
            }
        }
    }

    fn create_strategy(&self, option_chain: &OptionChain, legs: &StrategyLegs) -> CallButterfly {
        let (long_call, short_call_low, short_call_high) = match legs {
            StrategyLegs::ThreeLegs {
                first,
                second,
                third,
            } => (first, second, third),
            _ => panic!("Invalid number of legs for this strategy"),
        };

        if !long_call.validate() || !short_call_low.validate() || !short_call_high.validate() {
            panic!("Invalid options");
        }
        CallButterfly::new(
            option_chain.symbol.clone(),
            option_chain.underlying_price,
            long_call.strike_price,
            short_call_low.strike_price,
            short_call_high.strike_price,
            self.long_call.option.expiration_date.clone(),
            long_call.implied_volatility.unwrap(),
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long_call.call_ask.unwrap(),
            short_call_low.call_bid.unwrap(),
            short_call_high.call_bid.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call_low.open_fee,
            self.short_call_low.close_fee,
            self.short_call_high.open_fee,
            self.short_call_high.close_fee,
        )
    }
}

impl Profit for CallButterfly {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        let long_call_itm_profit = self.long_call.pnl_at_expiration(&price)?;
        let long_call_otm_profit = self.short_call_low.pnl_at_expiration(&price)?;
        let short_call_profit = self.short_call_high.pnl_at_expiration(&price)?;
        Ok(long_call_itm_profit + long_call_otm_profit + short_call_profit)
    }
}

impl Graph for CallButterfly {
    fn title(&self) -> String {
        let strategy_title = format!("Ratio Call Spread Strategy: {:?}", self.kind);
        let long_call_itm_title = self.long_call.title();
        let long_call_otm_title = self.short_call_low.title();
        let short_call_title = self.short_call_high.title();

        format!(
            "{}\n\t{}\n\t{}\n\t{}",
            strategy_title, long_call_itm_title, long_call_otm_title, short_call_title
        )
    }

    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        let vertical_lines = vec![ChartVerticalLine {
            x_coordinate: self.long_call.option.underlying_price.to_f64(),
            y_range: (f64::NEG_INFINITY, f64::INFINITY),
            label: format!(
                "Current Price: {:.2}",
                self.long_call.option.underlying_price
            ),
            label_offset: (-24.0, -1.0),
            line_color: ORANGE,
            label_color: ORANGE,
            line_style: ShapeStyle::from(&ORANGE).stroke_width(2),
            font_size: 18,
        }];

        vertical_lines
    }

    fn get_points(&self) -> Vec<ChartPoint<(f64, f64)>> {
        let mut points: Vec<ChartPoint<(f64, f64)>> = Vec::new();

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Low Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(-55.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("High Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(3.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                self.calculate_profit_at(self.long_call.option.strike_price)
                    .unwrap()
                    .to_f64()
                    .unwrap(),
            ),
            label: format!(
                "Left Loss\n\n{:.2}",
                self.calculate_profit_at(self.long_call.option.strike_price)
                    .unwrap()
            ),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let lower_loss = self
            .calculate_profit_at(self.short_call_low.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        let upper_loss = self
            .calculate_profit_at(self.short_call_high.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();

        points.push(ChartPoint {
            coordinates: (self.short_call_low.option.strike_price.to_f64(), lower_loss),
            label: format!("Left High {:.2}", lower_loss),
            label_offset: LabelOffsetType::Relative(3.0, -3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (
                self.short_call_high.option.strike_price.to_f64(),
                upper_loss,
            ),
            label: format!("Right High {:.2}", upper_loss),
            label_offset: LabelOffsetType::Relative(3.0, 3.0),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for CallButterfly {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.long_call.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call_low.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_points[0]),
            Some(break_even_points[1]),
            Positive::ZERO,
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
        let break_even_points = self.get_break_even_points()?;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call_low.option.implied_volatility,
            self.short_call_high.option.implied_volatility,
        ]);

        let mut loss_range_lower =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        let mut loss_range_upper =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        loss_range_lower.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        loss_range_upper.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            self.get_expiration()?,
            self.get_risk_free_rate(),
        )?;

        Ok(vec![loss_range_lower, loss_range_upper])
    }
}

impl Greeks for CallButterfly {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.long_call.option,
            &self.short_call_low.option,
            &self.short_call_high.option,
        ])
    }
}

impl DeltaNeutrality for CallButterfly {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_itm_delta = self.short_call_low.option.delta();
        let long_call_otm_delta = self.short_call_high.option.delta();
        let short_call_delta = self.long_call.option.delta();
        let threshold = DELTA_THRESHOLD;
        let l_ci_delta = long_call_itm_delta.unwrap();
        let l_co_delta = long_call_otm_delta.unwrap();
        let s_c_delta = short_call_delta.unwrap();

        let delta = l_ci_delta + l_co_delta + s_c_delta;
        DeltaInfo {
            net_delta: delta,
            individual_deltas: vec![l_ci_delta, l_co_delta, s_c_delta],
            is_neutral: (delta).abs() < threshold,
            underlying_price: self.short_call_low.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.short_call_low.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta_low = self.short_call_low.option.delta().unwrap();
        let delta_high = self.short_call_high.option.delta().unwrap();
        let qty_low = Positive((net_delta.abs() / delta_low).abs());
        let qty_high = Positive((net_delta.abs() / delta_high).abs());

        vec![
            DeltaAdjustment::SellOptions {
                quantity: qty_low * self.short_call_low.option.quantity,
                strike: self.short_call_low.option.strike_price,
                option_type: OptionStyle::Call,
            },
            DeltaAdjustment::SellOptions {
                quantity: qty_high * self.short_call_high.option.quantity,
                strike: self.short_call_high.option.strike_price,
                option_type: OptionStyle::Call,
            },
        ]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let delta = self.long_call.option.delta().unwrap();
        let qty = Positive((net_delta.abs() / delta).abs());

        vec![DeltaAdjustment::BuyOptions {
            quantity: qty * self.long_call.option.quantity,
            strike: self.long_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }
}

#[cfg(test)]
mod tests_call_butterfly {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn setup() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
            pos!(157.5),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(45.0),
            pos!(30.0),
            pos!(20.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "AAPL");
        assert_eq!(strategy.kind, StrategyType::CallButterfly);
        assert!(strategy
            .description
            .contains("A Ratio Call Spread involves"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 150.1);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 172.0;
        assert!(strategy.calculate_profit_at(pos!(price)).unwrap() < Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit() {
        let strategy = setup();
        assert!(strategy.max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_relative_eq!(
            strategy.net_premium_received().unwrap().to_f64(),
            4.9,
            epsilon = 0.0001
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let strategy = setup();
        assert_relative_eq!(
            strategy.fees().unwrap().to_f64(),
            0.6,
            epsilon = f64::EPSILON
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_methods() {
        let strategy = setup();

        let vertical_lines = strategy.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].label, "Current Price: 150.00");

        let data = vec![
            pos!(150.0),
            pos!(155.0),
            pos!(160.0),
            pos!(165.0),
            pos!(170.0),
        ];
        let values = strategy.get_values(&data);
        for (i, &price) in data.iter().enumerate() {
            assert_eq!(
                values[i],
                strategy
                    .calculate_profit_at(price)
                    .unwrap()
                    .to_f64()
                    .unwrap()
            );
        }

        let title = strategy.title();
        assert!(title.contains("Ratio Call Spread Strategy"));
        assert!(title.contains("Call"));
    }
}

#[cfg(test)]
mod tests_call_butterfly_validation {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn setup_basic_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(7.0),
            pos!(5.0),
            pos!(3.0),
            pos!(4.0),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_empty_symbol() {
        let mut strategy = setup_basic_strategy();
        strategy.name = "".to_string();
        assert!(!strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_invalid_underlying_price() {
        let mut strategy = setup_basic_strategy();
        strategy.underlying_price = Positive::ZERO;
        assert!(!strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_valid_strategy() {
        let strategy = setup_basic_strategy();
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_call_butterfly_pnl {
    use super::*;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn setup_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(150.0),
            pos!(155.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(7.0),
            pos!(5.0),
            pos!(3.0),
            pos!(4.0),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_below_lower_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(pos!(140.0)).unwrap();
        assert!(profit <= Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_above_upper_strike() {
        let strategy = setup_test_strategy();
        let profit = strategy.calculate_profit_at(pos!(160.0)).unwrap();
        assert!(profit <= Decimal::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio() {
        let strategy = setup_test_strategy();
        let ratio = strategy.profit_ratio().unwrap();
        assert!(ratio > Decimal::ZERO);
    }
}

#[cfg(test)]
mod tests_call_butterfly_graph {
    use super::*;
    use crate::model::types::ExpirationDate;
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn setup_test_strategy() -> CallButterfly {
        CallButterfly::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0),
            pos!(155.0),
            pos!(150.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(7.0),
            pos!(5.0),
            pos!(3.0),
            pos!(4.0),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vertical_lines() {
        let butterfly = setup_test_strategy();
        let lines = butterfly.get_vertical_lines();

        assert_eq!(lines.len(), 1, "Should have exactly one vertical line");

        let line = &lines[0];
        assert_relative_eq!(line.x_coordinate, 150.0, epsilon = 0.001);
        assert_eq!(line.y_range, (f64::NEG_INFINITY, f64::INFINITY));
        assert!(line.label.contains("Current Price: 150.00"));
        assert_eq!(line.font_size, 18);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_points_count_and_labels() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Should have 6 points: 2 break-even, 1 left loss, 2 max profit points, and current price
        assert_eq!(points.len(), 6, "Should have exactly 6 points");

        let labels: Vec<&str> = points.iter().map(|p| p.label.as_str()).collect();

        // Verify all required labels are present
        assert!(labels.iter().any(|&l| l.contains("Low Break Even")));
        assert!(labels.iter().any(|&l| l.contains("High Break Even")));
        assert!(labels.iter().any(|&l| l.contains("Left Loss")));
        assert!(labels.iter().any(|&l| l.contains("Left High")));
        assert!(labels.iter().any(|&l| l.contains("Right High")));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_points_coordinates() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Check for points at strike prices
        let strike_coordinates: Vec<f64> = points.iter().map(|p| p.coordinates.0).collect();

        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 145.0).abs() < 0.001),
            "Missing point near 145.0 strike"
        );
        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 150.0).abs() < 0.001),
            "Missing point near 150.0 strike"
        );
        assert!(
            strike_coordinates
                .iter()
                .any(|&x| (x - 155.0).abs() < 0.001),
            "Missing point near 155.0 strike"
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_point_styling() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        for point in points {
            assert_eq!(point.point_size, 5, "Point size should be 5");
            assert_eq!(point.font_size, 18, "Font size should be 18");

            // Verify label offset is set
            match point.label_offset {
                LabelOffsetType::Relative(x, y) => {
                    assert!(x != 0.0, "X offset should not be 0");
                    assert!(y != 0.0, "Y offset should not be 0");
                }
                _ => panic!("Expected Relative label offset"),
            }
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_points() {
        let butterfly = setup_test_strategy();
        let points = butterfly.get_points();

        // Find break-even points
        let break_even_points: Vec<&ChartPoint<(f64, f64)>> = points
            .iter()
            .filter(|p| p.label.contains("Break Even"))
            .collect();

        assert_eq!(
            break_even_points.len(),
            2,
            "Should have exactly 2 break-even points"
        );

        // Verify break-even points have y-coordinate = 0
        for point in break_even_points {
            assert_relative_eq!(point.coordinates.1, 0.0, epsilon = 0.001);
        }
    }
}

#[cfg(test)]
mod tests_iron_condor_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::call_butterfly::CallButterfly;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5750.0),     // long_strike_itm
            pos!(5850.0),     // long_strike_otm
            pos!(5800.0),     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(95.8),     // short_quantity
            pos!(85.04),    // premium_long_itm
            pos!(31.65),    // premium_long_otm
            pos!(53.04),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5901.88));
        let size = dec!(-0.687410);
        let delta = pos!(0.7040502965074396);
        let k = pos!(5750.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));
        let size = dec!(0.055904);
        let delta1 = pos!(0.28356181440213835);
        let delta2 = pos!(0.1338190182607754);
        let k1 = pos!(5850.0);
        let k2 = pos!(5800.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion_zero = binding.first().unwrap();
        let suggestion_one = binding.last().unwrap();
        match suggestion_zero {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        match suggestion_one {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = delta1;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5794.4));

        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_iron_condor_delta_size {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::call_butterfly::CallButterfly;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            pos!(5750.0),     // long_strike_itm
            pos!(5850.0),     // long_strike_otm
            pos!(5800.0),     // short_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // long quantity
            pos!(97.8),     // short_quantity
            pos!(85.04),    // premium_long_itm
            pos!(31.65),    // premium_long_otm
            pos!(53.04),    // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.73),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(5881.88));
        let size = dec!(-0.5699325);
        let delta = pos!(0.5948524360242063);
        let k = pos!(5750.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(5781.88));
        let size = dec!(0.05590);
        let delta1 = pos!(0.28356181440213835);
        let delta2 = pos!(0.1338190182607754);
        let k1 = pos!(5850.0);
        let k2 = pos!(5800.0);
        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.suggest_delta_adjustments();
        let suggestion_zero = binding.first().unwrap();
        let suggestion_one = binding.last().unwrap();
        match suggestion_zero {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        match suggestion_one {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_type,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_type, OptionStyle::Call);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call_low.option.clone();
        option.quantity = delta1;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(5794.4));

        assert_decimal_eq!(
            strategy.calculate_net_delta().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_call_butterfly_optimizable {
    use super::*;
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-19".to_string(), None, None);

        // Add options with different strikes
        chain.add_option(
            pos!(95.0),      // strike
            spos!(6.0),      // call_bid
            spos!(6.2),      // call_ask
            spos!(1.0),      // put_bid
            spos!(1.2),      // put_ask
            spos!(0.2),      // iv
            Some(dec!(0.4)), // delta
            spos!(100.0),    // volume
            Some(50),        // open interest
        );

        chain.add_option(
            pos!(100.0),
            spos!(3.0),
            spos!(3.2),
            spos!(3.0),
            spos!(3.2),
            spos!(0.2),
            Some(dec!(0.5)),
            spos!(200.0),
            Some(100),
        );

        chain.add_option(
            pos!(105.0),
            spos!(1.0),
            spos!(1.2),
            spos!(6.0),
            spos!(6.2),
            spos!(0.2),
            Some(dec!(0.6)),
            spos!(100.0),
            Some(50),
        );

        chain
    }

    fn setup_test_butterfly() -> CallButterfly {
        CallButterfly::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(95.0),
            pos!(100.0),
            pos!(105.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(1.0),
            pos!(6.2), // long call ask
            pos!(3.0), // short call bid low
            pos!(1.0), // short call bid high
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_ratio() {
        let mut butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        // Verify the optimization resulted in valid strikes
        assert!(
            butterfly.long_call.option.strike_price < butterfly.short_call_low.option.strike_price
        );
        assert!(
            butterfly.short_call_low.option.strike_price
                < butterfly.short_call_high.option.strike_price
        );

        // Verify the strategy is valid
        assert!(butterfly.validate());
        assert!(butterfly.max_profit().is_ok());
        assert!(butterfly.max_loss().is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_area() {
        let mut butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        // Verify the optimization resulted in valid strikes
        assert!(
            butterfly.long_call.option.strike_price < butterfly.short_call_low.option.strike_price
        );
        assert!(
            butterfly.short_call_low.option.strike_price
                < butterfly.short_call_high.option.strike_price
        );

        // Verify the strategy is valid
        assert!(butterfly.validate());
        assert!(butterfly.max_profit().is_ok());
        assert!(butterfly.max_loss().is_ok());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_strategy() {
        let butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        let legs = StrategyLegs::ThreeLegs {
            first: chain.options.iter().next().unwrap(),
            second: chain.options.iter().nth(1).unwrap(),
            third: chain.options.iter().nth(2).unwrap(),
        };

        let new_strategy = butterfly.create_strategy(&chain, &legs);

        // Verify the new strategy has correct properties
        assert_relative_eq!(
            new_strategy.get_underlying_price().to_f64(),
            100.0,
            epsilon = 0.001
        );
        assert!(new_strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    #[should_panic(expected = "Invalid number of legs for this strategy")]
    fn test_create_strategy_invalid_legs() {
        let butterfly = setup_test_butterfly();
        let chain = create_test_option_chain();

        let legs = StrategyLegs::TwoLegs {
            first: chain.options.iter().next().unwrap(),
            second: chain.options.iter().nth(1).unwrap(),
        };

        butterfly.create_strategy(&chain, &legs); // Should panic
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_filter_combinations_empty_chain() {
        let butterfly = setup_test_butterfly();
        let empty_chain =
            OptionChain::new("TEST", pos!(100.0), "2024-12-19".to_string(), None, None);

        let combinations: Vec<_> = butterfly
            .filter_combinations(&empty_chain, FindOptimalSide::All)
            .collect();

        assert!(
            combinations.is_empty(),
            "Empty chain should yield no combinations"
        );
    }
}

#[cfg(test)]
mod tests_call_butterfly_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use crate::{assert_pos_relative_eq, pos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    /// Creates a test Call Butterfly with standard parameters based on SP500
    fn create_test_butterfly() -> CallButterfly {
        CallButterfly::new(
            "SP500".to_string(),
            pos!(5781.88), // underlying_price
            pos!(5750.0),  // long_call_strike
            pos!(5800.0),  // short_call_low_strike
            pos!(5850.0),  // short_call_high_strike
            ExpirationDate::Days(pos!(2.0)),
            pos!(0.18),     // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(3.0),      // long quantity
            pos!(85.04),    // premium_long_itm
            pos!(53.04),    // premium_long_otm
            pos!(28.85),    // premium_short
            pos!(0.78),     // premium_short
            pos!(0.78),     // open_fee_long
            pos!(0.78),     // close_fee_long
            pos!(0.73),     // close_fee_short
            pos!(0.73),     // close_fee_short
            pos!(0.72),     // open_fee_short
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let butterfly = create_test_butterfly();
        let result = butterfly.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 2.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let butterfly = create_test_butterfly();
        assert_eq!(
            butterfly.get_risk_free_rate().unwrap().to_f64().unwrap(),
            0.05
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let butterfly = create_test_butterfly();
        let result = butterfly.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 1);
        let range = &ranges[0];

        // Verify range bounds
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= pos!(1.0));

        // Verify bounds are within strike prices
        assert!(range.lower_bound.unwrap() >= butterfly.long_call.option.strike_price);
        assert!(range.upper_bound.unwrap() >= butterfly.short_call_high.option.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let butterfly = create_test_butterfly();
        let result = butterfly.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 2); // Should have two loss ranges

        // Test lower loss range
        let lower_range = &ranges[0];
        assert!(lower_range.lower_bound.is_none());
        assert!(lower_range.upper_bound.is_some());
        assert!(lower_range.probability > Positive::ZERO);

        // Test upper loss range
        let upper_range = &ranges[1];
        assert!(upper_range.lower_bound.is_some());
        assert!(upper_range.upper_bound.is_none());
        assert!(upper_range.probability > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_sum_to_one() {
        let butterfly = create_test_butterfly();

        let profit_ranges = butterfly.get_profit_ranges().unwrap();
        let loss_ranges = butterfly.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(total_profit_prob + total_loss_prob, pos!(1.0), pos!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_points_validity() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        // Break-even points should be within strike prices
        assert!(break_even_points[0] >= butterfly.long_call.option.strike_price);
        assert!(break_even_points[1] >= butterfly.short_call_high.option.strike_price);
        // Break-even points should be between adjacent strikes
        assert!(break_even_points[0] < butterfly.short_call_low.option.strike_price);
        assert!(break_even_points[1] > butterfly.short_call_low.option.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_volatility_adjustment() {
        let butterfly = create_test_butterfly();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let prob = butterfly.probability_of_profit(vol_adj, None);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_price_trend() {
        let butterfly = create_test_butterfly();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let prob = butterfly.probability_of_profit(None, trend);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_analyze_probabilities() {
        let butterfly = create_test_butterfly();
        let analysis = butterfly.analyze_probabilities(None, None).unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.expected_value > Positive::ZERO);
        assert_eq!(analysis.break_even_points.len(), 2);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_near_expiration() {
        let mut butterfly = create_test_butterfly();
        butterfly.long_call.option.expiration_date = ExpirationDate::Days(pos!(0.5));
        butterfly.short_call_low.option.expiration_date = ExpirationDate::Days(pos!(0.5));
        butterfly.short_call_high.option.expiration_date = ExpirationDate::Days(pos!(0.5));

        let prob = butterfly.probability_of_profit(None, None).unwrap();
        // Near expiration probabilities should be more extreme
        assert!(prob < pos!(0.3) || prob > pos!(0.7));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_high_volatility_scenario() {
        let mut butterfly = create_test_butterfly();
        butterfly.long_call.option.implied_volatility = pos!(0.5);
        butterfly.short_call_low.option.implied_volatility = pos!(0.5);
        butterfly.short_call_high.option.implied_volatility = pos!(0.5);

        let analysis = butterfly.analyze_probabilities(None, None).unwrap();
        // Higher volatility should increase potential profit/loss ranges
        assert!(analysis.expected_value > pos!(10.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_extreme_probabilities() {
        let butterfly = create_test_butterfly();
        let result = butterfly.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();

        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}
