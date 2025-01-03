/*
Iron Condor Strategy

An iron condor involves selling an out-of-the-money put spread and an out-of-the-money call spread with the same expiration date.
This strategy is used when little volatility in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential
- Limited risk
- Profit is highest when the underlying asset price remains between the two sold options at expiration
*/
use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::OptionChain;
use crate::chains::utils::OptionDataGroup;
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::greeks::equations::{Greek, Greeks};
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::Options;
use crate::{d2fu, f2p, Positive};
use chrono::Utc;
use num_traits::{FromPrimitive, ToPrimitive};
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use tracing::{error, info};

const IRON_CONDOR_DESCRIPTION: &str =
    "An Iron Condor is a neutral options strategy combining a bull put spread with a bear call spread. \
    It involves selling an out-of-the-money put and call while buying further out-of-the-money put and call options. \
    This strategy is used when low volatility is expected and the underlying asset's price is anticipated to remain \
    within a specific range.";

#[derive(Clone, Debug)]
pub struct IronCondor {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub break_even_points: Vec<Positive>,
    short_call: Position,
    short_put: Position,
    long_call: Position,
    long_put: Position,
}

impl IronCondor {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        short_call_strike: Positive,
        short_put_strike: Positive,
        long_call_strike: Positive,
        long_put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: f64,
        risk_free_rate: f64,
        dividend_yield: f64,
        quantity: Positive,
        premium_short_call: f64,
        premium_short_put: f64,
        premium_long_call: f64,
        premium_long_put: f64,
        open_fee: f64,
        close_fee: f64,
    ) -> Self {
        let mut strategy = IronCondor {
            name: "Iron Condor".to_string(),
            kind: StrategyType::IronCondor,
            description: IRON_CONDOR_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
            long_call: Position::default(),
            long_put: Position::default(),
        };

        // Short Call
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_call_strike,
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
            open_fee,
            close_fee,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call");

        // Short Put
        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_put_strike,
            expiration.clone(),
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
            open_fee,
            close_fee,
        );
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid short put");

        // Long Call
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
            open_fee,
            close_fee,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid long call");

        // Long Put
        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol,
            long_put_strike,
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
            open_fee,
            close_fee,
        );
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid long put");

        // Calculate break-even points
        let net_credit = (strategy.long_put.premium + strategy.long_call.premium)
            + strategy.fees().unwrap().to_f64().unwrap()
            - (strategy.short_put.premium + strategy.short_call.premium);
        strategy
            .break_even_points
            .push(short_put_strike + net_credit);
        strategy
            .break_even_points
            .push(short_call_strike - net_credit);

        strategy
    }
}

impl Validable for IronCondor {
    fn validate(&self) -> bool {
        let order = self.long_put.option.strike_price <= self.short_put.option.strike_price
            && self.short_put.option.strike_price <= self.short_call.option.strike_price
            && self.short_call.option.strike_price <= self.long_call.option.strike_price;

        if !order {
            error!("Invalid order of strikes");
        }

        self.short_call.validate()
            && self.short_put.validate()
            && self.long_call.validate()
            && self.long_put.validate()
            && order
    }
}

impl Positionable for IronCondor {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (
            position.option.option_style.clone(),
            position.option.side.clone(),
        ) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
                Ok(())
            }
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![
            &self.short_call,
            &self.short_put,
            &self.long_call,
            &self.long_put,
        ])
    }
}

impl Strategies for IronCondor {
    fn get_underlying_price(&self) -> Positive {
        self.long_put.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let left_profit = self.calculate_profit_at(self.short_call.option.strike_price);
        let right_profit = self.calculate_profit_at(self.short_put.option.strike_price);
        if left_profit < ZERO || right_profit < ZERO {
            return Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ));
        }

        Ok(f2p!(
            self.calculate_profit_at(self.short_call.option.strike_price)
        ))
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let left_loss = self.calculate_profit_at(self.long_put.option.strike_price);
        let right_loss = self.calculate_profit_at(self.long_call.option.strike_price);
        if left_loss > ZERO || right_loss > ZERO {
            return Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ));
        }
        Ok(f2p!(left_loss.abs().max(right_loss.abs())))
    }

    fn total_cost(&self) -> Positive {
        f2p!(
            self.short_call.net_cost()
                + self.short_put.net_cost()
                + self.long_call.net_cost()
                + self.long_put.net_cost()
        )
    }

    fn net_premium_received(&self) -> Result<Decimal, StrategyError> {
        let result = self.short_call.net_premium_received() + self.short_put.net_premium_received()
            - self.long_call.total_cost()
            - self.long_put.total_cost();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn fees(&self) -> Result<Decimal, StrategyError> {
        let restul = self.short_call.open_fee
            + self.short_call.close_fee
            + self.short_put.open_fee
            + self.short_put.close_fee
            + self.long_call.open_fee
            + self.long_call.close_fee
            + self.long_put.open_fee
            + self.long_put.close_fee;
        Ok(Decimal::from_f64(restul).unwrap())
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let inner_width =
            (self.short_call.option.strike_price - self.short_put.option.strike_price).to_f64();
        let outer_width =
            (self.long_call.option.strike_price - self.long_put.option.strike_price).to_f64();
        let height = self.max_profit().unwrap_or(Positive::ZERO);

        let inner_area = inner_width * height;
        let outer_triangles = (outer_width - inner_width) * height / 2.0;

        let result =
            (inner_area + outer_triangles) / self.short_call.option.underlying_price.to_f64();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
    }

    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }
}

