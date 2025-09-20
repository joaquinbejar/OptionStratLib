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
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::chains::OptionData;
use crate::{
    ExpirationDate, Options, Positive,
    chains::{StrategyLegs, chain::OptionChain},
    constants::ZERO,
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
    test_strategy_traits,
};
use chrono::Utc;
use num_traits::FromPrimitive;
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::debug;
use utoipa::ToSchema;

pub(super) const PMCC_DESCRIPTION: &str = "A Poor Man's Covered Call (PMCC) is an options strategy that simulates a covered call \
    using long-term equity anticipation securities (LEAPS) instead of the underlying stock. \
    It involves buying a long-term in-the-money call option and selling a short-term out-of-the-money call option. \
    This strategy aims to generate income while reducing the capital required compared to a traditional covered call.";

/// # PoorMansCoveredCall
///
/// Represents a Poor Man's Covered Call options trading strategy. This strategy is a cost-effective
/// alternative to the traditional covered call, using a deep in-the-money long-term call option
/// instead of owning the underlying stock, while selling shorter-term out-of-the-money call options.
///
/// A Poor Man's Covered Call (also known as a PMCC or Diagonal Debit Call Spread) requires less capital
/// than a standard covered call while still providing similar profit potential and risk profile.
///
/// ## Fields
/// * `name`: A descriptive name for the specific strategy instance.
/// * `kind`: The type of strategy, which is `StrategyType::PoorMansCoveredCall`.
/// * `description`: A detailed description of this specific strategy instance.
/// * `break_even_points`: The price points at which the strategy breaks even (neither profit nor loss).
/// * `long_call`: The long call position (typically a LEAP - Long-Term Equity Anticipation Security).
/// * `short_call`: The short call position (shorter-term, out-of-the-money call).
///
/// ## Risk and Reward
/// The maximum risk in this strategy is limited to the net debit paid (cost of the long call minus
/// the premium received for the short call).
///
/// The maximum profit is capped and occurs when the underlying price at expiration of the short call
/// equals or exceeds the strike price of the short call.
///
/// ## Break-Even Point
/// The break-even point at expiration of the short call is approximately the strike price of the long call
/// plus the net debit paid for the spread.
///
/// ## Strategy Usage
/// This strategy is typically used when:
/// - The trader is moderately to strongly bullish on the underlying asset
/// - The trader wants to generate income while still participating in potential upside movement
/// - The trader wants to implement a covered call strategy with less capital investment
/// - Implied volatility is relatively high for near-term options
///
/// ## Management Considerations
/// - The strategy often involves rolling the short call forward to continue generating income
/// - The long call should have sufficient time value to avoid assignment complications
/// - Ideally implemented when the underlying asset has a strong positive outlook over the long term
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct PoorMansCoveredCall {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a PoorMansCoveredCall strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The long-term in-the-money call option (usually a LEAP)
    pub(super) long_call: Position,
    /// The shorter-term out-of-the-money call option that is sold
    pub(super) short_call: Position,
}

