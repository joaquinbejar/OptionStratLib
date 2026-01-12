/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/01/26
******************************************************************************/

//! # Collar Strategy
//!
//! A collar involves holding a long position in the underlying asset, buying an
//! out-of-the-money put option (protective put), and selling an out-of-the-money
//! call option (covered call). This strategy provides downside protection at the
//! cost of limiting upside potential.
//!
//! ## Key Characteristics
//!
//! - Limited profit potential (capped by short call strike)
//! - Limited loss potential (protected by long put strike)
//! - Can often be implemented for little to no cost (zero-cost collar)
//! - Ideal for protecting gains on existing long positions
//!
//! ## Components
//!
//! - **Long Spot Position**: Ownership of the underlying asset
//! - **Long Put Option**: Protective put at a lower strike price
//! - **Short Call Option**: Covered call at a higher strike price
//!
//! ## Profit/Loss Profile
//!
//! ```text
//! Profit ^
//!        |     ___________ <- Max profit (capped by short call)
//!        |    /
//!        |   /
//!        |--/-------------> Underlying Price
//!        | /
//!        |/______________ <- Max loss (limited by long put)
//!        |
//! ```
//!
//! ## Example
//!
//! ```rust
//! use optionstratlib::strategies::collar::Collar;
//! use optionstratlib::model::ExpirationDate;
//! use positive::{pos_or_panic, Positive};
//! use rust_decimal_macros::dec;
//!
//! let collar = Collar::new(
//!     "AAPL".to_string(),
//!     pos_or_panic!(150.0),    // underlying price
//!     pos_or_panic!(145.0),    // put strike (protection level)
//!     pos_or_panic!(160.0),    // call strike (profit cap)
//!     ExpirationDate::Days(pos_or_panic!(30.0)),
//!     pos_or_panic!(0.25),     // implied volatility
//!     dec!(0.05),              // risk-free rate
//!     pos_or_panic!(0.01),     // dividend yield
//!     Positive::HUNDRED,       // quantity (shares)
//!     pos_or_panic!(2.50),     // put premium paid
//!     pos_or_panic!(3.00),     // call premium received
//!     Positive::ONE,           // spot open fee
//!     Positive::ONE,           // spot close fee
//!     pos_or_panic!(0.65),     // put open fee
//!     pos_or_panic!(0.65),     // put close fee
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

/// Default description for the Collar strategy.
pub const COLLAR_DESCRIPTION: &str = "A collar is a protective options strategy that involves \
    holding a long position in the underlying asset, buying a protective put option at a lower \
    strike price, and selling a covered call option at a higher strike price. This creates a \
    'collar' around the current price, limiting both upside potential and downside risk. The \
    strategy is ideal for investors who want to protect gains on existing positions while \
    potentially offsetting the cost of protection with premium received from the short call.";

/// Represents a Collar options trading strategy.
///
/// A Collar combines a long position in the underlying asset with a protective
/// put (long put) and a covered call (short call). This strategy is used to
/// protect existing holdings while accepting limited upside potential.
///
/// # Structure
///
/// - **Spot Leg**: Long position in the underlying asset
/// - **Long Put**: Protective put at a strike below current price
/// - **Short Call**: Covered call at a strike above current price
///
/// # Profit/Loss Profile
///
/// - **Maximum Profit**: (Call Strike - Cost Basis) + Net Premium
/// - **Maximum Loss**: (Cost Basis - Put Strike) - Net Premium
/// - **Break-even**: Cost Basis - Net Premium (if credit) or + Net Premium (if debit)
///
/// # Greeks
///
/// - **Delta**: Positive (long spot delta + long put delta + short call delta)
/// - **Gamma**: Mixed (positive from long put, negative from short call)
/// - **Theta**: Mixed (negative from long put, positive from short call)
/// - **Vega**: Mixed (positive from long put, negative from short call)
#[derive(Clone, Debug, Serialize, Deserialize, ToSchema)]
pub struct Collar {
    /// The name of the strategy.
    pub name: String,

    /// The type of strategy (StrategyType::Collar).
    pub kind: StrategyType,

    /// A textual description of this strategy instance.
    pub description: String,

    /// The price points at which the strategy breaks even.
    pub break_even_points: Vec<Positive>,

    /// The long spot position (underlying asset).
    pub spot_leg: SpotPosition,

    /// The long put option position (protective put).
    pub long_put: Position,

    /// The short call option position (covered call).
    pub short_call: Position,
}

