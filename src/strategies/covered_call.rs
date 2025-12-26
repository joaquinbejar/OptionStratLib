/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Covered Call Strategy
//!
//! A covered call involves holding a long position in the underlying asset and
//! selling a call option on that same asset. This strategy provides limited upside
//! potential but offers some downside protection in the form of the premium received
//! for selling the call option.
//!
//! ## Key Characteristics
//!
//! - Limited profit potential (capped at strike price + premium received)
//! - Provides some downside protection via premium received
//! - Reduces the cost basis of the underlying asset
//! - Ideal for neutral to slightly bullish outlook
//!
//! ## Components
//!
//! - **Long Spot Position**: Ownership of the underlying asset
//! - **Short Call Option**: Sold call option at a strike above current price
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::strategies::covered_call::CoveredCall;
//! use optionstratlib::model::ExpirationDate;
//! use positive::{pos_or_panic, Positive};
//! use rust_decimal_macros::dec;
//!
//! let covered_call = CoveredCall::new(
//!     "AAPL".to_string(),
//!     pos_or_panic!(150.0),    // underlying price
//!     pos_or_panic!(155.0),    // call strike
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.25),     // implied volatility
//!     dec!(0.05),     // risk-free rate
//!     pos_or_panic!(0.01),     // dividend yield
//!     Positive::HUNDRED,    // quantity (shares)
//!     pos_or_panic!(3.50),     // call premium received
//!     Positive::ONE,      // spot open fee
//!     Positive::ONE,      // spot close fee
//!     pos_or_panic!(0.65),     // call open fee
//!     pos_or_panic!(0.65),     // call close fee
//! );
//! ```

use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::Options;
use crate::error::position::PositionValidationErrorKind;
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

/// Default description for the Covered Call strategy.
pub const COVERED_CALL_DESCRIPTION: &str = "A covered call is created by holding a long position \
    in the underlying asset and selling a call option on that same asset. This strategy provides \
    limited upside potential (capped at the strike price plus premium received) but offers some \
    downside protection through the premium received. It is ideal for investors with a neutral \
    to slightly bullish outlook who want to generate income from their holdings.";

/// Represents a Covered Call options trading strategy.
///
/// A Covered Call combines a long position in the underlying asset with a short
/// call option. This strategy is used to generate income from existing holdings
/// while accepting limited upside potential.
///
/// # Structure
///
/// - **Spot Leg**: Long position in the underlying asset
/// - **Option Leg**: Short call option at a strike price above current price
///
/// # Profit/Loss Profile
///
/// - **Maximum Profit**: (Strike Price - Cost Basis) + Premium Received
/// - **Maximum Loss**: Cost Basis - Premium Received (if underlying goes to zero)
/// - **Break-even**: Cost Basis - Premium Received per share
///
/// # Greeks
///
/// - **Delta**: Positive (long spot delta + short call delta)
/// - **Gamma**: Negative (from short call)
/// - **Theta**: Positive (benefits from time decay of short call)
/// - **Vega**: Negative (benefits from volatility decrease)
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct CoveredCall {
    /// The name of the strategy.
    pub name: String,

    /// The type of strategy (StrategyType::CoveredCall).
    pub kind: StrategyType,

    /// A textual description of this strategy instance.
    pub description: String,

    /// The price points at which the strategy breaks even.
    pub break_even_points: Vec<Positive>,

    /// The long spot position (underlying asset).
    pub spot_leg: SpotPosition,

    /// The short call option position.
    pub short_call: Position,
}