impl Optimizable for IronCondor {
    type Strategy = IronCondor;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_quad_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |(long_put, short_put, short_call, long_call)| {
                long_put.is_valid_optimal_side(underlying_price, &side)
                    && short_put.is_valid_optimal_side(underlying_price, &side)
                    && short_call.is_valid_optimal_side(underlying_price, &side)
                    && long_call.is_valid_optimal_side(underlying_price, &side)
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long_put, short_put, short_call, long_call)| {
                long_put.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_put.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short_call.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && long_call.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long_put, short_put, short_call, long_call)| {
                let legs = StrategyLegs::FourLegs {
                    first: long_put,
                    second: short_put,
                    third: short_call,
                    fourth: long_call,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate() && strategy.max_profit().is_ok() && strategy.max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long_put, short_put, short_call, long_call)| {
                OptionDataGroup::Four(long_put, short_put, short_call, long_call)
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
            let (long_put, short_put, short_call, long_call) = match option_data_group {
                OptionDataGroup::Four(first, second, third, fourth) => {
                    (first, second, third, fourth)
                }
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::FourLegs {
                first: long_put,
                second: short_put,
                third: short_call,
                fourth: long_call,
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

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        match legs {
            StrategyLegs::FourLegs {
                first: long_put,
                second: short_put,
                third: short_call,
                fourth: long_call,
            } => IronCondor::new(
                chain.symbol.clone(),
                chain.underlying_price,
                short_call.strike_price,
                short_put.strike_price,
                long_call.strike_price,
                long_put.strike_price,
                self.short_call.option.expiration_date.clone(),
                short_put.implied_volatility.unwrap().to_f64() / 100.0,
                self.short_call.option.risk_free_rate,
                self.short_call.option.dividend_yield,
                self.short_call.option.quantity,
                short_call.call_bid.unwrap().to_f64(),
                short_put.put_bid.unwrap().to_f64(),
                long_call.call_ask.unwrap().to_f64(),
                long_put.put_ask.unwrap().to_f64(),
                self.fees().unwrap().to_f64().unwrap() / 8.0,
                self.fees().unwrap().to_f64().unwrap() / 8.0,
            ),
            _ => panic!("Invalid number of legs for Iron Condor strategy"),
        }
    }
}

impl Profit for IronCondor {
    fn calculate_profit_at(&self, price: Positive) -> f64 {
        let price = Some(price);
        self.short_call.pnl_at_expiration(&price)
            + self.short_put.pnl_at_expiration(&price)
            + self.long_call.pnl_at_expiration(&price)
            + self.long_put.pnl_at_expiration(&price)
    }
}

impl Graph for IronCondor {
    fn title(&self) -> String {
        let strategy_title = format!(
            "{:?} Strategy on {} Size {}:",
            self.kind, self.short_put.option.underlying_symbol, self.short_put.option.quantity
        );
        let leg_titles: Vec<String> = [
            format!("Long Put: ${}", self.long_put.option.strike_price),
            format!("Short Put: ${}", self.short_put.option.strike_price),
            format!("Short Call: ${}", self.short_call.option.strike_price),
            format!("Long Call: ${}", self.long_call.option.strike_price),
            format!(
                "Expire: {}",
                self.short_put.option.expiration_date.get_date_string()
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
            x_coordinate: self.short_call.option.underlying_price.to_f64(),
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
        let max_profit = self.max_profit().unwrap_or(Positive::ZERO);
        let short_call_strike_price = &self.short_call.option.strike_price;
        let short_put_strike_price = &self.short_put.option.strike_price;
        let long_call_strike_price = &self.long_call.option.strike_price;
        let long_put_strike_price = &self.long_put.option.strike_price;
        let current_price = &self.short_call.option.underlying_price;

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Left Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(5.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        points.push(ChartPoint {
            coordinates: (self.break_even_points[1].to_f64(), 0.0),
            label: format!("Right Break Even\n\n{}", self.break_even_points[1]),
            label_offset: LabelOffsetType::Relative(5.0, 5.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            short_call_strike_price.to_f64() / 2000.0,
            max_profit.to_f64() / 5.0,
        );
        points.push(ChartPoint {
            coordinates: (short_call_strike_price.to_f64(), max_profit.to_f64()),
            label: format!(
                "High Max Profit {:.2} at {:.0}",
                max_profit, short_call_strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordiantes.0, coordiantes.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let coordinates: (f64, f64) = (
            -short_put_strike_price.to_f64() / 35.0,
            max_profit.to_f64() / 5.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_put.option.strike_price.to_f64(),
                max_profit.to_f64(),
            ),
            label: format!(
                "Low Max Profit {:.2} at {:.0}",
                max_profit, self.short_put.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let loss = self.calculate_profit_at(*long_call_strike_price);
        let coordinates: (f64, f64) = (-short_put_strike_price.to_f64() / 35.0, loss / 50.0);
        points.push(ChartPoint {
            coordinates: (self.long_call.option.strike_price.to_f64(), loss),
            label: format!(
                "Right Max Loss {:.2} at {:.0}",
                loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        let loss = self.calculate_profit_at(*long_put_strike_price);

        let coordinates: (f64, f64) = (long_put_strike_price.to_f64() / 2000.0, loss / 50.0);
        points.push(ChartPoint {
            coordinates: (long_put_strike_price.to_f64(), loss),
            label: format!("Left Max Loss {:.2} at {:.0}", loss, long_put_strike_price),
            label_offset: LabelOffsetType::Relative(coordinates.0, coordinates.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(*current_price));

        points
    }
}

impl Greeks for IronCondor {
    fn greeks(&self) -> Greek {
        let long_call_greek = self.long_call.greeks();
        let long_put_greek = self.long_put.greeks();
        let short_call_greek = self.short_call.greeks();
        let short_put_greek = self.short_put.greeks();

        Greek {
            delta: long_call_greek.delta
                + long_put_greek.delta
                + short_call_greek.delta
                + short_put_greek.delta,
            gamma: long_call_greek.gamma
                + long_put_greek.gamma
                + short_call_greek.gamma
                + short_put_greek.gamma,
            theta: long_call_greek.theta
                + long_put_greek.theta
                + short_call_greek.theta
                + short_put_greek.theta,
            vega: long_call_greek.vega
                + long_put_greek.vega
                + short_call_greek.vega
                + short_put_greek.vega,
            rho: long_call_greek.rho
                + long_put_greek.rho
                + short_call_greek.rho
                + short_put_greek.rho,
            rho_d: long_call_greek.rho_d
                + long_put_greek.rho_d
                + short_call_greek.rho_d
                + short_put_greek.rho_d,
        }
    }
}

impl DeltaNeutrality for IronCondor {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_delta = self.long_call.option.delta();
        let long_put_delta = self.long_put.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let short_put_delta = self.short_put.option.delta();
        let threshold = DELTA_THRESHOLD;
        let l_c_delta = d2fu!(long_call_delta.unwrap()).unwrap();
        let l_p_delta = d2fu!(long_put_delta.unwrap()).unwrap();
        let s_c_delta = d2fu!(short_call_delta.unwrap()).unwrap();
        let s_p_delta = d2fu!(short_put_delta.unwrap()).unwrap();

        let delta = l_c_delta + l_p_delta + s_c_delta + s_p_delta;
        DeltaInfo {
            net_delta: delta,
            individual_deltas: vec![l_c_delta, l_p_delta, s_c_delta, s_p_delta],
            is_neutral: (delta).abs() < threshold,
            underlying_price: self.long_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let l_p_delta = d2fu!(self.long_put.option.delta().unwrap()).unwrap();
        let s_c_delta = d2fu!(self.short_call.option.delta().unwrap()).unwrap();
        vec![
            DeltaAdjustment::BuyOptions {
                quantity: f2p!((net_delta.abs() / l_p_delta).abs()) * self.long_put.option.quantity,
                strike: self.long_put.option.strike_price,
                option_type: OptionStyle::Put,
            },
            DeltaAdjustment::SellOptions {
                quantity: f2p!((net_delta.abs() / s_c_delta).abs())
                    * self.short_call.option.quantity,
                strike: self.short_call.option.strike_price,
                option_type: OptionStyle::Call,
            },
        ]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let s_p_delta = d2fu!(self.short_put.option.delta().unwrap()).unwrap();
        let l_c_delta = d2fu!(self.long_call.option.delta().unwrap()).unwrap();
        vec![
            DeltaAdjustment::BuyOptions {
                quantity: f2p!((net_delta.abs() / l_c_delta).abs())
                    * self.long_call.option.quantity,
                strike: self.long_call.option.strike_price,
                option_type: OptionStyle::Call,
            },
            DeltaAdjustment::SellOptions {
                quantity: f2p!((net_delta.abs() / s_p_delta).abs())
                    * self.short_put.option.quantity,
                strike: self.short_put.option.strike_price,
                option_type: OptionStyle::Put,
            },
        ]
    }
}

#[cfg(test)]
mod tests_iron_condor {
    use super::*;
    use crate::f2p;
    use chrono::{TimeZone, Utc};

    #[test]
    fn test_iron_condor_creation() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            f2p!(150.0),
            f2p!(155.0),
            f2p!(145.0),
            f2p!(160.0),
            f2p!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(iron_condor.name, "Iron Condor");
        assert_eq!(iron_condor.description, IRON_CONDOR_DESCRIPTION.to_string());
        assert_eq!(iron_condor.kind, StrategyType::IronCondor);
        assert_eq!(iron_condor.break_even_points.len(), 2);
        assert_eq!(iron_condor.short_call.option.strike_price, 155.0);
        assert_eq!(iron_condor.short_put.option.strike_price, 145.0);
        assert_eq!(iron_condor.long_call.option.strike_price, 160.0);
        assert_eq!(iron_condor.long_put.option.strike_price, 140.0);
    }

    #[test]
    fn test_max_loss() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            f2p!(150.0),
            f2p!(120.0),
            f2p!(110.0),
            f2p!(130.0),
            f2p!(100.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(iron_condor.max_loss().unwrap_or(Positive::ZERO), 51.3);
    }

    #[test]
    fn test_max_profit() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            f2p!(150.0),
            f2p!(155.0),
            f2p!(145.0),
            f2p!(160.0),
            f2p!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            3.5,
            3.3,
            3.0,
            2.8,
            0.07,
            0.07,
        );

        let expected_profit = iron_condor
            .net_premium_received()
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(
            iron_condor.max_profit().unwrap_or(Positive::ZERO),
            expected_profit
        );
    }

    #[test]
    fn test_get_break_even_points() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            f2p!(150.0),
            f2p!(155.0),
            f2p!(145.0),
            f2p!(160.0),
            f2p!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        assert_eq!(
            iron_condor.get_break_even_points().unwrap()[0],
            iron_condor.break_even_points[0]
        );
    }

    #[test]
    fn test_fees() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            f2p!(150.0),
            f2p!(155.0),
            f2p!(145.0),
            f2p!(160.0),
            f2p!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        let expected_fees = iron_condor.short_call.open_fee
            + iron_condor.short_call.close_fee
            + iron_condor.short_put.open_fee
            + iron_condor.short_put.close_fee
            + iron_condor.long_call.open_fee
            + iron_condor.long_call.close_fee
            + iron_condor.long_put.open_fee
            + iron_condor.long_put.close_fee;
        assert_eq!(iron_condor.fees().unwrap().to_f64().unwrap(), expected_fees);
    }

    #[test]
    fn test_calculate_profit_at() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            f2p!(150.0),
            f2p!(155.0),
            f2p!(145.0),
            f2p!(160.0),
            f2p!(140.0),
            ExpirationDate::DateTime(date),
            0.2,
            0.01,
            0.02,
            Positive::ONE,
            1.5,
            1.0,
            2.0,
            1.8,
            5.0,
            5.0,
        );

        let price = f2p!(150.0);
        let expected_profit = iron_condor.short_call.pnl_at_expiration(&Some(price))
            + iron_condor.short_put.pnl_at_expiration(&Some(price))
            + iron_condor.long_call.pnl_at_expiration(&Some(price))
            + iron_condor.long_put.pnl_at_expiration(&Some(price));
        assert_eq!(iron_condor.calculate_profit_at(price), expected_profit);
    }
}

#[cfg(test)]
mod tests_iron_condor_validable {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_valid_position(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        quantity: Positive,
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
                f2p!(100.0),
                0.05,
                option_style,
                0.0,
                None,
            ),
            1.0,
            Utc::now(),
            0.0,
            0.0,
        )
    }

    fn create_valid_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        )
    }

    #[test]
    fn test_validate_valid_condor() {
        let condor = create_valid_condor();
        assert!(condor.validate());
    }

    #[test]
    fn test_validate_invalid_short_call() {
        let mut condor = create_valid_condor();
        // Make short call invalid by setting quantity to zero
        condor.short_call =
            create_valid_position(Side::Short, OptionStyle::Call, f2p!(105.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_invalid_short_put() {
        let mut condor = create_valid_condor();
        // Make short put invalid by setting quantity to zero
        condor.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, f2p!(95.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_invalid_long_call() {
        let mut condor = create_valid_condor();
        // Make long call invalid by setting quantity to zero
        condor.long_call =
            create_valid_position(Side::Long, OptionStyle::Call, f2p!(110.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_invalid_long_put() {
        let mut condor = create_valid_condor();
        // Make long put invalid by setting quantity to zero
        condor.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, f2p!(90.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_all_invalid() {
        let mut condor = create_valid_condor();
        // Make all positions invalid
        condor.short_call =
            create_valid_position(Side::Short, OptionStyle::Call, f2p!(105.0), Positive::ZERO);
        condor.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, f2p!(95.0), Positive::ZERO);
        condor.long_call =
            create_valid_position(Side::Long, OptionStyle::Call, f2p!(110.0), Positive::ZERO);
        condor.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, f2p!(90.0), Positive::ZERO);
        assert!(!condor.validate());
    }
}

#[cfg(test)]
mod tests_iron_condor_strategies {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.5,       // open_fee
            0.5,       // closing fee
        )
    }

    #[test]
    fn test_add_leg() {
        let mut condor = create_test_condor();

        // Test adding a short call
        let new_short_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                "TEST".to_string(),
                f2p!(106.0),
                ExpirationDate::Days(30.0),
                0.20,
                f2p!(1.0),
                f2p!(100.0),
                0.05,
                OptionStyle::Call,
                0.0,
                None,
            ),
            2.5,
            Utc::now(),
            0.5,
            0.5,
        );
        condor
            .add_position(&new_short_call.clone())
            .expect("Invalid short call");
        assert_eq!(condor.short_call.option.strike_price, f2p!(106.0));

        // Test adding a long put
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                f2p!(89.0),
                ExpirationDate::Days(30.0),
                0.20,
                f2p!(1.0),
                f2p!(100.0),
                0.05,
                OptionStyle::Put,
                0.0,
                None,
            ),
            1.5,
            Utc::now(),
            0.5,
            0.5,
        );
        condor
            .add_position(&new_long_put.clone())
            .expect("Invalid long put");
        assert_eq!(condor.long_put.option.strike_price, f2p!(89.0));
    }

    #[test]
    fn test_get_legs() {
        let condor = create_test_condor();
        let legs = condor.get_positions().expect("Invalid legs");

        assert_eq!(legs.len(), 4);
        assert_eq!(legs[0].option.option_style, OptionStyle::Call);
        assert_eq!(legs[0].option.side, Side::Short);
        assert_eq!(legs[1].option.option_style, OptionStyle::Put);
        assert_eq!(legs[1].option.side, Side::Short);
        assert_eq!(legs[2].option.option_style, OptionStyle::Call);
        assert_eq!(legs[2].option.side, Side::Long);
        assert_eq!(legs[3].option.option_style, OptionStyle::Put);
        assert_eq!(legs[3].option.side, Side::Long);
    }

    #[test]
    fn test_get_break_even_points() {
        let condor = create_test_condor();
        let break_even_points = condor.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        assert!(break_even_points[0] > condor.short_put.option.strike_price);
        assert!(break_even_points[1] < condor.short_call.option.strike_price);
    }

    #[test]
    fn test_max_profit() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            10.0,      // premium_short_put
            10.0,      // premium_long_call
            10.0,      // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        );
        let max_profit = condor.max_profit().unwrap();
        assert_eq!(max_profit, f2p!(ZERO));
    }

    #[test]
    fn test_max_profit_bis() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            20.0,      // premium_short_call
            20.0,      // premium_short_put
            10.0,      // premium_long_call
            10.0,      // premium_long_put
            0.09,      // open_fee
            0.09,      // closing fee
        );
        let max_profit = condor.max_profit().unwrap();
        assert_eq!(max_profit, f2p!(19.28));
    }

    #[test]
    fn test_max_loss() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            10.0,      // premium_short_put
            11.1,      // premium_long_call
            11.1,      // premium_long_put
            0.1,       // open_fee
            0.1,       // closing fee
        );
        let max_loss = condor.max_loss().unwrap();
        assert_eq!(max_loss, f2p!(7.9999999999999964));
    }

    #[test]
    fn test_max_loss_with_uneven_wings() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(105.0),
            f2p!(95.0),
            f2p!(115.0), // Wider call wing
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            2.0,
            2.0,
            1.0,
            1.0,
            0.5,
            0.5,
        );

        let max_loss = condor.max_loss().unwrap();
        assert_eq!(max_loss, f2p!(12.0));
    }

    #[test]
    fn test_total_cost() {
        let condor = create_test_condor();
        // Total cost = 2.0 + 2.0 + 1.0 + 1.0 = 6.0
        assert_eq!(condor.total_cost(), f2p!(6.0));
    }

    #[test]
    fn test_net_premium_received() {
        let condor = create_test_condor();
        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            -2.0
        );
    }

