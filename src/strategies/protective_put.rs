/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! # Protective Put Strategy
//!
//! A protective put (also known as a "married put") involves holding a long
//! position in the underlying asset and buying a put option on that same asset.
//! This strategy provides unlimited upside potential while limiting downside risk.

use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::Options;
use crate::error::probability::ProbabilityError;
use crate::error::{GreeksError, PositionError, PricingError, StrategyError};
use crate::greeks::Greeks;
use crate::model::ExpirationDate;
use crate::model::ProfitLossRange;
use crate::model::leg::traits::LegAble;
use crate::model::leg::{Leg, SpotPosition};
use crate::model::position::Position;
use crate::model::types::{OptionBasicType, OptionStyle, OptionType, Side};
use crate::pnl::PnLCalculator;
use crate::pricing::payoff::Profit;
use crate::strategies::delta_neutral::DeltaNeutrality;
use crate::strategies::probabilities::core::ProbabilityAnalysis;
use crate::strategies::probabilities::utils::VolatilityAdjustment;
use crate::strategies::{BasicAble, Strategies};
use chrono::Utc;
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::debug;
use utoipa::ToSchema;

/// Default description for the Protective Put strategy.
pub const PROTECTIVE_PUT_DESCRIPTION: &str = "A protective put (married put) is a hedging strategy \
    that involves holding a long position in the underlying asset and buying a put option on that \
    same asset. This provides downside protection while maintaining unlimited upside potential.";

/// Represents a Protective Put options trading strategy.
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct ProtectivePut {
    /// The name of the strategy.
    pub name: String,
    /// The type of strategy.
    pub kind: StrategyType,
    /// A textual description of this strategy instance.
    pub description: String,
    /// The price points at which the strategy breaks even.
    pub break_even_points: Vec<Positive>,
    /// The long spot position (underlying asset).
    pub spot_leg: SpotPosition,
    /// The long put option position (protective put).
    pub long_put: Position,
}

impl ProtectivePut {
    /// Creates a new Protective Put strategy.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_put: Positive,
        spot_open_fee: Positive,
        spot_close_fee: Positive,
        put_open_fee: Positive,
        put_close_fee: Positive,
    ) -> Self {
        let spot_leg = SpotPosition::new(
            underlying_symbol.clone(),
            quantity,
            underlying_price,
            Side::Long,
            Utc::now(),
            spot_open_fee,
            spot_close_fee,
        );

        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            put_strike,
            expiration,
            implied_volatility,
            quantity / Positive::HUNDRED,
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
            put_open_fee,
            put_close_fee,
            None,
            None,
        );

        let mut strategy = ProtectivePut {
            name: format!("ProtectivePut_{}", underlying_symbol),
            kind: StrategyType::ProtectivePut,
            description: PROTECTIVE_PUT_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            spot_leg,
            long_put,
        };

        strategy.validate();
        strategy
            .update_break_even_points()
            .expect("Failed to calculate break-even points");
        strategy
    }

    /// Returns the spot leg as a Leg enum.
    #[must_use]
    pub fn get_spot_leg(&self) -> Leg {
        Leg::Spot(self.spot_leg.clone())
    }

    /// Returns the long put leg as a Leg enum.
    #[must_use]
    pub fn get_put_leg(&self) -> Leg {
        Leg::Option(self.long_put.clone())
    }

    /// Returns all legs of the strategy.
    #[must_use]
    pub fn get_legs(&self) -> Vec<Leg> {
        vec![self.get_spot_leg(), self.get_put_leg()]
    }

    /// Returns the put strike price.
    #[must_use]
    pub fn put_strike(&self) -> Positive {
        self.long_put.option.strike_price
    }

    /// Returns the underlying price (cost basis).
    #[must_use]
    pub fn underlying_price(&self) -> Positive {
        self.spot_leg.cost_basis
    }

    /// Returns the quantity of shares.
    #[must_use]
    pub fn quantity(&self) -> Positive {
        self.spot_leg.quantity
    }

    /// Calculates the net delta of the strategy.
    pub fn net_delta(&self) -> Result<Decimal, GreeksError> {
        let spot_delta = self.spot_leg.delta()?;
        let put_delta = self.long_put.delta()?;
        Ok(spot_delta + put_delta)
    }

    /// Calculates the maximum loss potential.
    pub fn max_loss_potential(&self) -> Result<Positive, PricingError> {
        let put_strike = self.put_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        let put_premium = self.long_put.premium * self.long_put.option.quantity;
        let total_fees = self.total_fees();

        if cost_basis >= put_strike {
            let capital_loss = (cost_basis - put_strike) * quantity;
            let total_loss = capital_loss.to_dec() + put_premium.to_dec() + total_fees.to_dec();
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        } else {
            let capital_gain = (put_strike - cost_basis) * quantity;
            let total_loss = put_premium.to_dec() + total_fees.to_dec() - capital_gain.to_dec();
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO)).unwrap_or(Positive::ZERO))
        }
    }

    /// Calculates total fees for all positions.
    #[must_use]
    pub fn total_fees(&self) -> Positive {
        self.spot_leg.open_fee
            + self.spot_leg.close_fee
            + self.long_put.open_fee
            + self.long_put.close_fee
    }

    /// Returns the protection level as a percentage below current price.
    #[must_use]
    pub fn protection_level(&self) -> Decimal {
        let current_price = self.spot_leg.cost_basis.to_dec();
        let put_strike = self.long_put.option.strike_price.to_dec();
        ((current_price - put_strike) / current_price) * Decimal::ONE_HUNDRED
    }

    /// Checks if the put is out-of-the-money.
    #[must_use]
    pub fn is_put_otm(&self) -> bool {
        self.spot_leg.cost_basis > self.long_put.option.strike_price
    }
}