impl CoveredCall {
    /// Creates a new Covered Call strategy.
    ///
    /// # Arguments
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `call_strike` - The strike price for the short call option
    /// * `expiration` - The expiration date for the call option
    /// * `implied_volatility` - The implied volatility for option pricing
    /// * `risk_free_rate` - The risk-free interest rate
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of shares (typically 100 per option contract)
    /// * `premium_short_call` - The premium received for selling the call
    /// * `spot_open_fee` - Fee to open the spot position
    /// * `spot_close_fee` - Fee to close the spot position
    /// * `call_open_fee` - Fee to open the call position
    /// * `call_close_fee` - Fee to close the call position
    ///
    /// # Returns
    ///
    /// A fully configured `CoveredCall` strategy instance.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        call_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        spot_open_fee: Positive,
        spot_close_fee: Positive,
        call_open_fee: Positive,
        call_close_fee: Positive,
    ) -> Self {
        // Create the spot position (long underlying)
        let spot_leg = SpotPosition::new(
            underlying_symbol.clone(),
            quantity,
            underlying_price,
            Side::Long,
            Utc::now(),
            spot_open_fee,
            spot_close_fee,
        );

        // Create the short call option
        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            call_strike,
            expiration,
            implied_volatility,
            quantity / Positive::HUNDRED, // Convert shares to contracts
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
            call_open_fee,
            call_close_fee,
            None,
            None,
        );

        let mut strategy = CoveredCall {
            name: "Covered Call".to_string(),
            kind: StrategyType::CoveredCall,
            description: COVERED_CALL_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            spot_leg,
            short_call,
        };

        strategy.validate();
        strategy
            .update_break_even_points()
            .expect("Failed to calculate break-even points");

        strategy
    }

    /// Returns the spot leg as a `Leg` enum.
    #[must_use]
    pub fn get_spot_leg(&self) -> Leg {
        Leg::Spot(self.spot_leg.clone())
    }

    /// Returns the option leg as a `Leg` enum.
    #[must_use]
    pub fn get_option_leg(&self) -> Leg {
        Leg::Option(self.short_call.clone())
    }

    /// Returns all legs of the strategy.
    #[must_use]
    pub fn get_legs(&self) -> Vec<Leg> {
        vec![self.get_spot_leg(), self.get_option_leg()]
    }

    /// Returns the call strike price.
    #[must_use]
    pub fn call_strike(&self) -> Positive {
        self.short_call.option.strike_price
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

    /// Calculates the net delta of the covered call.
    ///
    /// Net Delta = Spot Delta + Option Delta
    /// For a covered call: typically positive but less than 1.0 per share
    pub fn net_delta(&self) -> Result<Decimal, GreeksError> {
        let spot_delta = self.spot_leg.delta()?;
        let option_delta = self.short_call.delta()?;
        Ok(spot_delta + option_delta)
    }

    /// Calculates the effective cost basis after receiving premium.
    ///
    /// Effective Cost Basis = Original Cost Basis - Premium Received per Share
    #[must_use]
    pub fn effective_cost_basis(&self) -> Positive {
        let premium_per_share =
            self.short_call.premium * self.short_call.option.quantity / self.spot_leg.quantity;
        if self.spot_leg.cost_basis > premium_per_share {
            self.spot_leg.cost_basis - premium_per_share
        } else {
            Positive::ZERO
        }
    }

    /// Calculates the maximum profit potential.
    ///
    /// Max Profit = (Strike - Cost Basis) × Quantity + Premium Received - Fees
    pub fn max_profit_potential(&self) -> Result<Positive, PricingError> {
        let strike = self.call_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        let premium_received = self.short_call.premium * self.short_call.option.quantity;
        let total_fees =
            self.spot_leg.fees() + self.short_call.open_fee + self.short_call.close_fee;

        if strike >= cost_basis {
            let capital_gain = (strike - cost_basis) * quantity;
            Ok(capital_gain + premium_received - total_fees)
        } else {
            // Strike below cost basis - max profit is just premium minus loss
            let capital_loss = (cost_basis - strike) * quantity;
            if premium_received > capital_loss + total_fees {
                Ok(premium_received - capital_loss - total_fees)
            } else {
                Ok(Positive::ZERO)
            }
        }
    }

    /// Calculates the maximum loss potential.
    ///
    /// Max Loss = Cost Basis × Quantity - Premium Received + Fees
    /// (occurs if underlying goes to zero)
    pub fn max_loss_potential(&self) -> Result<Positive, PricingError> {
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        let premium_received = self.short_call.premium * self.short_call.option.quantity;
        let total_fees =
            self.spot_leg.fees() + self.short_call.open_fee + self.short_call.close_fee;

        let total_investment = cost_basis * quantity;
        if total_investment + total_fees > premium_received {
            Ok(total_investment + total_fees - premium_received)
        } else {
            Ok(Positive::ZERO)
        }
    }

    /// Checks if the call is currently in-the-money.
    #[must_use]
    pub fn is_call_itm(&self, current_price: Positive) -> bool {
        current_price > self.call_strike()
    }

    /// Calculates the probability of the call being assigned.
    ///
    /// This is a simplified calculation based on moneyness.
    #[must_use]
    pub fn assignment_probability(&self, current_price: Positive) -> Decimal {
        if current_price >= self.call_strike() {
            Decimal::ONE
        } else {
            current_price.to_dec() / self.call_strike().to_dec()
        }
    }
}