    #[test]
    fn test_net_premium_received_bis_i() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            10.0,      // premium_short_put
            10.0,      // premium_long_call
            10.0,      // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        );
        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            ZERO
        );
    }

    #[test]
    fn test_net_premium_received_bis_ii() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            10.0,      // premium_short_put
            10.0,      // premium_long_call
            10.0,      // premium_long_put
            1.0,       // open_fee
            1.0,       // closing fee
        );
        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            -8.0
        );
    }

    #[test]
    fn test_net_premium_received_bis_iii() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            20.0,      // premium_short_put
            20.0,      // premium_long_call
            10.0,      // premium_long_put
            1.0,       // open_fee
            1.0,       // closing fee
        );
        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            -8.0
        );
    }

    #[test]
    fn test_net_premium_received_bis_iv() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            20.0,      // premium_short_put
            10.0,      // premium_long_call
            10.0,      // premium_long_put
            1.0,       // open_fee
            1.0,       // closing fee
        );
        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            2.0
        );
    }

    #[test]
    fn test_net_premium_received_bis_v() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            10.0,      // premium_short_call
            10.0,      // premium_short_put
            10.0,      // premium_long_call
            20.0,      // premium_long_put
            1.0,       // open_fee
            1.0,       // closing fee
        );
        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            -18.0
        );
    }

    #[test]
    fn test_fees() {
        let condor = create_test_condor();
        // Total fees = (0.5 + 0.5) * 4 = 4.0
        assert_eq!(condor.fees().unwrap().to_f64().unwrap(), 4.0);
    }

    #[test]
    fn test_profit_area() {
        let condor = create_test_condor();
        assert_eq!(condor.profit_area().unwrap().to_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_best_range_to_show() {
        let condor = create_test_condor();
        let range = condor.best_range_to_show(f2p!(1.0)).unwrap();

        assert!(!range.is_empty());
        assert!(range[0] < condor.long_put.option.strike_price);
        assert!(range[range.len() - 1] > condor.long_call.option.strike_price);
    }

    #[test]
    fn test_with_multiple_contracts() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(105.0),
            f2p!(95.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(2.0), // quantity = 2
            2.0,
            2.0,
            1.0,
            1.0,
            0.5,
            0.5,
        );

        assert!(condor.max_profit().is_err());
        assert_eq!(condor.max_loss().unwrap(), f2p!(14.0));
    }

    #[test]
    fn test_with_no_premium() {
        let mut condor = create_test_condor();
        condor.short_call.premium = 1.0;
        condor.short_put.premium = 1.0;
        condor.long_call.premium = 1.0;
        condor.long_put.premium = 1.0;

        assert_eq!(
            condor.net_premium_received().unwrap().to_f64().unwrap(),
            -4.0
        );
        assert!(condor.max_profit().is_err());
    }
}