impl PoorMansCoveredCall {
    /// # Creates a new Poor Man's Covered Call strategy instance
    ///
    /// This function constructs and initializes a Poor Man's Covered Call (PMCC) options strategy, which simulates
    /// a covered call using a long-term deep in-the-money call option (LEAPS) instead of owning the underlying stock,
    /// paired with selling a shorter-term out-of-the-money call.
    ///
    /// ## Parameters
    /// * `underlying_symbol`: Symbol of the underlying security (e.g., "AAPL")
    /// * `underlying_price`: Current market price of the underlying security
    /// * `long_call_strike`: Strike price for the long-term call option (LEAPS)
    /// * `short_call_strike`: Strike price for the short-term call option
    /// * `long_call_expiration`: Expiration date for the long-term call option
    /// * `short_call_expiration`: Expiration date for the short-term call option
    /// * `implied_volatility`: Implied volatility used for options pricing
    /// * `risk_free_rate`: Risk-free interest rate used in options pricing models
    /// * `dividend_yield`: Expected dividend yield of the underlying security
    /// * `quantity`: Number of contracts for both legs of the strategy
    /// * `premium_long_call`: Premium paid for the long-term call option
    /// * `premium_short_call`: Premium received for the short-term call option
    /// * `open_fee_long_call`: Transaction fee for opening the long call position
    /// * `close_fee_long_call`: Transaction fee for closing the long call position
    /// * `open_fee_short_call`: Transaction fee for opening the short call position
    /// * `close_fee_short_call`: Transaction fee for closing the short call position
    ///
    /// ## Returns
    /// A fully initialized `PoorMansCoveredCall` instance with both option positions and calculated break-even points.
    ///
    /// ## Strategy Details
    /// The Poor Man's Covered Call is a diagonal spread that consists of:
    /// 1. Buying a deep in-the-money, longer-term call option (LEAPS)
    /// 2. Selling an out-of-the-money, shorter-term call option
    ///
    /// This strategy provides similar benefits to a traditional covered call but requires less capital,
    /// as purchasing a LEAPS call option is less expensive than buying 100 shares of the underlying stock.
    ///
    /// ## Risk-Reward Profile
    /// * Maximum profit: Limited to the short call's strike price minus the long call's strike price,
    ///   plus the net credit received (or minus the net debit paid)
    /// * Maximum loss: Limited to the net debit paid for the position (long call premium minus short call premium)
    ///   plus transaction fees
    /// * Break-even point: Long call strike price plus the net debit paid
    ///
    /// ## Ideal Market Conditions
    /// This strategy is optimal when:
    /// * The investor is moderately bullish on the underlying asset
    /// * The investor wants to generate income from the short calls while maintaining upside potential
    /// * The investor seeks a capital-efficient alternative to traditional covered calls
    ///
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
        let mut strategy = PoorMansCoveredCall::default();

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
            None,
            None,
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
            None,
            None,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Invalid short call option");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for PoorMansCoveredCall {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a poor man's covered call
        if vec_positions.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Poor Man's Covered Call get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Sort options by strike price to identify long and short positions
        let mut sorted_positions = vec_positions.to_vec();
        sorted_positions.sort_by(|a, b| {
            a.option
                .strike_price
                .partial_cmp(&b.option.strike_price)
                .unwrap()
        });

        let lower_strike_position = &sorted_positions[0];
        let higher_strike_position = &sorted_positions[1];

        // Validate options are calls
        if lower_strike_position.option.option_style != OptionStyle::Call
            || higher_strike_position.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Poor Man's Covered Call get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option sides
        if lower_strike_position.option.side != Side::Long
            || higher_strike_position.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "Poor Man's Covered Call get_strategy".to_string(),
                reason: "Poor Man's Covered Call requires a long lower strike call and a short higher strike call".to_string(),
            }));
        }

        // Create positions
        let long_call = Position::new(
            lower_strike_position.option.clone(),
            lower_strike_position.premium,
            Utc::now(),
            lower_strike_position.open_fee,
            lower_strike_position.close_fee,
            lower_strike_position.epic.clone(),
            lower_strike_position.extra_fields.clone(),
        );

        let short_call = Position::new(
            higher_strike_position.option.clone(),
            higher_strike_position.premium,
            Utc::now(),
            higher_strike_position.open_fee,
            higher_strike_position.close_fee,
            higher_strike_position.epic.clone(),
            higher_strike_position.extra_fields.clone(),
        );

        // Create strategy
        let mut strategy = PoorMansCoveredCall {
            name: "Poor Man's Covered Call".to_string(),
            kind: StrategyType::PoorMansCoveredCall,
            description: PMCC_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call,
            short_call,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for PoorMansCoveredCall {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let net_debit = self.get_net_cost()? / self.long_call.option.quantity;

        self.break_even_points
            .push((self.long_call.option.strike_price + net_debit).round_to(2));

        Ok(())
    }
}

