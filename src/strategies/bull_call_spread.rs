/*
Bull Call Spread Strategy

A bull call spread involves buying a call option with a lower strike price and selling a call option with a higher strike price,
both with the same expiration date. This strategy is used when a moderate rise in the underlying asset's price is expected.

Key characteristics:
- Limited profit potential (difference between strikes minus net debit)
- Limited risk (net debit paid)
- Bullish strategy that profits from price increase
- Both options have same expiration date
- Lower cost than buying calls outright
- Lower risk than naked call buying
- Maximum profit achieved when price rises above higher strike
- Also known as a vertical call debit spread
*/
use super::base::{
    BreakEvenable, Optimizable, Positionable, Strategable, StrategyBasics, StrategyType, Validable,
};
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
use positive::{Positive, pos_or_panic};
use pretty_simple_display::{DebugPretty, DisplaySimple};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, error, info};
use utoipa::ToSchema;

/// The default description for the Bull Call Spread strategy.
pub const BULL_CALL_SPREAD_DESCRIPTION: &str = "A bull call spread is created by buying a call option with a lower strike price \
    and simultaneously selling a call option with a higher strike price, both with the same \
    expiration date. This strategy is used when you expect a moderate increase in the underlying \
    asset's price. The maximum profit is limited to the difference between strike prices minus \
    the net debit paid, while the maximum loss is limited to the net debit paid.";

/// Represents a Bull Call Spread options trading strategy.
///
/// A Bull Call Spread is a vertical spread strategy that involves buying a call option
/// with a lower strike price and selling another call option with a higher strike price,
/// both with the same expiration date. This strategy is typically used when an investor
/// expects a moderate rise in the price of the underlying asset.
///
/// # Advantages
/// - Limited risk (maximum loss is the net debit paid)
/// - Lower cost than buying a call option outright
/// - Potential for profit if the underlying price rises
///
/// # Disadvantages
/// - Limited profit potential (capped by the difference between strike prices minus the net debit)
/// - Requires more capital than a single option position
/// - Loses value as expiration approaches if the underlying price doesn't rise
#[derive(Clone, DebugPretty, DisplaySimple, Serialize, Deserialize, ToSchema)]
pub struct BullCallSpread {
    /// The name of the strategy, typically including underlying asset information.
    pub name: String,

    /// The type of strategy, which is StrategyType::BullCallSpread for this struct.
    pub kind: StrategyType,

    /// A textual description of this specific bull call spread instance.
    pub description: String,

    /// The price points at which the strategy breaks even (typically one point).
    pub break_even_points: Vec<Positive>,

    /// The long call position (lower strike price).
    pub long_call: Position,

    /// The short call position (higher strike price).
    pub short_call: Position,
}

impl BullCallSpread {
    /// Creates a new Bull Call Spread strategy.
    ///
    /// A Bull Call Spread is created by buying a call option with a lower strike price
    /// and simultaneously selling a call option with a higher strike price, both with the same
    /// expiration date. This strategy benefits from moderate increases in the underlying asset's price.
    ///
    /// # Arguments
    ///
    /// * `underlying_symbol` - The ticker symbol of the underlying asset.
    /// * `underlying_price` - The current market price of the underlying asset.
    /// * `long_strike` - The strike price for the long call option. If set to zero, defaults to the underlying price.
    /// * `short_strike` - The strike price for the short call option. If set to zero, defaults to the underlying price.
    /// * `expiration` - The expiration date for both options.
    /// * `implied_volatility` - The implied volatility value used for option pricing.
    /// * `risk_free_rate` - The risk-free interest rate used in option pricing calculations.
    /// * `dividend_yield` - The dividend yield of the underlying asset.
    /// * `quantity` - The number of contracts to create for both positions.
    /// * `premium_long_call` - The premium paid for the long call position.
    /// * `premium_short_call` - The premium received for the short call position.
    /// * `open_fee_long_call` - The fee paid when opening the long call position.
    /// * `close_fee_long_call` - The fee that will be paid when closing the long call position.
    /// * `open_fee_short_call` - The fee paid when opening the short call position.
    /// * `close_fee_short_call` - The fee that will be paid when closing the short call position.
    ///
    /// # Returns
    ///
    /// Returns a fully configured `BullCallSpread` strategy instance with positions and break-even points calculated.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    /// - The long call position cannot be added to the strategy
    /// - The short call position cannot be added to the strategy
    /// - Break-even points cannot be calculated
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        underlying_symbol: String,
        underlying_price: Positive,
        mut long_strike: Positive,
        mut short_strike: Positive,
        expiration: ExpirationDate,
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
        if long_strike == Positive::ZERO {
            long_strike = underlying_price;
        }
        if short_strike == Positive::ZERO {
            short_strike = underlying_price;
        }

        let mut strategy = BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: BULL_CALL_SPREAD_DESCRIPTION.to_string(),
            break_even_points: Vec::new(),
            long_call: Position::default(),
            short_call: Position::default(),
        };

        let long_call_option = Options::new(
            OptionType::European,
            Side::Long,
            underlying_symbol.clone(),
            long_strike,
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
            open_fee_long_call,
            close_fee_long_call,
            None,
            None,
        );
        strategy
            .add_position(&long_call.clone())
            .expect("Failed to add long call");

        let short_call_option = Options::new(
            OptionType::European,
            Side::Short,
            underlying_symbol,
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
            open_fee_short_call,
            close_fee_short_call,
            None,
            None,
        );
        strategy
            .add_position(&short_call.clone())
            .expect("Failed to add short call");

        strategy.validate();

        strategy
            .update_break_even_points()
            .expect("Unable to update break even points");
        strategy
    }
}