#[cfg(test)]
mod tests_iron_condor_optimizable {
    use super::*;
    use crate::chains::chain::OptionData;
    use crate::f2p;
    use crate::model::types::ExpirationDate;
    use crate::spos;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.5,       // open_fee
            0.5,       // closing fee
        )
    }

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", f2p!(100.0), "2024-12-31".to_string(), None, None);

        // Add options at various strikes
        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                f2p!(strike),
                spos!(5.0),   // call_bid
                spos!(5.2),   // call_ask
                spos!(5.0),   // put_bid
                spos!(5.2),   // put_ask
                spos!(0.2),   // implied_volatility
                None,         // delta
                spos!(100.0), // volume
                Some(50),     // open_interest
            );
        }
        chain
    }

    #[test]
    fn test_find_optimal_lower_side() {
        let mut condor = create_test_condor();
        let chain = create_test_chain();

        condor.find_optimal(&chain, FindOptimalSide::Lower, OptimizationCriteria::Ratio);

        assert!(condor.validate());
        assert!(condor.long_put.option.strike_price <= chain.underlying_price);
        assert!(condor.short_put.option.strike_price <= chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_upper_side() {
        let mut condor = create_test_condor();
        let chain = create_test_chain();

        condor.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        assert!(condor.validate());
        assert!(condor.short_call.option.strike_price >= chain.underlying_price);
        assert!(condor.long_call.option.strike_price >= chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_range() {
        let mut condor = create_test_condor();
        let chain = create_test_chain();

        condor.find_optimal(
            &chain,
            FindOptimalSide::Range(f2p!(95.0), f2p!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(condor.validate());
        assert!(condor.short_put.option.strike_price >= f2p!(95.0));
        assert!(condor.short_call.option.strike_price <= f2p!(105.0));
    }

    #[test]
    fn test_find_optimal_by_area() {
        let mut condor = create_test_condor();
        let chain = create_test_chain();

        let initial_area = condor.profit_area().unwrap();
        condor.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(condor.validate());
        assert!(condor.profit_area().unwrap() >= initial_area);
    }

    #[test]
    fn test_is_valid_long_option() {
        let condor = create_test_condor();
        let option = OptionData::new(
            f2p!(90.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        // Test with different sides
        assert!(condor.is_valid_long_option(&option, &FindOptimalSide::All));
        assert!(condor.is_valid_long_option(&option, &FindOptimalSide::Lower));
        assert!(!condor.is_valid_long_option(&option, &FindOptimalSide::Upper));
        assert!(
            condor.is_valid_long_option(&option, &FindOptimalSide::Range(f2p!(85.0), f2p!(95.0)))
        );
    }

    #[test]
    fn test_is_valid_short_option() {
        let condor = create_test_condor();
        let option = OptionData::new(
            f2p!(105.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            spos!(0.2),
            None,
            spos!(100.0),
            Some(50),
        );

        // Test with different sides
        assert!(condor.is_valid_short_option(&option, &FindOptimalSide::All));
        assert!(!condor.is_valid_short_option(&option, &FindOptimalSide::Lower));
        assert!(condor.is_valid_short_option(&option, &FindOptimalSide::Upper));
        assert!(condor
            .is_valid_short_option(&option, &FindOptimalSide::Range(f2p!(100.0), f2p!(110.0))));
    }

    #[test]
    fn test_create_strategy() {
        let condor = create_test_condor();
        let chain = create_test_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::FourLegs {
            first: options[1],  // 90.0 strike for long put
            second: options[2], // 95.0 strike for short put
            third: options[4],  // 105.0 strike for short call
            fourth: options[5], // 110.0 strike for long call
        };

        let new_strategy = condor.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
        assert_eq!(new_strategy.long_put.option.strike_price, f2p!(90.0));
        assert_eq!(new_strategy.short_put.option.strike_price, f2p!(95.0));
        assert_eq!(new_strategy.short_call.option.strike_price, f2p!(105.0));
        assert_eq!(new_strategy.long_call.option.strike_price, f2p!(110.0));
    }

    #[test]
    #[should_panic(expected = "Invalid number of legs for Iron Condor strategy")]
    fn test_create_strategy_invalid_legs() {
        let condor = create_test_condor();
        let chain = create_test_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::TwoLegs {
            first: options[0],
            second: options[1],
        };

        let _ = condor.create_strategy(&chain, &legs);
    }
}

#[cfg(test)]
mod tests_iron_condor_profit {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        )
    }

    #[test]
    fn test_profit_at_max_profit_price() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(100.0));
        // Net premium = (2.0 + 2.0) - (1.0 + 1.0) = 2.0
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_below_long_put() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(85.0));
        // (95 - 90) - net_premium = 5 - 2 = 3
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_at_long_put() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(90.0));
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_between_puts() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(92.5));
        assert!(profit > -3.0 && profit < 2.0);
    }

    #[test]
    fn test_profit_at_short_put() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(95.0));
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_in_profit_zone() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(100.0));
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_at_short_call() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(105.0));
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_between_calls() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(107.5));
        assert!(profit > -3.0 && profit < 2.0);
    }

    #[test]
    fn test_profit_at_long_call() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(110.0));
        // (110 - 105) - net_premium = 5 - 2 = 3
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_above_long_call() {
        let condor = create_test_condor();
        let profit = condor.calculate_profit_at(f2p!(115.0));
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_with_fees() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(105.0),
            f2p!(95.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(1.0),
            2.0,
            2.0,
            1.0,
            1.0,
            0.5, // open_fee
            0.5, // closing fee
        );

        let profit = condor.calculate_profit_at(f2p!(100.0));
        // Net premium = 2.0 - fees = 2.0 - 4.0 = -2.0
        assert_eq!(profit, -2.0);
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0),
            f2p!(105.0),
            f2p!(95.0),
            f2p!(110.0),
            f2p!(90.0),
            ExpirationDate::Days(30.0),
            0.20,
            0.05,
            0.0,
            f2p!(2.0), // quantity = 2
            2.0,
            2.0,
            1.0,
            1.0,
            0.0,
            0.0,
        );

        let profit = condor.calculate_profit_at(f2p!(100.0));
        // Net premium * quantity = 2.0 * 2 = 4.0
        assert_eq!(profit, 4.0);
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let condor = create_test_condor();

        let lower_break_even = f2p!(93.0); // 95 - 2
        let upper_break_even = f2p!(107.0); // 105 + 2

        let lower_profit = condor.calculate_profit_at(lower_break_even);
        let upper_profit = condor.calculate_profit_at(upper_break_even);

        assert!(lower_profit.abs() < 0.001);
        assert!(upper_profit.abs() < 0.001);
    }
}

