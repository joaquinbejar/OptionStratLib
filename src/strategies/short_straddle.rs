/*
Straddle Strategy

A straddle involves simultaneously buying a call and a put option with the same strike price and expiration date.
This strategy is used when a significant move in the underlying asset's price is expected, but the direction is uncertain.

Key characteristics:
- Unlimited profit potential
- High cost due to purchasing both a call and a put
- Profitable only with a large move in either direction
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
use crate::{
    ExpirationDate, Options, Positive,
    chains::{StrategyLegs, chain::OptionChain, utils::OptionDataGroup},
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
    strategies::{Strategies,
                 BasicAble, StrategyConstructor,
                 delta_neutral::DeltaNeutrality,
                 probabilities::{core::ProbabilityAnalysis, utils::VolatilityAdjustment},
                 utils::{FindOptimalSide, OptimizationCriteria},
    },
    visualization::{Graph, GraphData},
};
use chrono::Utc;
use num_traits::{FromPrimitive, ToPrimitive};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::error::Error;
use tracing::{info, trace};

/// A Short Straddle is an options trading strategy that involves simultaneously selling
/// a put and a call option with the same strike price and expiration date. This neutral
/// strategy profits from low volatility and time decay, as the trader collects premium
/// from both options. Maximum profit is limited to the total premium collected, while
/// potential loss is unlimited. The strategy is most profitable when the underlying
/// asset stays close to the strike price through expiration.
///
/// Key characteristics:
/// - Sell 1 ATM Call
/// - Sell 1 ATM Put
/// - Same strike price
/// - Same expiration date
/// - Maximum profit: Total premium received
/// - Maximum loss: Unlimited
/// - Break-even points: Strike price +/- total premium received
/// - Ideal market forecast: Range-bound, low volatility
const SHORT_STRADDLE_DESCRIPTION: &str = "Short Straddle strategy involves simultaneously \
selling a put and a call option with identical strike prices and expiration dates. \
Profits from decreased volatility and time decay, with maximum gain limited to premium \
received and unlimited potential loss. Most effective in range-bound markets with low \
volatility expectations.";

/// # ShortStraddle
///
/// Represents a Short Straddle options trading strategy. This strategy involves selling both a call option and a put option
/// with the same strike price and expiration date.
///
/// A short straddle is a neutral strategy that profits from low volatility, where the underlying asset price remains close
/// to the strike price through expiration.
///
/// ## Structure
///
/// * `name`: A descriptive name for the specific strategy instance.
/// * `kind`: The type of strategy, which is `StrategyType::ShortStraddle`.
/// * `description`: A detailed description of this specific strategy instance.
/// * `break_even_points`: The price points at which the strategy breaks even (neither profit nor loss).
/// * `short_call`: The short call position component of the strategy.
/// * `short_put`: The short put position component of the strategy.
///
/// ## Risk Profile
///
/// A short straddle has unlimited risk if the stock price moves significantly in either direction,
/// but has limited profit potential equal to the total premium collected from both options.
///
/// ## Maximum Profit
///
/// The maximum profit is achieved when the underlying asset price at expiration equals exactly the strike price
/// of both options. In this case, both options expire worthless and the trader keeps the full premium collected.
///
/// ## Maximum Loss
///
/// The potential loss is theoretically unlimited on the upside (if the stock price rises significantly)
/// and is limited to the strike price minus the premium received on the downside (if the stock price falls to zero).
///
/// ## Break-Even Points
///
/// There are two break-even points in a short straddle:
/// 1. Upper break-even point: Strike price + total premium received
/// 2. Lower break-even point: Strike price - total premium received
///
/// ## Use Cases
///
/// This strategy is typically used when:
/// - The trader expects low volatility in the underlying asset
/// - The trader believes the price will remain close to the current level
/// - Implied volatility is high (making the options more expensive to sell)
///
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ShortStraddle {
    /// Name identifier for this specific strategy instance
    pub name: String,
    /// Identifies this as a ShortStraddle strategy type
    pub kind: StrategyType,
    /// Detailed description of this strategy instance
    pub description: String,
    /// Price points where the strategy neither makes nor loses money
    pub break_even_points: Vec<Positive>,
    /// The short call leg of the strategy
    short_call: Position,
    /// The short put leg of the strategy  
    short_put: Position,
}

impl ShortStraddle {
    /// # ShortStraddle Constructor
    ///
    /// Creates a new Short Straddle options strategy, which involves simultaneously selling a call and a put option
    /// with the same strike price and expiration date.
    ///
    /// ## Parameters
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset
    /// * `underlying_price` - The current market price of the underlying asset
    /// * `strike` - The strike price for both options. If set to zero, it defaults to the underlying price (at-the-money)
    /// * `expiration` - The expiration date for both options
    /// * `implied_volatility` - The implied volatility used for option pricing
    /// * `risk_free_rate` - The risk-free interest rate used for option pricing
    /// * `dividend_yield` - The dividend yield of the underlying asset
    /// * `quantity` - The number of option contracts
    /// * `premium_short_call` - The premium received for selling the call option
    /// * `premium_short_put` - The premium received for selling the put option
    /// * `open_fee_short_call` - The transaction fee for opening the short call position
    /// * `close_fee_short_call` - The transaction fee for closing the short call position
    /// * `open_fee_short_put` - The transaction fee for opening the short put position
    /// * `close_fee_short_put` - The transaction fee for closing the short put position
    ///
    /// ## Returns
    ///
    /// A fully configured `ShortStraddle` strategy instance with:
    /// - Initialized short call and short put positions
    /// - Calculated break-even points
    /// - Strategy metadata (name, description, etc.)
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut strike: Positive,
        expiration: ExpirationDate,
        implied_volatility: Positive,
        risk_free_rate: Decimal,
        dividend_yield: Positive,
        quantity: Positive,
        premium_short_call: Positive,
        premium_short_put: Positive,
        open_fee_short_call: Positive,
        close_fee_short_call: Positive,
        open_fee_short_put: Positive,
        close_fee_short_put: Positive,
    ) -> Self {
        if strike == Positive::ZERO {
            strike = underlying_price;
        }

        let mut strategy = ShortStraddle {
            name: "Short Straddle".to_string(),
            kind: StrategyType::ShortStraddle,
            description: SHORT_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call: Position::default(),
            short_put: Position::default(),
        };

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol.clone(),
            strike,
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
            .expect("Invalid short call");

        let short_put_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
            strike,
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
            open_fee_short_put,
            close_fee_short_put,
        );
        strategy
            .add_position(&short_put.clone())
            .expect("Invalid short put");

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for ShortStraddle {
    fn get_strategy(vec_options: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a short straddle
        if vec_options.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Straddle get_strategy".to_string(),
                    reason: "Must have exactly 2 options".to_string(),
                },
            ));
        }

        // Find call and put positions
        let mut call_position = None;
        let mut put_position = None;

        for option in vec_options {
            match option.option.option_style {
                OptionStyle::Call => call_position = Some(option),
                OptionStyle::Put => put_position = Some(option),
            }
        }

        // Validate we have both positions
        let (call_position, put_position) = match (call_position, put_position) {
            (Some(call), Some(put)) => (call, put),
            _ => {
                return Err(StrategyError::OperationError(
                    OperationErrorKind::InvalidParameters {
                        operation: "Short Straddle get_strategy".to_string(),
                        reason: "Must have one call and one put option".to_string(),
                    },
                ));
            }
        };

        // Validate strike prices match
        if call_position.option.strike_price != put_position.option.strike_price {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Straddle get_strategy".to_string(),
                    reason: "Options must have the same strike price".to_string(),
                },
            ));
        }

        // Validate both positions are short
        if call_position.option.side != Side::Short || put_position.option.side != Side::Short {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Straddle get_strategy".to_string(),
                    reason: "Both options must be short positions".to_string(),
                },
            ));
        }

        // Validate expiration dates match
        if call_position.option.expiration_date != put_position.option.expiration_date {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Short Straddle get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let short_call = Position::new(
            call_position.option.clone(),
            call_position.premium,
            Utc::now(),
            call_position.open_fee,
            call_position.close_fee,
        );

        let short_put = Position::new(
            put_position.option.clone(),
            put_position.premium,
            Utc::now(),
            put_position.open_fee,
            put_position.close_fee,
        );

        // Create strategy
        let mut strategy = ShortStraddle {
            name: "Short Straddle".to_string(),
            kind: StrategyType::ShortStraddle,
            description: SHORT_STRADDLE_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            short_call,
            short_put,
        };

        // Validate and update break-even points
        strategy.validate();
        strategy.update_break_even_points()?;

        Ok(strategy)
    }
}

impl BreakEvenable for ShortStraddle {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        let total_premium = self.get_net_premium_received()?;

        self.break_even_points.push(
            (self.short_put.option.strike_price
                - (total_premium / self.short_put.option.quantity).to_dec())
            .round_to(2),
        );

        self.break_even_points.push(
            (self.short_call.option.strike_price
                + (total_premium / self.short_call.option.quantity))
                .round_to(2),
        );

        self.break_even_points.sort();
        Ok(())
    }
}

impl Positionable for ShortStraddle {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.option_style {
            OptionStyle::Call => {
                self.short_call = position.clone();
                Ok(())
            }
            OptionStyle::Put => {
                self.short_put = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.short_call, &self.short_put])
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
            (Side::Long, _, _) => Err(PositionError::invalid_position_type(
                *side,
                "Position side is Long, it is not valid for ShortStraddle".to_string(),
            )),
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

        if position.option.side == Side::Long {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Position side is Long, it is not valid for ShortStraddle".to_string(),
            ));
        }

        if position.option.strike_price != self.short_call.option.strike_price
            && position.option.strike_price != self.short_put.option.strike_price
        {
            return Err(PositionError::invalid_position_type(
                position.option.side,
                "Strike not found in positions".to_string(),
            ));
        }

        if position.option.option_style == OptionStyle::Call {
            self.short_call = position.clone();
        }

        if position.option.option_style == OptionStyle::Put {
            self.short_put = position.clone();
        }

        Ok(())
    }
}

impl Strategable for ShortStraddle {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for ShortStraddle {
    fn get_title(&self) -> String {
        let strategy_title = format!("{:?} Strategy: ", self.kind);
        let leg_titles: Vec<String> = [self.short_call.get_title(), self.short_put.get_title()]
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
        self.short_call.premium = Positive::from(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_put.option.underlying_price = *price;
        self.short_put.premium =
            Positive::from(self.short_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
    fn set_implied_volatility(&mut self, volatility: &Positive) -> Result<(), StrategyError> {
        self.short_call.option.implied_volatility = *volatility;
        self.short_put.option.implied_volatility = *volatility;
        self.short_call.premium = Positive(
            self.short_call
                .option
                .calculate_price_black_scholes()?
                .abs(),
        );
        self.short_put.premium =
            Positive(self.short_put.option.calculate_price_black_scholes()?.abs());
        Ok(())
    }
}

impl Strategies for ShortStraddle {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let max_profit = self.get_net_premium_received()?.to_f64();
        if max_profit < ZERO {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Max profit is negative".to_string(),
                },
            ))
        } else {
            Ok(max_profit.into())
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        Ok(Positive::INFINITY)
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let strike_diff = self.break_even_points[1] - self.break_even_points[0];
        let cat = (strike_diff / 2.0_f64.sqrt()).to_f64();
        let result = (cat.powf(2.0)) / (2.0 * 10.0_f64.powf(cat.log10().ceil()));
        Ok(Decimal::from_f64(result).unwrap())
    }
    fn get_profit_ratio(&self) -> Result<Decimal, StrategyError> {
        let break_even_diff = self.break_even_points[1] - self.break_even_points[0];
        let result =
            self.get_max_profit().unwrap_or(Positive::ZERO).to_f64() / break_even_diff * 100.0;
        Ok(Decimal::from_f64(result).unwrap())
    }
}

impl Validable for ShortStraddle {
    fn validate(&self) -> bool {
        self.short_call.validate()
            && self.short_put.validate()
            && self.short_call.option.strike_price == self.short_put.option.strike_price
    }
}

impl Optimizable for ShortStraddle {
    type Strategy = ShortStraddle;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_single_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |both| {
                if side == FindOptimalSide::Center {
                    let atm_strike = match option_chain.atm_strike() {
                        Ok(atm_strike) => atm_strike,
                        Err(_) => return false,
                    };
                    both.is_valid_optimal_side(
                        underlying_price,
                        &FindOptimalSide::Range(*atm_strike, *atm_strike),
                    )
                } else {
                    both.is_valid_optimal_side(underlying_price, &side)
                }
            })
            .filter(|both| {
                both.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && both.put_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |both| {
                let legs = StrategyLegs::TwoLegs {
                    first: both,
                    second: both,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(OptionDataGroup::One)
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
            let both = match option_data_group {
                OptionDataGroup::One(first) => first,
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: both,
                second: both,
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
        let (call, put) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };

        if !call.validate() {
            panic!("Invalid Call option");
        }

        if !put.validate() {
            panic!("Invalid Put option");
        }

        let implied_volatility = call.implied_volatility.unwrap();
        assert!(implied_volatility <= Positive::ONE);
        ShortStraddle::new(
            chain.symbol.clone(),
            chain.underlying_price,
            call.strike_price,
            self.short_call.option.expiration_date,
            implied_volatility,
            self.short_call.option.risk_free_rate,
            self.short_call.option.dividend_yield,
            self.short_call.option.quantity,
            call.call_bid.unwrap(),
            put.put_bid.unwrap(),
            self.short_call.open_fee,
            self.short_call.close_fee,
            self.short_put.open_fee,
            self.short_put.close_fee,
        )
    }
}

impl Profit for ShortStraddle {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, Box<dyn Error>> {
        let price = Some(price);
        trace!(
            "Price: {:?} Strike: {} Call: {:.2} Strike: {} Put: {:.2} Profit: {:.2}",
            price,
            self.short_call.option.strike_price,
            self.short_call.pnl_at_expiration(&price)?,
            self.short_put.option.strike_price,
            self.short_put.pnl_at_expiration(&price)?,
            self.short_call.pnl_at_expiration(&price)?
                + self.short_put.pnl_at_expiration(&price)?
        );
        Ok(
            self.short_call.pnl_at_expiration(&price)?
                + self.short_put.pnl_at_expiration(&price)?,
        )
    }
}

impl Graph for ShortStraddle {
    fn graph_data(&self) -> GraphData {
        todo!()
    }
}

impl ProbabilityAnalysis for ShortStraddle {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_points = &self.get_break_even_points()?;
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
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
        let break_even_points = &self.get_break_even_points()?;
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            option.implied_volatility,
            self.short_put.option.implied_volatility,
        ]);

        let mut lower_loss_range =
            ProfitLossRange::new(None, Some(break_even_points[0]), Positive::ZERO)?;

        lower_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        let mut upper_loss_range =
            ProfitLossRange::new(Some(break_even_points[1]), None, Positive::ZERO)?;

        upper_loss_range.calculate_probability(
            self.get_underlying_price(),
            Some(VolatilityAdjustment {
                base_volatility: mean_volatility,
                std_dev_adjustment: std_dev,
            }),
            None,
            expiration_date,
            Some(risk_free_rate),
        )?;

        Ok(vec![lower_loss_range, upper_loss_range])
    }
}

impl Greeks for ShortStraddle {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.short_call.option, &self.short_put.option])
    }
}

impl DeltaNeutrality for ShortStraddle {}

impl PnLCalculator for ShortStraddle {
    fn calculate_pnl(
        &self,
        market_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        Ok(self
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
            .short_call
            .calculate_pnl_at_expiration(underlying_price)?
            + self
                .short_put
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

#[cfg(test)]
mod tests_short_straddle {
    use super::*;
    use crate::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
    use crate::{pos, spos};
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    fn setup() -> ShortStraddle {
        ShortStraddle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(150.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        )
    }

    #[test]
    fn test_atm_strike_initialization() {
        let underlying_price = pos!(150.0);
        let strategy = ShortStraddle::new(
            "AAPL".to_string(),
            underlying_price,
            Positive::ZERO,
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        );

        assert_eq!(
            strategy.short_call.option.strike_price, underlying_price,
            "Strike should default to underlying price when Positive::ZERO is provided"
        );
    }

    #[test]
    fn test_new() {
        let strategy = setup();
        assert_eq!(strategy.name, "Short Straddle");
        assert_eq!(strategy.kind, StrategyType::ShortStraddle);
        assert_eq!(
            strategy.description,
            "Short Straddle strategy involves simultaneously selling a put and a call option with \
            identical strike prices and expiration dates. Profits from decreased volatility and \
            time decay, with maximum gain limited to premium received and unlimited potential \
            loss. Most effective in range-bound markets with low volatility expectations."
        );
    }

    #[test]
    fn test_strikes_are_equal() {
        let strategy = setup();
        assert_eq!(
            strategy.short_call.option.strike_price, strategy.short_put.option.strike_price,
            "Call and Put strikes should be equal in a Straddle"
        );
    }

    #[test]
    fn test_validate() {
        let strategy = setup();
        assert!(
            strategy.validate(),
            "Strategy should be valid with equal strikes"
        );

        let valid_strategy = ShortStraddle::new(
            "AAPL".to_string(),
            pos!(150.0),
            pos!(145.0), // Diferente strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            dec!(0.01),
            pos!(0.02),
            pos!(100.0),
            Positive::TWO,
            pos!(1.5),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
            pos!(0.1),
        );
        assert!(valid_strategy.validate());
        assert_eq!(
            valid_strategy.short_call.option.strike_price,
            valid_strategy.short_put.option.strike_price
        );
    }

    #[test]
    fn test_get_break_even_points() {
        let strategy = setup();
        assert_eq!(strategy.get_break_even_points().unwrap()[0], 146.9);
    }

    #[test]
    fn test_calculate_profit_at() {
        let strategy = setup();
        let price = 150.0;
        assert_eq!(
            strategy
                .calculate_profit_at(&pos!(price))
                .unwrap()
                .to_f64()
                .unwrap(),
            310.0
        );
    }

    #[test]
    fn test_max_profit() {
        let strategy = setup();
        assert_eq!(
            strategy.get_max_profit().unwrap_or(Positive::ZERO),
            strategy.get_net_premium_received().unwrap().to_f64()
        );
    }

    #[test]
    fn test_max_loss() {
        let strategy = setup();
        assert_eq!(
            strategy.get_max_loss().unwrap_or(Positive::ZERO),
            Positive::INFINITY
        );
    }

    #[test]
    fn test_total_cost() {
        let strategy = setup();
        assert_eq!(strategy.get_total_cost().unwrap(), 40.0);
    }

    #[test]
    fn test_net_premium_received() {
        let strategy = setup();
        assert_eq!(
            strategy.get_net_premium_received().unwrap().to_f64(),
            strategy.short_call.net_premium_received().unwrap()
                + strategy.short_put.net_premium_received().unwrap()
        );
    }

    #[test]
    fn test_fees() {
        let strategy = setup();
        let expected_fees = 40.0;
        assert_eq!(strategy.get_fees().unwrap().to_f64(), expected_fees);
    }

    #[test]
    fn test_area() {
        let strategy = setup();
        assert_eq!(strategy.get_profit_area().unwrap().to_f64().unwrap(), 0.961);
    }
    

    #[test]
    fn test_add_leg() {
        let mut strategy = setup();
        let original_call = strategy.short_call.clone();
        let original_put = strategy.short_put.clone();

        // Test adding a new call leg
        strategy
            .add_position(&original_call.clone())
            .expect("Invalid call");
        assert_eq!(strategy.short_call, original_call);

        // Test adding a new put leg
        strategy
            .add_position(&original_put.clone())
            .expect("Invalid put");
        assert_eq!(strategy.short_put, original_put);
    }

    #[test]
    fn test_profit_ratio() {
        let strategy = setup();
        let break_even_diff = strategy.break_even_points[1] - strategy.break_even_points[0];
        let expected_ratio =
            strategy.get_max_profit().unwrap_or(Positive::ZERO) / break_even_diff * 100.0;
        assert_eq!(
            strategy.get_profit_ratio().unwrap().to_f64().unwrap(),
            expected_ratio.to_f64()
        );
    }

    #[test]
    fn test_best_ratio() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        info!("{}", option_chain);
        strategy.get_best_ratio(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_area() {
        let mut strategy = setup();
        let option_chain = create_test_option_chain();

        strategy.get_best_area(&option_chain, FindOptimalSide::All);
        assert!(strategy.validate());
    }

    #[test]
    fn test_best_range_to_show() {
        let strategy = setup();
        let step = pos!(1.0);

        let range = strategy.get_best_range_to_show(step).unwrap();
        assert!(!range.is_empty());
        assert!(range[0] <= strategy.break_even_points[0]);
        assert!(*range.last().unwrap() >= strategy.break_even_points[1]);
    }

    #[test]
    fn test_is_valid_short_option() {
        let strategy = setup();
        let option_chain = create_test_option_chain();
        let option_data = option_chain.options.first().unwrap();
        let min_strike = option_chain.options.first().unwrap().strike_price;
        let max_strike = option_chain.options.last().unwrap().strike_price;

        // Test FindOptimalSide::Lower
        assert!(strategy.is_valid_optimal_option(option_data, &FindOptimalSide::Lower));

        // Test FindOptimalSide::Upper
        assert!(!strategy.is_valid_optimal_option(option_data, &FindOptimalSide::Upper));

        // Test FindOptimalSide::All
        assert!(strategy.is_valid_optimal_option(option_data, &FindOptimalSide::All));

        // Test FindOptimalSide::Range
        assert!(
            strategy.is_valid_optimal_option(
                option_data,
                &FindOptimalSide::Range(min_strike, max_strike)
            )
        );
    }

    #[test]
    fn test_create_strategy() {
        let strategy = setup();
        let chain = create_test_option_chain();
        let call_option = chain
            .options
            .iter()
            .rev()
            .find(|option_data| option_data.valid_call())
            .unwrap();
        let put_option = chain
            .options
            .iter()
            .find(|option_data| option_data.valid_put())
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: call_option,
            second: put_option,
        };
        let new_strategy = strategy.create_strategy(&chain, &legs);
        assert!(new_strategy.validate());
    }
    

    fn create_test_option_chain() -> OptionChain {
        let option_data_price_params = OptionDataPriceParams::new(
            pos!(150.0),
            ExpirationDate::Days(pos!(30.0)),
            spos!(0.2),
            dec!(0.01),
            pos!(0.02),
            None,
        );
        let option_chain_build_params = OptionChainBuildParams::new(
            "AAPL".to_string(),
            spos!(1.0),
            10,
            spos!(10.0),
            dec!(-0.2),
            dec!(0.1),
            pos!(0.01),
            2,
            option_data_price_params,
        );
        OptionChain::build_chain(&option_chain_build_params)
    }
}

#[cfg(test)]
mod tests_short_straddle_probability {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    /// Helper function that creates a basic short Straddle for testing purposes
    /// Returns a ShortStraddle instance with predefined test values
    fn create_test_short_straddle() -> ShortStraddle {
        ShortStraddle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]
    fn test_probability_of_profit_basic() {
        let straddle = create_test_short_straddle();
        let result = straddle.probability_of_profit(None, None);

        assert!(result.is_ok(), "Probability calculation should succeed");
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_volatility_adjustment() {
        let straddle = create_test_short_straddle();
        let vol_adj = VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.05),
        };

        let result = straddle.probability_of_profit(Some(vol_adj), None);

        assert!(
            result.is_ok(),
            "Probability calculation with volatility adjustment should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_trend() {
        let straddle = create_test_short_straddle();
        let trend = PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        };

        let result = straddle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_probability_of_profit_with_downward_trend() {
        let straddle = create_test_short_straddle();
        let trend = PriceTrend {
            drift_rate: -0.1,
            confidence: 0.90,
        };

        let result = straddle.probability_of_profit(None, Some(trend));

        assert!(
            result.is_ok(),
            "Probability calculation with downward trend should succeed"
        );
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO, "Probability should be positive");
        assert!(prob <= pos!(1.0), "Probability should not exceed 1.0");
    }

    #[test]
    fn test_get_reference_price() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_underlying_price();

        assert_eq!(
            result,
            &pos!(100.0),
            "Reference price should match underlying price"
        );
    }

    #[test]
    fn test_get_expiration() {
        let straddle = create_test_short_straddle();
        let expiration_date = *straddle.get_expiration().values().next().unwrap();
        assert_eq!(expiration_date, &ExpirationDate::Days(pos!(30.0)));
    }

    #[test]
    fn test_get_profit_ranges() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_profit_ranges();

        assert!(result.is_ok(), "Profit ranges calculation should succeed");
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1, "Should have exactly one profit range");

        let range = &ranges[0];
        assert!(range.lower_bound.is_some(), "Lower bound should be defined");
        assert!(range.upper_bound.is_some(), "Upper bound should be defined");
        assert!(
            range.probability > Positive::ZERO,
            "Probability should be positive"
        );
    }
}

#[cfg(test)]
mod tests_short_straddle_probability_bis {
    use super::*;
    use crate::model::ExpirationDate;
    use crate::pos;
    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    fn create_test_short_straddle() -> ShortStraddle {
        ShortStraddle::new(
            "TEST".to_string(),
            pos!(100.0),                      // underlying_price
            pos!(110.0),                      // strike
            ExpirationDate::Days(pos!(30.0)), // expiration
            pos!(0.2),                        // implied_volatility
            dec!(0.05),                       // risk_free_rate
            Positive::ZERO,                   // dividend_yield
            pos!(1.0),                        // quantity
            Positive::TWO,                    // premium_short_call
            Positive::TWO,                    // premium_short_put
            Positive::ZERO,                   // open_fee_short_call
            Positive::ZERO,                   // close_fee_short_call
            Positive::ZERO,                   // open_fee_short_put
            Positive::ZERO,                   // close_fee_short_put
        )
    }

    #[test]
    fn test_get_risk_free_rate() {
        let straddle = create_test_short_straddle();
        assert_eq!(
            **straddle.get_risk_free_rate().values().next().unwrap(),
            dec!(0.05)
        );
    }

    #[test]
    fn test_get_profit_ranges() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_profit_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1); // Short Straddle has one profit range

        let range = &ranges[0];
        assert!(range.lower_bound.is_some());
        assert!(range.upper_bound.is_some());
    }

    #[test]
    fn test_get_loss_ranges() {
        let straddle = create_test_short_straddle();
        let result = straddle.get_loss_ranges();

        assert!(result.is_ok());
        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 2); // Short Straddle has two loss ranges

        // Verify ranges have correct bounds
        assert!(ranges[0].lower_bound.is_none()); // First loss range extends to negative infinity
        assert!(ranges[1].upper_bound.is_none()); // Second loss range extends to positive infinity
    }

    #[test]
    fn test_probability_of_profit() {
        let straddle = create_test_short_straddle();
        let result = straddle.probability_of_profit(None, None);

        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let straddle = create_test_short_straddle();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos!(0.25),
            std_dev_adjustment: pos!(0.1),
        });

        let result = straddle.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_probability_with_trend() {
        let straddle = create_test_short_straddle();
        let trend = Some(PriceTrend {
            drift_rate: 0.1,
            confidence: 0.95,
        });

        let result = straddle.probability_of_profit(None, trend);
        assert!(result.is_ok());
        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= pos!(1.0));
    }

    #[test]
    fn test_analyze_probabilities() {
        let straddle = create_test_short_straddle();
        let result = straddle.analyze_probabilities(None, None);

        assert!(result.is_ok());
        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let straddle = create_test_short_straddle();
        let result = straddle.calculate_extreme_probabilities(None, None);

        assert!(result.is_ok());
        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= pos!(1.0));
    }
}

#[cfg(test)]
mod tests_short_straddle_delta {
    use super::*;
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn get_strategy(strike: Positive) -> ShortStraddle {
        let underlying_price = pos!(7138.5);
        ShortStraddle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            strike,           // call_strike 7450
            ExpirationDate::Days(pos!(45.0)),
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
    fn create_test_short_straddle_reducing_adjustments() {
        let strategy = get_strategy(pos!(7460.0));
        let size = dec!(0.1759865);
        let delta = pos!(0.42714475673336616);
        let k = pos!(7460.0);
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
        let strategy = get_strategy(pos!(7050.0));
        let size = dec!(-0.164378449);
        let delta = pos!(0.3934279797271222);
        let k = pos!(7050.0);
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
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
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
        let strategy = get_strategy(pos!(7245.0));

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
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{ExpirationDate, Side, assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use std::str::FromStr;

    fn get_strategy(strike: Positive) -> ShortStraddle {
        let underlying_price = pos!(7138.5);
        ShortStraddle::new(
            "CL".to_string(),
            underlying_price, // underlying_price
            strike,           // call_strike 7450
            ExpirationDate::Days(pos!(45.0)),
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
    fn create_test_short_straddle_reducing_adjustments() {
        let strategy = get_strategy(pos!(7460.0));
        let size = dec!(0.3519);
        let delta =
            Positive::new_decimal(Decimal::from_str("0.8542895134667324").unwrap()).unwrap();

        let k = pos!(7460.0);
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
        let strategy = get_strategy(pos!(7050.0));
        let size = dec!(-0.3287);
        let delta =
            Positive::new_decimal(Decimal::from_str("0.7868559594542444").unwrap()).unwrap();
        let k = pos!(7050.0);
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
                assert_eq!(*option_style, OptionStyle::Put);
                assert_eq!(*side, Side::Short);
            }
            _ => panic!("Invalid suggestion"),
        }

        let mut option = strategy.short_put.option.clone();
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
        let strategy = get_strategy(pos!(7245.0));

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
mod tests_short_strategy_constructor {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = ShortStraddle::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.short_call.option.strike_price, pos!(100.0));
        assert_eq!(strategy.short_put.option.strike_price, pos!(100.0));
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        )];

        let result = ShortStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Straddle get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_missing_put_option() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = ShortStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Straddle get_strategy" && reason == "Must have one call and one put option"
        ));
    }

    #[test]
    fn test_get_strategy_different_strikes() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
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

        let result = ShortStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Straddle get_strategy" && reason == "Options must have the same strike price"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
            create_sample_position(
                OptionStyle::Put,
                Side::Short,
                pos!(100.0),
                pos!(1.0),
                pos!(100.0),
                pos!(0.2),
            ),
        ];

        let result = ShortStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Straddle get_strategy" && reason == "Both options must be short positions"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0),
            pos!(1.0),
            pos!(100.0),
            pos!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos!(60.0));

        let options = vec![option1, option2];
        let result = ShortStraddle::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Short Straddle get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_short_straddle_pnl {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::{assert_decimal_eq, assert_pos_relative_eq, pos};
    use rust_decimal_macros::dec;

    fn create_test_short_straddle() -> Result<ShortStraddle, StrategyError> {
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(100.0), // Underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Strike price (ATM)
            pos!(0.2),   // Implied volatility
        );

        let short_put = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(100.0), // Same underlying price
            pos!(1.0),   // Quantity
            pos!(100.0), // Same strike price
            pos!(0.2),   // Implied volatility
        );

        ShortStraddle::get_strategy(&vec![short_call, short_put])
    }

    #[test]
    fn test_calculate_pnl_at_strike() {
        let straddle = create_test_short_straddle().unwrap();
        let market_price = pos!(100.0); // At strike price
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options ATM, should be near max profit
        assert_pos_relative_eq!(pnl.initial_income, pos!(10.0), pos!(1e-6)); // Premium from both options
        assert_pos_relative_eq!(pnl.initial_costs, pos!(2.0), pos!(1e-6)); // Total fees
    }

    #[test]
    fn test_calculate_pnl_below_strike() {
        let straddle = create_test_short_straddle().unwrap();
        let market_price = pos!(90.0); // Below strike
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Put ITM, call OTM
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be a loss
    }

    #[test]
    fn test_calculate_pnl_above_strike() {
        let straddle = create_test_short_straddle().unwrap();
        let market_price = pos!(110.0); // Above strike
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.2);

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Call ITM, put OTM
        assert!(pnl.unrealized.unwrap() < dec!(0.0)); // Should be a loss
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let straddle = create_test_short_straddle().unwrap();
        let market_price = pos!(100.0);
        let expiration_date = ExpirationDate::Days(pos!(20.0));
        let implied_volatility = pos!(0.4); // Higher volatility

        let result = straddle.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Higher volatility should result in larger losses
        assert!(pnl.unrealized.unwrap() < dec!(-2.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let straddle = create_test_short_straddle().unwrap();
        let underlying_price = pos!(100.0); // At strike price

        let result = straddle.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At strike price at expiration, both options expire worthless
        // Max profit is the net premium received minus fees
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(8.0), dec!(1e-6)); // Premium received - costs
        assert_eq!(pnl.initial_income, pos!(10.0));
        assert_eq!(pnl.initial_costs, pos!(2.0));
    }
}

#[cfg(test)]
mod tests_straddle_position_management {
    use super::*;
    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};
    use crate::pos;
    use rust_decimal_macros::dec;
    use tracing::error;

    fn create_test_short_straddle() -> ShortStraddle {
        ShortStraddle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_short_straddle_get_position() {
        let mut straddle = create_test_short_straddle();

        // Test getting short call position
        let call_position = straddle.get_position(&OptionStyle::Call, &Side::Short, &pos!(110.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting short put position
        let put_position = straddle.get_position(&OptionStyle::Put, &Side::Short, &pos!(110.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos!(110.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Put);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            straddle.get_position(&OptionStyle::Call, &Side::Short, &pos!(100.0));
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
    fn test_short_straddle_modify_position() {
        let mut straddle = create_test_short_straddle();

        // Modify short call position
        let mut modified_call = straddle.short_call.clone();
        modified_call.option.quantity = pos!(2.0);
        let result = straddle.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(straddle.short_call.option.quantity, pos!(2.0));

        // Modify short put position
        let mut modified_put = straddle.short_put.clone();
        modified_put.option.quantity = pos!(2.0);
        let result = straddle.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(straddle.short_put.option.quantity, pos!(2.0));

        // Test modifying with invalid position
        let mut invalid_position = straddle.short_call.clone();
        invalid_position.option.strike_price = pos!(95.0);
        let result = straddle.modify_position(&invalid_position);
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

    fn create_test_short_straddle() -> ShortStraddle {
        ShortStraddle::new(
            "TEST".to_string(),
            pos!(100.0), // underlying_price
            pos!(110.0), // strike
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),      // implied_volatility
            dec!(0.05),     // risk_free_rate
            Positive::ZERO, // dividend_yield
            pos!(1.0),      // quantity
            pos!(2.0),      // premium_short_call
            pos!(2.0),      // premium_short_put
            pos!(0.1),      // open_fee_short_call
            pos!(0.1),      // close_fee_short_call
            pos!(0.1),      // open_fee_short_put
            pos!(0.1),      // close_fee_short_put
        )
    }

    #[test]
    fn test_adjust_existing_call_position_short() {
        let mut strategy = create_test_short_straddle();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(110.0),
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
    fn test_adjust_existing_put_position_short() {
        let mut strategy = create_test_short_straddle();
        let initial_quantity = strategy.short_put.option.quantity;
        let adjustment = pos!(1.0);

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos!(110.0),
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
    fn test_adjust_nonexistent_position_short() {
        let mut strategy = create_test_short_straddle();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos!(100.0),
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
                assert_eq!(
                    reason,
                    "Position side is Long, it is not valid for ShortStraddle"
                );
            }
            _ => panic!("Expected PositionError::ValidationError"),
        }
    }

    #[test]
    fn test_adjust_with_invalid_strike_short() {
        let mut strategy = create_test_short_straddle();

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
    fn test_zero_quantity_adjustment_short() {
        let mut strategy = create_test_short_straddle();
        let initial_quantity = strategy.short_call.option.quantity;

        let result = strategy.adjust_option_position(
            Decimal::ZERO,
            &pos!(110.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}