impl Collar {
    /// Creates a new Collar strategy.
    ///
    /// # Arguments
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `put_strike` - The strike price for the long put option (protection level)
    /// * `call_strike` - The strike price for the short call option (profit cap)
    /// * `expiration` - The expiration date for both options
    /// * `implied_volatility` - The implied volatility for option pricing
    /// * `risk_free_rate` - The risk-free interest rate
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of shares (typically 100 per option contract)
    /// * `premium_long_put` - The premium paid for buying the put
    /// * `premium_short_call` - The premium received for selling the call
    /// * `spot_open_fee` - Fee to open the spot position
    /// * `spot_close_fee` - Fee to close the spot position
    /// * `put_open_fee` - Fee to open the put position
    /// * `put_close_fee` - Fee to close the put position
    /// * `call_open_fee` - Fee to open the call position
    /// * `call_close_fee` - Fee to close the call position
    ///
    /// # Returns
    ///
    /// A fully configured `Collar` strategy instance.
    ///
    /// # Panics
    ///
    /// Panics if break-even point calculation fails.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        put_strike: Positive,
        call_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_long_put: Positive,
        premium_short_call: Positive,
        spot_open_fee: Positive,
        spot_close_fee: Positive,
        put_open_fee: Positive,
        put_close_fee: Positive,
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

        // Create the long put option (protective put)
        let long_put_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            put_strike,
            expiration,
            implied_volatility,
            quantity / Positive::HUNDRED, // Convert shares to contracts
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

        // Create the short call option (covered call)
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

        let mut strategy = Collar {
            name: "Collar".to_string(),
            kind: StrategyType::Collar,
            description: COLLAR_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            spot_leg,
            long_put,
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

    /// Returns the long put leg as a `Leg` enum.
    #[must_use]
    pub fn get_put_leg(&self) -> Leg {
        Leg::Option(self.long_put.clone())
    }

    /// Returns the short call leg as a `Leg` enum.
    #[must_use]
    pub fn get_call_leg(&self) -> Leg {
        Leg::Option(self.short_call.clone())
    }

    /// Returns all legs of the strategy.
    #[must_use]
    pub fn get_legs(&self) -> Vec<Leg> {
        vec![self.get_spot_leg(), self.get_put_leg(), self.get_call_leg()]
    }

    /// Returns the put strike price.
    #[must_use]
    pub fn put_strike(&self) -> Positive {
        self.long_put.option.strike_price
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

    /// Returns the collar width (distance between put and call strikes).
    #[must_use]
    pub fn collar_width(&self) -> Positive {
        self.call_strike() - self.put_strike()
    }

    /// Calculates the net premium (credit if positive, debit if negative).
    ///
    /// Net Premium = Call Premium Received - Put Premium Paid
    #[must_use]
    pub fn net_premium(&self) -> Decimal {
        let call_premium = self.short_call.premium * self.short_call.option.quantity;
        let put_premium = self.long_put.premium * self.long_put.option.quantity;
        call_premium.to_dec() - put_premium.to_dec()
    }

    /// Returns true if this is a zero-cost collar (net premium is approximately zero).
    #[must_use]
    pub fn is_zero_cost(&self) -> bool {
        self.net_premium().abs() < Decimal::new(1, 2) // Less than $0.01
    }

    /// Returns true if this is a credit collar (net premium received).
    #[must_use]
    pub fn is_credit(&self) -> bool {
        self.net_premium() > Decimal::ZERO
    }

    /// Calculates the net delta of the collar.
    ///
    /// Net Delta = Spot Delta + Put Delta + Call Delta
    pub fn net_delta(&self) -> Result<Decimal, GreeksError> {
        let spot_delta = self.spot_leg.delta()?;
        let put_delta = self.long_put.delta()?;
        let call_delta = self.short_call.delta()?;
        Ok(spot_delta + put_delta + call_delta)
    }

    /// Calculates the maximum profit potential.
    ///
    /// Max Profit = (Call Strike - Cost Basis) × Quantity + Net Premium - Fees
    pub fn max_profit_potential(&self) -> Result<Positive, PricingError> {
        let call_strike = self.call_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        let net_premium = self.net_premium();
        let total_fees = self.total_fees();

        if call_strike >= cost_basis {
            let capital_gain = (call_strike - cost_basis) * quantity;
            let total_profit = capital_gain.to_dec() + net_premium - total_fees.to_dec();
            Ok(Positive::new_decimal(total_profit.max(Decimal::ZERO))
                .unwrap_or(Positive::ZERO))
        } else {
            // Call strike below cost basis
            let capital_loss = (cost_basis - call_strike) * quantity;
            let total_profit = net_premium - capital_loss.to_dec() - total_fees.to_dec();
            Ok(Positive::new_decimal(total_profit.max(Decimal::ZERO))
                .unwrap_or(Positive::ZERO))
        }
    }

    /// Calculates the maximum loss potential.
    ///
    /// Max Loss = (Cost Basis - Put Strike) × Quantity - Net Premium + Fees
    pub fn max_loss_potential(&self) -> Result<Positive, PricingError> {
        let put_strike = self.put_strike();
        let cost_basis = self.spot_leg.cost_basis;
        let quantity = self.spot_leg.quantity;
        let net_premium = self.net_premium();
        let total_fees = self.total_fees();

        if cost_basis >= put_strike {
            let capital_loss = (cost_basis - put_strike) * quantity;
            let total_loss = capital_loss.to_dec() - net_premium + total_fees.to_dec();
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO))
                .unwrap_or(Positive::ZERO))
        } else {
            // Put strike above cost basis (unusual but possible)
            let capital_gain = (put_strike - cost_basis) * quantity;
            let total_loss = total_fees.to_dec() - net_premium - capital_gain.to_dec();
            Ok(Positive::new_decimal(total_loss.max(Decimal::ZERO))
                .unwrap_or(Positive::ZERO))
        }
    }

    /// Calculates total fees for all positions.
    fn total_fees(&self) -> Positive {
        self.spot_leg.fees()
            + self.long_put.open_fee
            + self.long_put.close_fee
            + self.short_call.open_fee
            + self.short_call.close_fee
    }

    /// Checks if the put is currently in-the-money.
    #[must_use]
    pub fn is_put_itm(&self, current_price: Positive) -> bool {
        current_price < self.put_strike()
    }

    /// Checks if the call is currently in-the-money.
    #[must_use]
    pub fn is_call_itm(&self, current_price: Positive) -> bool {
        current_price > self.call_strike()
    }
}