impl StrategyConstructor for BullCallSpread {
    fn get_strategy(vec_positions: &[Position]) -> Result<Self, StrategyError> {
        // Need exactly 2 options for a bull call spread
        if vec_positions.len() != 2 {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Spread get_strategy".to_string(),
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

        let lower_strike_option = &sorted_positions[0];
        let higher_strike_option = &sorted_positions[1];

        // Validate options are calls
        if lower_strike_option.option.option_style != OptionStyle::Call
            || higher_strike_option.option.option_style != OptionStyle::Call
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Spread get_strategy".to_string(),
                    reason: "Options must be calls".to_string(),
                },
            ));
        }

        // Validate option sides
        if lower_strike_option.option.side != Side::Long
            || higher_strike_option.option.side != Side::Short
        {
            return Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters {
                operation: "Bull Call Spread get_strategy".to_string(),
                reason: "Bull Call Spread requires a long lower strike call and a short higher strike call".to_string(),
            }));
        }

        // Validate expiration dates match
        if lower_strike_option.option.expiration_date != higher_strike_option.option.expiration_date
        {
            return Err(StrategyError::OperationError(
                OperationErrorKind::InvalidParameters {
                    operation: "Bull Call Spread get_strategy".to_string(),
                    reason: "Options must have the same expiration date".to_string(),
                },
            ));
        }

        // Create positions
        let long_call = Position::new(
            lower_strike_option.option.clone(),
            lower_strike_option.premium,
            Utc::now(),
            lower_strike_option.open_fee,
            lower_strike_option.close_fee,
            lower_strike_option.epic.clone(),
            lower_strike_option.extra_fields.clone(),
        );

        let short_call = Position::new(
            higher_strike_option.option.clone(),
            higher_strike_option.premium,
            Utc::now(),
            higher_strike_option.open_fee,
            higher_strike_option.close_fee,
            higher_strike_option.epic.clone(),
            higher_strike_option.extra_fields.clone(),
        );

        // Create strategy
        let mut strategy = BullCallSpread {
            name: "Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: BULL_CALL_SPREAD_DESCRIPTION.to_string(),
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

impl BreakEvenable for BullCallSpread {
    fn get_break_even_points(&self) -> Result<&Vec<Positive>, StrategyError> {
        Ok(&self.break_even_points)
    }

    fn update_break_even_points(&mut self) -> Result<(), StrategyError> {
        self.break_even_points = Vec::new();

        self.break_even_points.push(
            (self.long_call.option.strike_price
                + self.get_net_cost()? / self.long_call.option.quantity)
                .round_to(2),
        );

        Ok(())
    }
}

impl Positionable for BullCallSpread {
    fn add_position(&mut self, position: &Position) -> Result<(), PositionError> {
        match position.option.side {
            Side::Short => {
                self.short_call = position.clone();
                Ok(())
            }
            Side::Long => {
                self.long_call = position.clone();
                Ok(())
            }
        }
    }

    fn get_positions(&self) -> Result<Vec<&Position>, PositionError> {
        Ok(vec![&self.long_call, &self.short_call])
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
                "Put is not valid for BearCallSpread".to_string(),
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

impl Strategable for BullCallSpread {
    fn info(&self) -> Result<StrategyBasics, StrategyError> {
        Ok(StrategyBasics {
            name: self.name.clone(),
            kind: self.kind.clone(),
            description: self.description.clone(),
        })
    }
}

impl BasicAble for BullCallSpread {
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

impl Strategies for BullCallSpread {
    fn get_max_profit(&self) -> Result<Positive, StrategyError> {
        let profit = self.calculate_profit_at(&self.short_call.option.strike_price)?;
        if profit >= Decimal::ZERO {
            Ok(profit.into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxProfitError {
                    reason: "Net premium received is negative".to_string(),
                },
            ))
        }
    }
    fn get_max_loss(&self) -> Result<Positive, StrategyError> {
        let loss = self.calculate_profit_at(&self.long_call.option.strike_price)?;
        if loss <= Decimal::ZERO {
            Ok(loss.abs().into())
        } else {
            Err(StrategyError::ProfitLossError(
                ProfitLossErrorKind::MaxLossError {
                    reason: "Max loss is negative".to_string(),
                },
            ))
        }
    }
    fn get_profit_area(&self) -> Result<Decimal, StrategyError> {
        let high = self.get_max_profit().unwrap_or(Positive::ZERO);
        let base = self.short_call.option.strike_price - self.break_even_points[0];
        Ok((high * base / 200.0).into())
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

impl Validable for BullCallSpread {
    fn validate(&self) -> bool {
        if !self.long_call.validate() {
            debug!("Long call is invalid");
            return false;
        }
        if !self.short_call.validate() {
            debug!("Short call is invalid");
            return false;
        }
        if self.long_call.option.strike_price >= self.short_call.option.strike_price {
            error!(
                "Long call strike price {} must be lower than short call strike price {}",
                self.long_call.option.strike_price, self.short_call.option.strike_price
            );
            return false;
        }

        true
    }
}

impl Optimizable for BullCallSpread {
    type Strategy = BullCallSpread;

    fn filter_combinations<'a>(
        &'a self,
        option_chain: &'a OptionChain,
        side: FindOptimalSide,
    ) -> impl Iterator<Item = OptionDataGroup<'a>> {
        let underlying_price = self.get_underlying_price();
        let strategy = self.clone();
        option_chain
            .get_double_iter()
            // Filter out invalid combinations based on FindOptimalSide
            .filter(move |&(long, short)| {
                if side == FindOptimalSide::Center {
                    long.is_valid_optimal_side(underlying_price, &FindOptimalSide::Lower)
                        && short.is_valid_optimal_side(underlying_price, &FindOptimalSide::Upper)
                } else {
                    long.is_valid_optimal_side(underlying_price, &side)
                        && short.is_valid_optimal_side(underlying_price, &side)
                }
            })
            // Filter out options with invalid bid/ask prices
            .filter(|(long, short)| {
                long.call_ask.unwrap_or(Positive::ZERO) > Positive::ZERO
                    && short.call_bid.unwrap_or(Positive::ZERO) > Positive::ZERO
            })
            // Filter out options that don't meet strategy constraints
            .filter(move |(long, short)| {
                let legs = StrategyLegs::TwoLegs {
                    first: long,
                    second: short,
                };
                let strategy = strategy.create_strategy(option_chain, &legs);
                strategy.validate()
                    && strategy.get_max_profit().is_ok()
                    && strategy.get_max_loss().is_ok()
            })
            // Map to OptionDataGroup
            .map(move |(long, short)| OptionDataGroup::Two(long, short))
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
            let (long, short) = match option_data_group {
                OptionDataGroup::Two(first, second) => (first, second),
                _ => panic!("Invalid OptionDataGroup"),
            };

            let legs = StrategyLegs::TwoLegs {
                first: long,
                second: short,
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
        let (long, short) = match legs {
            StrategyLegs::TwoLegs { first, second } => (first, second),
            _ => panic!("Invalid number of legs for this strategy"),
        };
        let implied_volatility = long.implied_volatility;
        assert!(implied_volatility <= Positive::ONE);
        BullCallSpread::new(
            chain.symbol.clone(),
            chain.underlying_price,
            long.strike_price,
            short.strike_price,
            self.long_call.option.expiration_date,
            implied_volatility,
            self.long_call.option.risk_free_rate,
            self.long_call.option.dividend_yield,
            self.long_call.option.quantity,
            long.call_ask.unwrap(),
            short.call_bid.unwrap(),
            self.long_call.open_fee,
            self.long_call.close_fee,
            self.short_call.open_fee,
            self.short_call.close_fee,
        )
    }
}

impl Profit for BullCallSpread {
    fn calculate_profit_at(&self, price: &Positive) -> Result<Decimal, PricingError> {
        let price = Some(price);
        Ok(
            self.long_call.pnl_at_expiration(&price)?
                + self.short_call.pnl_at_expiration(&price)?,
        )
    }
}

impl ProbabilityAnalysis for BullCallSpread {
    fn get_profit_ranges(&self) -> Result<Vec<ProfitLossRange>, ProbabilityError> {
        let break_even_point = self.get_break_even_points()?[0];
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call.option.implied_volatility,
        ]);

        let mut profit_range = ProfitLossRange::new(
            Some(break_even_point),
            Some(self.short_call.option.strike_price),
            pos_or_panic!(self.get_max_profit()?.to_f64()),
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
        let break_even_point = self.get_break_even_points()?[0];
        let option = &self.short_call.option;
        let expiration_date = &option.expiration_date;
        let risk_free_rate = option.risk_free_rate;

        let (mean_volatility, std_dev) = mean_and_std(vec![
            self.long_call.option.implied_volatility,
            self.short_call.option.implied_volatility,
        ]);

        let mut loss_range = ProfitLossRange::new(
            Some(self.long_call.option.strike_price),
            Some(break_even_point),
            pos_or_panic!(self.get_max_loss()?.to_f64()),
        )?;

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

impl Greeks for BullCallSpread {
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.long_call.option, &self.short_call.option])
    }
}

impl DeltaNeutrality for BullCallSpread {}

impl PnLCalculator for BullCallSpread {
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
                .short_call
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
                .short_call
                .calculate_pnl_at_expiration(underlying_price)?)
    }
}