impl Validable for PoorMansCoveredCall {
    fn validate(&self) -> bool {
        self.short_call.validate() && self.long_call.validate()
    }
}

impl Positionable for PoorMansCoveredCall {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match (position.option.option_style, position.option.side) {
            (OptionStyle::Call, Side::Long) => {
                self.long_call = position.clone();
                Ok(())
            }
            (OptionStyle::Call, Side::Short) => {
                self.short_call = position.clone();
                Ok(())
            }
            _ => Err(PositionError::invalid_position_style(
                position.option.option_style,
                "Position is a Put, it is not valid for PoorMansCoveredCall".to_string(),
            )),
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.long_call])
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
            (_, OptionStyle::Put, _) => Err(PositionError::invalid_position_type(
                *side,
                "Put is not valid for PoorMansCoveredCall".to_string(),
            )),
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                Ok(vec![&mut self.long_call])
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                Ok(vec![&mut self.short_call])
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

        match (
            &position.option.side,
            &position.option.option_style,
            &position.option.strike_price,
        ) {
            (_, OptionStyle::Put, _) => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Put is not valid for PoorMansCoveredCall".to_string(),
                ));
            }
            (Side::Long, OptionStyle::Call, strike)
                if *strike == self.long_call.option.strike_price =>
            {
                self.long_call = position.clone();
            }
            (Side::Short, OptionStyle::Call, strike)
                if *strike == self.short_call.option.strike_price =>
            {
                self.short_call = position.clone();
            }
            _ => {
                return Err(PositionError::invalid_position_type(
                    position.option.side,
                    "Strike not found in positions".to_string(),
                ));
            }
        }

        Ok(())
    }
}

impl Strategable for PoorMansCoveredCall {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for PoorMansCoveredCall {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.get_title(), self.long_call.get_title()]
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
        let long_call = &self.long_call.option;
        hash_set.insert(OptionBasicType {
            option_style: &short_call.option_style,
            side: &short_call.side,
            strike_price: &short_call.strike_price,
            expiration_date: &short_call.expiration_date,
        });
        hash_set.insert(OptionBasicType {
            option_style: &long_call.option_style,
            side: &long_call.side,
            strike_price: &long_call.strike_price,
            expiration_date: &long_call.expiration_date,
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
                &self.long_call.option,
                &self.long_call.option.implied_volatility,
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
            (&self.long_call.option, &self.long_call.option.quantity),
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
        self.long_call.option.expiration_date = expiration_date;
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
        self.long_call.option.underlying_price = *price;
        self.long_call.premium =
            Positive::from(self.long_call.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.long_call.option.implied_volatility = *volatility;
        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.long_call.premium =
            Positive(self.long_call.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Strategies for PoorMansCoveredCall {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.short_call.option.strike_price)?;
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

    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(&self.long_call.option.strike_price)?;
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

    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let base = (self.short_call.option.strike_price
            - (self.short_call.option.strike_price
                - self.get_max_profit().unwrap_or(Positive::ZERO)))
        .to_f64();
        let high = self.get_max_profit().unwrap_or(Positive::ZERO).to_f64();
        let result = base * high / 200.0;
        Ok(Decimal::from_f64(result).unwrap())
    }

    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let result = match (self.get_max_profit(), self.get_max_loss()) {
            (Ok(profit), Ok(loss)) => (profit / loss).to_f64() * 100.0,
            _ => ZERO,
        };
        Ok(Decimal::from_f64(result).unwrap())
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

                if side == FindOptimalSide::Center {
                    if !self.is_valid_optimal_option(short_call_option, &FindOptimalSide::Upper)
                        || !self.is_valid_optimal_option(long_call_option, &FindOptimalSide::Lower)
                    {
                        debug!("Invalid option");
                        continue;
                    }
                } else if !self.is_valid_optimal_option(short_call_option, &side)
                    || !self.is_valid_optimal_option(long_call_option, &side)
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
                    OptimizationCriteria::Ratio => strategy.get_profit_ratio().unwrap(),
                    OptimizationCriteria::Area => strategy.get_profit_area().unwrap(),
                };

                if current_value > best_value {
                    best_value = current_value;
                    *self = strategy.clone();
                }
            }
        }
    }

    fn create_strategy(&self, chain: &OptionChain, legs: &StrategyLegs) -> Self::Strategy {
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        let implied_volatility = short.implied_volatility;
        assert!(implied_volatility <= Positive::ONE);

        PoorMansCoveredCall::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_call.option.expiration_date,
            self.short_call.option.expiration_date,
            implied_volatility,
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
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        Ok(
            self.long_call.pnl_at_expiration(&price)?
                + self.short_call.pnl_at_expiration(&price)?,
        )
    }
}

