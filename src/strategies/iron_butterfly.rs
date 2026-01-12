/*
Iron Butterfly Strategy

An iron butterfly involves selling both a put and call at the same strike price (at-the-money)
and buying a put at a lower strike and a call at a higher strike.
This strategy is used when very low volatility in the underlying asset's price is expected.

Key characteristics:
- Maximum profit at the short strike price
- Limited risk
- High probability of small profit
- Requires very low volatility
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use super::shared::ButterflyStrategy;
use crate::{
    ExpirationDate, Options,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
    error::{
        GreeksError, OperationErrorKind, PricingError,
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
    test_strategy_traits,
};
use chrono::Utc;
use num_traits::FromPrimitive;
use positive::Positive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{error, info};
use utoipa::ToSchema;

/// The default description for the Iron Butterfly strategy.
pub const IRON_BUTTERFLY_DESCRIPTION: &str = "An Iron Butterfly is a neutral options strategy combining selling an at-the-money put and call \
    while buying an out-of-the-money call and an out-of-the-money put. The short options have the same \
    strike price. This strategy profits from low volatility and time decay, with maximum profit when \
    the underlying price equals the strike price of the short options at expiration.";

/// # IronButterfly
///
/// Represents an Iron Butterfly options trading strategy. This strategy involves four options positions:
/// a short call and short put at the same middle strike price, along with a long call at a higher strike
/// and a long put at a lower strike, all with the same expiration date.
///
/// An Iron Butterfly is a neutral options strategy that profits from low volatility, where the underlying
/// asset price remains close to the middle strike price through expiration.
///
/// ## Fields
/// * `name`: A descriptive name for the specific strategy instance.
/// * `kind`: The type of strategy, which is `StrategyType::IronButterfly`.
/// * `description`: A detailed description of this specific strategy instance.
/// * `break_even_points`: The price points at which the strategy breaks even (neither profit nor loss).
/// * `short_call`: The short call position component at the middle strike price.
/// * `short_put`: The short put position component at the same middle strike price.
/// * `long_call`: The long call position component at a higher strike price.
/// * `long_put`: The long put position component at a lower strike price.
///
/// ## Risk Profile
/// An Iron Butterfly has limited risk and limited profit potential. The maximum risk is defined
/// by the difference between the middle strike and either wing strike, minus the net premium received.
///
/// ## Maximum Profit
/// The maximum profit is achieved when the underlying asset price at expiration equals exactly the middle strike price.
/// In this scenario, all options expire worthless except the short positions, and the trader keeps the full premium collected.
///
/// ## Maximum Loss
/// The maximum loss is limited and occurs when the price of the underlying asset moves beyond either the upper
/// or lower strike price. The worst-case scenario loss is the difference between strike prices minus the premium received.
///
/// ## Break-even Points
/// There are two break-even points in an Iron Butterfly:
/// 1. Upper break-even point: Middle strike price + net premium received
/// 2. Lower break-even point: Middle strike price - net premium received
///
/// ## Strategy Usage
/// This strategy is typically used when:
/// - The trader expects low volatility in the underlying asset
/// - The trader believes the price will remain close to the current level
/// - Implied volatility is high (making the sold options more expensive)
/// - The trader wants defined risk/reward parameters compared to a short straddle
///
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct IronButterfly {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as an IronButterfly strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call position at the middle strike
    pub short_call: Position,
    /// The short put position at the middle strike
    pub short_put: Position,
    /// The long call position at a higher strike price
    pub long_call: Position,
    /// The long put position at a lower strike price
    pub long_put: Position,
}

impl IronButterfly {
    /// # Iron Butterfly Strategy Constructor
    ///
    /// Creates a new Iron Butterfly options trading strategy instance. An Iron Butterfly is a neutral options strategy
    /// that combines selling an at-the-money put and call while buying an out-of-the-money call and an out-of-the-money put.
    ///
    /// The Iron Butterfly consists of four options contracts:
    /// - Short call at the middle strike price
    /// - Short put at the same middle strike price
    /// - Long call at a higher strike price
    /// - Long put at a lower strike price
    ///
    /// ## Parameters
    ///
    /// * `underlying_symbol` - Symbol of the underlying asset (e.g., "SPY")
    /// * `underlying_price` - Current market price of the underlying asset
    /// * `short_strike` - Strike price for both short call and short put options
    /// * `long_call_strike` - Strike price for the long call option (higher than the short strike)
    /// * `long_put_strike` - Strike price for the long put option (lower than the short strike)
    /// * `expiration` - Expiration date for all options in the strategy
    /// * `implied_volatility` - Implied volatility used for option pricing
    /// * `risk_free_rate` - Risk-free interest rate as a decimal
    /// * `dividend_yield` - Dividend yield of the underlying asset
    /// * `quantity` - Number of contracts for each position in the strategy
    /// * `premium_short_call` - Premium received for selling the call option
    /// * `premium_short_put` - Premium received for selling the put option
    /// * `premium_long_call` - Premium paid for buying the call option
    /// * `premium_long_put` - Premium paid for buying the put option
    /// * `open_fee` - Transaction fee for opening positions
    /// * `close_fee` - Transaction fee for closing positions
    ///
    /// ## Return Value
    ///
    /// Returns a fully configured `IronButterfly` strategy instance with all positions established and break-even points calculated.
    ///
    /// ## Strategy Characteristics
    ///
    /// - **Maximum Profit**: Achieved when the underlying asset price equals the short strike price at expiration.
    ///   The maximum profit equals the net credit received (premiums from short options minus premiums paid for long options).
    ///
    /// - **Maximum Risk**: Limited and defined by the difference between adjacent strike prices minus the net credit received.
    ///
    /// - **Break-even Points**: There are two break-even points:
    ///   1. Lower break-even = short strike price - net credit received
    ///   2. Upper break-even = short strike price + net credit received
    ///
    /// - **Ideal Market Outlook**: Neutral, expecting low volatility with the underlying price remaining near the short strike price.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        short_strike: Positive,
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
        let mut strategy = IronButterfly {
            name: "Iron Butterfly".to_string(),
            kind: StrategyType::IronButterfly,
            description: IRON_BUTTERFLY_DESCRIPTION.to_string(),
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
            open_fee,
            close_fee,
            None,
            None,
        );
        strategy
            .add_position(&short_call)
            .expect("Invalid short call");

        // Short Put
        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            short_strike,
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
            .add_position(&short_put)
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
            .add_position(&long_call)
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
        strategy.add_position(&long_put).expect("Invalid long put");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for IronButterfly {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 4 options for an Iron Butterfly
        if vec_positions.len() != 4 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "Must have exactly 4 options".to_string(),
                },
            ));
        }

        // Sort options by strike price to identify positions
        let mut sorted_positions = vec_positions.to_vec();
        sorted_positions.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        // Validate the positions and their structure
        // In Iron Butterfly, all strikes must be equidistant
        let strike_prices: Vec<Positive> = sorted_positions
            .iter()
            .map(|opt| opt.option.strike_price)
            .collect();

        if strike_prices[1] - strike_prices[0] != strike_prices[3] - strike_prices[2] {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "Strike prices must be equidistant".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        let exp_date = sorted_positions[0].option.expiration_date;
        if !sorted_positions
            .iter()
            .all(|opt| opt.option.expiration_date == exp_date)
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "All options must have the same expiration date".to_string(),
                },
            ));
        }

        // Find and validate the positions
        let long_put = sorted_positions
            .iter()
            .find(|opt| {
                opt.option.option_style == OptionStyle::Put && opt.option.side == Side::Long
            })
            .ok_or(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "Missing long put position".to_string(),
                },
            ))?;

        let short_put = sorted_positions
            .iter()
            .find(|opt| {
                opt.option.option_style == OptionStyle::Put && opt.option.side == Side::Short
            })
            .ok_or(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "Missing short put position".to_string(),
                },
            ))?;

        let short_call = sorted_positions
            .iter()
            .find(|opt| {
                opt.option.option_style == OptionStyle::Call && opt.option.side == Side::Short
            })
            .ok_or(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "Missing short call position".to_string(),
                },
            ))?;

        let long_call = sorted_positions
            .iter()
            .find(|opt| {
                opt.option.option_style == OptionStyle::Call && opt.option.side == Side::Long
            })
            .ok_or(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Iron Butterfly get_strategy".to_string(),
                    reason: "Missing long call position".to_string(),
                },
            ))?;

        // Create strategy
        let mut strategy = IronButterfly {
            name: "Iron Butterfly".to_string(),
            kind: StrategyType::IronButterfly,
            description: IRON_BUTTERFLY_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: short_call.clone(),
            short_put: short_put.clone(),
            long_call: long_call.clone(),
            long_put: long_put.clone(),
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for IronButterfly {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let net_credit = self.get_net_premium_received()? / self.short_call.option.quantity;

        self.break_even_points
            .push((self.short_call.option.strike_price + net_credit).round_to(2));

        self.break_even_points
            .push((self.short_call.option.strike_price - net_credit).round_to(2));

        self.break_even_points.sort();
        Ok(())
    }
}

impl Validable for IronButterfly {
    fn validate(&self) -> bool {
        let order = self.long_put.option.strike_price < self.short_put.option.strike_price
            && self.short_put.option.strike_price == self.short_call.option.strike_price
            && self.short_call.option.strike_price < self.long_call.option.strike_price;

        if !order {
            error!("Invalid order of strikes or short strikes not equal");
        }

        self.short_call.validate()
            && self.short_put.validate()
            && self.long_call.validate()
            && self.long_put.validate()
            && order
    }
}

impl Positionable for IronButterfly {
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
            (Side::Long, OptionStyle::Put, _) if *strike == self.long_put.option.strike_price => {
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

impl Strategable for IronButterfly {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for IronButterfly {
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
    fn get_option_basic_type(&self) -> HashSet<OptionBasicType<'_>> {
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
    fn get_implied_volatility(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
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
    fn get_quantity(&self) -> HashMap<OptionBasicType<'_>, &Positive> {
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
        Ok(())
    }
    fn set_underlying_price(&mut self, price: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.underlying_price = *price;
        self.short_call.premium = Positive::new_decimal(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )
        .unwrap_or(Positive::ZERO);
        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::new_decimal(self.short_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::new_decimal(self.long_call.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        self.long_put.option.underlying_price = *price;
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.short_put.option.implied_volatility = *volatility;
        self.long_call.option.implied_volatility = *volatility;
        self.long_put.option.implied_volatility = *volatility;
        self.short_call.premium = Positive::new_decimal(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        )
        .unwrap_or(Positive::ZERO);
        self.short_put.premium =
            Positive::new_decimal(self.short_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        self.long_call.premium =
            Positive::new_decimal(self.long_call.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        self.long_put.premium =
            Positive::new_decimal(self.long_put.option.calculate_price_black_scholes()?.abs())
                .unwrap_or(Positive::ZERO);
        Ok(())
    }
}

impl Strategies for IronButterfly {
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

        Ok(
            Positive::new_decimal(self.calculate_profit_at(&self.short_call.option.strike_price)?)
                .unwrap_or(Positive::ZERO),
        )
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
        Ok(Positive::new_decimal(left_loss.abs().max(right_loss.abs())).unwrap_or(Positive::ZERO))
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
            _ => Ok(
                Decimal::from_f64(max_profit.to_f64() / max_loss.to_f64() * 100.0)
                    .unwrap_or(Decimal::ZERO),
            ),
        }
    }
}

impl Optimizable for IronButterfly {
    type Strategy = IronButterfly;

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
            .filter(move |(low, mid, high)| {
                if side == FindOptimalSide::Center {
                    let atm_strike = match option_chain.atm_strike() {
                        Ok(atm_strike) => atm_strike,
                        Err(_) => return false,
                    };
                    low.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && mid.is_valid_optimal_side(
                            underlying_price,
                            &FindOptimalSide::Range(*atm_strike, *atm_strike),
                        )
                        && high.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    low.is_valid_optimal_side(underlying_price, &side)
                        && mid.is_valid_optimal_side(underlying_price, &side)
                        && high.is_valid_optimal_side(underlying_price, &side)
                }
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(low, mid, high)| {
                low.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && mid.put_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && high.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(low, mid, high)| {
                let legs = StrategyLegs::FourLegs {
                    first: low,
                    second: mid,
                    third: mid,
                    fourth: high,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(low, mid, high)| OptionDataGroup::Three(low, mid, high))
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
            let (low, mid, high) = match option_data_group {
                OptionDataGroup::Three(first, second, third) => (first, second, third),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::FourLegs {
                first: low,
                second: mid,
                third: mid,
                fourth: high,
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
                second: short_strike,
                third: _,
                fourth: long_call,
            } => {
                let implied_volatility = short_strike.implied_volatility;
                assert!(implied_volatility <= Positive::ONE);
                IronButterfly::new(
                    chain.symbol.clone(),
                    chain.underlying_price,
                    short_strike.strike_price,
                    long_call.strike_price,
                    long_put.strike_price,
                    self.short_call.option.expiration_date,
                    implied_volatility,
                    self.short_call.option.risk_free_rate,
                    self.short_call.option.dividend_yield,
                    self.short_call.option.quantity,
                    short_strike.call_bid.unwrap(),
                    short_strike.put_bid.unwrap(),
                    long_call.call_ask.unwrap(),
                    long_put.put_ask.unwrap(),
                    self.get_fees().unwrap() / 8.0,
                    self.get_fees().unwrap() / 8.0,
                )
            }
            _ => panic!("Invalid number of legs for Iron Butterfly strategy"),
        }
    }
}

impl Profit for IronButterfly {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(self.short_call.pnl_at_expiration(&price)?
            + self.short_put.pnl_at_expiration(&price)?
            + self.long_call.pnl_at_expiration(&price)?
            + self.long_put.pnl_at_expiration(&price)?)
    }
}

impl ProbabilityAnalysis for IronButterfly {
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

impl Greeks for IronButterfly {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![
            &self.short_call.option,
            &self.short_put.option,
            &self.long_call.option,
            &self.long_put.option,
        ])
    }
}

impl DeltaNeutrality for IronButterfly {}

impl ButterflyStrategy for IronButterfly {
    fn wing_strikes(&self) -> (Positive, Positive) {
        (
            self.long_put.option.strike_price,
            self.long_call.option.strike_price,
        )
    }

    fn body_strike(&self) -> Positive {
        self.short_call.option.strike_price
    }

    fn get_butterfly_positions(&self) -> Vec<&Position> {
        vec![
            &self.long_put,
            &self.short_put,
            &self.short_call,
            &self.long_call,
        ]
    }
}

impl PnLCalculator for IronButterfly {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, PricingError> {
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
    ) -> Result<PnL, PricingError> {
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

test_strategy_traits!(IronButterfly, test_short_call_implementations);

#[cfg(test)]
mod tests_iron_butterfly {
    use super::*;

    use chrono::{TimeZone, Utc};
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    #[test]
    fn test_iron_butterfly_creation() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            pos_or_panic!(150.0), // underlying price
            pos_or_panic!(150.0), // short strike (at the money)
            pos_or_panic!(160.0), // long call strike
            pos_or_panic!(140.0), // long put strike
            ExpirationDate::DateTime(date),
            pos_or_panic!(0.2),  // implied volatility
            dec!(0.01),          // risk free rate
            pos_or_panic!(0.02), // dividend yield
            Positive::ONE,       // quantity
            pos_or_panic!(1.5),  // premium short call
            pos_or_panic!(1.5),  // premium short put
            Positive::ONE,       // premium long call
            Positive::ONE,       // premium long put
            pos_or_panic!(5.0),  // open fee
            pos_or_panic!(5.0),  // close fee
        );

        assert_eq!(butterfly.name, "Iron Butterfly");
        assert_eq!(
            butterfly.description,
            IRON_BUTTERFLY_DESCRIPTION.to_string()
        );
        assert_eq!(butterfly.kind, StrategyType::IronButterfly);
        assert_eq!(butterfly.break_even_points.len(), 2);
        assert_eq!(butterfly.short_call.option.strike_price, 150.0);
        assert_eq!(butterfly.short_put.option.strike_price, 150.0);
        assert_eq!(butterfly.long_call.option.strike_price, 160.0);
        assert_eq!(butterfly.long_put.option.strike_price, 140.0);
    }

    #[test]
    fn test_max_loss() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            Positive::HUNDRED,    // underlying price
            Positive::HUNDRED,    // short strike (at the money)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::DateTime(date),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(1.5),
            pos_or_panic!(1.5),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
        );

        assert_eq!(butterfly.get_max_loss().unwrap(), 49.0);
    }

    #[test]
    fn test_max_profit() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            Positive::HUNDRED,    // underlying price
            Positive::HUNDRED,    // short strike (at the money)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::DateTime(date),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(3.5),
            pos_or_panic!(3.5),
            Positive::TWO,
            Positive::TWO,
            pos_or_panic!(0.07),
            pos_or_panic!(0.07),
        );

        let expected_profit: Positive = butterfly.get_net_premium_received().unwrap();
        assert_eq!(butterfly.get_max_profit().unwrap(), expected_profit);
    }

    #[test]
    fn test_get_break_even_points() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            Positive::HUNDRED,    // underlying price
            Positive::HUNDRED,    // short strike (at the money)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::DateTime(date),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(1.5),
            pos_or_panic!(1.5),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
        );

        assert_eq!(
            butterfly.get_break_even_points().unwrap()[0],
            butterfly.break_even_points[0]
        );
        assert_eq!(
            butterfly.get_break_even_points().unwrap()[1],
            butterfly.break_even_points[1]
        );

        // Break-even points should be equidistant from short strike
        let distance_up = butterfly.break_even_points[1] - butterfly.short_call.option.strike_price;
        let distance_down =
            butterfly.short_put.option.strike_price - butterfly.break_even_points[0];
        assert!((distance_up - distance_down) < pos_or_panic!(0.01));
    }

    #[test]
    fn test_fees() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            ExpirationDate::DateTime(date),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(1.5),
            pos_or_panic!(1.5),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
        );

        let expected_fees = butterfly.short_call.open_fee
            + butterfly.short_call.close_fee
            + butterfly.short_put.open_fee
            + butterfly.short_put.close_fee
            + butterfly.long_call.open_fee
            + butterfly.long_call.close_fee
            + butterfly.long_put.open_fee
            + butterfly.long_put.close_fee;
        assert_eq!(butterfly.get_fees().unwrap(), expected_fees);
    }

    #[test]
    fn test_calculate_profit_at() {
        let date = Utc.with_ymd_and_hms(2024, 12, 1, 0, 0, 0).unwrap();
        let butterfly = IronButterfly::new(
            "AAPL".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            ExpirationDate::DateTime(date),
            pos_or_panic!(0.2),
            dec!(0.01),
            pos_or_panic!(0.02),
            Positive::ONE,
            pos_or_panic!(1.5),
            pos_or_panic!(1.5),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(5.0),
            pos_or_panic!(5.0),
        );

        // Test at short strike (maximum profit point)
        let price = butterfly.short_call.option.strike_price;
        let expected_profit = butterfly
            .short_call
            .pnl_at_expiration(&Some(&price))
            .unwrap()
            + butterfly
                .short_put
                .pnl_at_expiration(&Some(&price))
                .unwrap()
            + butterfly
                .long_call
                .pnl_at_expiration(&Some(&price))
                .unwrap()
            + butterfly.long_put.pnl_at_expiration(&Some(&price)).unwrap();
        assert_eq!(
            butterfly.calculate_profit_at(&price).unwrap(),
            expected_profit
        );
    }
}

#[cfg(test)]
mod tests_iron_butterfly_validable {
    use super::*;
    use positive::pos_or_panic;

    use crate::model::ExpirationDate;

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
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                quantity,
                Positive::HUNDRED,
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

    fn create_valid_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,    // underlying_price
            Positive::HUNDRED,    // short strike (both call and put)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2), // implied_volatility
            dec!(0.05),         // risk_free_rate
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // quantity
            Positive::TWO,      // premium_short_call
            Positive::TWO,      // premium_short_put
            Positive::ONE,      // premium_long_call
            Positive::ONE,      // premium_long_put
            Positive::ZERO,     // open_fee
            Positive::ZERO,     // closing fee
        )
    }

    #[test]
    fn test_validate_valid_butterfly() {
        let butterfly = create_valid_butterfly();
        assert!(butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_short_call() {
        let mut butterfly = create_valid_butterfly();
        // Make short call invalid by setting quantity to zero
        butterfly.short_call = create_valid_position(
            Side::Short,
            OptionStyle::Call,
            Positive::HUNDRED,
            Positive::ZERO,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_short_put() {
        let mut butterfly = create_valid_butterfly();
        // Make short put invalid by setting quantity to zero
        butterfly.short_put = create_valid_position(
            Side::Short,
            OptionStyle::Put,
            Positive::HUNDRED,
            Positive::ZERO,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_long_call() {
        let mut butterfly = create_valid_butterfly();
        // Make long call invalid by setting quantity to zero
        butterfly.long_call = create_valid_position(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(110.0),
            Positive::ZERO,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_invalid_long_put() {
        let mut butterfly = create_valid_butterfly();
        // Make long put invalid by setting quantity to zero
        butterfly.long_put = create_valid_position(
            Side::Long,
            OptionStyle::Put,
            pos_or_panic!(90.0),
            Positive::ZERO,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_all_invalid() {
        let mut butterfly = create_valid_butterfly();
        // Make all positions invalid
        butterfly.short_call = create_valid_position(
            Side::Short,
            OptionStyle::Call,
            Positive::HUNDRED,
            Positive::ZERO,
        );
        butterfly.short_put = create_valid_position(
            Side::Short,
            OptionStyle::Put,
            Positive::HUNDRED,
            Positive::ZERO,
        );
        butterfly.long_call = create_valid_position(
            Side::Long,
            OptionStyle::Call,
            pos_or_panic!(110.0),
            Positive::ZERO,
        );
        butterfly.long_put = create_valid_position(
            Side::Long,
            OptionStyle::Put,
            pos_or_panic!(90.0),
            Positive::ZERO,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_different_short_strikes() {
        let mut butterfly = create_valid_butterfly();
        // Make short strikes different
        butterfly.short_call = create_valid_position(
            Side::Short,
            OptionStyle::Call,
            pos_or_panic!(105.0),
            Positive::ONE,
        );
        butterfly.short_put = create_valid_position(
            Side::Short,
            OptionStyle::Put,
            pos_or_panic!(95.0),
            Positive::ONE,
        );
        assert!(!butterfly.validate());
    }

    #[test]
    fn test_validate_inverted_strikes() {
        let mut butterfly = create_valid_butterfly();
        // Invert the strikes
        butterfly.long_put = create_valid_position(
            Side::Long,
            OptionStyle::Put,
            pos_or_panic!(105.0),
            Positive::ONE,
        );
        butterfly.short_put = create_valid_position(
            Side::Short,
            OptionStyle::Put,
            pos_or_panic!(110.0),
            Positive::ONE,
        );
        assert!(!butterfly.validate());
    }
}

#[cfg(test)]
mod tests_iron_butterfly_strategies {
    use super::*;

    use crate::model::ExpirationDate;

    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,    // underlying_price
            Positive::HUNDRED,    // short strike (both call and put)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2), // implied_volatility
            dec!(0.05),         // risk_free_rate
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // quantity
            Positive::TWO,      // premium_short_call
            Positive::TWO,      // premium_short_put
            Positive::ONE,      // premium_long_call
            Positive::ONE,      // premium_long_put
            pos_or_panic!(0.5), // open_fee
            pos_or_panic!(0.5), // closing fee
        )
    }

    #[test]
    fn test_add_leg() {
        let mut butterfly = create_test_butterfly();

        // Test adding a short call at the same strike as short put
        let new_short_call = Position::new(
            Options::new(
                OptionType::European,
                Side::Short,
                "TEST".to_string(),
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos_or_panic!(2.5),
            Utc::now(),
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
            None,
            None,
        );
        butterfly
            .add_position(&new_short_call)
            .expect("Failed to add short call");
        assert_eq!(
            butterfly.short_call.option.strike_price,
            butterfly.short_put.option.strike_price
        );

        // Test adding a long put
        let new_long_put = Position::new(
            Options::new(
                OptionType::European,
                Side::Long,
                "TEST".to_string(),
                pos_or_panic!(90.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Put,
                Positive::ZERO,
                None,
            ),
            pos_or_panic!(1.5),
            Utc::now(),
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
            None,
            None,
        );
        butterfly
            .add_position(&new_long_put)
            .expect("Failed to add long put");
        assert_eq!(butterfly.long_put.option.strike_price, pos_or_panic!(90.0));
    }

    #[test]
    fn test_get_legs() {
        let butterfly = create_test_butterfly();
        let legs = butterfly.get_positions().expect("Failed to get legs");

        assert_eq!(legs.len(), 4);
        assert_eq!(legs[0].option.option_style, OptionStyle::Call);
        assert_eq!(legs[0].option.side, Side::Short);
        assert_eq!(legs[1].option.option_style, OptionStyle::Put);
        assert_eq!(legs[1].option.side, Side::Short);
        assert_eq!(legs[2].option.option_style, OptionStyle::Call);
        assert_eq!(legs[2].option.side, Side::Long);
        assert_eq!(legs[3].option.option_style, OptionStyle::Put);
        assert_eq!(legs[3].option.side, Side::Long);

        // Verify short strikes are equal
        assert_eq!(legs[0].option.strike_price, legs[1].option.strike_price);
    }

    #[test]
    fn test_get_break_even_points() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);

        // Break-even points should be equidistant from short strike
        let short_strike = butterfly.short_call.option.strike_price;
        let upper_distance = break_even_points[1] - short_strike;
        let lower_distance = short_strike - break_even_points[0];
        assert!((upper_distance - lower_distance) < pos_or_panic!(0.01));
    }

    #[test]
    fn test_max_profit() {
        let butterfly = create_test_butterfly();
        assert!(butterfly.get_max_profit().is_err());
    }

    #[test]
    fn test_max_loss() {
        let butterfly = create_test_butterfly();
        let max_loss = butterfly.get_max_loss().unwrap().to_dec();

        // Max loss should be equal at both wings
        let loss_at_long_put = butterfly
            .calculate_profit_at(&butterfly.long_put.option.strike_price)
            .unwrap();
        let loss_at_long_call = butterfly
            .calculate_profit_at(&butterfly.long_call.option.strike_price)
            .unwrap();
        assert!((loss_at_long_put - loss_at_long_call).abs() < dec!(0.01));
        assert_eq!(max_loss, loss_at_long_put.abs());
    }

    #[test]
    fn test_total_cost() {
        let butterfly = create_test_butterfly();
        let total_cost = butterfly.get_total_cost().unwrap();
        let expected_cost = pos_or_panic!(6.0); // 2.0 + 2.0 + 1.0 + 1.0
        assert_eq!(total_cost, expected_cost);
    }

    #[test]
    fn test_net_premium_received() {
        let butterfly = create_test_butterfly();
        assert_eq!(butterfly.get_net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    fn test_fees() {
        let butterfly = create_test_butterfly();
        let expected_fees = 4.0; // (0.5 + 0.5) * 4 legs
        assert_eq!(butterfly.get_fees().unwrap(), expected_fees);
    }

    #[test]
    fn test_profit_area() {
        let butterfly = create_test_butterfly();
        // Profit area should be smaller than Iron Condor due to higher concentration
        assert!(butterfly.get_profit_area().unwrap().to_f64().unwrap() < 1.0);
    }

    #[test]
    fn test_with_multiple_contracts() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO, // quantity = 2
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
        );

        assert_eq!(butterfly.get_net_premium_received().unwrap().to_f64(), 0.0);
    }

    #[test]
    fn test_with_asymmetric_premiums() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(3.0), // Higher short call premium
            Positive::TWO,      // Lower short put premium
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.5),
            pos_or_panic!(0.5),
        );

        assert_eq!(butterfly.get_net_premium_received().unwrap().to_f64(), 0.0);
    }
}

#[cfg(test)]
mod tests_iron_butterfly_optimizable {
    use super::*;
    use positive::{pos_or_panic, spos};

    use crate::chains::OptionData;
    use crate::model::ExpirationDate;

    use rust_decimal_macros::dec;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,    // underlying_price
            Positive::HUNDRED,    // short strike (both call and put)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2), // implied_volatility
            dec!(0.05),         // risk_free_rate
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // quantity
            Positive::TWO,      // premium_short_call
            Positive::TWO,      // premium_short_put
            Positive::ONE,      // premium_long_call
            Positive::ONE,      // premium_long_put
            pos_or_panic!(0.5), // open_fee
            pos_or_panic!(0.5), // closing fee
        )
    }

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            None,
            None,
        );

        // Add options at various strikes
        for strike in [85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0] {
            chain.add_option(
                pos_or_panic!(strike),
                spos!(5.0),         // call_bid
                spos!(5.2),         // call_ask
                spos!(5.0),         // put_bid
                spos!(5.2),         // put_ask
                pos_or_panic!(0.2), // implied_volatility
                None,               // delta
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
    fn test_find_optimal_at_the_money() {
        let mut butterfly = create_test_butterfly();
        let chain = create_test_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(butterfly.validate());
        // Short strikes should be at or very near the money
        let diff = (butterfly.short_call.option.strike_price.to_f64()
            - chain.underlying_price.to_f64())
        .abs();
        assert!(diff <= 5.0); // Allow some flexibility in strike selection
        assert_eq!(
            butterfly.short_call.option.strike_price,
            butterfly.short_put.option.strike_price
        );
    }

    #[test]
    fn test_find_optimal_symmetric_wings() {
        let mut butterfly = create_test_butterfly();
        let chain = create_test_chain();

        butterfly.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(butterfly.validate());
        // Wings should be roughly symmetric
        let upper_wing =
            butterfly.long_call.option.strike_price - butterfly.short_call.option.strike_price;
        let lower_wing =
            butterfly.short_put.option.strike_price - butterfly.long_put.option.strike_price;
        assert!((upper_wing - lower_wing).to_f64().abs() <= 5.0);
    }

    #[test]
    fn test_find_optimal_range() {
        let mut butterfly = create_test_butterfly();
        let chain = create_test_chain();

        butterfly.find_optimal(
            &chain,
            FindOptimalSide::Range(pos_or_panic!(95.0), pos_or_panic!(105.0)),
            OptimizationCriteria::Ratio,
        );

        assert!(butterfly.validate());
        // Short strikes should be within the specified range
        assert!(butterfly.short_call.option.strike_price >= pos_or_panic!(95.0));
        assert!(butterfly.short_call.option.strike_price <= pos_or_panic!(105.0));
        // And should be equal
        assert_eq!(
            butterfly.short_call.option.strike_price,
            butterfly.short_put.option.strike_price
        );
    }

    #[test]
    fn test_is_valid_long_option() {
        let butterfly = create_test_butterfly();
        let option = OptionData::new(
            pos_or_panic!(90.0),
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            pos_or_panic!(0.2),
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
        assert!(butterfly.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(butterfly.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(!butterfly.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_is_valid_short_option() {
        let butterfly = create_test_butterfly();
        let option = OptionData::new(
            Positive::HUNDRED, // At the money
            spos!(5.0),
            spos!(5.2),
            spos!(5.0),
            spos!(5.2),
            pos_or_panic!(0.2),
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

        // Test with different sides - should prefer at-the-money options
        assert!(butterfly.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(butterfly.is_valid_optimal_option(
            &option,
            &FindOptimalSide::Range(pos_or_panic!(95.0), pos_or_panic!(105.0))
        ));
    }

    #[test]
    fn test_create_strategy() {
        let butterfly = create_test_butterfly();
        let chain = create_test_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::FourLegs {
            first: options[1],  // 90.0 strike for long put
            second: options[3], // 100.0 strike for both shorts
            third: options[3],  // 100.0 strike for both shorts
            fourth: options[5], // 110.0 strike for long call
        };

        let new_strategy = butterfly.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
        assert_eq!(
            new_strategy.long_put.option.strike_price,
            pos_or_panic!(90.0)
        );
        assert_eq!(
            new_strategy.short_put.option.strike_price,
            Positive::HUNDRED
        );
        assert_eq!(
            new_strategy.short_call.option.strike_price,
            Positive::HUNDRED
        );
        assert_eq!(
            new_strategy.long_call.option.strike_price,
            pos_or_panic!(110.0)
        );
    }

    #[test]
    #[should_panic(expected = "Invalid number of legs for Iron Butterfly strategy")]
    fn test_create_strategy_invalid_legs() {
        let butterfly = create_test_butterfly();
        let chain = create_test_chain();
        let options: Vec<&OptionData> = chain.options.iter().collect();

        let legs = StrategyLegs::TwoLegs {
            first: options[0],
            second: options[1],
        };

        let _ = butterfly.create_strategy(&chain, &legs);
    }
}

#[cfg(test)]
mod tests_iron_butterfly_profit {
    use super::*;

    use crate::model::ExpirationDate;

    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
    use rust_decimal_macros::dec;

    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,    // underlying_price
            Positive::HUNDRED,    // short strike (both call and put)
            pos_or_panic!(110.0), // long call strike
            pos_or_panic!(90.0),  // long put strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2), // implied_volatility
            dec!(0.05),         // risk_free_rate
            Positive::ZERO,     // dividend_yield
            Positive::ONE,      // quantity
            Positive::TWO,      // premium_short_call
            Positive::TWO,      // premium_short_put
            Positive::ONE,      // premium_long_call
            Positive::ONE,      // premium_long_put
            Positive::ZERO,     // open_fee
            Positive::ZERO,     // closing fee
        )
    }

    #[test]
    fn test_profit_at_max_profit_price() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&butterfly.short_call.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        // Net premium = (2.0 + 2.0) - (1.0 + 1.0) = 2.0
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_below_long_put() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&pos_or_panic!(85.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_at_long_put() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&butterfly.long_put.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_between_put_wing() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&pos_or_panic!(95.0))
            .unwrap()
            .to_f64()
            .unwrap();
        let max_loss = -8.0;
        let max_profit = 2.0;
        assert!(profit > max_loss && profit < max_profit);
    }

    #[test]
    fn test_profit_at_short_strike() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&butterfly.short_call.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        // Maximum profit = net premium = 2.0
        assert_eq!(profit, 2.0);
    }

    #[test]
    fn test_profit_between_call_wing() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&pos_or_panic!(105.0))
            .unwrap()
            .to_f64()
            .unwrap();
        let max_loss = -8.0;
        let max_profit = 2.0;
        assert!(profit > max_loss && profit < max_profit);
    }

    #[test]
    fn test_profit_at_long_call() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&butterfly.long_call.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_above_long_call() {
        let butterfly = create_test_butterfly();
        let profit = butterfly
            .calculate_profit_at(&pos_or_panic!(115.0))
            .unwrap()
            .to_f64()
            .unwrap();
        // Maximum loss = width of wing - net premium = 10 - 2 = 8
        assert_eq!(profit, -8.0);
    }

    #[test]
    fn test_profit_with_fees() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.5), // open_fee
            pos_or_panic!(0.5), // closing fee
        );

        let profit = butterfly
            .calculate_profit_at(&Positive::HUNDRED)
            .unwrap()
            .to_f64()
            .unwrap();
        // Net premium = 2.0 - fees = 2.0 - 4.0 = -2.0
        assert_eq!(profit, -2.0);
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let butterfly = IronButterfly::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(90.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO, // quantity = 2
            Positive::TWO,
            Positive::TWO,
            Positive::ONE,
            Positive::ONE,
            Positive::ZERO,
            Positive::ZERO,
        );

        let profit = butterfly
            .calculate_profit_at(&butterfly.short_call.option.strike_price)
            .unwrap()
            .to_f64()
            .unwrap();
        // Net premium * quantity = 2.0 * 2 = 4.0
        assert_eq!(profit, 4.0);
    }

    #[test]
    fn test_profit_at_break_even_points() {
        let butterfly = create_test_butterfly();

        // Break-evens should be equidistant from short strike
        let short_strike = butterfly.short_call.option.strike_price;
        let lower_break_even = pos_or_panic!((short_strike - 2.0).to_f64());
        let upper_break_even = pos_or_panic!((short_strike + 2.0).to_f64());

        let lower_profit = butterfly.calculate_profit_at(&lower_break_even).unwrap();
        let upper_profit = butterfly.calculate_profit_at(&upper_break_even).unwrap();

        assert!(lower_profit.abs() < dec!(0.001));
        assert!(upper_profit.abs() < dec!(0.001));

        // Break-evens should be equidistant from short strike
        assert!(
            (lower_break_even.to_f64() - short_strike.to_f64()).abs()
                == (upper_break_even.to_f64() - short_strike.to_f64()).abs()
        );
    }

    #[test]
    fn test_symmetric_profits() {
        let butterfly = create_test_butterfly();
        let short_strike = butterfly.short_call.option.strike_price;

        // Test points equidistant from short strike should have equal profits
        for offset in [2.0, 4.0, 6.0, 8.0] {
            let up_profit = butterfly
                .calculate_profit_at(&pos_or_panic!((short_strike + offset).to_f64()))
                .unwrap();
            let down_profit = butterfly
                .calculate_profit_at(&pos_or_panic!((short_strike - offset).to_f64()))
                .unwrap();
            assert!((up_profit - down_profit).abs() < dec!(0.001));
        }
    }
}

#[cfg(test)]
mod tests_iron_butterfly_delta {
    use super::*;
    use positive::{assert_pos_relative_eq, pos_or_panic};

    use crate::assert_decimal_eq;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_butterfly::IronButterfly;
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            underlying_price,      // underlying_price
            pos_or_panic!(2725.0), // short_call_strike
            pos_or_panic!(2800.0), // long_call_strike
            pos_or_panic!(2500.0), // long_put_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.1548), // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::ONE,         // quantity
            pos_or_panic!(38.8),   // premium_short_call
            pos_or_panic!(30.4),   // premium_short_put
            pos_or_panic!(23.3),   // premium_long_call
            pos_or_panic!(16.8),   // premium_long_put
            pos_or_panic!(0.96),   // open_fee
            pos_or_panic!(0.96),   // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(2900.0));
        let size = dec!(-0.053677);
        let delta1 = pos_or_panic!(0.0573840487746411);
        let delta2 = pos_or_panic!(0.8309463413138215);
        let k1 = pos_or_panic!(2725.0);
        let k2 = pos_or_panic!(2725.0);
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
        let strategy = get_strategy(pos_or_panic!(2500.0));
        let size = dec!(0.485367);
        let delta1 = pos_or_panic!(14.3398655875839);
        let delta2 = pos_or_panic!(0.50237115863231);
        let k1 = pos_or_panic!(2725.0);
        let k2 = pos_or_panic!(2725.0);
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
        let strategy = get_strategy(pos_or_panic!(2100.0));

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
mod tests_iron_butterfly_delta_size {
    use super::*;
    use positive::{assert_pos_relative_eq, pos_or_panic};

    use crate::assert_decimal_eq;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::iron_butterfly::IronButterfly;
    use rust_decimal_macros::dec;

    fn get_strategy(underlying_price: Positive) -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            underlying_price,      // underlying_price
            pos_or_panic!(2725.0), // short_call_strike
            pos_or_panic!(2800.0), // long_call_strike
            pos_or_panic!(2500.0), // long_put_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.1548), // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::TWO,         // quantity
            pos_or_panic!(38.8),   // premium_short_call
            pos_or_panic!(30.4),   // premium_short_put
            pos_or_panic!(23.3),   // premium_long_call
            pos_or_panic!(16.8),   // premium_long_put
            pos_or_panic!(0.96),   // open_fee
            pos_or_panic!(0.96),   // close_fee
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(2900.0));
        let size = dec!(-0.107354);
        let delta1 = pos_or_panic!(0.1147680975492);
        let delta2 = pos_or_panic!(1.6618926826276);
        let k1 = pos_or_panic!(2725.0);
        let k2 = pos_or_panic!(2725.0);
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
        let strategy = get_strategy(pos_or_panic!(2700.0));
        let size = dec!(0.5645588522918766);
        let delta1 = pos_or_panic!(1.219357854222913);
        let delta2 = pos_or_panic!(1.051313866824854);
        let k1 = pos_or_panic!(2725.0);
        let k2 = pos_or_panic!(2725.0);
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
        let strategy = get_strategy(pos_or_panic!(2090.0));

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
mod tests_iron_butterfly_probability {
    use super::*;

    use crate::strategies::probabilities::utils::PriceTrend;
    use num_traits::ToPrimitive;
    use positive::{assert_pos_relative_eq, pos_or_panic};
    use rust_decimal_macros::dec;

    /// Creates a test Iron Butterfly with standard parameters
    fn create_test_butterfly() -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            pos_or_panic!(2646.9), // underlying_price
            pos_or_panic!(2725.0), // short_call_strike
            pos_or_panic!(2800.0), // long_call_strike
            pos_or_panic!(2500.0), // long_put_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.1548), // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::ONE,         // quantity
            pos_or_panic!(38.8),   // premium_short_call
            pos_or_panic!(30.4),   // premium_short_put
            pos_or_panic!(23.3),   // premium_long_call
            pos_or_panic!(16.8),   // premium_long_put
            pos_or_panic!(0.96),   // open_fee
            pos_or_panic!(0.96),   // close_fee
        )
    }

    #[test]
    fn test_get_expiration() {
        let butterfly = create_test_butterfly();
        let expiration = *butterfly.get_expiration().values().next().unwrap();
        assert_eq!(expiration, &ExpirationDate::Days(pos_or_panic!(30.0)));
    }

    #[test]
    fn test_get_risk_free_rate() {
        let butterfly = create_test_butterfly();
        assert_eq!(
            butterfly
                .get_risk_free_rate()
                .values()
                .next()
                .unwrap()
                .to_f64()
                .unwrap(),
            0.05
        );
    }

    #[test]
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
        assert!(range.probability <= Positive::ONE);
    }

    #[test]
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
    fn test_probability_sum_to_one() {
        let butterfly = create_test_butterfly();

        let profit_ranges = butterfly.get_profit_ranges().unwrap();
        let loss_ranges = butterfly.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        // Total probability should be approximately 1
        assert_pos_relative_eq!(
            total_profit_prob + total_loss_prob,
            Positive::ONE,
            pos_or_panic!(0.0001)
        );
    }

    #[test]
    fn test_break_even_points_validity() {
        let butterfly = create_test_butterfly();
        let break_even_points = butterfly.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 2);
        // Lower break-even should be between long put and short put
        assert!(break_even_points[0] >= butterfly.long_put.option.strike_price);
        assert!(break_even_points[0] <= butterfly.short_put.option.strike_price);
        // Upper break-even should be between short call and long call
        assert!(break_even_points[1] >= butterfly.short_call.option.strike_price);
        assert!(break_even_points[1] <= butterfly.long_call.option.strike_price);
    }

    #[test]
    fn test_with_volatility_adjustment() {
        let butterfly = create_test_butterfly();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.05),
        });

        let prob = butterfly.probability_of_profit(vol_adj, None);
        assert!(prob.is_ok());
        let probability = prob.unwrap();
        assert!(probability > Positive::ZERO);
        assert!(probability <= Positive::ONE);
    }

    #[test]
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
        assert!(probability <= Positive::ONE);
    }

    #[test]
    fn test_extreme_probabilities() {
        let butterfly = create_test_butterfly();
        let result = butterfly.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();

        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        // Sum of extreme probabilities should be less than or equal to 1
        assert!(max_profit_prob + max_loss_prob <= Positive::ONE);
    }

    #[test]
    fn test_zero_volatility() {
        let mut butterfly = create_test_butterfly();
        // Set invalid volatility for one leg
        butterfly.short_call.option.implied_volatility = Positive::ZERO;

        let result = butterfly.get_profit_ranges();
        assert!(result.is_ok());
    }

    #[test]
    fn test_different_expirations() {
        let mut butterfly = create_test_butterfly();
        let expirations = vec![
            ExpirationDate::Days(pos_or_panic!(7.0)),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            ExpirationDate::Days(pos_or_panic!(90.0)),
        ];

        for expiration in expirations {
            butterfly.long_put.option.expiration_date = expiration;
            butterfly.short_put.option.expiration_date = expiration;
            butterfly.short_call.option.expiration_date = expiration;
            butterfly.long_call.option.expiration_date = expiration;

            let result = butterfly.probability_of_profit(None, None);
            assert!(result.is_ok());
            let prob = result.unwrap();
            assert!(prob > Positive::ZERO);
            assert!(prob <= Positive::ONE);
        }
    }
}

#[cfg(test)]
mod tests_iron_butterfly_position_management {
    use super::*;
    use positive::pos_or_panic;

    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    fn create_test_iron_butterfly() -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            pos_or_panic!(2646.9), // underlying_price
            pos_or_panic!(2725.0), // short_call_strike
            pos_or_panic!(2800.0), // long_call_strike
            pos_or_panic!(2500.0), // long_put_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.1548), // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::TWO,         // quantity
            pos_or_panic!(38.8),   // premium_short_call
            pos_or_panic!(30.4),   // premium_short_put
            pos_or_panic!(23.3),   // premium_long_call
            pos_or_panic!(16.8),   // premium_long_put
            pos_or_panic!(0.96),   // open_fee
            pos_or_panic!(0.96),   // close_fee
        )
    }

    #[test]
    fn test_short_iron_butterfly_get_position() {
        let mut iron_butterfly = create_test_iron_butterfly();

        // Test getting short call position
        let call_position =
            iron_butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(2725.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(2725.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position =
            iron_butterfly.get_position(&OptionStyle::Put, &Side::Short, &pos_or_panic!(2725.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(2725.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            iron_butterfly.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(2715.0));
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
    fn test_long_iron_butterfly_get_position() {
        let mut iron_butterfly = create_test_iron_butterfly();

        // Test getting short call position
        let call_position =
            iron_butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos_or_panic!(2800.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(2800.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            iron_butterfly.get_position(&OptionStyle::Put, &Side::Long, &pos_or_panic!(2500.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(2500.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting non-existent position
        let invalid_position =
            iron_butterfly.get_position(&OptionStyle::Call, &Side::Long, &pos_or_panic!(2715.0));
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
    fn test_short_iron_butterfly_modify_position() {
        let mut iron_butterfly = create_test_iron_butterfly();

        // Modify short call position
        let mut modified_call = iron_butterfly.short_call.clone();
        modified_call.option.quantity = Positive::TWO;
        let result = iron_butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(iron_butterfly.short_call.option.quantity, Positive::TWO);

        // Modify short put position
        let mut modified_put = iron_butterfly.short_put.clone();
        modified_put.option.quantity = Positive::TWO;
        let result = iron_butterfly.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(iron_butterfly.short_put.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = iron_butterfly.short_call.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
        let result = iron_butterfly.modify_position(&invalid_position);
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
    fn test_long_iron_butterfly_modify_position() {
        let mut iron_butterfly = create_test_iron_butterfly();

        // Modify long call position
        let mut modified_call = iron_butterfly.long_call.clone();
        modified_call.option.quantity = Positive::TWO;
        let result = iron_butterfly.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(iron_butterfly.long_call.option.quantity, Positive::TWO);

        // Modify long put position
        let mut modified_put = iron_butterfly.long_put.clone();
        modified_put.option.quantity = Positive::TWO;
        let result = iron_butterfly.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(iron_butterfly.long_put.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = iron_butterfly.long_call.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
        let result = iron_butterfly.modify_position(&invalid_position);
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
    use positive::pos_or_panic;

    use crate::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> IronButterfly {
        IronButterfly::new(
            "GOLD".to_string(),
            pos_or_panic!(2646.9), // underlying_price
            pos_or_panic!(2725.0), // short_call_strike
            pos_or_panic!(2800.0), // long_call_strike
            pos_or_panic!(2500.0), // long_put_strike
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.1548), // implied_volatility
            dec!(0.05),            // risk_free_rate
            Positive::ZERO,        // dividend_yield
            Positive::TWO,         // quantity
            pos_or_panic!(38.8),   // premium_short_call
            pos_or_panic!(30.4),   // premium_short_put
            pos_or_panic!(23.3),   // premium_long_call
            pos_or_panic!(16.8),   // premium_long_put
            pos_or_panic!(0.96),   // open_fee
            pos_or_panic!(0.96),   // close_fee
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(2725.0),
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
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(2725.0),
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
            &pos_or_panic!(110.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        // StrategyError wraps PositionError, so we check the error message
        assert!(err.to_string().contains("Strike not found in positions"));
    }

    #[test]
    fn test_adjust_with_invalid_strike() {
        let mut strategy = create_test_strategy();

        // Try to adjust position with wrong strike price
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &Positive::HUNDRED, // Invalid strike price
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
            &pos_or_panic!(2725.0),
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
    use positive::pos_or_panic;

    use crate::model::utils::create_sample_position;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = IronButterfly::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_put.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(strategy.short_put.option.strike_price, Positive::HUNDRED);
        assert_eq!(strategy.short_call.option.strike_price, Positive::HUNDRED);
        assert_eq!(strategy.long_call.option.strike_price, pos_or_panic!(105.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
        ];

        let result = IronButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Butterfly get_strategy" && reason == "Must have exactly 4 options"
        ));
    }

    #[test]
    fn test_get_strategy_non_equidistant_strikes() {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(90.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(115.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = IronButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Butterfly get_strategy" && reason == "Strike prices must be equidistant"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        options[3].option.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));

        let result = IronButterfly::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Iron Butterfly get_strategy" && reason == "All options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_iron_butterfly_pnl {
    use super::*;
    use positive::pos_or_panic;

    use crate::assert_decimal_eq;
    use crate::model::utils::create_sample_position;
    use rust_decimal_macros::dec;

    fn create_test_iron_butterfly() -> Result<IronButterfly, StrategyError> {
        let options = vec![
            create_sample_position(
                OptionStyle::Put,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                Positive::HUNDRED,
                Positive::ONE,
                Positive::HUNDRED,
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                Positive::HUNDRED,
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        IronButterfly::get_strategy(&options)
    }

    #[test]
    fn test_calculate_pnl_at_middle_strike() {
        let iron_butterfly = create_test_iron_butterfly().unwrap();
        let underlying_price = Positive::HUNDRED; // At middle strike

        let result = iron_butterfly.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Maximum profit at middle strike
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-4.0), dec!(0.01));
    }

    #[test]
    fn test_calculate_pnl_at_wing_strikes() {
        let iron_butterfly = create_test_iron_butterfly().unwrap();

        // Test at lower wing
        let result_lower = iron_butterfly.calculate_pnl_at_expiration(&pos_or_panic!(95.0));
        assert!(result_lower.is_ok());
        let pnl_lower = result_lower.unwrap();
        assert_decimal_eq!(pnl_lower.realized.unwrap(), dec!(-9.0), dec!(0.01));

        // Test at upper wing
        let result_upper = iron_butterfly.calculate_pnl_at_expiration(&pos_or_panic!(105.0));
        assert!(result_upper.is_ok());
        let pnl_upper = result_upper.unwrap();
        assert_decimal_eq!(pnl_upper.realized.unwrap(), dec!(-9.0), dec!(0.01));
    }

    #[test]
    fn test_calculate_pnl_beyond_wings() {
        let iron_butterfly = create_test_iron_butterfly().unwrap();

        // Test far below wings
        let result_lower = iron_butterfly.calculate_pnl_at_expiration(&pos_or_panic!(90.0));
        assert!(result_lower.is_ok());
        let pnl_lower = result_lower.unwrap();
        assert_decimal_eq!(pnl_lower.realized.unwrap(), dec!(-9.0), dec!(0.01));

        // Test far above wings
        let result_upper = iron_butterfly.calculate_pnl_at_expiration(&pos_or_panic!(110.0));
        assert!(result_upper.is_ok());
        let pnl_upper = result_upper.unwrap();
        assert_decimal_eq!(pnl_upper.realized.unwrap(), dec!(-9.0), dec!(0.01));
    }
}