test_strategy_traits!(BullCallSpread, test_short_call_implementations);

#[cfg(test)]
fn bull_call_spread_test() -> BullCallSpread {
    use rust_decimal_macros::dec;
    let underlying_price = pos_or_panic!(5781.88);
    BullCallSpread::new(
        "SP500".to_string(),
        underlying_price,      // underlying_price
        pos_or_panic!(5750.0), // long_strike_itm
        pos_or_panic!(5820.0), // short_strike
        ExpirationDate::Days(Positive::TWO),
        pos_or_panic!(0.18),  // implied_volatility
        dec!(0.05),           // risk_free_rate
        Positive::ZERO,       // dividend_yield
        pos_or_panic!(3.0),   // long quantity
        pos_or_panic!(85.04), // premium_long
        pos_or_panic!(29.85), // premium_short
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.78),  // open_fee_long
        pos_or_panic!(0.73),  // close_fee_long
        pos_or_panic!(0.73),  // close_fee_short
    )
}

#[cfg(test)]
mod tests_bull_call_spread_strategy {
    use super::*;

    use crate::model::ExpirationDate;

    use approx::assert_relative_eq;
    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_new_bull_call_spread() {
        let spread = bull_call_spread_test();

        assert_eq!(spread.name, "Bull Call Spread");
        assert_eq!(spread.kind, StrategyType::BullCallSpread);
        assert!(!spread.description.is_empty());
        assert_eq!(spread.get_underlying_price(), &pos_or_panic!(5781.88));
        assert_eq!(spread.long_call.option.strike_price, pos_or_panic!(5750.0));
        assert_eq!(spread.short_call.option.strike_price, pos_or_panic!(5820.0));
    }

    #[test]
    fn test_add_leg() {
        let mut spread = bull_call_spread_test();
        let new_long_call = Position::new(
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
                OptionStyle::Call,
                Positive::ZERO,
                None,
            ),
            pos_or_panic!(1.5),
            Utc::now(),
            Positive::ZERO,
            Positive::ZERO,
            None,
            None,
        );

        spread
            .add_position(&new_long_call.clone())
            .expect("Failed to add long call");
        assert_eq!(spread.long_call.option.strike_price, pos_or_panic!(90.0));
    }

    #[test]
    fn test_get_legs() {
        let spread = bull_call_spread_test();
        let legs = spread.get_positions().expect("Failed to get positions");

        assert_eq!(legs.len(), 2);
        assert_eq!(legs[0].option.side, Side::Long);
        assert_eq!(legs[1].option.side, Side::Short);
        assert_eq!(legs[0].option.option_style, OptionStyle::Call);
        assert_eq!(legs[1].option.option_style, OptionStyle::Call);
    }

    #[test]
    fn test_max_profit() {
        let spread = bull_call_spread_test();
        let max_profit = spread.get_max_profit().unwrap();
        assert_eq!(max_profit, pos_or_panic!(35.37));
    }

    #[test]
    fn test_max_loss() {
        let spread = bull_call_spread_test();
        let max_loss = spread.get_max_loss().unwrap();
        assert_eq!(max_loss, pos_or_panic!(174.63));
    }

    #[test]
    fn test_total_cost() {
        let spread = bull_call_spread_test();
        assert_eq!(spread.get_total_cost().unwrap(), pos_or_panic!(264.18));
    }

    #[test]
    fn test_fees() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            pos_or_panic!(0.5), // open_fee_long_call
            pos_or_panic!(0.5), // close_fee_long_call
            pos_or_panic!(0.5), // open_fee_short_call
            pos_or_panic!(0.5), // close_fee_short_call
        );

        assert_eq!(spread.get_fees().unwrap().to_f64(), 2.0);
    }

    #[test]
    fn test_break_even_points() {
        let spread = bull_call_spread_test();
        let break_even_points = spread.get_break_even_points().unwrap();

        assert_eq!(break_even_points.len(), 1);
        assert_eq!(break_even_points[0], pos_or_panic!(5808.21));
    }

    #[test]
    fn test_profit_area() {
        let spread = bull_call_spread_test();
        let area = spread.get_profit_area().unwrap().to_f64().unwrap();
        assert_eq!(area, 2.0850615);
    }

    #[test]
    fn test_profit_ratio() {
        let spread = bull_call_spread_test();
        let ratio = spread.get_profit_ratio().unwrap().to_f64().unwrap();
        assert_relative_eq!(ratio, 20.25425, epsilon = 0.0001);
    }

    #[test]
    fn test_default_strikes() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            Positive::ZERO, // long_strike = default
            Positive::ZERO, // short_strike = default
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        assert_eq!(spread.long_call.option.strike_price, Positive::HUNDRED);
        assert_eq!(spread.short_call.option.strike_price, Positive::HUNDRED);
    }

    #[test]
    fn test_invalid_strikes() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            Positive::TWO,
            Positive::ONE,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        assert!(!spread.validate());
    }
}