impl Validable for CoveredCall {
    fn validate(&self) -> bool {
        // Validate spot position
        if self.spot_leg.quantity == Positive::ZERO {
            debug!("Invalid: Spot quantity is zero");
            return false;
        }

        if self.spot_leg.side != Side::Long {
            debug!("Invalid: Spot position must be long");
            return false;
        }

        // Validate short call
        if self.short_call.option.side != Side::Short {
            debug!("Invalid: Call option must be short");
            return false;
        }

        if self.short_call.option.option_style != OptionStyle::Call {
            debug!("Invalid: Option must be a call");
            return false;
        }

        true
    }
}

impl BreakEvenable for CoveredCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points.clear();

        // Break-even = Cost Basis - Premium Received per Share
        let premium_per_share =
            self.short_call.premium * self.short_call.option.quantity / self.spot_leg.quantity;
        let fees_per_share = self.spot_leg.fees() / self.spot_leg.quantity;

        let break_even = if self.spot_leg.cost_basis > premium_per_share - fees_per_share {
            self.spot_leg.cost_basis - premium_per_share + fees_per_share
        } else {
            Positive::ZERO
        };

        self.break_even_points.push(break_even.round_to(2));
        Ok(())
    }
}

impl Positionable for CoveredCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        // Only allow adding/updating the short call position
        if position.option.side != Side::Short || position.option.option_style != OptionStyle::Call
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "CoveredCall only accepts short call positions".to_string(),
            ));
        }

        self.short_call = position.clone();
        Ok(())
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call])
    }

    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        if *option_style == OptionStyle::Call
            && *side == Side::Short
            && *strike == self.short_call.option.strike_price
        {
            Ok(vec![&mut self.short_call])
        } else {
            Err(PositionError::invalid_position(
                "Position not found in CoveredCall",
            ))
        }
    }

    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        if position.option.side == Side::Short
            && position.option.option_style == OptionStyle::Call
            && position.option.strike_price == self.short_call.option.strike_price
        {
            self.short_call = position.clone();
            Ok(())
        } else {
            Err(PositionError::invalid_position(
                "Position does not match existing short call",
            ))
        }
    }
}

impl Strategable for CoveredCall {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for CoveredCall {
    fn get_title(&self) -> String {
        format!(
            "CoveredCall Strategy:\n\t{} {} {} @ {}\n\t{}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.symbol,
            self.spot_leg.cost_basis,
            self.short_call.get_title()
        )
    }

    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
        let mut hash_set = HashSet::new();
        let short_call = &self.short_call.option;
        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });
        hash_set
    }

    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut map = HashMap::new();
        let short_call = &self.short_call.option;
        map.insert(
            OptionBasicType {
                option_style: &short_call.option_style,
                side: &short_call.side,
                strike_price: &short_call.strike_price,
                expiration_date: &short_call.expiration_date,
            },
            &short_call.implied_volatility,
        );
        map
    }

    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
        let mut map = HashMap::new();
        let short_call = &self.short_call.option;
        map.insert(
            OptionBasicType {
                option_style: &short_call.option_style,
                side: &short_call.side,
                strike_price: &short_call.strike_price,
                expiration_date: &short_call.expiration_date,
            },
            &short_call.quantity,
        );
        map
    }
}

impl Strategies for CoveredCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        self.max_profit_potential().map_err(StrategyError::from)
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        self.max_loss_potential().map_err(StrategyError::from)
    }
}

impl Profit for CoveredCall {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        // Spot P&L
        let spot_pnl = self.spot_leg.pnl_at_price(*price);

        // Option P&L at expiration
        let option_pnl = self
            .short_call
            .pnl_at_expiration(&Some(price))
            .unwrap_or(Decimal::ZERO);

        Ok(spot_pnl + option_pnl)
    }
}

impl Greeks for CoveredCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option])
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        self.net_delta()
    }
}

impl PnLCalculator for CoveredCall {
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
        let premium_received = self.short_call.premium * self.short_call.option.quantity;

        Ok(crate::pnl::utils::PnL {
            realized: None,
            unrealized: Some(profit),
            initial_costs: spot_cost,
            initial_income: premium_received,
            date_time: Utc::now(),
        })
    }
}

impl DeltaNeutrality for CoveredCall {}

impl Optimizable for CoveredCall {
    type Strategy = CoveredCall;
}

impl crate::strategies::StrategyConstructor for CoveredCall {}

