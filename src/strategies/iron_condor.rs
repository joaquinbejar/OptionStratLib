/*
Iron Condor Strategy

An iron condor involves selling an out-of-the-money put spread and an out-of-the-money call spread with the same expiration date.
This strategy is used when little volatility in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential
- Limited risk
- Profit is highest when the underlying asset price remains between the two sold options at expiration
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::{
    ExpirationDate, Options, Positive,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
    error::{
        GreeksError, OperationErrorKind,
        position::{PositionError, PositionValidationErrorKind},
        probability::ProbabilityError,
        strategies::{ProfitLossErrorKind, StrategyError},
    },
    greeks::Greeks,
    model::{
        ProfitLossRange,
        position::Position,
        types::{OptionBasicType, OptionStyle, OptionType, Side},
        utils::mean_and_std,
    },
    pnl::{PnLCalculator, utils::PnL},
    pricing::payoff::Profit,
    strategies::{
        BasicAble, Strategies, StrategyConstructor,
        delta_neutral::DeltaNeutrality,
        probabilities::{core::ProbabilityAnalysis, utils::VolatilityAdjustment},
        utils::{FindOptimalSide, OptimizationCriteria},
    },
};
use chrono::Utc;
use num_traits::FromPrimitive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::{error, info};

pub const IRON_CONDOR_DESCRIPTION: &str = "An Iron Condor is a neutral options strategy combining a bull put spread with a bear call spread. \
    It involves selling an out-of-the-money put and call while buying further out-of-the-money put and call options. \
    This strategy is used when low volatility is expected and the underlying asset's price is anticipated to remain \
    within a specific range.";

/// # IronCondor
///
/// Represents an Iron Condor options trading strategy. This strategy involves four options positions:
/// selling an out-of-the-money put, buying a further out-of-the-money put, selling an out-of-the-money call,
/// and buying a further out-of-the-money call - all with the same expiration date.
///
/// An Iron Condor is a neutral strategy that profits from low volatility, where the underlying asset price
/// remains within a specific range until expiration.
///
/// ## Fields
/// * `name`: A descriptive name for the specific strategy instance.
/// * `kind`: The type of strategy, which is `StrategyType::IronCondor`.
/// * `description`: A detailed description of this specific strategy instance.
/// * `break_even_points`: The price points at which the strategy breaks even (neither profit nor loss).
/// * `short_call`: The short call position component of the strategy.
/// * `short_put`: The short put position component of the strategy.
/// * `long_call`: The long call position component of the strategy (typically with a higher strike than the short call).
/// * `long_put`: The long put position component of the strategy (typically with a lower strike than the short put).
///
/// ## Risk Profile
/// An Iron Condor has limited risk and limited profit potential. The maximum loss is limited to the difference between
/// either the call strikes or the put strikes (whichever is greater) minus the net credit received.
///
/// ## Profit Profile
/// The maximum profit is limited to the net premium (credit) received when establishing the position. This maximum profit
/// is realized when all options expire worthless, which occurs when the underlying asset's price at expiration is between
/// the short put and short call strike prices.
///
/// ## Break-Even Points
/// There are two break-even points in an Iron Condor:
/// 1. Upper break-even point: Short call strike price + net premium received
/// 2. Lower break-even point: Short put strike price - net premium received
///
/// ## Typical Usage
/// This strategy is typically used when:
/// - The trader expects low volatility in the underlying asset
/// - The trader believes the price will remain within a specific range
/// - Implied volatility is relatively high (making the sold options more valuable)
/// - The trader wants to generate income while having defined risk parameters
///
/// ## Construction
/// An Iron Condor consists of:
/// - Selling an out-of-the-money put (short put)
/// - Buying a further out-of-the-money put (long put)
/// - Selling an out-of-the-money call (short call)
/// - Buying a further out-of-the-money call (long call)
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct IronCondor {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as an IronCondor strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call leg of the strategy (middle-upper strike)
    pub short_call: Position,
    /// The short put leg of the strategy (middle-lower strike)
    pub short_put: Position,
    /// The long call leg of the strategy (highest strike)
    pub long_call: Position,
    /// The long put leg of the strategy (lowest strike)
    pub long_put: Position,
}

impl IronCondor {
    /// # Creates a new Iron Condor options strategy
    ///
    /// This function constructs a new Iron Condor strategy, which combines a bull put spread with a bear call spread.
    /// It involves selling an out-of-the-money put and call while buying further out-of-the-money put and call options.
    ///
    /// ## Parameters
    ///
    /// * `underlying_symbol`: The symbol/ticker of the underlying asset
    /// * `underlying_price`: The current price of the underlying asset
    /// * `short_call_strike`: Strike price for the short call option
    /// * `short_put_strike`: Strike price for the short put option
    /// * `long_call_strike`: Strike price for the long call option
    /// * `long_put_strike`: Strike price for the long put option
    /// * `expiration`: The expiration date for all options in the strategy
    /// * `implied_volatility`: The implied volatility used for option pricing
    /// * `risk_free_rate`: Risk-free interest rate used in option pricing models
    /// * `dividend_yield`: Expected dividend yield of the underlying asset
    /// * `quantity`: Number of contracts for each option position
    /// * `premium_short_call`: Premium received for selling the call option
    /// * `premium_short_put`: Premium received for selling the put option
    /// * `premium_long_call`: Premium paid for buying the call option
    /// * `premium_long_put`: Premium paid for buying the put option
    /// * `open_fee`: Transaction fee for opening each position
    /// * `close_fee`: Transaction fee for closing each position
    ///
    /// ## Returns
    ///
    /// Returns a configured `IronCondor` strategy instance with all positions initialized and break-even points calculated.
    ///
    /// ## Strategy Details
    ///
    /// An Iron Condor is a neutral options strategy that profits when the underlying asset price
    /// remains within a specific range until expiration. The strategy:
    ///
    /// * Has limited risk (difference between strikes minus net premium received)
    /// * Has limited profit potential (net premium received minus fees)
    /// * Achieves maximum profit when the underlying price stays between the short strikes
    /// * Has two break-even points calculated based on short strikes and net premium received
    /// * Is typically used in low volatility environments when minimal price movement is expected
    ///
    /// ## Structure
    ///
    /// The strategy consists of four legs:
    /// 1. Short Call: Selling an out-of-the-money call option
    /// 2. Long Call: Buying a further out-of-the-money call option (protection)
    /// 3. Short Put: Selling an out-of-the-money put option
    /// 4. Long Put: Buying a further out-of-the-money put option (protection)
    ///
    /// The short options generate income, while the long options limit the potential loss.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        short_call_strike: Positive,
        short_put_strike: Positive,
        long_call_strike: Positive,
        long_put_strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        premium_short_put: Positive,
        premium_long_call: Positive,
        premium_long_put: Positive,
        open_fee: Positive,
        close_fee: Positive,
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
            open_fee,
            close_fee,
            None,
            None,
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
            open_fee,
            close_fee,
            None,
            None,
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
            expiration,
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
            None,
            None,
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
            None,
            None,
        );
        strategy
            .add_position(&long_put.clone())
            .expect("Invalid long put");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for IronCondor {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 4 options for an Iron Condor
        if vec_positions.len() != 4 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Condor get_strategy".to_string(),
                    reason: "Must have exactly 4 options".to_string(),
                },
            ));
        }

        // Sort options by strike price to identify each position
        let mut sorted_options = vec_positions.to_vec();
        sorted_options.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let lowest_strike = &sorted_options[0];
        let lower_middle_strike = &sorted_options[1];
        let upper_middle_strike = &sorted_options[2];
        let highest_strike = &sorted_options[3];

        // Validate option types
        if lowest_strike.option.option_style != OptionStyle::Put
            || lower_middle_strike.option.option_style != OptionStyle::Put
            || upper_middle_strike.option.option_style != OptionStyle::Call
            || highest_strike.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Condor get_strategy".to_string(),
                    reason: "Invalid option types for Iron Condor".to_string(),
                },
            ));
        }

        // Validate option sides
        if lowest_strike.option.side != Side::Long
            || lower_middle_strike.option.side != Side::Short
            || upper_middle_strike.option.side != Side::Short
            || highest_strike.option.side != Side::Long
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Condor get_strategy".to_string(),
                    reason: "Invalid option sides for Iron Condor".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        let expiry_date = lowest_strike.option.expiration_date;
        if !sorted_options
            .iter()
            .all(|pos| pos.option.expiration_date == expiry_date)
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Condor get_strategy".to_string(),
                    reason: "All options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_put = Position::new(
            lowest_strike.option.clone(),
            lowest_strike.premium,
            Utc::now(),
            lowest_strike.open_fee,
            lowest_strike.close_fee,
            lowest_strike.epic.clone(),
            lowest_strike.extra_fields.clone(),
        );

        let short_put = Position::new(
            lower_middle_strike.option.clone(),
            lower_middle_strike.premium,
            Utc::now(),
            lower_middle_strike.open_fee,
            lower_middle_strike.close_fee,
            lower_middle_strike.epic.clone(),
            lower_middle_strike.extra_fields.clone(),
        );

        let short_call = Position::new(
            upper_middle_strike.option.clone(),
            upper_middle_strike.premium,
            Utc::now(),
            upper_middle_strike.open_fee,
            upper_middle_strike.close_fee,
            upper_middle_strike.epic.clone(),
            upper_middle_strike.extra_fields.clone(),
        );

        let long_call = Position::new(
            highest_strike.option.clone(),
            highest_strike.premium,
            Utc::now(),
            highest_strike.open_fee,
            highest_strike.close_fee,
            highest_strike.epic.clone(),
            highest_strike.extra_fields.clone(),
        );

        // Create strategy
        let mut strategy = IronCondor {
            name: "Iron Condor".to_string(),
            kind: StrategyType::IronCondor,
            description: "Iron Condor strategy description".to_string(), // Replace with actual description
            break_even_points: Vec::new(),
            long_put,
            short_put,
            short_call,
            long_call,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for IronCondor {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let net_credit = self.get_net_premium_received()? / self.short_call.option.quantity;

        self.break_even_points
            .push((self.short_call.option.strike_price + net_credit).round_to(2));

        self.break_even_points
            .push((self.short_put.option.strike_price - net_credit).round_to(2));

        self.break_even_points.sort();
        Ok(())
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
        match (position.option.option_style, position.option.side) {
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

    /// Gets mutable positions matching the specified criteria from the strategy.
    ///
    /// # Arguments
    /// * `option_style` - The style of the option (Put/Call)
    /// * `side` - The side of the position (Long/Short)
    /// * `strike` - The strike price of the option
    ///
    /// # Returns
    /// * `Ok(Vec<&mut Position>)` - A vector containing mutable references to matching positions
    /// * `Err(PositionError)` - If there was an error retrieving positions
    fn get_position(
        &mut self,
        option_style: &OptionStyle,
        side: &Side,
        strike: &Positive,
    ) -> Result<Vec<&mut Position>, PositionError> {
        match (side, option_style, strike) {
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
            }
            (Side::Short, OptionStyle::Put, strike)
                if *strike == self.short_put.option.strike_price =>
            {
                Ok(vec![&mut self.short_put])
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (Side::Long, OptionStyle::Put, strike)
                if *strike == self.long_put.option.strike_price =>
            {
                Ok(vec![&mut self.long_put])
            }
            _ => Err(PositionError::invalid_position_type(
                *side,
                "Strike not found in positions".to_string(),
            )),
        }
    }

    /// Modifies an existing position in the strategy.
    ///
    /// # Arguments
    /// * `position` - The new position data to update
    ///
    /// # Returns
    /// * `Ok(())` if position was successfully modified
    /// * `Err(PositionError)` if position was not found or validation failed
    fn modify_position(&mut self, position: &Position) -> Result<(), PositionError> {
        if !position.validate() {
            return Err(PositionError::ValidationError(
                PositionValidationErrorKind::InvalidPosition {
                    reason: "Invalid position data".to_string(),
                },
            ));
        }

        if position.option.strike_price != self.long_call.option.strike_price
            && position.option.strike_price != self.long_put.option.strike_price
            && position.option.strike_price != self.short_call.option.strike_price
            && position.option.strike_price != self.short_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        match (&position.option.option_style, &position.option.side) {
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
            }
            (OptionStyle::Put, Side::Short) => {
                self.short_put = position.clone();
            }
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
            }
            (OptionStyle::Put, Side::Long) => {
                self.long_put = position.clone();
            }
        }

        Ok(())
    }
}

impl Strategable for IronCondor {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for IronCondor {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [
            self.short_call.get_title(),
            self.short_put.get_title(),
            self.long_call.get_title(),
            self.long_put.get_title(),
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
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType> {
        let mut hash_set = HashSet::new();
        let short_call = &self.short_call.option;
        let short_put = &self.short_put.option;
        let long_call = &self.long_call.option;
        let long_put = &self.long_put.option;
        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &short_put.option_style,
            side: &short_put.side,
            strike_price: &short_put.strike_price,
            expiration_date: &short_put.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &long_call.option_style,
            side: &long_call.side,
            strike_price: &long_call.strike_price,
            expiration_date: &long_call.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &long_put.option_style,
            side: &long_put.side,
            strike_price: &long_put.strike_price,
            expiration_date: &long_put.expiration_date,
        });

        hash_set
    }
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType, &Positive> {
        let options = [
            (
                &self.short_call.option,
                &self.short_call.option.implied_volatility,
            ),
            (
                &self.short_put.option,
                &self.short_put.option.implied_volatility,
            ),
            (
                &self.long_call.option,
                &self.long_call.option.implied_volatility,
            ),
            (
                &self.long_put.option,
                &self.long_put.option.implied_volatility,
            ),
        ];

        options
            .into_iter()
            .map(|(option, iv)| {
                (
                    OptionBasicType {
                        option_style: &option.option_style,
                        side: &option.side,
                        strike_price: &option.strike_price,
                        expiration_date: &option.expiration_date,
                    },
                    iv,
                )
            })
            .collect()
    }
    fn get_quantity(&self) -> HashMap<OptionBasicType, &Positive> {
        let options = [
            (&self.short_call.option, &self.short_call.option.quantity),
            (&self.short_put.option, &self.short_put.option.quantity),
            (&self.long_call.option, &self.long_call.option.quantity),
            (&self.long_put.option, &self.long_put.option.quantity),
        ];

        options
            .into_iter()
            .map(|(option, quantity)| {
                (
                    OptionBasicType {
                        option_style: &option.option_style,
                        side: &option.side,
                        strike_price: &option.strike_price,
                        expiration_date: &option.expiration_date,
                    },
                    quantity,
                )
            })
            .collect()
    }
    fn one_option(&self) -> &Options {
        self.short_call.one_option()
    }
    fn one_option_mut(&mut self) -> &mut Options {
        self.short_call.one_option_mut()
    }
    fn set_expiration_date(
        &mut self,
        expiration_date: ExpirationDate,
    ) -> Result<(), StrategyError> {
        self.short_call.option.expiration_date = expiration_date;
        self.short_put.option.expiration_date = expiration_date;
        self.long_call.option.expiration_date = expiration_date;
        self.long_put.option.expiration_date = expiration_date;
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.underlying_price = *price;
        self.short_call.premium = Positive::from(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );

        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::from(self.short_put.option.calculate_price_black_scholes()?.abs());

        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::from(self.long_call.option.calculate_price_black_scholes()?.abs());
        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::from(self.long_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.short_put.option.implied_volatility = *volatility;
        self.long_call.option.implied_volatility = *volatility;
        self.long_put.option.implied_volatility = *volatility;

        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_put.premium =
            Positive(self.short_put.option.calculate_price_black_scholes()?.abs());
        self.long_call.premium =
            Positive(self.long_call.option.calculate_price_black_scholes()?.abs());
        self.long_put.premium =
            Positive(self.long_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Strategies for IronCondor {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let left_profit = self.calculate_profit_at(&self.short_call.option.strike_price)?;
        let right_profit = self.calculate_profit_at(&self.short_put.option.strike_price)?;
        if left_profit < Decimal::ZERO || right_profit < Decimal::ZERO {
            return Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ));
        }

        Ok(self
            .calculate_profit_at(&self.short_call.option.strike_price)?
            .into())
    }

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let left_loss = self.calculate_profit_at(&self.long_put.option.strike_price)?;
        let right_loss = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if left_loss > Decimal::ZERO || right_loss > Decimal::ZERO {
            return Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ));
        }
        let result = left_loss.abs().max(right_loss.abs());
        Ok(result.into())
    }

    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let inner_width =
            (self.short_call.option.strike_price - self.short_put.option.strike_price).to_f64();
        let outer_width =
            (self.long_call.option.strike_price - self.long_put.option.strike_price).to_f64();
        let height = self.get_max_profit().unwrap_or(Positive::ZERO);

        let inner_area = inner_width * height;
        let outer_triangles = (outer_width - inner_width) * height / 2.0;

        let result =
            (inner_area + outer_triangles) / self.short_call.option.underlying_price.to_f64();
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let max_profit = self.get_max_profit().unwrap_or(Positive::ZERO);
        let max_loss = self.get_max_loss().unwrap_or(Positive::ZERO);
        match (max_profit, max_loss) {
            (value, _) if value == Positive::ZERO => Ok(Decimal::ZERO),
            (_, value) if value == Positive::ZERO => Ok(Decimal::MAX),
            _ => Ok((max_profit / max_loss * 100.0).into()),
        }
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
                if side == FindOptimalSide::Center {
                    long_put.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short_put
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short_call
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                        && long_call
                            .is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    long_put.is_valid_optimal_side(underlying_price, &side)
                        && short_put.is_valid_optimal_side(underlying_price, &side)
                        && short_call.is_valid_optimal_side(underlying_price, &side)
                        && long_call.is_valid_optimal_side(underlying_price, &side)
                }
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
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
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
                OptimizationCriteria::Ratio => strategy.get_profit_ratio().unwrap(),
                OptimizationCriteria::Area => strategy.get_profit_area().unwrap(),
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
            } => {
                let implied_volatility = short_call.implied_volatility;
                assert!(implied_volatility <= Positive::ONE);

                IronCondor::new(
                    chain.symbol.clone(),
                    chain.underlying_price,
                    short_call.strike_price,
                    short_put.strike_price,
                    long_call.strike_price,
                    long_put.strike_price,
                    self.short_call.option.expiration_date,
                    implied_volatility,
                    self.short_call.option.risk_free_rate,
                    self.short_call.option.dividend_yield,
                    self.short_call.option.quantity,
                    short_call.call_bid.unwrap(),
                    short_put.put_bid.unwrap(),
                    long_call.call_ask.unwrap(),
                    long_put.put_ask.unwrap(),
                    self.get_fees().unwrap() / 8.0,
                    self.get_fees().unwrap() / 8.0,
                )
            }
            _ => panic!("Invalid number of legs for Iron Condor strategy"),
        }
    }
}

impl Profit for IronCondor {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        Ok(self.short_call.pnl_at_expiration(&price)?
            + self.short_put.pnl_at_expiration(&price)?
            + self.long_call.pnl_at_expiration(&price)?
            + self.long_put.pnl_at_expiration(&price)?)
    }
}

impl ProbabilityAnalysis for IronCondor {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call.option.implied_volatility,
            self.short_put.option.implied_volatility,
            self.long_call.option.implied_volatility,
            self.long_put.option.implied_volatility,
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
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = self.get_break_even_points()?;
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.short_call.option.implied_volatility,
            self.short_put.option.implied_volatility,
            self.long_call.option.implied_volatility,
            self.long_put.option.implied_volatility,
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
            expiration_date,
            Some(risk_free_rate),
        )?;

        loss_range_upper.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range_lower, loss_range_upper])
    }
}

impl Greeks for IronCondor {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.short_call.option,
            &self.short_put.option,
            &self.long_call.option,
            &self.long_put.option,
        ])
    }
}

impl DeltaNeutrality for IronCondor {}

impl PnLCalculator for IronCondor {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .long_put
                .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .short_call
                .calculate_pnl(market_price, expiration_date, implied_volatility)?
            + self
                .short_put
                .calculate_pnl(market_price, expiration_date, implied_volatility)?)
    }

    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
            .long_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .long_put
                .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_call
                .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

#[cfg(test)]
mod tests_iron_condor {
    use super::*;
    use crate::pos;
    use chrono::{TimeZone, Utc};
    use rust_decimal_macros::dec;

    #[test]
    fn test_iron_condor_creation() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            Positive::ONE,
            pos!(1.5),
            Positive::ONE,
            Positive::TWO,
            pos!(1.8),
            pos!(5.0),
            pos!(5.0),
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
            pos!(150.0),
            pos!(120.0),
            pos!(110.0),
            pos!(130.0),
            pos!(100.0),
            ExpirationDate::DateTime(date),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            Positive::ONE,
            pos!(1.5),
            Positive::ONE,
            Positive::TWO,
            pos!(1.8),
            pos!(5.0),
            pos!(5.0),
        );

        assert_eq!(iron_condor.get_max_loss().unwrap_or(Positive::ZERO), 51.3);
    }

    #[test]
    fn test_max_profit() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            Positive::ONE,
            pos!(3.5),
            pos!(3.3),
            pos!(3.0),
            pos!(2.8),
            pos!(0.07),
            pos!(0.07),
        );

        let expected_profit = iron_condor.get_net_premium_received().unwrap().to_f64();
        assert_eq!(
            iron_condor.get_max_profit().unwrap_or(Positive::ZERO),
            expected_profit
        );
    }

    #[test]
    fn test_get_break_even_points() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            Positive::ONE,
            pos!(1.5),
            Positive::ONE,
            Positive::TWO,
            pos!(1.8),
            pos!(5.0),
            pos!(5.0),
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
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            Positive::ONE,
            pos!(1.5),
            Positive::ONE,
            Positive::TWO,
            pos!(1.8),
            pos!(5.0),
            pos!(5.0),
        );

        let expected_fees = iron_condor.short_call.open_fee
            + iron_condor.short_call.close_fee
            + iron_condor.short_put.open_fee
            + iron_condor.short_put.close_fee
            + iron_condor.long_call.open_fee
            + iron_condor.long_call.close_fee
            + iron_condor.long_put.open_fee
            + iron_condor.long_put.close_fee;
        assert_eq!(iron_condor.get_fees().unwrap().to_f64(), expected_fees);
    }

    #[test]
    fn test_calculate_profit_at() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let iron_condor = IronCondor::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(155.0),
            pos!(145.0),
            pos!(160.0),
            pos!(140.0),
            ExpirationDate::DateTime(date),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            Positive::ONE,
            pos!(1.5),
            Positive::ONE,
            Positive::TWO,
            pos!(1.8),
            pos!(5.0),
            pos!(5.0),
        );

        let price = pos!(150.0);
        let expected_profit = iron_condor
            .short_call
            .pnl_at_expiration(&Some(&price))
            .unwrap()
            + iron_condor
                .short_put
                .pnl_at_expiration(&Some(&price))
                .unwrap()
            + iron_condor
                .long_call
                .pnl_at_expiration(&Some(&price))
                .unwrap()
            + iron_condor
                .long_put
                .pnl_at_expiration(&Some(&price))
                .unwrap();
        assert_eq!(
            iron_condor.calculate_profit_at(&price).unwrap(),
            expected_profit
        );
    }
}

#[cfg(test)]
mod tests_iron_condor_validable {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use rust_decimal_macros::dec;

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
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                quantity,
                pos!(100.0),
                dec!(0.05),
                option_style,
                Positive::ZERO,
                None,
            ),
            Positive::ONE,
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        )
    }

    fn create_valid_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            Positive::TWO,  // premium_short_call
            Positive::TWO,  // premium_short_put
            Positive::ONE,  // premium_long_call
            Positive::ONE,  // premium_long_put
            Positive::ZERO, // open_fee
            Positive::ZERO, // closing fee
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
            create_valid_position(Side::Short, OptionStyle::Call, pos!(105.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_invalid_short_put() {
        let mut condor = create_valid_condor();
        // Make short put invalid by setting quantity to zero
        condor.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, pos!(95.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_invalid_long_call() {
        let mut condor = create_valid_condor();
        // Make long call invalid by setting quantity to zero
        condor.long_call =
            create_valid_position(Side::Long, OptionStyle::Call, pos!(110.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_invalid_long_put() {
        let mut condor = create_valid_condor();
        // Make long put invalid by setting quantity to zero
        condor.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, pos!(90.0), Positive::ZERO);
        assert!(!condor.validate());
    }

    #[test]
    fn test_validate_all_invalid() {
        let mut condor = create_valid_condor();
        // Make all positions invalid
        condor.short_call =
            create_valid_position(Side::Short, OptionStyle::Call, pos!(105.0), Positive::ZERO);
        condor.short_put =
            create_valid_position(Side::Short, OptionStyle::Put, pos!(95.0), Positive::ZERO);
        condor.long_call =
            create_valid_position(Side::Long, OptionStyle::Call, pos!(110.0), Positive::ZERO);
        condor.long_put =
            create_valid_position(Side::Long, OptionStyle::Put, pos!(90.0), Positive::ZERO);
        assert!(!condor.validate());
    }
}

#[cfg(test)]
mod tests_iron_condor_strategies {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            Positive::TWO,  // premium_short_call
            Positive::TWO,  // premium_short_put
            Positive::ONE,  // premium_long_call
            Positive::ONE,  // premium_long_put
            pos!(0.5),      // open_fee
            pos!(0.5),      // closing fee
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
                pos!(106.0),
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos!(2.5),
            Utc::now(),
            pos!(0.5),
            pos!(0.5),
            None,
            None,
        );
        condor
            .add_position(&new_short_call.clone())
            .expect("Invalid short call");
        assert_eq!(condor.short_call.option.strike_price, pos!(106.0));

        // Test adding a long put
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos!(89.0),
                ExpirationDate::Days(pos!(30.0)),
                pos!(0.2),
                pos!(1.0),
                pos!(100.0),
                dec!(0.05),
                OptionStyle::Put,
                Positive::ZERO,
                None,
            ),
            pos!(1.5),
            Utc::now(),
            pos!(0.5),
            pos!(0.5),
            None,
            None,
        );
        condor
            .add_position(&new_long_put.clone())
            .expect("Invalid long put");
        assert_eq!(condor.long_put.option.strike_price, pos!(89.0));
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
        let condor = IronCondor::new(
            "GOLD".to_string(),
            pos!(2646.9), // underlying_price
            pos!(2725.0), // short_call_strike
            pos!(2560.0), // short_put_strike
            pos!(2800.0), // long_call_strike
            pos!(2500.0), // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.1548),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(38.8),     // premium_short_call
            pos!(30.4),     // premium_short_put
            pos!(23.3),     // premium_long_call
            pos!(16.8),     // premium_long_put
            pos!(0.96),     // open_fee
            pos!(0.96),     // close_fee
        );
        let break_even_points = condor.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        assert!(break_even_points[0] < condor.short_put.option.strike_price);
        assert!(break_even_points[1] > condor.short_call.option.strike_price);
    }

    #[test]
    fn test_max_profit() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(10.0),     // premium_short_put
            pos!(10.0),     // premium_long_call
            pos!(10.0),     // premium_long_put
            Positive::ZERO, // open_fee
            Positive::ZERO, // closing fee
        );
        let max_profit = condor.get_max_profit().unwrap();
        assert_eq!(max_profit, pos!(ZERO));
    }

    #[test]
    fn test_max_profit_bis() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(20.0),     // premium_short_call
            pos!(20.0),     // premium_short_put
            pos!(10.0),     // premium_long_call
            pos!(10.0),     // premium_long_put
            pos!(0.09),     // open_fee
            pos!(0.09),     // closing fee
        );
        let max_profit = condor.get_max_profit().unwrap();
        assert_eq!(max_profit, pos!(19.28));
    }

    #[test]
    fn test_max_loss() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(10.0),     // premium_short_put
            pos!(11.1),     // premium_long_call
            pos!(11.1),     // premium_long_put
            pos!(0.1),      // open_fee
            pos!(0.1),      // closing fee
        );
        let max_loss = condor.get_max_loss().unwrap();
        assert_eq!(max_loss, pos!(7.9999999999999964));
    }

    #[test]
    fn test_max_loss_with_uneven_wings() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            pos!(115.0), // Wider call wing
            pos!(90.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        );

        let max_loss = condor.get_max_loss().unwrap();
        assert_eq!(max_loss, pos!(12.0));
    }

    #[test]
    fn test_total_cost() {
        let condor = create_test_condor();
        // Total cost = 2.0 + 2.0 + 1.0 + 1.0 = 6.0
        assert_eq!(condor.get_total_cost().unwrap(), pos!(6.0));
    }

    #[test]
    fn test_net_premium_received() {
        let condor = create_test_condor();
        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    fn test_net_premium_received_bis_i() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(10.0),     // premium_short_put
            pos!(10.0),     // premium_long_call
            pos!(10.0),     // premium_long_put
            Positive::ZERO, // open_fee
            Positive::ZERO, // closing fee
        );
        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), ZERO);
    }

    #[test]
    fn test_net_premium_received_bis_ii() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(10.0),     // premium_short_put
            pos!(10.0),     // premium_long_call
            pos!(10.0),     // premium_long_put
            Positive::ONE,  // open_fee
            Positive::ONE,  // closing fee
        );
        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    fn test_net_premium_received_bis_iii() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(20.0),     // premium_short_put
            pos!(20.0),     // premium_long_call
            pos!(10.0),     // premium_long_put
            Positive::ONE,  // open_fee
            Positive::ONE,  // closing fee
        );
        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    fn test_net_premium_received_bis_iv() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(20.0),     // premium_short_put
            pos!(10.0),     // premium_long_call
            pos!(10.0),     // premium_long_put
            Positive::ONE,  // open_fee
            Positive::ONE,  // closing fee
        );
        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), 2.0);
    }

    #[test]
    fn test_net_premium_received_bis_v() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(10.0),     // premium_short_call
            pos!(10.0),     // premium_short_put
            pos!(10.0),     // premium_long_call
            pos!(20.0),     // premium_long_put
            Positive::ONE,  // open_fee
            Positive::ONE,  // closing fee
        );
        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    fn test_fees() {
        let condor = create_test_condor();
        // Total fees = (0.5 + 0.5) * 4 = 4.0
        assert_eq!(condor.get_fees().unwrap().to_f64(), 4.0);
    }

    #[test]
    fn test_profit_area() {
        let condor = create_test_condor();
        assert_eq!(condor.get_profit_area().unwrap().to_f64().unwrap(), 0.0);
    }

    #[test]
    fn test_best_range_to_show() {
        let condor = create_test_condor();
        let range = condor.get_best_range_to_show(pos!(1.0)).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] < condor.long_put.option.strike_price);
        assert!(range[range.len() - 1] > condor.long_call.option.strike_price);
    }

    #[test]
    fn test_with_multiple_contracts() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            pos!(110.0),
            pos!(90.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            pos!(0.5),
            pos!(0.5),
        );

        assert!(condor.get_max_profit().is_err());
        assert_eq!(condor.get_max_loss().unwrap(), pos!(14.0));
    }

    #[test]
    fn test_with_no_premium() {
        let mut condor = create_test_condor();
        condor.short_call.premium = Positive::ONE;
        condor.short_put.premium = Positive::ONE;
        condor.long_call.premium = Positive::ONE;
        condor.long_put.premium = Positive::ONE;

        assert_eq!(condor.get_net_premium_received().unwrap().to_f64(), 0.0);
        assert!(condor.get_max_profit().is_err());
    }
}

#[cfg(test)]
mod tests_iron_condor_optimizable {
    use super::*;
    use crate::chains::OptionData;
    use crate::model::ExpirationDate;
    use crate::{pos, spos};

    use rust_decimal_macros::dec;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            Positive::TWO,  // premium_short_call
            Positive::TWO,  // premium_short_put
            Positive::ONE,  // premium_long_call
            Positive::ONE,  // premium_long_put
            pos!(0.5),      // open_fee
            pos!(0.5),      // closing fee
        )
    }

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new("TEST", pos!(100.0), "2024-12-31".to_string(), None, None);

        // Add options at various strikes
        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                pos!(strike),
                spos!(5.0), // call_bid
                spos!(5.2), // call_ask
                spos!(5.0), // put_bid
                spos!(5.2), // put_ask
                pos!(0.2),  // implied_volatility
                None,       // delta
                None,
                None,
                spos!(100.0), // volume
                Some(50),     // open_interest
                None,
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
            FindOptimalSide::Range(pos!(95.0), pos!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(condor.validate());
        assert!(condor.short_put.option.strike_price >= pos!(95.0));
        assert!(condor.short_call.option.strike_price <= pos!(105.0));
    }

    #[test]
    fn test_find_optimal_by_area() {
        let mut condor = create_test_condor();
        let chain = create_test_chain();

        let initial_area = condor.get_profit_area().unwrap();
        condor.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(condor.validate());
        assert!(condor.get_profit_area().unwrap() >= initial_area);
    }

    #[test]
    fn test_is_valid_long_option() {
        let condor = create_test_condor();
        let option = OptionData::new(
            pos!(90.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            pos!(0.2),
            None,
            None,
            None,
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // Test with different sides
        assert!(condor.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(condor.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(!condor.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
        assert!(
            condor
                .is_valid_optimal_option(&option, &FindOptimalSide::Range(pos!(85.0), pos!(95.0)))
        );
    }

    #[test]
    fn test_is_valid_short_option() {
        let condor = create_test_condor();
        let option = OptionData::new(
            pos!(105.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            pos!(0.2),
            None,
            None,
            None,
            spos!(100.0),
            Some(50),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );

        // Test with different sides
        assert!(condor.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(!condor.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(condor.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
        assert!(
            condor.is_valid_optimal_option(
                &option,
                &FindOptimalSide::Range(pos!(100.0), pos!(110.0))
            )
        );
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
        assert_eq!(new_strategy.long_put.option.strike_price, pos!(90.0));
        assert_eq!(new_strategy.short_put.option.strike_price, pos!(95.0));
        assert_eq!(new_strategy.short_call.option.strike_price, pos!(105.0));
        assert_eq!(new_strategy.long_call.option.strike_price, pos!(110.0));
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
    use crate::model::ExpirationDate;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(105.0), // short_call_strike
            pos!(95.0),  // short_put_strike
            pos!(110.0), // long_call_strike
            pos!(90.0),  // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            Positive::TWO,  // premium_short_call
            Positive::TWO,  // premium_short_put
            Positive::ONE,  // premium_long_call
            Positive::ONE,  // premium_long_put
            Positive::ZERO, // open_fee
            Positive::ZERO, // closing fee
        )
    }

    #[test]
    fn test_profit_at_max_profit_price() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // Net premium = (2.0 + 2.0) - (1.0 + 1.0) = 2.0
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_below_long_put() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // (95 - 90) - net_premium = 5 - 2 = 3
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_at_long_put() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(90.0))
            .unwrap()
            .to_f64()
            .unwrap();

        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_between_puts() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(92.5))
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(profit > -3.0 && profit < 2.0);
    }

    #[test]
    fn test_profit_at_short_put() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(95.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_in_profit_zone() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_at_short_call() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(105.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_between_calls() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(107.5))
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(profit > -3.0 && profit < 2.0);
    }

    #[test]
    fn test_profit_at_long_call() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(110.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // (110 - 105) - net_premium = 5 - 2 = 3
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_above_long_call() {
        let condor = create_test_condor();
        let profit = condor
            .calculate_profit_at(&pos!(115.0))
            .unwrap()
            .to_f64()
            .unwrap();
        assert_eq!(profit, -3.0);
    }

    #[test]
    fn test_profit_with_fees() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            pos!(110.0),
            pos!(90.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(1.0),
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            pos!(0.5), // open_fee
            pos!(0.5), // closing fee
        );

        let profit = condor
            .calculate_profit_at(&pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();

        // Net premium = 2.0 - fees = 2.0 - 4.0 = -2.0
        assert_eq!(profit, -2.0);
    }

    #[test]
    fn test_profit_with_fees_qty() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            pos!(110.0),
            pos!(90.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(3.0),
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            pos!(0.5), // open_fee
            pos!(0.5), // closing fee
        );

        let profit = condor
            .calculate_profit_at(&pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // Net premium = 6.0 - fees = 6.0 - 12.0 = -6.0
        assert_eq!(profit, -6.0);
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let condor = IronCondor::new(
            "TEST".to_string(),
            pos!(100.0),
            pos!(105.0),
            pos!(95.0),
            pos!(110.0),
            pos!(90.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos!(2.0), // quantity = 2
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            Positive::ZERO,
            Positive::ZERO,
        );

        let profit = condor
            .calculate_profit_at(&pos!(100.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // Net premium * quantity = 2.0 * 2 = 4.0
        assert_eq!(profit, 4.0);
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let condor = create_test_condor();

        let lower_break_even = pos!(93.0); // 95 - 2
        let upper_break_even = pos!(107.0); // 105 + 2

        let lower_profit = condor
            .calculate_profit_at(&lower_break_even)
            .unwrap()
            .to_f64()
            .unwrap();

        let upper_profit = condor
            .calculate_profit_at(&upper_break_even)
            .unwrap()
            .to_f64()
            .unwrap();

        assert!(lower_profit.abs() < 0.001);
        assert!(upper_profit.abs() < 0.001);
    }
}

#[cfg(test)]
mod tests_iron_condor_delta {
    use super::*;
    use crate::strategies::{DELTA_THRESHOLD, DeltaAdjustment};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            underlying_price, // underlying_price
            pos!(2725.0),     // short_call_strike
            pos!(2560.0),     // short_put_strike
            pos!(2800.0),     // long_call_strike
            pos!(2500.0),     // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.1548),   // implied_volatility
            Decimal::ZERO,  // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(38.8),     // premium_short_call
            pos!(30.4),     // premium_short_put
            pos!(23.3),     // premium_long_call
            pos!(16.8),     // premium_long_put
            pos!(0.96),     // open_fee
            pos!(0.96),     // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(2800.0));
        let size = dec!(-0.2124);
        let delta1 = pos!(0.2883434684073368);
        let delta2 = pos!(10.312_565_341_673_713);
        let k1 = pos!(2725.0);
        let k2 = pos!(2560.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[0] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
        option.quantity = delta2;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(2500.0));
        let size = dec!(0.18282752);
        let delta1 = pos!(6.659_872_649_379_905);
        let delta2 = pos!(0.262_778_440_960_323_7);
        let k1 = pos!(2725.0);
        let k2 = pos!(2560.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[0] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta1;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(2100.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_iron_condor_delta_size {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_condor::IronCondor;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            underlying_price, // underlying_price
            pos!(2725.0),     // short_call_strike
            pos!(2560.0),     // short_put_strike
            pos!(2800.0),     // long_call_strike
            pos!(2500.0),     // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.1548),   // implied_volatility
            Decimal::ZERO,  // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(2.0),      // quantity
            pos!(38.8),     // premium_short_call
            pos!(30.4),     // premium_short_put
            pos!(23.3),     // premium_long_call
            pos!(16.8),     // premium_long_put
            pos!(0.96),     // open_fee
            pos!(0.96),     // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos!(2800.9));
        let size = dec!(-0.42443);
        let delta1 = pos!(0.5740980208367);
        let delta2 = pos!(20.961338283036);
        let k1 = pos!(2725.0);
        let k2 = pos!(2560.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        match &binding[0] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
        option.quantity = delta2;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos!(2500.9));
        let size = dec!(0.3656);
        let delta1 = pos!(13.074221057885);
        let delta2 = pos!(0.5277336881936);
        let k1 = pos!(2725.0);
        let k2 = pos!(2560.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[0] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta1, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k1, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        match &binding[1] {
            DeltaAdjustment::SellOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta2, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k2, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta1;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos!(2100.0));

        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
        assert!(strategy.is_delta_neutral());
        let suggestion = strategy.delta_adjustments().unwrap();
        assert_eq!(suggestion[0], DeltaAdjustment::NoAdjustmentNeeded);
    }
}

#[cfg(test)]
mod tests_iron_condor_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use crate::{assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    /// Creates a test Iron Condor with standard parameters
    fn create_test_condor() -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            pos!(2646.9), // underlying_price
            pos!(2725.0), // short_call_strike
            pos!(2560.0), // short_put_strike
            pos!(2800.0), // long_call_strike
            pos!(2500.0), // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.1548),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(38.8),     // premium_short_call
            pos!(30.4),     // premium_short_put
            pos!(23.3),     // premium_long_call
            pos!(16.8),     // premium_long_put
            pos!(0.96),     // open_fee
            pos!(0.96),     // close_fee
        )
    }

    #[test]
    fn test_get_expiration() {
        let condor = create_test_condor();
        let expiration_date = *condor.get_expiration().values().next().unwrap();
        assert_eq!(expiration_date, &ExpirationDate::Days(pos!(30.0)));
    }

    #[test]
    fn test_get_risk_free_rate() {
        let condor = create_test_condor();
        let risk_free_rate = **condor.get_risk_free_rate().values().next().unwrap();
        assert_eq!(risk_free_rate, dec!(0.05));
    }

    #[test]
    fn test_get_profit_ranges() {
        let condor = create_test_condor();
        let result = condor.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();

        assert_eq!(ranges.len(), 1);
        let range = &ranges[0];

        // Verify range bounds
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
        assert!(range.probability > Positive::ZERO);
        assert!(range.probability <= pos!(1.0));

        // Verify profit range is between short strikes
        assert!(range.lower_bound.unwrap() <= condor.short_put.option.strike_price);
        assert!(range.upper_bound.unwrap() >= condor.short_call.option.strike_price);
    }

    #[test]
    fn test_get_loss_ranges() {
        let condor = create_test_condor();
        let result = condor.get_loss_ranges();

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
    fn test_probability_sum_to_one() {
        let condor = create_test_condor();

        let profit_ranges = condor.get_profit_ranges().unwrap();
        let loss_ranges = condor.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(total_profit_prob + total_loss_prob, pos!(1.0), pos!(0.0001));
    }

    #[test]
    fn test_strike_prices_validity() {
        let condor = create_test_condor();
        // Verify strike price ordering
        assert!(condor.long_put.option.strike_price < condor.short_put.option.strike_price);
        assert!(condor.short_put.option.strike_price < condor.short_call.option.strike_price);
        assert!(condor.short_call.option.strike_price < condor.long_call.option.strike_price);
    }

    #[test]
    fn test_break_even_points_validity() {
        let condor = create_test_condor();
        let break_even_points = condor.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        // Break-even points should be between respective strikes
        assert!(break_even_points[0] > condor.long_put.option.strike_price);
        assert!(break_even_points[0] < condor.short_put.option.strike_price);
        assert!(break_even_points[1] > condor.short_call.option.strike_price);
        assert!(break_even_points[1] < condor.long_call.option.strike_price);
    }

    #[test]
    fn test_with_volatility_adjustment() {
        let condor = create_test_condor();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        });

        let prob = condor.probability_of_profit(vol_adj, None);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    fn test_with_price_trend() {
        let condor = create_test_condor();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let prob = condor.probability_of_profit(None, trend);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= pos!(1.0));
    }

    #[test]
    fn test_analyze_probabilities() {
        let condor = create_test_condor();
        let analysis = condor.analyze_probabilities(None, None).unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.expected_value >= Positive::ZERO);
        assert_eq!(analysis.break_even_points.len(), 2);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_high_volatility_scenario() {
        let mut condor = create_test_condor();
        let high_vol = pos!(0.5);
        condor.long_call.option.implied_volatility = high_vol;
        condor.short_call.option.implied_volatility = high_vol;
        condor.short_put.option.implied_volatility = high_vol;
        condor.long_put.option.implied_volatility = high_vol;

        let analysis = condor.analyze_probabilities(None, None).unwrap();
        // Higher volatility should decrease probability of profit
        assert!(analysis.probability_of_profit < pos!(0.7));
    }

    #[test]
    fn test_extreme_probabilities() {
        let condor = create_test_condor();
        let result = condor.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();

        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_iron_condor_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_iron_condor() -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            pos!(2646.9), // underlying_price
            pos!(2725.0), // short_call_strike
            pos!(2560.0), // short_put_strike
            pos!(2800.0), // long_call_strike
            pos!(2500.0), // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.1548),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(38.8),     // premium_short_call
            pos!(30.4),     // premium_short_put
            pos!(23.3),     // premium_long_call
            pos!(16.8),     // premium_long_put
            pos!(0.96),     // open_fee
            pos!(0.96),     // close_fee
        )
    }

    #[test]
    fn test_short_iron_condor_get_position() {
        let mut iron_condor = create_test_iron_condor();

        // Test getting short call position
        let call_position =
            iron_condor.get_position(&OptionStyle::Call, &Side::Short, &pos!(2725.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(2725.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position = iron_condor.get_position(&OptionStyle::Put, &Side::Short, &pos!(2560.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(2560.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            iron_condor.get_position(&OptionStyle::Call, &Side::Short, &pos!(2715.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => {
                error!("Unexpected error: {:?}", invalid_position);
                panic!()
            }
        }
    }

    #[test]
    fn test_long_iron_condor_get_position() {
        let mut iron_condor = create_test_iron_condor();

        // Test getting short call position
        let call_position =
            iron_condor.get_position(&OptionStyle::Call, &Side::Long, &pos!(2800.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(2800.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position = iron_condor.get_position(&OptionStyle::Put, &Side::Long, &pos!(2500.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(2500.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position =
            iron_condor.get_position(&OptionStyle::Call, &Side::Long, &pos!(2715.0));
        assert!(invalid_position.is_err());
        match invalid_position {
            Err(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => {
                error!("Unexpected error: {:?}", invalid_position);
                panic!()
            }
        }
    }

    #[test]
    fn test_short_iron_condor_modify_position() {
        let mut iron_condor = create_test_iron_condor();

        // Modify short call position
        let mut modified_call = iron_condor.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = iron_condor.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(iron_condor.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = iron_condor.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = iron_condor.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(iron_condor.short_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = iron_condor.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = iron_condor.modify_position(&invalid_position);
        assert!(result.is_err());
        match result {
            Err(PositionError::ValidationError(kind)) => match kind {
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                } => {
                    assert_eq!(reason, "Strike not found in positions");
                }
                _ => panic!("Expected ValidationError::InvalidPosition"),
            },
            _ => panic!("Expected ValidationError"),
        }
    }

    #[test]
    fn test_long_iron_condor_modify_position() {
        let mut iron_condor = create_test_iron_condor();

        // Modify long call position
        let mut modified_call = iron_condor.long_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = iron_condor.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(iron_condor.long_call.option.quantity, pos!(2.0));

        // Modify long put position
        let mut modified_put = iron_condor.long_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = iron_condor.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(iron_condor.long_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = iron_condor.long_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = iron_condor.modify_position(&invalid_position);
        assert!(result.is_err());
        match result {
            Err(PositionError::ValidationError(kind)) => match kind {
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                } => {
                    assert_eq!(reason, "Strike not found in positions");
                }
                _ => panic!("Expected ValidationError::InvalidPosition"),
            },
            _ => panic!("Expected ValidationError"),
        }
    }
}

#[cfg(test)]
mod tests_adjust_option_position {
    use super::*;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> IronCondor {
        IronCondor::new(
            "GOLD".to_string(),
            pos!(2646.9), // underlying_price
            pos!(2725.0), // short_call_strike
            pos!(2560.0), // short_put_strike
            pos!(2800.0), // long_call_strike
            pos!(2500.0), // long_put_strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.1548),   // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(38.8),     // premium_short_call
            pos!(30.4),     // premium_short_put
            pos!(23.3),     // premium_long_call
            pos!(16.8),     // premium_long_put
            pos!(0.96),     // open_fee
            pos!(0.96),     // close_fee
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(2725.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_call.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(2560.0),
            &OptionStyle::Put,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.short_put.option.quantity,
            initial_quantity + adjustment
        );
    }

    #[test]
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_err());
        match result.unwrap_err().downcast_ref::<PositionError>() {
            Some(PositionError::ValidationError(
                PositionValidationErrorKind::IncompatibleSide {
                    position_side: _,
                    reason,
                },
            )) => {
                assert_eq!(reason, "Strike not found in positions");
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0), // Invalid strike price
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_err());
    }

    #[test]
    fn test_zero_quantity_adjustment() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(2725.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_strategy_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(90.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        let result = IronCondor::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_put.option.strike_price, pos!(90.0));
        assert_eq!(strategy.short_put.option.strike_price, pos!(95.0));
        assert_eq!(strategy.short_call.option.strike_price, pos!(105.0));
        assert_eq!(strategy.long_call.option.strike_price, pos!(110.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(90.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
        ];

        let result = IronCondor::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Condor get_strategy" && reason == "Must have exactly 4 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_types() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call, // Should be Put
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(90.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        let result = IronCondor::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Condor get_strategy" && reason == "Invalid option types for Iron Condor"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Short, // Should be Long
                pos!(100.0),
                pos!(1.0),
                pos!(90.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        let result = IronCondor::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Condor get_strategy" && reason == "Invalid option sides for Iron Condor"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(90.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(110.0),
                pos!(0.2),
            ),
        ];

        // Change expiration date of one option
        options[0].option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let result = IronCondor::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Condor get_strategy" && reason == "All options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_iron_condor_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;
    use rust_decimal_macros::dec;

    fn create_test_iron_condor() -> Result<IronCondor, StrategyError> {
        // Create long put (lowest strike)
        let long_put = create_sample_position(
            OptionStyle::Put,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(90.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create short put
        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create short call
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        // Create long call (highest strike)
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(110.0), // Strike price
            pos!(0.2),   // Implied volatility
        );

        IronCondor::get_strategy(&vec![long_put, short_put, short_call, long_call])
    }

    #[test]
    fn test_calculate_pnl_below_all_strikes() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(85.0); // Below all strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Long put ITM, other options OTM
        // Loss should be capped at long put protection level
        let max_loss = dec!(-5.0); // Width of put spread
        assert!(pnl.unrealized.unwrap() >= max_loss);
    }

    #[test]
    fn test_calculate_pnl_in_profit_zone_lower() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(97.0); // Between put strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Price in profit zone, should show positive PnL
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
    }

    #[test]
    fn test_calculate_pnl_at_middle() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(100.0); // At middle of range
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Maximum profit zone
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
    }

    #[test]
    fn test_calculate_pnl_in_profit_zone_upper() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(101.0); // Between call strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Price in profit zone, should show positive PnL
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
    }

    #[test]
    fn test_calculate_pnl_above_all_strikes() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(115.0); // Above all strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Long call ITM, other options OTM
        // Loss should be capped at long call protection level
        let max_loss = dec!(-5.0); // Width of call spread
        assert!(pnl.unrealized.unwrap() >= max_loss);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let condor = create_test_iron_condor().unwrap();
        let underlying_price = pos!(101.0); // At middle of range

        let result = condor.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At expiration, all options expire worthless
        // Max profit is the net premium received minus fees
        assert!(pnl.realized.unwrap() < dec!(0.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss_lower() {
        let condor = create_test_iron_condor().unwrap();
        let underlying_price = pos!(85.0); // Below all strikes

        let result = condor.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss = width of put spread - net premium received + fees
        let max_loss = dec!(-9.0); // Width of put spread
        assert!(pnl.realized.unwrap() >= max_loss);
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss_upper() {
        let condor = create_test_iron_condor().unwrap();
        let underlying_price = pos!(115.0); // Above all strikes

        let result = condor.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Max loss = width of call spread - net premium received + fees
        let max_loss = dec!(-9.0); // Width of call spread
        assert!(pnl.realized.unwrap() >= max_loss);
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, all options are worth more
        // Net effect should be negative as short gamma position
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still capped by the spread width
        assert!(pnl.unrealized.unwrap() >= dec!(-5.0));
    }

    #[test]
    fn test_calculate_pnl_near_expiration() {
        let condor = create_test_iron_condor().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(1.0)); // Near expiration
        let implied_volatility = pos!(0.2);

        let result = condor.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Near expiration, theta decay should be minimal
        // and profit should be near maximum if price is in the middle
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
    }
}