#[cfg(test)]
mod tests_iron_condor_graph {
    use super::*;
    use crate::f2p;
    use crate::model::types::ExpirationDate;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            f2p!(100.0), // underlying_price
            f2p!(105.0), // short_call_strike
            f2p!(95.0),  // short_put_strike
            f2p!(110.0), // long_call_strike
            f2p!(90.0),  // long_put_strike
            ExpirationDate::Days(30.0),
            0.20,      // implied_volatility
            0.05,      // risk_free_rate
            0.0,       // dividend_yield
            f2p!(2.0), // quantity
            2.0,       // premium_short_call
            2.0,       // premium_short_put
            1.0,       // premium_long_call
            1.0,       // premium_long_put
            0.0,       // open_fee
            0.0,       // closing fee
        )
    }

    #[test]
    fn test_title_format() {
        let condor = create_test_condor();
        let title = condor.title();

        assert!(title.contains("IronCondor Strategy"));
        assert!(title.contains("TEST")); // Symbol
        assert!(title.contains("Size 2")); // Quantity

        assert!(title.contains("Long Put: $90"));
        assert!(title.contains("Short Put: $95"));
        assert!(title.contains("Short Call: $105"));
        assert!(title.contains("Long Call: $110"));

        assert!(title.contains("Expire:"));
    }

    #[test]
    fn test_get_vertical_lines() {
        let condor = create_test_condor();
        let lines = condor.get_vertical_lines();

        assert_eq!(lines.len(), 1);
        let line = &lines[0];

        assert_eq!(line.x_coordinate, 100.0);
        assert_eq!(line.y_range, (-50000.0, 50000.0));
        assert!(line.label.contains("Current Price: 100"));
        assert_eq!(line.label_offset, (5.0, 5.0));
        assert_eq!(line.line_color, ORANGE);
        assert_eq!(line.label_color, ORANGE);
        assert_eq!(line.font_size, 18);
    }

    #[test]
    fn test_get_points() {
        let condor = create_test_condor();
        let points = condor.get_points();

        assert_eq!(points.len(), 7);

        let lower_break_even = &points[0];
        let upper_break_even = &points[1];
        assert_eq!(lower_break_even.coordinates.1, 0.0);
        assert_eq!(upper_break_even.coordinates.1, 0.0);
        assert!(lower_break_even.label.contains("Left Break Even"));
        assert!(upper_break_even.label.contains("Right Break Even"));

        let lower_max_profit = &points[2];
        let upper_max_profit = &points[3];
        assert_eq!(lower_max_profit.coordinates.0, 105.0);
        assert_eq!(upper_max_profit.coordinates.0, 95.0);
        assert!(lower_max_profit.label.contains("High Max Profit"));
        assert!(upper_max_profit.label.contains("Low Max Profit"));

        let right_max_loss = &points[4];
        let left_max_loss = &points[5];
        assert_eq!(right_max_loss.coordinates.0, 110.0);
        assert_eq!(left_max_loss.coordinates.0, 90.0);
        assert!(right_max_loss.label.contains("Right Max Loss"));
        assert!(left_max_loss.label.contains("Left Max Loss"));
    }

    #[test]
    fn test_point_colors() {
        let condor = create_test_condor();
        let points = condor.get_points();

        for point in &points {
            match point.label.as_str() {
                label if label.contains("Break Even") => {
                    assert_eq!(point.point_color, DARK_BLUE);
                    assert_eq!(point.label_color, DARK_BLUE);
                }
                label if label.contains("Max Profit") => {
                    assert_eq!(point.point_color, DARK_GREEN);
                    assert_eq!(point.label_color, DARK_GREEN);
                }
                label if label.contains("Max Loss") => {
                    assert_eq!(point.point_color, RED);
                    assert_eq!(point.label_color, RED);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn test_point_styles() {
        let condor = create_test_condor();
        let points = condor.get_points();

        for point in points {
            assert_eq!(point.point_size, 5);
            assert_eq!(point.font_size, 18);
            assert_eq!(point.label_color, point.point_color);
        }
    }

    #[test]
    fn test_zero_profit_points() {
        let mut condor = create_test_condor();
        condor.short_call.premium = 1.0;
        condor.short_put.premium = 1.0;
        condor.long_call.premium = 1.0;
        condor.long_put.premium = 1.0;

        let points = condor.get_points();
        let max_profit_point = &points[2];

        assert_eq!(max_profit_point.coordinates.1, 0.0);
        assert!(max_profit_point.label.contains("0.00"));
    }

    #[test]
    fn test_points_with_different_quantities() {
        let condor = create_test_condor();
        let points = condor.get_points();

        let max_profit_point = &points[2];
        let max_loss_point = &points[4];

        assert_eq!(max_profit_point.coordinates.1, 4.0); // 2 * 2.0
        assert_eq!(max_loss_point.coordinates.1, -6.0); // 2 * -3.0
    }

    #[test]
    fn test_current_price_point() {
        let condor = create_test_condor();
        let points = condor.get_points();
        let current_price_point = points.last().unwrap();

        assert_eq!(
            current_price_point.coordinates.0,
            condor.long_call.option.underlying_price.to_f64()
        );

        let expected_profit = condor.calculate_profit_at(condor.long_call.option.underlying_price);
        assert_eq!(current_price_point.coordinates.1, expected_profit);
    }
}

#[cfg(test)]
mod tests_iron_condor_delta {
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_condor::IronCondor;
    use crate::{d2fu, f2p, Positive};
    use approx::assert_relative_eq;

    fn get_strategy(underlying_price: Positive) -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            underlying_price, // underlying_price
            f2p!(2725.0),     // short_call_strike
            f2p!(2560.0),     // short_put_strike
            f2p!(2800.0),     // long_call_strike
            f2p!(2500.0),     // long_put_strike
            ExpirationDate::Days(30.0),
            0.1548,    // implied_volatility
            0.0,       // risk_free_rate
            0.0,       // dividend_yield
            f2p!(1.0), // quantity
            38.8,      // premium_short_call
            30.4,      // premium_short_put
            23.3,      // premium_long_call
            16.8,      // premium_long_put
            0.96,      // open_fee
            0.96,      // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(f2p!(2800.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.2124,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.4175977387296557),
                strike: f2p!(2800.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(10.312565341673709),
                strike: f2p!(2560.0),
                option_type: OptionStyle::Put
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = f2p!(0.4175977387296557);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.21249, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(f2p!(2500.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.18282752,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.37224508871224055),
                strike: f2p!(2500.0),
                option_type: OptionStyle::Put
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(6.659872649379908),
                strike: f2p!(2725.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_put.option.clone();
        option.quantity = f2p!(0.37224508871224055);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, -0.1828275205, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(f2p!(2100.0));

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
mod tests_iron_condor_delta_size {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_condor::IronCondor;
    use crate::{d2fu, f2p};
    use approx::assert_relative_eq;

    fn get_strategy(underlying_price: Positive) -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            underlying_price, // underlying_price
            f2p!(2725.0),     // short_call_strike
            f2p!(2560.0),     // short_put_strike
            f2p!(2800.0),     // long_call_strike
            f2p!(2500.0),     // long_put_strike
            ExpirationDate::Days(30.0),
            0.1548,    // implied_volatility
            0.0,       // risk_free_rate
            0.0,       // dividend_yield
            f2p!(2.0), // quantity
            38.8,      // premium_short_call
            30.4,      // premium_short_put
            23.3,      // premium_long_call
            16.8,      // premium_long_put
            0.96,      // open_fee
            0.96,      // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(f2p!(2800.9));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.42443,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.829_398_651_588_599),
                strike: f2p!(2800.0),
                option_type: OptionStyle::Call
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(20.96133828303674),
                strike: f2p!(2560.0),
                option_type: OptionStyle::Put
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = f2p!(0.8351954774593114);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.42740, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(f2p!(2500.9));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.3656,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: f2p!(0.749453805542630),
                strike: f2p!(2500.0),
                option_type: OptionStyle::Put
            }
        );
        assert_eq!(
            suggestion[1],
            DeltaAdjustment::SellOptions {
                quantity: f2p!(13.07422105788514),
                strike: f2p!(2725.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_put.option.clone();
        option.quantity = f2p!(0.749453805542630);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, -0.36565, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(f2p!(2100.0));

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
