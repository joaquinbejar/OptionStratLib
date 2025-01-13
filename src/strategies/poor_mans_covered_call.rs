/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 25/9/24
******************************************************************************/

//!
//! The "Poor Man's Covered Call" is an options strategy designed to simulate a traditional covered call,
//! but with a lower capital requirement. In a standard covered call, an investor holds a long position
//! in the underlying asset (e.g., a stock) and sells a call option against it to generate premium income.
//! This strategy works well for neutral to slightly bullish market outlooks.
//! However, instead of purchasing the underlying asset (which can be capital-intensive), the "Poor Man's
//! Covered Call" involves buying a deep-in-the-money LEAP (Long-term Equity Anticipation Security) call
//! option with a long expiration date and selling a short-term out-of-the-money call option against it.
//! By using a LEAP, the investor still benefits from the movement of the underlying asset while avoiding
//! the need to purchase it outright. The premium collected from selling the short-term call generates income
//! and helps offset the cost of the LEAP.
//!
//! The strategy has two main components:
//! 1. **Long LEAP Call**: This serves as a substitute for holding the underlying asset. The deep-in-the-money
//!    LEAP behaves similarly to the underlying asset's price movement but costs a fraction of its price.
//!    The LEAP should have a delta close to 1, meaning it moves nearly dollar-for-dollar with the underlying asset.
//! 2. **Short Call**: A short-term out-of-the-money call is sold against the long LEAP. This generates premium
//!    income, and if the underlying asset's price rises above the strike price of the short call, the investor may
//!    need to sell the asset (or close the position), locking in potential gains.
//!
//! The goal is to capture some upside potential of the underlying asset while reducing risk through a lower capital
//! commitment. The key risks involve the loss of the premium collected if the underlying asset does not move favorably
//! and potential limitations on profits if the underlying asset's price rises sharply, triggering the short call.
//! This strategy is often used by investors who are moderately bullish on an asset but wish to reduce the cost
//! and risk associated with traditional covered call strategies.
//!

use super::base::{Optimizable, Positionable, Strategies, StrategyType, Validable};
use crate::chains::chain::{OptionChain, OptionData};
use crate::chains::StrategyLegs;
use crate::constants::{DARK_BLUE, DARK_GREEN, ZERO};
use crate::error::position::PositionError;
use crate::error::strategies::{ProfitLossErrorKind, StrategyError};
use crate::error::ProbabilityError;
use crate::greeks::equations::{Greek, Greeks};
use crate::model::position::Position;
use crate::model::types::{ExpirationDate, OptionStyle, OptionType, Side};
use crate::model::utils::mean_and_std;
use crate::model::ProfitLossRange;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::{
    DeltaAdjustment, DeltaInfo, DeltaNeutrality, DELTA_THRESHOLD,
};
use crate::strategies::probabilities::{ProbabilityAnalysis, VolatilityAdjustment};
use crate::strategies::utils::{FindOptimalSide, OptimizationCriteria};
use crate::visualization::model::{ChartPoint, ChartVerticalLine, LabelOffsetType};
use crate::visualization::utils::Graph;
use crate::Options;
use crate::{d2fu, pos, Positive};
use chrono::Utc;
use num_traits::FromPrimitive;
use plotters::prelude::full_palette::ORANGE;
use plotters::prelude::{ShapeStyle, RED};
use rust_decimal::Decimal;
use std::error::Error;
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
    pub break_even_points: Vec<Positive>,
    long_call: Position,
    short_call: Position,
}

impl PoorMansCoveredCall {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        long_call_strike: Positive,
        short_call_strike: Positive,
        long_call_expiration: ExpirationDate,
        short_call_expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_call: Positive,
        premium_short_call: Positive,
        open_fee_long_call: Positive,
        close_fee_long_call: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
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
        strategy
            .add_position(&long_call.clone())
            .expect("Invalid long call option");

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
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call option");

        let net_debit = strategy.net_cost().unwrap() / quantity;

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

impl Positionable for PoorMansCoveredCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (
            position.option.option_style.clone(),
            position.option.side.clone(),
        ) {
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style.clone(),
                "Position is a Put, it is not valid for PoorMansCoveredCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.long_call])
    }
}