#[cfg(test)]
mod tests_bull_call_spread_validation {
    use super::*;

    use crate::model::ExpirationDate;
    use chrono::Utc;
    use rust_decimal_macros::dec;

    fn create_valid_position(
        side: Side,
        strike_price: Positive,
        expiration: ExpirationDate,
    ) -> Position {
        Position::new(
            Options::new(
                OptionType::European,
                side,
                "TEST".to_string(),
                strike_price,
                expiration,
                pos_or_panic!(0.2),
                Positive::ONE,
                Positive::HUNDRED,
                dec!(0.05),
                OptionStyle::Call,
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

    #[test]
    fn test_valid_bull_call_spread() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };

        assert!(spread.validate(), "Valid spread should pass validation");
    }

    #[test]
    fn test_invalid_long_call() {
        let mut invalid_long = create_valid_position(
            Side::Long,
            pos_or_panic!(95.0),
            ExpirationDate::Days(pos_or_panic!(30.0)),
        );
        invalid_long.option.quantity = Positive::ZERO;

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: invalid_long,
            short_call: create_valid_position(
                Side::Short,
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with invalid long call should fail validation"
        );
    }

    #[test]
    fn test_invalid_short_call() {
        let mut invalid_short = create_valid_position(
            Side::Short,
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
        );
        invalid_short.option.quantity = Positive::ZERO;

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_call: invalid_short,
        };

        assert!(
            !spread.validate(),
            "Spread with invalid short call should fail validation"
        );
    }