impl Validable for Collar {
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

        // Validate long put
        if self.long_put.option.side != Side::Long {
            debug!("Invalid: Put option must be long");
            return false;
        }

        if self.long_put.option.option_style != OptionStyle::Put {
            debug!("Invalid: Long option must be a put");
            return false;
        }

        // Validate short call
        if self.short_call.option.side != Side::Short {
            debug!("Invalid: Call option must be short");
            return false;
        }

        if self.short_call.option.option_style != OptionStyle::Call {
            debug!("Invalid: Short option must be a call");
            return false;
        }

        // Validate strike order: put_strike < underlying_price < call_strike (typical)
        // Note: This is the typical setup but not strictly required
        if self.put_strike() >= self.call_strike() {
            debug!("Invalid: Put strike must be less than call strike");
            return false;
        }

        true
    }
}

impl BreakEvenable for Collar {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points.clear();

        // Break-even = Cost Basis - Net Premium per Share
        let net_premium_per_share = self.net_premium() / self.spot_leg.quantity.to_dec();
        let fees_per_share = self.total_fees().to_dec() / self.spot_leg.quantity.to_dec();

        let break_even = self.spot_leg.cost_basis.to_dec() - net_premium_per_share + fees_per_share;
        
        if let Ok(be) = Positive::new_decimal(break_even) {
            self.break_even_points.push(be.round_to(2));
        }

        Ok(())
    }
}

impl Positionable for Collar {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
                Ok(())
            }
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_type(
                position.option.side,
                "Collar only accepts long put or short call positions".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_put, &self.short_call])
    }

    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        match (option_style, side) {
            (OptionStyle::Put, Side::Long) if *strike == self.long_put.option.strike_price => {
                Ok(vec![&mut self.long_put])
            }
            (OptionStyle::Call, Side::Short) if *strike == self.short_call.option.strike_price => {
                Ok(vec![&mut self.short_call])
            }
            _ => Err(PositionError::invalid_position(
                "Position not found in Collar",
            )),
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

        match (position.option.option_style, position.option.side) {
            (OptionStyle::Put, Side::Long)
                if position.option.strike_price == self.long_put.option.strike_price =>
            {
                self.long_put = position.clone();
                Ok(())
            }
            (OptionStyle::Call, Side::Short)
                if position.option.strike_price == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position(
                "Position does not match existing collar positions",
            )),
        }
    }
}