impl Strategies for PoorMansCoveredCall {
    fn get_underlying_price(&self) -> Positive {
        self.long_call.option.underlying_price
    }

    fn max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(self.short_call.option.strike_price)?;
        if profit <= Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        } else {
            Ok(profit.into())
        }
    }

    fn max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(self.long_call.option.strike_price)?;
        if loss >= Decimal::ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss must be negative".to_string(),
                },
            ))
        } else {
            Ok(loss.abs().into())
        }
    }

    fn profit_area(&self) -> Result<Decimal, StrategyError> {
        let base = (self.short_call.option.strike_price
            - (self.short_call.option.strike_price - self.max_profit().unwrap_or(Positive::ZERO)))
        .to_f64();
        let high = self.max_profit().unwrap_or(Positive::ZERO).to_f64();
        let result = base * high / 200.0;
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let result = match (self.max_profit(), self.max_loss()) {
            (Ok(profit), Ok(loss)) => (profit / loss).to_f64() * 100.0,
            _ => ZERO,
        };
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
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
        let mut best_value = Decimal::MIN;

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

                let legs = StrategyLegs::TwoLegs {
                    first: long_call_option,
                    second: short_call_option,
                };
                let strategy: PoorMansCoveredCall = self.create_strategy(option_chain, &legs);

                if !strategy.validate() {
                    debug!("Invalid strategy");
                    continue;
                }

                let current_value = match criteria {
                    OptimizationCriteria::Ratio => strategy.profit_ratio().unwrap(),
                    OptimizationCriteria::Area => strategy.profit_area().unwrap(),
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
        if underlying_price == Positive::ZERO {
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
        if underlying_price == Positive::ZERO {
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

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };

        PoorMansCoveredCall::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_call.option.expiration_date.clone(),
            self.short_call.option.expiration_date.clone(),
            short.implied_volatility.unwrap() / 100.0,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            long.call_ask.unwrap(),
            short.call_bid.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
        )
    }
}

impl Profit for PoorMansCoveredCall {
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(&price);
        Ok(
            self.long_call.pnl_at_expiration(&price)?
                + self.short_call.pnl_at_expiration(&price)?,
        )
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
                self.short_call
                    .option
                    .expiration_date
                    .get_date_string()
                    .unwrap()
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
        let max_loss = self.max_loss().unwrap_or(Positive::ZERO);

        points.push(ChartPoint {
            coordinates: (self.break_even_points[0].to_f64(), 0.0),
            label: format!("Break Even\n\n{}", self.break_even_points[0]),
            label_offset: LabelOffsetType::Relative(-30.0, 15.0),
            point_color: DARK_BLUE,
            label_color: DARK_BLUE,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.short_call.option.strike_price.to_f64() / 2000.0,
            max_profit.to_f64() / 10.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.short_call.option.strike_price.to_f64(),
                max_profit.to_f64(),
            ),
            label: format!(
                "Max Profit {:.2} at {:.0}",
                max_profit, self.short_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordiantes.0, coordiantes.1),
            point_color: DARK_GREEN,
            label_color: DARK_GREEN,
            point_size: 5,
            font_size: 18,
        });

        let coordiantes: (f64, f64) = (
            self.long_call.option.strike_price.to_f64() / 2000.0,
            -max_loss.to_f64() / 50.0,
        );
        points.push(ChartPoint {
            coordinates: (
                self.long_call.option.strike_price.to_f64(),
                -max_loss.to_f64(),
            ),
            label: format!(
                "Max Loss {:.2} at {:.0}",
                max_loss, self.long_call.option.strike_price
            ),
            label_offset: LabelOffsetType::Relative(coordiantes.0, coordiantes.1),
            point_color: RED,
            label_color: RED,
            point_size: 5,
            font_size: 18,
        });

        points.push(self.get_point_at_price(self.long_call.option.underlying_price));

        points
    }
}

impl ProbabilityAnalysis for PoorMansCoveredCall {
    fn get_expiration(&self) -> Result<ExpirationDate, ProbabilityError> {
        Ok(self.long_call.option.expiration_date.clone())
    }