impl ProbabilityAnalysis for PoorMansCoveredCall {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

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
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![profit_range])
    }

    fn get_loss_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

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
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![loss_range])
    }
}

impl Greeks for PoorMansCoveredCall {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option, &self.short_call.option])
    }
}

impl DeltaNeutrality for PoorMansCoveredCall {}

impl PnLCalculator for PoorMansCoveredCall {
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
                .short_call
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
                .short_call
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

test_strategy_traits!(PoorMansCoveredCall, test_short_call_implementations);

#[cfg(test)]
mod tests_pmcc_validation {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::error::position::PositionValidationErrorKind;
    use crate::pos;
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
    fn test_validate_valid_strategy() {
        let strategy = create_basic_strategy();
        assert!(strategy.validate());
    }

    #[test]
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
        let position = Position::new(
            option,
            pos!(15.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        strategy
            .add_position(&position.clone())
            .expect("Invalid long call option");
        assert_eq!(strategy.long_call, position);
    }

    #[test]
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
        let position = Position::new(
            option,
            pos!(5.0),
            Utc::now(),
            pos!(0.5),
            pos!(0.5),
            None,
            None,
        );
        strategy
            .add_position(&position.clone())
            .expect("Invalid short call option");
        assert_eq!(strategy.short_call, position);
    }