impl ProbabilityAnalysis for CoveredCall {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self
            .break_even_points
            .first()
            .copied()
            .ok_or_else(|| ProbabilityError::from("No break-even point found"))?;

        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        // Profit range: from break-even up to strike (capped profit)
        let mut profit_range = ProfitLossRange::new(
            Some(break_even_point),
            Some(self.call_strike()),
            Positive::ZERO,
        )?;

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

        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        // Loss range: from zero up to break-even
        let mut loss_range =
            ProfitLossRange::new(Some(Positive::ZERO), Some(break_even_point), Positive::ZERO)?;

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

impl std::fmt::Display for CoveredCall {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "CoveredCall: {} {} @ {} + Short {} Call @ {}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.cost_basis,
            self.short_call.option.strike_price,
            self.short_call.premium
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_covered_call() -> CoveredCall {
        CoveredCall::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),
            pos_or_panic!(155.0),
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
    fn test_covered_call_creation() {
        let cc = create_test_covered_call();

        assert_eq!(cc.name, "Covered Call");
        assert_eq!(cc.kind, StrategyType::CoveredCall);
        assert_eq!(cc.spot_leg.symbol, "AAPL");
        assert_eq!(cc.spot_leg.quantity, Positive::HUNDRED);
        assert_eq!(cc.spot_leg.cost_basis, pos_or_panic!(150.0));
        assert_eq!(cc.spot_leg.side, Side::Long);
        assert_eq!(cc.short_call.option.strike_price, pos_or_panic!(155.0));
        assert_eq!(cc.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_covered_call_validation() {
        let cc = create_test_covered_call();
        assert!(cc.validate());
    }

    #[test]
    fn test_break_even_calculation() {
        let cc = create_test_covered_call();

        // Break-even should be cost basis minus premium received per share
        assert!(!cc.break_even_points.is_empty());
        let break_even = cc.break_even_points[0];
        assert!(break_even < pos_or_panic!(150.0)); // Should be below cost basis
    }

    #[test]
    fn test_effective_cost_basis() {
        let cc = create_test_covered_call();
        let effective = cc.effective_cost_basis();

        // Effective cost basis should be lower than original
        assert!(effective < cc.spot_leg.cost_basis);
    }

    #[test]
    fn test_profit_at_strike() {
        let cc = create_test_covered_call();

        // At strike price, should have maximum profit
        let profit = cc.calculate_profit_at(&pos_or_panic!(155.0)).unwrap();
        assert!(profit > Decimal::ZERO);
    }

    #[test]
    fn test_profit_above_strike() {
        let cc = create_test_covered_call();

        // Above strike, the short call gets exercised
        // Spot gains continue but are offset by short call losses
        let profit_at_strike = cc.calculate_profit_at(&pos_or_panic!(155.0)).unwrap();
        let profit_above = cc.calculate_profit_at(&pos_or_panic!(170.0)).unwrap();

        // Both should be positive (profitable strategy when price rises)
        assert!(profit_at_strike > Decimal::ZERO);
        assert!(profit_above > Decimal::ZERO);
    }

    #[test]
    fn test_loss_at_zero() {
        let cc = create_test_covered_call();

        // At zero, maximum loss
        let loss = cc.calculate_profit_at(&pos_or_panic!(0.01)).unwrap();
        assert!(loss < Decimal::ZERO);
    }

    #[test]
    fn test_get_legs() {
        let cc = create_test_covered_call();
        let legs = cc.get_legs();

        assert_eq!(legs.len(), 2);
        assert!(legs[0].is_spot());
        assert!(legs[1].is_option());
    }

    #[test]
    fn test_net_delta() {
        let cc = create_test_covered_call();
        let delta = cc.net_delta().unwrap();

        // Net delta should be positive but less than spot quantity
        // (spot delta is +100, short call delta is negative)
        assert!(delta > Decimal::ZERO);
        assert!(delta < dec!(100.0));
    }

    #[test]
    fn test_is_call_itm() {
        let cc = create_test_covered_call();

        assert!(!cc.is_call_itm(pos_or_panic!(150.0))); // Below strike
        assert!(!cc.is_call_itm(pos_or_panic!(155.0))); // At strike
        assert!(cc.is_call_itm(pos_or_panic!(160.0))); // Above strike
    }

    #[test]
    fn test_display() {
        let cc = create_test_covered_call();
        let display = format!("{}", cc);

        assert!(display.contains("CoveredCall"));
        assert!(display.contains("Long"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_get_title() {
        let cc = create_test_covered_call();
        let title = cc.get_title();

        assert!(title.contains("CoveredCall"));
        assert!(title.contains("AAPL"));
    }

    #[test]
    fn test_underlying_price() {
        let cc = create_test_covered_call();
        assert_eq!(cc.underlying_price(), pos_or_panic!(150.0));
    }

    #[test]
    fn test_call_strike() {
        let cc = create_test_covered_call();
        assert_eq!(cc.call_strike(), pos_or_panic!(155.0));
    }

    #[test]
    fn test_quantity() {
        let cc = create_test_covered_call();
        assert_eq!(cc.quantity(), Positive::HUNDRED);
    }
}