impl Validable for ProtectivePut {
    fn validate(&self) -> bool {
        if !self.long_put.validate() {
            debug!("Long put validation failed");
            return false;
        }
        if self.long_put.option.option_style != OptionStyle::Put {
            debug!("Long put must be a put option");
            return false;
        }
        if self.long_put.option.side != Side::Long {
            debug!("Long put must be a long position");
            return false;
        }
        if self.spot_leg.side != Side::Long {
            debug!("Spot leg must be a long position");
            return false;
        }
        true
    }
}

impl BreakEvenable for ProtectivePut {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points.clear();
        let entry_price = self.spot_leg.cost_basis.to_dec();
        let put_premium = self.long_put.premium.to_dec();
        let quantity = self.spot_leg.quantity.to_dec();
        let total_fees = self.total_fees();
        let break_even = entry_price + put_premium + (total_fees.to_dec() / quantity);
        if let Ok(be) = Positive::new_decimal(break_even) {
            self.break_even_points.push(be.round_to(2));
        }
        Ok(())
    }
}

impl Positionable for ProtectivePut {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if position.option.option_style != OptionStyle::Put {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position must be a put option".to_string(),
            ));
        }
        self.long_put = position.clone();
        let _ = self.update_break_even_points();
        Ok(())
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put])
    }
}

impl Strategable for ProtectivePut {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for ProtectivePut {
    fn get_title(&self) -> String {
        format!(
            "Protective Put Strategy:\n\t{} {} {} @ {}\n\t{}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.symbol,
            self.spot_leg.cost_basis,
            self.long_put.get_title()
        )
    }

    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        let mut hash_set = HashSet::new();
        let long_put = &self.long_put.option;
        hash_set.insert(OptionBasicType {
            option_style: &long_put.option_style,
            side: &long_put.side,
            strike_price: &long_put.strike_price,
            expiration_date: &long_put.expiration_date,
        });
        hash_set
    }

    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut map = HashMap::new();
        let long_put = &self.long_put.option;
        map.insert(
            OptionBasicType {
                option_style: &long_put.option_style,
                side: &long_put.side,
                strike_price: &long_put.strike_price,
                expiration_date: &long_put.expiration_date,
            },
            &long_put.implied_volatility,
        );
        map
    }

    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut map = HashMap::new();
        let long_put = &self.long_put.option;
        map.insert(
            OptionBasicType {
                option_style: &long_put.option_style,
                side: &long_put.side,
                strike_price: &long_put.strike_price,
                expiration_date: &long_put.expiration_date,
            },
            &long_put.quantity,
        );
        map
    }
}