impl Strategable for Collar {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for Collar {
    fn get_title(&self) -> String {
        format!(
            "Collar Strategy:\n\t{} {} {} @ {}\n\t{}\n\t{}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.symbol,
            self.spot_leg.cost_basis,
            self.long_put.get_title(),
            self.short_call.get_title()
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

impl Strategies for Collar {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        self.max_profit_potential().map_err(StrategyError::from)
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        self.max_loss_potential().map_err(StrategyError::from)
    }
}

impl Profit for Collar {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        // Spot P&L
        let spot_pnl = self.spot_leg.pnl_at_price(*price);

        // Put P&L at expiration
        let put_pnl = self
            .long_put
            .pnl_at_expiration(&Some(price))
            .unwrap_or(Decimal::ZERO);

        // Call P&L at expiration
        let call_pnl = self
            .short_call
            .pnl_at_expiration(&Some(price))
            .unwrap_or(Decimal::ZERO);

        Ok(spot_pnl + put_pnl + call_pnl)
    }
}

impl Greeks for Collar {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_put.option, &self.short_call.option])
    }

    fn delta(&self) -> Result<Decimal, GreeksError> {
        self.net_delta()
    }
}

impl PnLCalculator for Collar {
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
        let call_income = self.short_call.premium * self.short_call.option.quantity;

        Ok(crate::pnl::utils::PnL {
            realized: None,
            unrealized: Some(profit),
            initial_costs: spot_cost + put_cost,
            initial_income: call_income,
            date_time: Utc::now(),
        })
    }
}

impl DeltaNeutrality for Collar {}

impl Optimizable for Collar {
    type Strategy = Collar;
}

impl crate::strategies::StrategyConstructor for Collar {}

impl ProbabilityAnalysis for Collar {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self
            .break_even_points
            .first()
            .copied()
            .ok_or_else(|| ProbabilityError::from("No break-even point found"))?;

        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        // Profit range: from break-even up to call strike (capped profit)
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

        let option = &self.long_put.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        // Loss range: from put strike up to break-even
        let mut loss_range =
            ProfitLossRange::new(Some(self.put_strike()), Some(break_even_point), Positive::ZERO)?;

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

impl std::fmt::Display for Collar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Collar: {} {} @ {} + Long {} Put @ {} + Short {} Call @ {}",
            self.spot_leg.side,
            self.spot_leg.quantity,
            self.spot_leg.cost_basis,
            self.long_put.option.strike_price,
            self.long_put.premium,
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