    fn get_risk_free_rate(&self) -> Option<Decimal> {
        Some(self.long_call.option.risk_free_rate)
    }

    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call.option.implied_volatility,
            self.long_call.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(Some(break_even_point), None, Positive::ZERO)?;

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
        let break_even_point = self.get_break_even_points()?[0];

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(None, Some(break_even_point), Positive::ZERO)?;

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

impl Greeks for PoorMansCoveredCall {
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

impl DeltaNeutrality for PoorMansCoveredCall {
    fn calculate_net_delta(&self) -> DeltaInfo {
        let long_call_delta = self.long_call.option.delta();
        let short_call_delta = self.short_call.option.delta();
        let threshold = DELTA_THRESHOLD;
        let l_c_delta = d2fu!(long_call_delta.unwrap()).unwrap();
        let s_c_delta = d2fu!(short_call_delta.unwrap()).unwrap();

        DeltaInfo {
            net_delta: l_c_delta + s_c_delta,
            individual_deltas: vec![l_c_delta, s_c_delta],
            is_neutral: (l_c_delta + s_c_delta).abs() < threshold,
            underlying_price: self.short_call.option.underlying_price,
            neutrality_threshold: threshold,
        }
    }

    fn get_atm_strike(&self) -> Positive {
        self.short_call.option.underlying_price
    }