    #[test]
    fn test_invalid_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with long strike price >= short strike price should fail validation"
        );
    }

    #[test]
    fn test_equal_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                Positive::HUNDRED,
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };

        assert!(
            !spread.validate(),
            "Spread with equal strike prices should fail validation"
        );
    }

    #[test]
    fn test_different_expiration_dates_same_day() {
        let date1 = ExpirationDate::DateTime(Utc::now() + chrono::Duration::days(30));
        let date2 = ExpirationDate::Days(pos_or_panic!(30.0));

        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(Side::Long, pos_or_panic!(95.0), date1),
            short_call: create_valid_position(Side::Short, Positive::HUNDRED, date2),
        };

        assert!(
            spread.validate(),
            "Spread with different ExpirationDate types but same date should pass validation"
        );
    }

    #[test]
    fn test_boundary_strike_prices() {
        let spread = BullCallSpread {
            name: "Test Bull Call Spread".to_string(),
            kind: StrategyType::BullCallSpread,
            description: "Test".to_string(),
            break_even_points: Vec::new(),
            long_call: create_valid_position(
                Side::Long,
                pos_or_panic!(94.99),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
            short_call: create_valid_position(
                Side::Short,
                pos_or_panic!(95.0),
                ExpirationDate::Days(pos_or_panic!(30.0)),
            ),
        };

        assert!(
            spread.validate(),
            "Spread with very close but valid strike prices should pass validation"
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_optimization {
    use super::*;

    use crate::chains::OptionData;
    use crate::model::ExpirationDate;

    use num_traits::ToPrimitive;
    use positive::spos;
    use rust_decimal_macros::dec;

    fn create_test_chain() -> OptionChain {
        let mut chain = OptionChain::new(
            "TEST",
            Positive::HUNDRED,
            "2024-12-31".to_string(),
            None,
            None,
        );

        chain.add_option(
            pos_or_panic!(85.0), // strike
            spos!(16.0),         // call_bid
            spos!(16.2),         // call_ask
            spos!(15.6),         // put_bid
            spos!(15.8),         // put_ask
            pos_or_panic!(0.2),  // implied_volatility
            Some(dec!(0.8)),     // delta
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(100.0), // volume
            Some(50),     // open_interest
            None,
        );

        chain.add_option(
            pos_or_panic!(90.0),
            spos!(11.5),
            spos!(11.7),
            spos!(11.1),
            spos!(11.3),
            pos_or_panic!(0.2),
            Some(dec!(0.7)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(150.0),
            Some(75),
            None,
        );

        chain.add_option(
            pos_or_panic!(95.0),
            spos!(7.0),
            spos!(7.2),
            spos!(6.6),
            spos!(6.8),
            pos_or_panic!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(200.0),
            Some(100),
            None,
        );

        chain.add_option(
            Positive::HUNDRED,
            spos!(3.5),
            spos!(3.7),
            spos!(3.1),
            spos!(3.3),
            pos_or_panic!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(250.0),
            Some(125),
            None,
        );

        chain.add_option(
            pos_or_panic!(105.0),
            spos!(1.0),
            spos!(1.2),
            spos!(0.6),
            spos!(0.8),
            pos_or_panic!(0.2),
            Some(dec!(0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
            spos!(300.0),
            Some(150),
            None,
        );

        chain
    }

    fn create_base_spread() -> BullCallSpread {
        BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(7.2), // premium_long_call
            pos_or_panic!(3.5), // premium_short_call
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        )
    }

    #[test]
    fn test_find_optimal_ratio() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Ratio);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.get_profit_ratio().unwrap().to_f64().unwrap() > 0.0,
            "Profit ratio should be positive"
        );
    }

    #[test]
    fn test_find_optimal_area() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::All, OptimizationCriteria::Area);

        assert!(spread.validate(), "Optimized spread should be valid");
        assert!(
            spread.get_profit_area().unwrap().to_f64().unwrap() > 0.0,
            "Profit area should be positive"
        );
    }

    #[test]
    fn test_find_optimal_upper_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Upper, OptimizationCriteria::Ratio);

        assert!(spread.short_call.option.strike_price >= chain.underlying_price);
        assert!(spread.long_call.option.strike_price >= chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_lower_side() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(&chain, FindOptimalSide::Lower, OptimizationCriteria::Ratio);

        assert!(spread.short_call.option.strike_price <= chain.underlying_price);
        assert!(spread.long_call.option.strike_price <= chain.underlying_price);
    }

    #[test]
    fn test_find_optimal_range() {
        let mut spread = create_base_spread();
        let chain = create_test_chain();

        spread.find_optimal(
            &chain,
            FindOptimalSide::Range(pos_or_panic!(90.0), Positive::HUNDRED),
            OptimizationCriteria::Ratio,
        );

        assert!(spread.short_call.option.strike_price <= Positive::HUNDRED);
        assert!(spread.short_call.option.strike_price >= pos_or_panic!(90.0));
        assert!(spread.long_call.option.strike_price <= Positive::HUNDRED);
        assert!(spread.long_call.option.strike_price >= pos_or_panic!(90.0));
    }

    #[test]
    fn test_is_valid_long_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos_or_panic!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            pos_or_panic!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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

        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(!spread.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
        assert!(spread.is_valid_optimal_option(
            &option,
            &FindOptimalSide::Range(pos_or_panic!(90.0), Positive::HUNDRED)
        ));
    }

    #[test]
    fn test_is_valid_short_option() {
        let spread = create_base_spread();
        let option = OptionData::new(
            pos_or_panic!(105.0),
            spos!(1.0),
            spos!(1.2),
            None,
            None,
            pos_or_panic!(0.2),
            Some(dec!(0.4)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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

        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::All));
        assert!(!spread.is_valid_optimal_option(&option, &FindOptimalSide::Lower));
        assert!(spread.is_valid_optimal_option(&option, &FindOptimalSide::Upper));
        assert!(!spread.is_valid_optimal_option(
            &option,
            &FindOptimalSide::Range(pos_or_panic!(90.0), Positive::HUNDRED)
        ));
    }

    #[test]
    fn test_are_valid_prices() {
        let spread = create_base_spread();
        let long_option = OptionData::new(
            pos_or_panic!(95.0),
            spos!(7.0),
            spos!(7.2),
            None,
            None,
            pos_or_panic!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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
        let short_option = OptionData::new(
            Positive::HUNDRED,
            spos!(3.5),
            spos!(3.7),
            None,
            None,
            pos_or_panic!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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

        let legs = StrategyLegs::TwoLegs {
            first: &long_option,
            second: &short_option,
        };
        assert!(spread.are_valid_legs(&legs));
    }

    #[test]
    fn test_invalid_prices() {
        let spread = create_base_spread();
        let long_option = OptionData::new(
            pos_or_panic!(95.0),
            spos!(7.2),
            Some(Positive::ZERO),
            None,
            None,
            pos_or_panic!(0.2),
            Some(dec!(0.6)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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
        let short_option = OptionData::new(
            Positive::HUNDRED,
            Some(Positive::ZERO),
            spos!(3.5),
            None,
            None,
            pos_or_panic!(0.2),
            Some(dec!(0.5)),
            Some(dec!(0.2)),
            Some(dec!(0.2)),
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

        let legs = StrategyLegs::TwoLegs {
            first: &long_option,
            second: &short_option,
        };
        assert!(!spread.are_valid_legs(&legs));
    }

    #[test]
    fn test_create_strategy() {
        let spread = create_base_spread();
        let chain = create_test_chain();
        let long_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == pos_or_panic!(95.0))
            .unwrap();
        let short_option = chain
            .options
            .iter()
            .find(|o| o.strike_price == Positive::HUNDRED)
            .unwrap();

        let legs = StrategyLegs::TwoLegs {
            first: long_option,
            second: short_option,
        };
        let new_strategy = spread.create_strategy(&chain, &legs);

        assert!(new_strategy.validate());
        assert_eq!(
            new_strategy.long_call.option.strike_price,
            pos_or_panic!(95.0)
        );
        assert_eq!(
            new_strategy.short_call.option.strike_price,
            Positive::HUNDRED
        );
        assert_eq!(
            new_strategy.long_call.option.option_style,
            OptionStyle::Call
        );
        assert_eq!(
            new_strategy.short_call.option.option_style,
            OptionStyle::Call
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_profit {
    use super::*;

    use crate::model::ExpirationDate;

    use num_traits::ToPrimitive;
    use rust_decimal_macros::dec;

    #[test]
    fn test_profit_below_long_strike() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5800.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -24.63
        );
    }

    #[test]
    fn test_profit_at_long_strike() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5807.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -3.63
        );
    }

    #[test]
    fn test_profit_between_strikes() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5810.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            5.37
        );
    }

    #[test]
    fn test_profit_at_get_break_even_points() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5808.21);
        assert!(spread.calculate_profit_at(&price).unwrap().abs() < dec!(0.001));
    }

    #[test]
    fn test_profit_at_short_strike() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5818.21);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            30.0
        );
    }

    #[test]
    fn test_profit_above_short_strike() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5908.21);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            35.37
        );
    }

    #[test]
    fn test_profit_with_multiple_contracts() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::TWO,
            pos_or_panic!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let price = pos_or_panic!(105.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            6.0
        );
    }

    #[test]
    fn test_profit_with_fees() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(4.0),
            Positive::TWO,
            pos_or_panic!(0.5), // open_fee_long_call
            pos_or_panic!(0.5), // close_fee_long_call
            pos_or_panic!(0.5), // open_fee_short_call
            pos_or_panic!(0.5), // close_fee_short_call
        );

        let price = pos_or_panic!(105.0);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            1.0
        );
    }

    #[test]
    fn test_maximum_profit() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5858.21);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            35.37
        );
    }

    #[test]
    fn test_maximum_loss() {
        let spread = bull_call_spread_test();
        let price = pos_or_panic!(5708.21);
        assert_eq!(
            spread
                .calculate_profit_at(&price)
                .unwrap()
                .to_f64()
                .unwrap(),
            -174.63
        );
    }
}

#[cfg(test)]
mod tests_bull_call_spread_graph {
    use super::*;

    use rust_decimal_macros::dec;

    #[test]
    fn test_title_format() {
        let spread = bull_call_spread_test();
        let title = spread.get_title();
        assert!(title.contains("BullCallSpread Strategy"));
        assert!(title.contains("SP500 @ $5820 Short Call European Option"));
        assert!(title.contains("SP500 @ $5750 Long Call European Option"));
    }