impl Strategies for ProtectivePut {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::new_decimal(Decimal::MAX).unwrap_or(Positive::ZERO))
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        self.max_loss_potential().map_err(StrategyError::from)
    }
}

impl Profit for ProtectivePut {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let spot_pnl = self.spot_leg.pnl_at_price(*price);
        let put_pnl = self
            .long_put
            .pnl_at_expiration(&Some(price))
            .unwrap_or(Decimal::ZERO);
        Ok(spot_pnl + put_pnl)
    }
}

impl Greeks for ProtectivePut {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option])
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        self.net_delta()
    }
}

impl PnLCalculator for ProtectivePut {
    fn calculate_pnl(
        &self,
        underlying_price: &Positive,
        _expiration_date: ExpirationDate,
        _implied_volatility: &Positive,
    ) -> Result<crate::pnl::utils::PnL, PricingError> {
        self.calculate_pnl_at_expiration(underlying_price)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<crate::pnl::utils::PnL, PricingError> {
        let profit = self.calculate_profit_at(underlying_price)?;
        let spot_cost = self.spot_leg.total_cost();
        let put_cost = self.long_put.premium * self.long_put.option.quantity;
        Ok(crate::pnl::utils::PnL {
            realized: None,
            unrealized: Some(profit),
            initial_costs: spot_cost + put_cost,
            initial_income: Positive::ZERO,
            date_time: Utc::now(),
        })
    }
}

impl DeltaNeutrality for ProtectivePut {}

impl Optimizable for ProtectivePut {
    type Strategy = ProtectivePut;
}

impl crate::strategies::StrategyConstructor for ProtectivePut {}

impl ProbabilityAnalysis for ProtectivePut {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self
            .break_even_points
            .first()
            .copied()
            .ok_or_else(|| ProbabilityError::from("No break-even point found"))?;
        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let mut profit_range = ProfitLossRange::new(Some(break_even_point), None, Positive::ZERO)?;
        profit_range.calculate_probability(
            &self.spot_leg.cost_basis,
            Some(VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;
        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self
            .break_even_points
            .first()
            .copied()
            .ok_or_else(|| ProbabilityError::from("No break-even point found"))?;
        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;
        let mut loss_range = ProfitLossRange::new(
            Some(self.put_strike()),
            Some(break_even_point),
            Positive::ZERO,
        )?;
        loss_range.calculate_probability(
            &self.spot_leg.cost_basis,
            Some(VolatilityAdjustment {
                base_volatility: option.implied_volatility,
                std_dev_adjustment: Positive::ZERO,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;
        Ok(vec![loss_range])
    }
}

impl std::fmt::Display for ProtectivePut {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Protective Put Strategy")?;
        writeln!(f, "======================")?;
        writeln!(f, "Symbol: {}", self.spot_leg.symbol)?;
        writeln!(f, "Underlying Price: ${:.2}", self.spot_leg.cost_basis)?;
        writeln!(f, "Put Strike: ${:.2}", self.long_put.option.strike_price)?;
        writeln!(f, "Put Premium: ${:.2}", self.long_put.premium)?;
        writeln!(f, "Quantity: {}", self.spot_leg.quantity)?;
        writeln!(f, "Expiration: {}", self.long_put.option.expiration_date)?;
        writeln!(f, "Protection Level: {:.2}%", self.protection_level())?;
        if let Ok(break_evens) = self.get_break_even_points() {
            writeln!(f, "Break-even: ${:.2}", break_evens[0])?;
        }
        if let Ok(max_loss) = self.max_loss_potential() {
            writeln!(f, "Max Loss: ${:.2}", max_loss)?;
        }
        writeln!(f, "Max Profit: Unlimited")?;
        if let Ok(delta) = self.net_delta() {
            writeln!(f, "Net Delta: {:.4}", delta)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_protective_put() -> ProtectivePut {
        ProtectivePut::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(145.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.25),
            dec!(0.05),
            pos_or_panic!(0.01),
            Positive::HUNDRED,
            pos_or_panic!(3.50),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.65),
            pos_or_panic!(0.65),
        )
    }

    #[test]
    fn test_new_protective_put() {
        let pp = create_test_protective_put();
        assert_eq!(pp.spot_leg.symbol, "AAPL");
        assert_eq!(pp.spot_leg.cost_basis, pos_or_panic!(150.0));
        assert_eq!(pp.long_put.option.strike_price, pos_or_panic!(145.0));
    }

    #[test]
    fn test_underlying_price() {
        let pp = create_test_protective_put();
        assert_eq!(pp.underlying_price(), pos_or_panic!(150.0));
    }

    #[test]
    fn test_put_strike() {
        let pp = create_test_protective_put();
        assert_eq!(pp.put_strike(), pos_or_panic!(145.0));
    }

    #[test]
    fn test_quantity() {
        let pp = create_test_protective_put();
        assert_eq!(pp.quantity(), Positive::HUNDRED);
    }

    #[test]
    fn test_break_even_points() {
        let pp = create_test_protective_put();
        let break_evens = pp.get_break_even_points().unwrap();
        assert_eq!(break_evens.len(), 1);
        assert!(break_evens[0] > pp.spot_leg.cost_basis);
    }

    #[test]
    fn test_max_loss() {
        let pp = create_test_protective_put();
        let max_loss = pp.max_loss_potential().unwrap();
        assert!(max_loss > Positive::ZERO);
    }

    #[test]
    fn test_validate() {
        let pp = create_test_protective_put();
        assert!(pp.validate());
    }

    #[test]
    fn test_profit_at_high_price() {
        let pp = create_test_protective_put();
        let profit = pp.calculate_profit_at(&pos_or_panic!(200.0)).unwrap();
        assert!(profit > Decimal::ZERO);
    }

    #[test]
    fn test_is_put_otm() {
        let pp = create_test_protective_put();
        assert!(pp.is_put_otm());
    }

    #[test]
    fn test_protection_level() {
        let pp = create_test_protective_put();
        let protection = pp.protection_level();
        assert!(protection > Decimal::ZERO);
    }

    #[test]
    fn test_get_legs() {
        let pp = create_test_protective_put();
        let legs = pp.get_legs();
        assert_eq!(legs.len(), 2);
        assert!(legs[0].is_spot());
        assert!(legs[1].is_option());
    }

    #[test]
    fn test_total_fees() {
        let pp = create_test_protective_put();
        let fees = pp.total_fees();
        assert!(fees > Positive::ZERO);
    }

    #[test]
    fn test_display() {
        let pp = create_test_protective_put();
        let display = format!("{}", pp);
        assert!(display.contains("Protective Put Strategy"));
        assert!(display.contains("AAPL"));
    }

    #[test]
    fn test_get_title() {
        let pp = create_test_protective_put();
        let title = pp.get_title();
        assert!(title.contains("Protective Put"));
        assert!(title.contains("AAPL"));
    }

    #[test]
    fn test_strategy_type() {
        let pp = create_test_protective_put();
        assert_eq!(pp.kind, StrategyType::ProtectivePut);
    }

    #[test]
    fn test_positions() {
        let pp = create_test_protective_put();
        let positions = Positionable::get_positions(&pp).unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
    }
}