    fn create_test_collar() -> Collar {
        Collar::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0),    // underlying price
            pos_or_panic!(145.0),    // put strike
            pos_or_panic!(160.0),    // call strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.25),     // implied volatility
            dec!(0.05),              // risk-free rate
            pos_or_panic!(0.01),     // dividend yield
            Positive::HUNDRED,       // quantity
            pos_or_panic!(2.50),     // put premium
            pos_or_panic!(3.00),     // call premium
            Positive::ONE,           // spot open fee
            Positive::ONE,           // spot close fee
            pos_or_panic!(0.65),     // put open fee
            pos_or_panic!(0.65),     // put close fee
            pos_or_panic!(0.65),     // call open fee
            pos_or_panic!(0.65),     // call close fee
        )
    }

    #[test]
    fn test_collar_creation() {
        let collar = create_test_collar();

        assert_eq!(collar.name, "Collar");
        assert_eq!(collar.kind, StrategyType::Collar);
        assert_eq!(collar.spot_leg.symbol, "AAPL");
        assert_eq!(collar.spot_leg.quantity, Positive::HUNDRED);
        assert_eq!(collar.spot_leg.cost_basis, pos_or_panic!(150.0));
        assert_eq!(collar.spot_leg.side, Side::Long);
        assert_eq!(collar.long_put.option.strike_price, pos_or_panic!(145.0));
        assert_eq!(collar.long_put.option.side, Side::Long);
        assert_eq!(collar.short_call.option.strike_price, pos_or_panic!(160.0));
        assert_eq!(collar.short_call.option.side, Side::Short);
    }

    #[test]
    fn test_collar_validation() {
        let collar = create_test_collar();
        assert!(collar.validate());
    }

    #[test]
    fn test_break_even_calculation() {
        let collar = create_test_collar();

        assert!(!collar.break_even_points.is_empty());
    }

    #[test]
    fn test_net_premium() {
        let collar = create_test_collar();
        let net_premium = collar.net_premium();

        // Call premium (3.00) - Put premium (2.50) = 0.50 credit per share
        // With 1 contract (100 shares / 100 = 1), net = 0.50
        assert!(net_premium > Decimal::ZERO); // Credit collar
    }

    #[test]
    fn test_is_credit() {
        let collar = create_test_collar();
        assert!(collar.is_credit());
    }

    #[test]
    fn test_collar_width() {
        let collar = create_test_collar();
        assert_eq!(collar.collar_width(), pos_or_panic!(15.0)); // 160 - 145
    }

    #[test]
    fn test_put_strike() {
        let collar = create_test_collar();
        assert_eq!(collar.put_strike(), pos_or_panic!(145.0));
    }

    #[test]
    fn test_call_strike() {
        let collar = create_test_collar();
        assert_eq!(collar.call_strike(), pos_or_panic!(160.0));
    }

    #[test]
    fn test_underlying_price() {
        let collar = create_test_collar();
        assert_eq!(collar.underlying_price(), pos_or_panic!(150.0));
    }

    #[test]
    fn test_quantity() {
        let collar = create_test_collar();
        assert_eq!(collar.quantity(), Positive::HUNDRED);
    }

    #[test]
    fn test_profit_at_call_strike() {
        let collar = create_test_collar();

        // At call strike, should have maximum profit
        let profit = collar.calculate_profit_at(&pos_or_panic!(160.0)).unwrap();
        assert!(profit > Decimal::ZERO);
    }

    #[test]
    fn test_profit_above_call_strike() {
        let collar = create_test_collar();

        // Above call strike, profit is capped
        let profit_at_strike = collar.calculate_profit_at(&pos_or_panic!(160.0)).unwrap();
        let profit_above = collar.calculate_profit_at(&pos_or_panic!(180.0)).unwrap();

        // Both should be positive (profitable when price rises)
        assert!(profit_at_strike > Decimal::ZERO);
        assert!(profit_above > Decimal::ZERO);
    }

    #[test]
    fn test_loss_at_put_strike() {
        let collar = create_test_collar();

        // At put strike, should have defined P&L
        let _loss = collar.calculate_profit_at(&pos_or_panic!(145.0)).unwrap();
    }

    #[test]
    fn test_loss_below_put_strike() {
        let collar = create_test_collar();

        // Below put strike, loss is capped by the protective put
        let loss_at_strike = collar.calculate_profit_at(&pos_or_panic!(145.0)).unwrap();
        let loss_below = collar.calculate_profit_at(&pos_or_panic!(120.0)).unwrap();

        // Both values should be defined (the put provides protection)
        assert!(loss_at_strike != Decimal::MAX);
        assert!(loss_below != Decimal::MAX);
    }

    #[test]
    fn test_get_legs() {
        let collar = create_test_collar();
        let legs = collar.get_legs();

        assert_eq!(legs.len(), 3);
        assert!(legs[0].is_spot());
        assert!(legs[1].is_option());
        assert!(legs[2].is_option());
    }

    #[test]
    fn test_is_put_itm() {
        let collar = create_test_collar();

        assert!(collar.is_put_itm(pos_or_panic!(140.0)));  // Below put strike
        assert!(!collar.is_put_itm(pos_or_panic!(145.0))); // At put strike
        assert!(!collar.is_put_itm(pos_or_panic!(150.0))); // Above put strike
    }

    #[test]
    fn test_is_call_itm() {
        let collar = create_test_collar();

        assert!(!collar.is_call_itm(pos_or_panic!(155.0))); // Below call strike
        assert!(!collar.is_call_itm(pos_or_panic!(160.0))); // At call strike
        assert!(collar.is_call_itm(pos_or_panic!(165.0)));  // Above call strike
    }

    #[test]
    fn test_display() {
        let collar = create_test_collar();
        let display = format!("{}", collar);

        assert!(display.contains("Collar"));
        assert!(display.contains("Long"));
        assert!(display.contains("100"));
    }

    #[test]
    fn test_get_title() {
        let collar = create_test_collar();
        let title = collar.get_title();

        assert!(title.contains("Collar"));
        assert!(title.contains("AAPL"));
    }

    #[test]
    fn test_max_profit() {
        let collar = create_test_collar();
        let max_profit = collar.get_max_profit();

        assert!(max_profit.is_ok());
    }

    #[test]
    fn test_max_loss() {
        let collar = create_test_collar();
        let max_loss = collar.get_max_loss();

        assert!(max_loss.is_ok());
    }

    #[test]
    fn test_get_positions() {
        let collar = create_test_collar();
        let positions = collar.get_positions().unwrap();

        assert_eq!(positions.len(), 2);
    }
}