    #[test]
    fn test_graph_at_extremes() {
        let spread = bull_call_spread_test();
        let profit_at_zero = spread.calculate_profit_at(&Positive::ZERO).unwrap();
        let profit_at_high = spread.calculate_profit_at(&pos_or_panic!(1000.0)).unwrap();

        assert_eq!(profit_at_zero, dec!(-174.63));
        assert_eq!(profit_at_high, dec!(-174.63));
    }
}

#[cfg(test)]
mod tests_bull_call_spread_probability {
    use super::*;
    use positive::assert_pos_relative_eq;

    use crate::strategies::probabilities::utils::PriceTrend;
    use rust_decimal_macros::dec;

    #[test]
    fn test_get_expiration() {
        let spread = bull_call_spread_test();
        let expiration_date = *spread.get_expiration().values().next().unwrap();
        assert_eq!(expiration_date, &ExpirationDate::Days(Positive::TWO));
    }

    #[test]
    fn test_get_risk_free_rate() {
        let spread = bull_call_spread_test();
        assert_eq!(
            **spread.get_risk_free_rate().values().next().unwrap(),
            dec!(0.05)
        );
    }

    #[test]
    fn test_get_profit_ranges() {
        let spread = bull_call_spread_test();
        let result = spread.get_profit_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.lower_bound.unwrap(), pos_or_panic!(5808.21)); // Break-even
        assert_eq!(range.upper_bound.unwrap(), pos_or_panic!(5820.0)); // Short strike
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_get_loss_ranges() {
        let spread = bull_call_spread_test();
        let result = spread.get_loss_ranges();
        assert!(result.is_ok());

        let ranges = result.unwrap();
        assert_eq!(ranges.len(), 1);

        let range = &ranges[0];
        assert_eq!(range.lower_bound.unwrap(), pos_or_panic!(5750.0)); // Long strike
        assert_eq!(range.upper_bound.unwrap(), pos_or_panic!(5808.21)); // Break-even
        assert!(range.probability > Positive::ZERO);
    }