    #[test]
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
        let position = Position::new(
            option,
            pos!(15.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
    use crate::chains::OptionData;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::{pos, spos};
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
                pos!(0.2),
                Some(dec!(0.5)),
                None,
                None,
                spos!(100.0),
                Some(50),
                None,
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
    fn test_is_valid_short_option() {
        let strategy = create_base_strategy();
        let option = OptionData::new(
            pos!(160.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(strategy.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_is_valid_long_option() {
        let strategy = create_base_strategy();
        let option = OptionData::new(
            pos!(140.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(strategy.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut strategy = create_base_strategy();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);
        assert!(strategy.validate());
    }

    #[test]
    fn test_find_optimal_area() {
        let mut strategy = create_base_strategy();
        let chain = create_test_option_chain();
        strategy.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);
        assert!(strategy.validate());
    }

    #[test]
    #[should_panic]
    fn test_invalid_short_option_zero_underlying() {
        let mut strategy = create_base_strategy();
        strategy.short_call.option.underlying_price = Positive::ZERO;
        let option = OptionData::new(
            pos!(160.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(!strategy.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
    }

    #[test]
    fn test_invalid_long_option_zero_underlying() {
        let mut strategy = create_base_strategy();
        let result = strategy.set_underlying_price(&Positive::ZERO);
        assert!(result.is_err());
        let option = OptionData::new(
            pos!(140.0),
            spos!(5.0),
            spos!(5.2),
            spos!(4.8),
            spos!(5.0),
            pos!(0.2),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(!strategy.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
    }
}

#[cfg(test)]
mod tests_pmcc_pnl {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::pos;
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
    fn test_calculate_profit_at_various_prices() {
        let strategy = create_test_strategy();

        // Below long strike
        let profit_below = strategy.calculate_profit_at(&pos!(130.0)).unwrap();
        assert!(profit_below < Decimal::ZERO);

        // Between strikes
        let profit_middle = strategy.calculate_profit_at(&pos!(150.0)).unwrap();
        assert!(profit_middle > profit_below);

        // At short strike
        let profit_short = strategy
            .calculate_profit_at(&strategy.short_call.option.strike_price)
            .unwrap();
        assert_eq!(
            profit_short,
            strategy.get_max_profit().unwrap_or(Positive::ZERO).to_dec()
        );

        // Above short strike
        let profit_above = strategy.calculate_profit_at(&pos!(170.0)).unwrap();
        assert_eq!(profit_above, profit_above);
    }

    #[test]
    fn test_break_even_point() {
        let strategy = create_test_strategy();
        assert_eq!(strategy.break_even_points.len(), 1);
        let break_even = strategy.break_even_points[0];
        let profit_at_be = strategy
            .calculate_profit_at(&break_even)
            .unwrap()
            .to_f64()
            .unwrap();
        assert!(profit_at_be.abs() < 0.01);
    }

    #[test]
    fn test_net_premium() {
        let strategy = create_test_strategy();
        let net_premium = strategy.get_net_premium_received().unwrap();
        assert_eq!(net_premium, 0.0);
    }

    #[test]
    fn test_max_profit_max_loss_relationship() {
        let strategy = create_test_strategy();
        assert!(strategy.get_max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(strategy.get_max_loss().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(
            strategy.get_max_loss().unwrap_or(Positive::ZERO)
                > strategy.get_max_profit().unwrap_or(Positive::ZERO)
        );
    }
}

#[cfg(test)]
mod tests_pmcc_best_area {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::pos;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn set_up() -> Result<(PoorMansCoveredCall, OptionChain), String> {
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
    fn test_best_area_all() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.get_best_area(&option_chain, FindOptimalSide::All);

        assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert!(strategy.get_total_cost().unwrap() > Positive::ZERO);
        assert!(strategy.get_fees().unwrap().to_f64() > 0.0);

        assert!(strategy.long_call.option.strike_price < strategy.short_call.option.strike_price);
    }

    #[test]
    fn test_best_area_upper() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.get_best_area(&option_chain, FindOptimalSide::Upper);

        assert!(strategy.long_call.option.strike_price >= *strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.get_max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
    }

    #[test]
    fn test_best_area_lower() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.get_best_area(&option_chain, FindOptimalSide::Lower);

        assert!(strategy.long_call.option.strike_price <= *strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.get_profit_area().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_pmcc_best_ratio {
    use super::*;
    use crate::constants::DAYS_IN_A_YEAR;
    use crate::pos;

    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn set_up() -> Result<(PoorMansCoveredCall, OptionChain), String> {
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
    fn test_best_ratio_all() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.get_best_ratio(&option_chain, FindOptimalSide::All);

        assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert_eq!(strategy.break_even_points.len(), 1);
        assert!(strategy.get_max_profit().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(strategy.get_max_loss().unwrap_or(Positive::ZERO) > Positive::ZERO);
        assert!(strategy.get_fees().unwrap().to_f64() > 0.0);
    }

    #[test]
    fn test_best_ratio_upper() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.get_best_ratio(&option_chain, FindOptimalSide::Upper);

        assert!(strategy.long_call.option.strike_price >= *strategy.get_underlying_price());
        assert!(strategy.short_call.option.strike_price > strategy.long_call.option.strike_price);

        assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_ratio_with_range() {
        let (mut strategy, option_chain) = set_up().unwrap();
        strategy.get_best_ratio(
            &option_chain,
            FindOptimalSide::Range(pos!(5750.0), pos!(5850.0)),
        );

        assert!(strategy.long_call.option.strike_price >= pos!(5750.0));
        assert!(strategy.short_call.option.strike_price <= pos!(5850.0));

        assert!(strategy.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0);
        assert!(strategy.validate());
    }
}

#[cfg(test)]
mod tests_short_straddle_delta {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::poor_mans_covered_call::PoorMansCoveredCall;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
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
            pos!(353.2),    // premium_long_call
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_long_call
            pos!(7.01),     // close_fee_long_call
        )
    }

    #[test]
    fn create_test_short_straddle_reducing_adjustments() {
        let strategy = get_strategy(pos!(7250.0), pos!(7300.0));
        let size = dec!(0.0887293);
        let delta = pos!(0.2168462168831);
        let k = pos!(7300.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_straddle_increasing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let size = dec!(-0.028694805);
        let delta = pos!(0.0689809869957862);
        let k = pos!(7450.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_straddle_no_adjustments() {
        let strategy = get_strategy(pos!(7379.0), pos!(7250.0));

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
mod tests_short_straddle_delta_size {
    use super::*;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::strategies::poor_mans_covered_call::PoorMansCoveredCall;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
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
            pos!(353.2),    // premium_long_call
            pos!(7.01),     // open_fee_short_call
            pos!(7.01),     // close_fee_short_call
            pos!(7.01),     // open_fee_long_call
            pos!(7.01),     // close_fee_long_call
        )
    }

    #[test]
    fn create_test_short_straddle_reducing_adjustments() {
        let strategy = get_strategy(pos!(7250.1), pos!(7300.0));
        let size = dec!(0.1773);
        let delta = pos!(0.4334878994986714);
        let k = pos!(7300.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();

        match &binding[1] {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_straddle_increasing_adjustments() {
        let strategy = get_strategy(pos!(7450.0), pos!(7250.0));
        let size = dec!(-0.057389);
        let delta = pos!(0.1379619739915724);
        let k = pos!(7450.0);
        assert_decimal_eq!(
            strategy.delta_neutrality().unwrap().net_delta,
            size,
            DELTA_THRESHOLD
        );
        assert!(!strategy.is_delta_neutral());
        let binding = strategy.delta_adjustments().unwrap();
        let suggestion = binding.first().unwrap();
        match suggestion {
            DeltaAdjustment::BuyOptions {
                quantity,
                strike,
                option_style,
                side,
            } => {
                assert_pos_relative_eq!(*quantity, delta, Positive(DELTA_THRESHOLD));
                assert_pos_relative_eq!(*strike, k, Positive(DELTA_THRESHOLD));
                assert_eq!(*option_style, OptionStyle::Call);
                assert_eq!(*side, Side::Long);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.long_call.option.clone();
        option.quantity = delta;
        let delta = option.delta().unwrap();
        assert_decimal_eq!(delta, -size, DELTA_THRESHOLD);
        assert_decimal_eq!(
            delta + strategy.delta_neutrality().unwrap().net_delta,
            Decimal::ZERO,
            DELTA_THRESHOLD
        );
    }

    #[test]
    fn create_test_short_straddle_no_adjustments() {
        let strategy = get_strategy(pos!(7387.5), pos!(7255.0));

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
mod tests_poor_mans_covered_call_probability {
    use super::*;
    use crate::strategies::probabilities::utils::PriceTrend;
    use crate::{assert_pos_relative_eq, pos};
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
            pos!(30.8),                        // premium_long_call
            pos!(1.74),                        // open_fee_short_call
            pos!(1.74),                        // close_fee_short_call
            pos!(0.85),                        // open_fee_long_call
            pos!(0.85),                        // close_fee_long_call
        )
    }

    #[test]
    fn test_get_expiration() {
        let pmcc = create_test_pmcc();
        let expected_dates = [
            ExpirationDate::Days(pos!(30.0)),
            ExpirationDate::Days(pos!(120.0)),
        ];

        for date in pmcc.get_expiration().values() {
            assert!(expected_dates.contains(date));
        }
    }

    #[test]
    fn test_get_risk_free_rate() {
        let pmcc = create_test_pmcc();
        assert_eq!(
            **pmcc.get_risk_free_rate().values().next().unwrap(),
            dec!(0.05)
        );
    }

    #[test]
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
    fn test_probability_sum_to_one() {
        let pmcc = create_test_pmcc();

        let profit_ranges = pmcc.get_profit_ranges().unwrap();
        let loss_ranges = pmcc.get_loss_ranges().unwrap();

        let total_profit_prob: Positive = profit_ranges.iter().map(|r| r.probability).sum();

        let total_loss_prob: Positive = loss_ranges.iter().map(|r| r.probability).sum();

        assert_pos_relative_eq!(total_profit_prob + total_loss_prob, pos!(1.0), pos!(0.0001));
    }

    #[test]
    fn test_break_even_points_validity() {
        let pmcc = create_test_pmcc();
        let break_even_points = pmcc.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        // Break-even point should be above long call strike and below short call strike
        assert!(break_even_points[0] > pmcc.long_call.option.strike_price);
        assert!(break_even_points[0] < pmcc.short_call.option.strike_price);
    }

    #[test]
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
    fn test_analyze_probabilities() {
        let pmcc = create_test_pmcc();
        let analysis = pmcc.analyze_probabilities(None, None).unwrap();

        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.expected_value >= Positive::ZERO);
        assert_eq!(analysis.break_even_points.len(), 1);
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
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
    fn test_strike_price_validity() {
        let pmcc = create_test_pmcc();
        // Short call strike should be higher than long call strike for a valid PMCC
        assert!(pmcc.short_call.option.strike_price > pmcc.long_call.option.strike_price);
    }
}

#[cfg(test)]
mod tests_poor_mans_covered_call_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_short_poor_mans_covered_call() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "GOLD".to_string(),                // underlying_symbol
            pos!(2703.3),                      // underlying_price
            pos!(2600.0),                      // long_call_strike
            pos!(2800.0),                      // short_call_strike OTM
            ExpirationDate::Days(pos!(120.0)), // long_call_expiration
            ExpirationDate::Days(pos!(30.0)), // short_call_expiration 30-45 days delta 0.30 or less
            pos!(0.17),                       // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(2.0),                        // quantity
            pos!(154.7),                      // premium_short_call
            pos!(30.8),                       // premium_long_call
            pos!(1.74),                       // open_fee_short_call
            pos!(1.74),                       // close_fee_short_call
            pos!(0.85),                       // open_fee_long_call
            pos!(0.85),                       // close_fee_long_call
        )
    }

    #[test]
    fn test_short_poor_mans_covered_call_get_position() {
        let mut poor_mans_covered_call = create_test_short_poor_mans_covered_call();

        // Test getting short call position
        let call_position =
            poor_mans_covered_call.get_position(&OptionStyle::Call, &Side::Long, &pos!(2600.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(2600.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            poor_mans_covered_call.get_position(&OptionStyle::Call, &Side::Short, &pos!(2800.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(2800.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            poor_mans_covered_call.get_position(&OptionStyle::Call, &Side::Short, &pos!(2801.0));
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
    fn test_short_poor_mans_covered_call_modify_position() {
        let mut poor_mans_covered_call = create_test_short_poor_mans_covered_call();

        // Modify short call position
        let mut modified_call = poor_mans_covered_call.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = poor_mans_covered_call.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(poor_mans_covered_call.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = poor_mans_covered_call.long_call.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = poor_mans_covered_call.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(poor_mans_covered_call.long_call.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = poor_mans_covered_call.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = poor_mans_covered_call.modify_position(&invalid_position);
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
    fn create_test_strategy() -> PoorMansCoveredCall {
        PoorMansCoveredCall::new(
            "GOLD".to_string(),                // underlying_symbol
            pos!(2703.3),                      // underlying_price
            pos!(2600.0),                      // long_call_strike
            pos!(2800.0),                      // short_call_strike OTM
            ExpirationDate::Days(pos!(120.0)), // long_call_expiration
            ExpirationDate::Days(pos!(30.0)), // short_call_expiration 30-45 days delta 0.30 or less
            pos!(0.17),                       // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(2.0),                        // quantity
            pos!(154.7),                      // premium_short_call
            pos!(30.8),                       // premium_long_call
            pos!(1.74),                       // open_fee_short_call
            pos!(1.74),                       // close_fee_short_call
            pos!(0.85),                       // open_fee_long_call
            pos!(0.85),                       // close_fee_long_call
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(2800.0),
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
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(2600.0),
            &OptionStyle::Call,
            &Side::Long,
        );

        assert!(result.is_ok());
        assert_eq!(
            strategy.long_call.option.quantity,
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
            &OptionStyle::Put,
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
                assert_eq!(reason, "Put is not valid for PoorMansCoveredCall");
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
            &pos!(2800.0),
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
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];

        let result = PoorMansCoveredCall::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_call.option.strike_price, pos!(95.0));
        assert_eq!(strategy.short_call.option.strike_price, pos!(105.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        )];

        let result = PoorMansCoveredCall::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Poor Man's Covered Call get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        option1.option.option_style = OptionStyle::Put;
        let option2 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(105.0),
            pos!(0.2),
        );

        let options = vec![option1, option2];
        let result = PoorMansCoveredCall::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Poor Man's Covered Call get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(90.0),
                pos!(1.0),
                pos!(95.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(90.0),
                pos!(1.0),
                pos!(105.0),
                pos!(0.2),
            ),
        ];
        let result = PoorMansCoveredCall::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Poor Man's Covered Call get_strategy"
                && reason == "Poor Man's Covered Call requires a long lower strike call and a short higher strike call"
        ));
    }
}

#[cfg(test)]
mod tests_poor_mans_covered_call_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_poor_mans_covered_call() -> Result<PoorMansCoveredCall, StrategyError> {
        // Create long call with lower strike
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(95.0),  // Strike price (Lower)
            pos!(0.2),   // Implied volatility
        );

        // Create short call with higher strike
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(105.0), // Higher strike price
            pos!(0.2),   // Implied volatility
        );

        PoorMansCoveredCall::get_strategy(&vec![long_call, short_call])
    }

    #[test]
    fn test_calculate_pnl_below_strikes() {
        let pmcc = create_test_poor_mans_covered_call().unwrap();
        let market_price = pos!(90.0); // Below both strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = pmcc.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options OTM
        assert_pos_relative_eq!(pnl.initial_income, pos!(5.0), pos!(1e-6));
        assert_pos_relative_eq!(pnl.initial_costs, pos!(7.0), pos!(1e-6));
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Loss due to time decay
    }

    #[test]
    fn test_calculate_pnl_between_strikes() {
        let pmcc = create_test_poor_mans_covered_call().unwrap();
        let market_price = pos!(101.0); // Between strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = pmcc.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Long call ITM, short call OTM
        assert!(pnl.unrealized.unwrap() > dec!(0.0)); // Should show some profit
    }

    #[test]
    fn test_calculate_pnl_above_strikes() {
        let pmcc = create_test_poor_mans_covered_call().unwrap();
        let market_price = pos!(110.0); // Above both strikes
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = pmcc.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options ITM, profit limited
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
        assert!(pnl.unrealized.unwrap() < dec!(10.0)); // Maximum profit is width of spread
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let pmcc = create_test_poor_mans_covered_call().unwrap();
        let market_price = pos!(105.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = pmcc.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Higher volatility should increase both option values
        // Net effect should be positive as long gamma position
        assert!(pnl.unrealized.unwrap() > dec!(0.0));
    }
}