    fn generate_delta_reducing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let l_c_delta = d2fu!(self.short_call.option.delta().unwrap()).unwrap();
        vec![DeltaAdjustment::SellOptions {
            quantity: pos!((net_delta.abs() / l_c_delta).abs()) * self.short_call.option.quantity,
            strike: self.short_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }

    fn generate_delta_increasing_adjustments(&self) -> Vec<DeltaAdjustment> {
        let net_delta = self.calculate_net_delta().net_delta;
        let l_c_delta = d2fu!(self.long_call.option.delta().unwrap()).unwrap();
        vec![DeltaAdjustment::BuyOptions {
            quantity: pos!((net_delta.abs() / l_c_delta).abs()) * self.long_call.option.quantity,
            strike: self.long_call.option.strike_price,
            option_type: OptionStyle::Call,
        }]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_pmcc_strategy() -> PoorMansCoveredCall {
        let underlying_symbol = "AAPL".to_string();
        let underlying_price = pos!(150.0);
        let long_call_strike = pos!(140.0);
        let short_call_strike = pos!(160.0);
        let long_call_expiration = ExpirationDate::Days(DAYS_IN_A_YEAR);
        let short_call_expiration = ExpirationDate::Days(pos!(30.0));
        let implied_volatility = pos!(0.20);
        let risk_free_rate = dec!(0.01);
        let dividend_yield = pos!(0.005);
        let quantity = pos!(1.0);
        let premium_long_call = pos!(15.0);
        let premium_short_call = pos!(5.0);
        let open_fee_long_call = pos!(1.0);
        let close_fee_long_call = pos!(1.0);
        let open_fee_short_call = pos!(0.5);
        let close_fee_short_call = pos!(0.5);

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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_create_pmcc_strategy() {
        let pmcc = create_pmcc_strategy();
        assert_eq!(pmcc.name, "Poor Man's Covered Call");
        assert_eq!(pmcc.long_call.option.strike_price, pos!(140.0));
        assert_eq!(pmcc.short_call.option.strike_price, pos!(160.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit() {
        let pmcc = create_pmcc_strategy();
        let max_profit = pmcc.max_profit().unwrap_or(Positive::ZERO);
        assert!(max_profit > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_loss() {
        let pmcc = create_pmcc_strategy();
        let max_loss = pmcc.max_loss().unwrap_or(Positive::ZERO);
        assert!(max_loss > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_break_even_points() {
        let pmcc = create_pmcc_strategy();
        let break_even = pmcc.get_break_even_points().unwrap();
        assert_eq!(break_even.len(), 1);
        assert!(break_even[0].to_f64() > 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_total_cost() {
        let pmcc = create_pmcc_strategy();
        let total_cost = pmcc.total_cost().unwrap();
        assert!(total_cost > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_fees() {
        let pmcc = create_pmcc_strategy();
        let fees = pmcc.fees().unwrap();
        assert!(fees > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_area() {
        let pmcc = create_pmcc_strategy();
        let profit_area = pmcc.profit_area().unwrap().to_f64().unwrap();
        assert!(profit_area > 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_profit_ratio() {
        let pmcc = create_pmcc_strategy();
        let profit_ratio = pmcc.profit_ratio().unwrap().to_f64().unwrap();
        assert!(profit_ratio > 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_best_range_to_show() {
        let pmcc = create_pmcc_strategy();
        let step = pos!(1.0);
        let range = pmcc.best_range_to_show(step);
        assert!(range.is_ok());
        let range_values = range.unwrap();
        assert!(!range_values.is_empty());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_profit_at() {
        let pmcc = create_pmcc_strategy();
        let profit = pmcc.calculate_profit_at(pos!(150.0)).unwrap();
        assert!(
            profit >= -pmcc.max_loss().unwrap_or(Positive::ZERO).to_dec()
                && profit <= pmcc.max_profit().unwrap_or(Positive::ZERO).to_dec()
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_title() {
        let pmcc = create_pmcc_strategy();
        let title = pmcc.title();
        assert!(title.contains("PoorMansCoveredCall Strategy"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vertical_lines() {
        let pmcc = create_pmcc_strategy();
        let vertical_lines = pmcc.get_vertical_lines();
        assert_eq!(vertical_lines.len(), 1);
        assert_eq!(vertical_lines[0].x_coordinate, 150.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_graph_points() {
        let pmcc = create_pmcc_strategy();
        let points = pmcc.get_points();
        assert!(!points.is_empty());
    }
}

#[cfg(test)]
mod tests_pmcc_validation {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::error::position::PositionValidationErrorKind;
    use rust_decimal_macros::dec;

    fn create_basic_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.005),
            pos!(1.0),
            pos!(15.0),
            pos!(5.0),
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_validate_valid_strategy() {
        let strategy = create_basic_strategy();
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_leg_long_call() {
        let mut strategy = create_basic_strategy();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(140.0),
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            pos!(0.2),
            pos!(1.0),
            pos!(150.0),
            dec!(0.01),
            OptionStyle::Call,
            pos!(0.005),
            None,
        );
        let position = Position::new(option, pos!(15.0), Utc::now(), Positive::ONE, Positive::ONE);
        strategy
            .add_position(&position.clone())
            .expect("Invalid long call option");
        assert_eq!(strategy.long_call, position);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_leg_short_call() {
        let mut strategy = create_basic_strategy();
        let option = Options::new(
            OptionType::European,
            Side::Short,
            "AAPL".to_string(),
            pos!(160.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            pos!(1.0),
            pos!(150.0),
            dec!(0.01),
            OptionStyle::Call,
            pos!(0.005),
            None,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), pos!(0.5), pos!(0.5));
        strategy
            .add_position(&position.clone())
            .expect("Invalid short call option");
        assert_eq!(strategy.short_call, position);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_add_leg_invalid_option() {
        let mut strategy = create_basic_strategy();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "AAPL".to_string(),
            pos!(140.0),
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            pos!(0.2),
            pos!(1.0),
            pos!(150.0),
            dec!(0.01),
            OptionStyle::Put,
            pos!(0.005),
            None,
        );
        let position = Position::new(option, pos!(15.0), Utc::now(), Positive::ONE, Positive::ONE);
        let err = strategy.add_position(&position).unwrap_err();
        assert!(matches!(
            err,
            PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleStyle {
                    style: OptionStyle::Put,
                    reason
                }
            ) if reason == "Position is a Put, it is not valid for PoorMansCoveredCall"
        ));
    }
}

#[cfg(test)]
mod tests_pmcc_optimization {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::spos;
    use rust_decimal_macros::dec;

    fn create_test_option_chain() -> OptionChain {
        let mut chain = OptionChain::new("AAPL", pos!(150.0), "2024-01-01".to_string(), None, None);

        // Add options at various strikes
        for strike in [140.0, 145.0, 150.0, 155.0, 160.0].iter() {
            chain.add_option(
                pos!(*strike),
                spos!(5.0),
                spos!(5.2),
                spos!(4.8),
                spos!(5.0),
                spos!(0.2),
                Some(dec!(0.5)),
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
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.005),
            pos!(1.0),
            pos!(15.0),
            pos!(5.0),
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_ratio() {
        let mut strategy = create_base_strategy();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_find_optimal_area() {
        let mut strategy = create_base_strategy();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_short_option_zero_underlying() {
        let mut strategy = create_base_strategy();
        strategy.short_call.option.underlying_price = Positive::ZERO;
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_invalid_long_option_zero_underlying() {
        let mut strategy = create_base_strategy();
        strategy.long_call.option.underlying_price = Positive::ZERO;
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
    use crate::constants::DAYS_IN_A_YEAR;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.005),
            pos!(1.0),
            pos!(15.0),
            pos!(5.0),
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_calculate_profit_at_various_prices() {
        let strategy = create_test_strategy();

        // Below long strike
        let profit_below = strategy.calculate_profit_at(pos!(130.0)).unwrap();
        assert!(profit_below < Decimal::ZERO);

        // Between strikes
        let profit_middle = strategy.calculate_profit_at(pos!(150.0)).unwrap();
        assert!(profit_middle > profit_below);

        // At short strike
        let profit_short = strategy
            .calculate_profit_at(strategy.short_call.option.strike_price)
            .unwrap();
        assert_eq!(
            profit_short,
            strategy.max_profit().unwrap_or(Positive::ZERO).to_dec()
        );

        // Above short strike
        let profit_above = strategy.calculate_profit_at(pos!(170.0)).unwrap();
        assert_eq!(profit_above, profit_above);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_point() {
        let strategy = create_test_strategy();
        assert_eq!(strategy.break_even_points.len(), 1);
        let break_even = strategy.break_even_points[0];
        let profit_at_be = strategy
            .calculate_profit_at(break_even)
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(profit_at_be.abs() < 0.01);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_net_premium() {
        let strategy = create_test_strategy();
        let net_premium = strategy.net_premium_received().unwrap();
        assert_eq!(net_premium, 0.0);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_max_profit_max_loss_relationship() {
        let strategy = create_test_strategy();
        assert!(strategy.max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(strategy.max_loss().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(
            strategy.max_loss().unwrap_or(Positive::ZERO)
                > strategy.max_profit().unwrap_or(Positive::ZERO)
        );
    }
}

#[cfg(test)]
mod tests_pmcc_graph {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use rust_decimal_macros::dec;

    fn create_test_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(140.0),
            pos!(160.0),
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.005),
            pos!(1.0),
            pos!(15.0),
            pos!(5.0),
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_title_format() {
        let strategy = create_test_strategy();
        let title = strategy.title();
        assert!(title.contains("PoorMansCoveredCall Strategy"));
        assert!(title.contains("AAPL"));
        assert!(title.contains("Long Call (LEAPS)"));
        assert!(title.contains("Short Call"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_vertical_lines_format() {
        let strategy = create_test_strategy();
        let lines = strategy.get_vertical_lines();

        assert_eq!(lines.len(), 1);
        assert_eq!(lines[0].x_coordinate, 150.0);
        assert!(lines[0].label.contains("Current Price"));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
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
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::utils::logger::setup_logger;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[cfg(not(target_arch = "wasm32"))]
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
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.005),
            pos!(1.0),
            pos!(15.0),
            pos!(5.0),
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        );

        Ok((strategy, option_chain))
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_best_area_all() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_area(&option_chain, FindOptimalSide::All);

        assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert!(strategy.total_cost().unwrap() > Positive::ZERO);
        assert!(strategy.fees().unwrap().to_f64() > 0.0);

        assert!(strategy.long_call.option.strike_price < strategy.short_call.option.strike_price);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_best_area_upper() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_area(&option_chain, FindOptimalSide::Upper);

        assert!(strategy.long_call.option.strike_price >= strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_best_area_lower() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_area(&option_chain, FindOptimalSide::Lower);

        assert!(strategy.long_call.option.strike_price <= strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.profit_area().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_pmcc_best_ratio {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::utils::logger::setup_logger;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[cfg(not(target_arch = "wasm32"))]
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
            ExpirationDate::Days(DAYS_IN_A_YEAR),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.005),
            pos!(1.0),
            pos!(15.0),
            pos!(5.0),
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        );

        Ok((strategy, option_chain))
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_best_ratio_all() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_ratio(&option_chain, FindOptimalSide::All);

        assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert!(strategy.max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(strategy.max_loss().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(strategy.fees().unwrap().to_f64() > 0.0);
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_best_ratio_upper() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_ratio(&option_chain, FindOptimalSide::Upper);

        assert!(strategy.long_call.option.strike_price >= strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.validate());
    }

    #[test]
    #[cfg(not(target_arch = "wasm32"))]
    fn test_best_ratio_with_range() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.best_ratio(
            &option_chain,
            FindOptimalSide::Range(pos!(5750.0), pos!(5850.0)),
        );

        assert!(strategy.long_call.option.strike_price >= pos!(5750.0));
        assert!(strategy.short_call.option.strike_price <= pos!(5850.0));

        assert!(strategy.profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_short_straddle_delta {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::poor_mans_covered_call::PoorMansCoveredCall;
    use crate::{d2fu, pos};
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> PoorMansCoveredCall {
        let underlying_price = pos!(7138.5);
        PoorMansCoveredCall::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            long_strike,      // call_strike 7450
            short_strike,     // put_strike 7050
            ExpirationDate::Days(pos!(45.0)),
            ExpirationDate::Days(pos!(15.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_short_straddle_reducing_adjustments() {
        let strategy = get_strategy(pos!(7250.0), pos!(7300.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            0.0887293,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::SellOptions {
                quantity: pos!(0.21684621688317646),
                strike: pos!(7300.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.short_call.option.clone();
        option.quantity = pos!(0.21684621688317646);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, -0.08872, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_short_straddle_increasing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.028694805,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.0689809869957862),
                strike: pos!(7450.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = pos!(0.0689809869957862);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.028694, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_short_straddle_no_adjustments() {
        let strategy = get_strategy(pos!(7390.0), pos!(7250.0));

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
mod tests_short_straddle_delta_size {
    use super::*;
    use crate::model::types::{ExpirationDate, OptionStyle};
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::poor_mans_covered_call::PoorMansCoveredCall;
    use crate::{d2fu, pos};
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> PoorMansCoveredCall {
        let underlying_price = pos!(7138.5);
        PoorMansCoveredCall::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            long_strike,      // call_strike 7450
            short_strike,     // put_strike 7050
            ExpirationDate::Days(pos!(45.0)),
            ExpirationDate::Days(pos!(15.0)),
            pos!(0.3745),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(84.2),     // premium_short_call
            pos!(353.2),    // premium_short_put
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_short_put
            pos!(7.01),     // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_short_straddle_reducing_adjustments() {
        let strategy = get_strategy(pos!(7250.1), pos!(7300.0));
        let size = 0.1773;
        let delta = pos!(0.4334878994986714);
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
                strike: pos!(7300.0),
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
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_short_straddle_increasing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));

        assert_relative_eq!(
            strategy.calculate_net_delta().net_delta,
            -0.057389,
            epsilon = 0.0001
        );
        assert!(!strategy.is_delta_neutral());
        let suggestion = strategy.suggest_delta_adjustments();
        assert_eq!(
            suggestion[0],
            DeltaAdjustment::BuyOptions {
                quantity: pos!(0.1379619739915724),
                strike: pos!(7450.0),
                option_type: OptionStyle::Call
            }
        );

        let mut option = strategy.long_call.option.clone();
        option.quantity = pos!(0.1379619739915724);
        let delta = d2fu!(option.delta().unwrap()).unwrap();
        assert_relative_eq!(delta, 0.05738, epsilon = 0.0001);
        assert_relative_eq!(
            delta + strategy.calculate_net_delta().net_delta,
            0.0,
            epsilon = DELTA_THRESHOLD
        );
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn create_test_short_straddle_no_adjustments() {
        let strategy = get_strategy(pos!(7390.0), pos!(7255.0));

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
mod tests_poor_mans_covered_call_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use crate::{assert_pos_relative_eq, pos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    /// Creates a test Poor Man's Covered Call with standard parameters
    fn create_test_pmcc() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "GOLD".to_string(),                // underlying_symbol
            pos!(2703.3),                      // underlying_price
            pos!(2600.0),                      // long_call_strike
            pos!(2800.0),                      // short_call_strike OTM
            ExpirationDate::Days(pos!(120.0)), // long_call_expiration
            ExpirationDate::Days(pos!(30.0)),  // short_call_expiration
            pos!(0.17),                        // implied_volatility
            dec!(0.05),                        // risk_free_rate
            Positive::ZERO,                    // dividend_yield
            pos!(3.0),                         // quantity
            pos!(154.7),                       // premium_short_call
            pos!(30.8),                        // premium_short_put
            pos!(1.74),                        // open_fee_short_call
            pos!(1.74),                        // close_fee_short_call
            pos!(0.85),                        // open_fee_short_put
            pos!(0.85),                        // close_fee_short_put
        )
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_expiration() {
        let pmcc = create_test_pmcc();
        let result = pmcc.get_expiration();
        assert!(result.is_ok());
        match result.unwrap() {
            ExpirationDate::Days(days) => assert_eq!(days, 120.0),
            _ => panic!("Expected ExpirationDate::Days"),
        }
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_risk_free_rate() {
        let pmcc = create_test_pmcc();
        assert_eq!(pmcc.get_risk_free_rate().unwrap().to_f64().unwrap(), 0.05);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_profit_ranges() {
        let pmcc = create_test_pmcc();
        let result = pmcc.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 1);
        let range = &ranges[0];

        // Verify range bounds
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_none()); // Unlimited upside
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= pos!(1.0));

        // Break-even point should be above long call strike
        assert!(range.lower_bound.unwrap() > pmcc.long_call.option.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_get_loss_ranges() {
        let pmcc = create_test_pmcc();
        let result = pmcc.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 1); // Should have one loss range

        let loss_range = &ranges[0];
        assert!(loss_range.lower_bound.is_none()); // No lower bound
        assert!(loss_range.upper_bound.is_some());
        assert!(loss_range.probability > Positive::ZERO);
        assert!(loss_range.probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_probability_sum_to_one() {
        let pmcc = create_test_pmcc();

        let profit_ranges = pmcc.get_profit_ranges().unwrap();
        let loss_ranges = pmcc.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(total_profit_prob + total_loss_prob, pos!(1.0), pos!(0.0001));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_break_even_points_validity() {
        let pmcc = create_test_pmcc();
        let break_even_points = pmcc.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        // Break-even point should be above long call strike and below short call strike
        assert!(break_even_points[0] > pmcc.long_call.option.strike_price);
        assert!(break_even_points[0] < pmcc.short_call.option.strike_price);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_volatility_adjustment() {
        let pmcc = create_test_pmcc();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let prob = pmcc.probability_of_profit(vol_adj, None);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_with_price_trend() {
        let pmcc = create_test_pmcc();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let prob = pmcc.probability_of_profit(None, trend);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_analyze_probabilities() {
        let pmcc = create_test_pmcc();
        let analysis = pmcc.analyze_probabilities(None, None).unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.expected_value >= Positive::ZERO);
        assert_eq!(analysis.break_even_points.len(), 1);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_different_expirations_validity() {
        let pmcc = create_test_pmcc();
        // Short expiration should be less than long expiration
        assert!(match pmcc.short_call.option.expiration_date {
            ExpirationDate::Days(short_days) => {
                match pmcc.long_call.option.expiration_date {
                    ExpirationDate::Days(long_days) => short_days < long_days,
                    _ => false,
                }
            }
            _ => false,
        });
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_high_volatility_scenario() {
        let pmcc = create_test_pmcc();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.7),
            std_dev_adjustment: pos!(0.05),
        });

        let analysis = pmcc.analyze_probabilities(vol_adj, None).unwrap();
        assert!(analysis.expected_value == Positive::ZERO);
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_extreme_probabilities() {
        let pmcc = create_test_pmcc();
        let result = pmcc.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();

        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }

    #[test]
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test::wasm_bindgen_test)]
    fn test_strike_price_validity() {
        let pmcc = create_test_pmcc();
        // Short call strike should be higher than long call strike for a valid PMCC
        assert!(pmcc.short_call.option.strike_price > pmcc.long_call.option.strike_price);
    }
}