    #[test]
    fn test_probability_of_profit() {
        let spread = bull_call_spread_test();
        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_with_volatility_adjustment() {
        let spread = bull_call_spread_test();
        let vol_adj = Some(VolatilityAdjustment {
            base_volatility: pos_or_panic!(0.25),
            std_dev_adjustment: pos_or_panic!(0.05),
        });

        let result = spread.probability_of_profit(vol_adj, None);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob > Positive::ZERO);
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_with_uptrend() {
        let spread = bull_call_spread_test();
        let trend = Some(PriceTrend {
            drift_rate: 0.8,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob < pos_or_panic!(0.5));
        assert!(prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_with_downtrend() {
        let spread = bull_call_spread_test();
        let trend = Some(PriceTrend {
            drift_rate: -0.1,
            confidence: 0.95,
        });

        let result = spread.probability_of_profit(None, trend);
        assert!(result.is_ok());

        let prob = result.unwrap();
        assert!(prob < pos_or_panic!(0.5));
        assert!(prob > Positive::ZERO);
    }

    #[test]
    fn test_analyze_probabilities() {
        let spread = bull_call_spread_test();
        let result = spread.analyze_probabilities(None, None);
        assert!(result.is_ok());

        let analysis = result.unwrap();
        assert!(analysis.probability_of_profit > Positive::ZERO);
        assert!(analysis.probability_of_max_profit >= Positive::ZERO);
        assert!(analysis.probability_of_max_loss >= Positive::ZERO);
        assert_pos_relative_eq!(
            analysis.expected_value,
            Positive::ZERO,
            pos_or_panic!(0.000001)
        );
        assert!(!analysis.break_even_points.is_empty());
        assert!(analysis.risk_reward_ratio > Positive::ZERO);
    }

    #[test]
    fn test_calculate_extreme_probabilities() {
        let spread = bull_call_spread_test();
        let result = spread.calculate_extreme_probabilities(None, None);
        assert!(result.is_ok());

        let (max_profit_prob, max_loss_prob) = result.unwrap();
        assert!(max_profit_prob >= Positive::ZERO);
        assert!(max_loss_prob >= Positive::ZERO);
        assert!(max_profit_prob + max_loss_prob <= Positive::ONE);
    }

    #[test]
    fn test_probability_near_expiration() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            ExpirationDate::Days(Positive::ONE),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());
        let prob = result.unwrap();

        assert!(prob < pos_or_panic!(0.5));
    }

    #[test]
    fn test_probability_with_high_volatility() {
        let spread = BullCallSpread::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            Positive::HUNDRED,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            pos_or_panic!(0.50), // Alta volatilidad
            dec!(0.05),
            Positive::ZERO,
            Positive::ONE,
            pos_or_panic!(4.0),
            Positive::TWO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
            Positive::ZERO,
        );

        let result = spread.probability_of_profit(None, None);
        assert!(result.is_ok());
        let prob = result.unwrap();

        assert!(prob < pos_or_panic!(0.3));
        assert!(prob < pos_or_panic!(0.7));
    }
}

#[cfg(test)]
mod tests_delta {
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::bull_call_spread::BullCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{ExpirationDate, Side, assert_decimal_eq};
    use positive::{Positive, assert_pos_relative_eq, pos_or_panic};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullCallSpread {
        let underlying_price = pos_or_panic!(5781.88);
        BullCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::ONE,        // long quantity
            pos_or_panic!(85.04), // premium_long
            pos_or_panic!(29.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strike = pos_or_panic!(5820.0);
        let strategy = get_strategy(pos_or_panic!(5750.0), strike);
        let size = dec!(0.3502);
        let delta = pos_or_panic!(1.092269393430898);
        let k = pos_or_panic!(5820.0);
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5850.0), pos_or_panic!(5820.0));
        let size = dec!(-0.1234671);
        let delta = pos_or_panic!(0.626251716937553);
        let k = pos_or_panic!(5850.0);
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
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5820.0), pos_or_panic!(5820.0));

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
mod tests_delta_size {
    use crate::greeks::Greeks;
    use crate::model::types::OptionStyle;
    use crate::strategies::bull_call_spread::BullCallSpread;
    use crate::strategies::delta_neutral::DELTA_THRESHOLD;
    use crate::strategies::delta_neutral::{DeltaAdjustment, DeltaNeutrality};
    use crate::{ExpirationDate, Side, assert_decimal_eq};
    use positive::{Positive, assert_pos_relative_eq, pos_or_panic};
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    fn get_strategy(long_strike: Positive, short_strike: Positive) -> BullCallSpread {
        let underlying_price = pos_or_panic!(5781.88);
        BullCallSpread::new(
            "SP500".to_string(),
            underlying_price, // underlying_price
            long_strike,      // long_strike
            short_strike,     // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::TWO,        // long quantity
            pos_or_panic!(85.04), // premium_long
            pos_or_panic!(29.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
    }

    #[test]
    fn create_test_reducing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5750.0), pos_or_panic!(5820.9));
        let size = dec!(0.7086);
        let delta = pos_or_panic!(2.239306943523854);
        let k = pos_or_panic!(5820.9);
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
    fn create_test_increasing_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5850.0), pos_or_panic!(5820.0));
        let size = dec!(-0.246934);
        let delta = pos_or_panic!(1.252503433875106);
        let k = pos_or_panic!(5850.0);
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
    fn create_test_no_adjustments() {
        let strategy = get_strategy(pos_or_panic!(5820.0), pos_or_panic!(5820.0));

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
mod tests_bull_call_spread_position_management {
    use super::*;

    use crate::error::position::PositionValidationErrorKind;
    use crate::model::types::{OptionStyle, Side};

    use rust_decimal_macros::dec;

    fn create_test_bull_call_spread() -> BullCallSpread {
        BullCallSpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5750.0),  // long_strike_itm
            pos_or_panic!(5820.0),  // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::TWO,        // long quantity
            pos_or_panic!(85.04), // premium_long
            pos_or_panic!(29.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
    }

    #[test]
    fn test_short_bull_call_spread_get_position() {
        let mut bull_call_spread = create_test_bull_call_spread();

        // Test getting short call position
        let call_position =
            bull_call_spread.get_position(&OptionStyle::Call, &Side::Long, &pos_or_panic!(5750.0));
        assert!(call_position.is_ok());
        let positions = call_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5750.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Long);

        // Test getting short put position
        let put_position =
            bull_call_spread.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(5820.0));
        assert!(put_position.is_ok());
        let positions = put_position.unwrap();
        assert_eq!(positions.len(), 1);
        assert_eq!(positions[0].option.strike_price, pos_or_panic!(5820.0));
        assert_eq!(positions[0].option.option_style, OptionStyle::Call);
        assert_eq!(positions[0].option.side, Side::Short);

        // Test getting non-existent position
        let invalid_position =
            bull_call_spread.get_position(&OptionStyle::Call, &Side::Short, &pos_or_panic!(5821.0));
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
    fn test_short_bull_call_spread_modify_position() {
        let mut bull_call_spread = create_test_bull_call_spread();

        // Modify short call position
        let mut modified_call = bull_call_spread.short_call.clone();
        modified_call.option.quantity = Positive::TWO;
        let result = bull_call_spread.modify_position(&modified_call);
        assert!(result.is_ok());
        assert_eq!(bull_call_spread.short_call.option.quantity, Positive::TWO);

        // Modify short put position
        let mut modified_put = bull_call_spread.long_call.clone();
        modified_put.option.quantity = Positive::TWO;
        let result = bull_call_spread.modify_position(&modified_put);
        assert!(result.is_ok());
        assert_eq!(bull_call_spread.long_call.option.quantity, Positive::TWO);

        // Test modifying with invalid position
        let mut invalid_position = bull_call_spread.short_call.clone();
        invalid_position.option.strike_price = pos_or_panic!(95.0);
        let result = bull_call_spread.modify_position(&invalid_position);
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

    use rust_decimal_macros::dec;

    // Helper function to create a test strategy
    fn create_test_strategy() -> BullCallSpread {
        BullCallSpread::new(
            "SP500".to_string(),
            pos_or_panic!(5781.88), // underlying_price
            pos_or_panic!(5750.0),  // long_strike_itm
            pos_or_panic!(5820.0),  // short_strike
            ExpirationDate::Days(Positive::TWO),
            pos_or_panic!(0.18),  // implied_volatility
            dec!(0.05),           // risk_free_rate
            Positive::ZERO,       // dividend_yield
            Positive::TWO,        // long quantity
            pos_or_panic!(85.04), // premium_long
            pos_or_panic!(29.85), // premium_short
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.78),  // open_fee_long
            pos_or_panic!(0.73),  // close_fee_long
            pos_or_panic!(0.73),  // close_fee_short
        )
    }

    #[test]
    fn test_adjust_existing_call_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.long_call.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5750.0),
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
    fn test_adjust_existing_put_position() {
        let mut strategy = create_test_strategy();
        let initial_quantity = strategy.short_call.option.quantity;
        let adjustment = Positive::ONE;

        let result = strategy.adjust_option_position(
            adjustment.to_dec(),
            &pos_or_panic!(5820.0),
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
    fn test_adjust_nonexistent_position() {
        let mut strategy = create_test_strategy();

        // Try to adjust a non-existent long call position
        let result = strategy.adjust_option_position(
            Decimal::ONE,
            &pos_or_panic!(110.0),
            &OptionStyle::Put,
            &Side::Long,
        );

        assert!(result.is_err());
        let err = result.unwrap_err();
        // StrategyError wraps PositionError, so we check the error message
        assert!(
            err.to_string()
                .contains("Put is not valid for BearCallSpread")
        );
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
            &pos_or_panic!(5820.0),
            &OptionStyle::Call,
            &Side::Short,
        );

        assert!(result.is_ok());
        assert_eq!(strategy.short_call.option.quantity, initial_quantity);
    }
}

#[cfg(test)]
mod tests_bull_call_spread_constructor {
    use super::*;

    use crate::model::utils::create_sample_position;

    #[test]
    fn test_get_strategy_valid() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];

        let result = BullCallSpread::get_strategy(&options);
        assert!(result.is_ok());

        let strategy = result.unwrap();
        assert_eq!(strategy.long_call.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(
            strategy.short_call.option.strike_price,
            pos_or_panic!(105.0)
        );
    }

    #[test]
    fn test_get_strategy_wrong_number_of_options() {
        let options = vec![create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        )];

        let result = BullCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Spread get_strategy" && reason == "Must have exactly 2 options"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_option_style() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        option1.option.option_style = OptionStyle::Put;
        let option2 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );

        let options = vec![option1, option2];
        let result = BullCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Spread get_strategy" && reason == "Options must be calls"
        ));
    }

    #[test]
    fn test_get_strategy_wrong_sides() {
        let options = vec![
            create_sample_position(
                OptionStyle::Call,
                Side::Short,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(95.0),
                pos_or_panic!(0.2),
            ),
            create_sample_position(
                OptionStyle::Call,
                Side::Long,
                pos_or_panic!(90.0),
                Positive::ONE,
                pos_or_panic!(105.0),
                pos_or_panic!(0.2),
            ),
        ];
        let result = BullCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Spread get_strategy"
                && reason == "Bull Call Spread requires a long lower strike call and a short higher strike call"
        ));
    }

    #[test]
    fn test_get_strategy_different_expiration_dates() {
        let mut option1 = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        let mut option2 = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(105.0),
            pos_or_panic!(0.2),
        );

        option1.option.expiration_date = ExpirationDate::Days(pos_or_panic!(30.0));
        option2.option.expiration_date = ExpirationDate::Days(pos_or_panic!(60.0));

        let options = vec![option1, option2];
        let result = BullCallSpread::get_strategy(&options);
        assert!(matches!(
            result,
            Err(StrategyError::OperationError(OperationErrorKind::InvalidParameters { operation, reason }))
            if operation == "Bull Call Spread get_strategy" && reason == "Options must have the same expiration date"
        ));
    }
}

#[cfg(test)]
mod tests_bull_call_spread_pnl {
    use super::*;
    use positive::assert_pos_relative_eq;

    use crate::assert_decimal_eq;
    use crate::model::utils::create_sample_position;
    use rust_decimal_macros::dec;

    /// Helper function to create a standard Bull Call Spread for testing
    fn create_test_bull_call_spread() -> Result<BullCallSpread, StrategyError> {
        // Create long call with lower strike
        let long_call = create_sample_position(
            OptionStyle::Call,
            Side::Long,
            Positive::HUNDRED,   // Underlying price
            Positive::ONE,       // Quantity
            pos_or_panic!(95.0), // Lower strike price
            pos_or_panic!(0.2),  // Implied volatility
        );

        // Create short call with higher strike
        let short_call = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            Positive::HUNDRED,    // Same underlying price
            Positive::ONE,        // Quantity
            pos_or_panic!(105.0), // Higher strike price
            pos_or_panic!(0.2),   // Implied volatility
        );

        BullCallSpread::get_strategy(&[long_call, short_call])
    }

    #[test]
    fn test_calculate_pnl_below_strikes() {
        let spread = create_test_bull_call_spread().unwrap();
        let market_price = pos_or_panic!(90.0); // Below both strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options OTM, should be close to max loss
        // Initial costs: Premium for long call (5.0) + total fees (2.0)
        // Initial income: Premium from short call (5.0)
        assert_pos_relative_eq!(pnl.initial_costs, pos_or_panic!(7.0), pos_or_panic!(1e-6));
        assert_pos_relative_eq!(pnl.initial_income, pos_or_panic!(5.0), pos_or_panic!(1e-6));
        assert!(pnl.unrealized.unwrap() < dec!(-2.0)); // Near max loss
    }

    #[test]
    fn test_calculate_pnl_above_strikes() {
        let spread = create_test_bull_call_spread().unwrap();
        let market_price = pos_or_panic!(110.0); // Above both strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.2);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // Both options ITM, should be close to max profit
        assert!(pnl.unrealized.unwrap() > dec!(-2.0)); // Near max profit
        assert!(pnl.unrealized.unwrap() < dec!(5.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_loss() {
        let spread = create_test_bull_call_spread().unwrap();
        let underlying_price = pos_or_panic!(90.0); // Well below both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // Adjusted expectation based on actual implementation
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos_or_panic!(5.0));
        assert_eq!(pnl.initial_costs, pos_or_panic!(7.0));
    }

    #[test]
    fn test_calculate_pnl_with_higher_volatility() {
        let spread = create_test_bull_call_spread().unwrap();
        let market_price = Positive::HUNDRED;
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.4); // Higher volatility

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());

        // With higher volatility, both options are worth more
        // Net effect should be slightly negative as short gamma position
        assert!(pnl.unrealized.unwrap() < dec!(0.0));
        // But still capped by the spread width
        assert!(pnl.unrealized.unwrap() > dec!(-5.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_at_long_strike() {
        let spread = create_test_bull_call_spread().unwrap();
        let underlying_price = pos_or_panic!(95.0); // At long strike

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());

        // At the long strike, long call is ATM
        // Loss should be just the costs minus income
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(-2.0), dec!(1e-6));
    }

    #[test]
    fn test_calculate_pnl_between_strikes() {
        let spread = create_test_bull_call_spread().unwrap();
        let market_price = pos_or_panic!(102.5); // Between strikes
        let expiration_date = ExpirationDate::Days(pos_or_panic!(20.0));
        let implied_volatility = pos_or_panic!(0.1);

        let result = spread.calculate_pnl(&market_price, expiration_date, &implied_volatility);
        assert!(result.is_ok());
        let pnl = result.unwrap();
        assert!(pnl.unrealized.is_some());
        assert!(pnl.unrealized.unwrap() > Decimal::ZERO); // Slight gain
        assert!(pnl.unrealized.unwrap() < dec!(3.0)); // But not too much gain
    }

    #[test]
    fn test_calculate_pnl_at_expiration_max_profit() {
        let spread = create_test_bull_call_spread().unwrap();
        let underlying_price = pos_or_panic!(110.0); // Above both strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(8.0), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos_or_panic!(5.0));
        assert_eq!(pnl.initial_costs, pos_or_panic!(7.0));
    }

    #[test]
    fn test_calculate_pnl_at_expiration_between_strikes() {
        let spread = create_test_bull_call_spread().unwrap();
        let underlying_price = pos_or_panic!(102.5); // Between strikes

        let result = spread.calculate_pnl_at_expiration(&underlying_price);
        assert!(result.is_ok());

        let pnl = result.unwrap();
        assert!(pnl.realized.is_some());
        assert_decimal_eq!(pnl.realized.unwrap(), dec!(5.5), dec!(1e-6));
        assert_eq!(pnl.initial_income, pos_or_panic!(5.0));
        assert_eq!(pnl.initial_costs, pos_or_panic!(7.0));
    }
}
