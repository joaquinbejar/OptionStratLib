/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/
use crate::chains::OptionData;
use crate::error::position::PositionValidationErrorKind;
use crate::error::{GreeksError, PositionError, TransactionError};
use crate::greeks::Greeks;
use crate::model::trade::TradeStatusAble;
use crate::model::types::{Action, OptionStyle, Side};
use crate::model::{Trade, TradeAble, TradeStatus};
use crate::pnl::utils::PnL;
use crate::pnl::{PnLCalculator, Transaction, TransactionAble};
use crate::pricing::payoff::Profit;
use crate::visualization::model::ChartVerticalLine;
use crate::visualization::utils::Graph;
use crate::{ExpirationDate, Options};
use crate::{Positive, pos};
use chrono::{DateTime, Utc};
use num_traits::ToPrimitive;
use plotters::prelude::{BLACK, ShapeStyle};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::error::Error;
use tracing::{debug, trace};

/// The `Position` struct represents a financial position in an options market.
///
/// This structure encapsulates all the necessary information to track an options position,
/// including the underlying option details, costs associated with the position, and the date
/// when the position was opened. It provides methods for analyzing profitability, time metrics,
/// and position characteristics.
///
/// # Examples
///
/// ```rust
/// use optionstratlib::{Options, pos, Side, OptionStyle};
/// use chrono::Utc;
/// use tracing::info;
/// use optionstratlib::model::Position;
/// use optionstratlib::model::utils::create_sample_option_simplest;
///
/// let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
/// let position = Position::new(
///     option,
///     pos!(5.25),           // premium per contract
///     Utc::now(),           // position open date
///     pos!(0.65),           // opening fee per contract
///     pos!(0.65),           // closing fee per contract
/// );
///
/// let total_cost = position.total_cost().unwrap();
/// info!("Total position cost: {}", total_cost);
/// ```
#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct Position {
    /// The detailed options contract information, including the type, strike price,
    /// expiration, underlying asset details, and other option-specific parameters.
    pub option: Options,

    /// The premium paid or received per contract. For long positions, this represents
    /// the cost per contract; for short positions, this is the credit received.
    pub premium: Positive,

    /// The date and time when the position was opened, used for calculating
    /// time-based metrics like days held and days to expiration.
    pub date: DateTime<Utc>,

    /// The fee paid to open the position per contract. This typically includes
    /// broker commissions and exchange fees.
    pub open_fee: Positive,

    /// The fee that will be paid to close the position per contract. This is used
    /// in profit/loss calculations to account for all transaction costs.
    pub close_fee: Positive,
}

impl Position {
    /// Creates a new options position.
    ///
    /// This constructor initializes a new `Position` instance representing an options trade,
    /// capturing all essential information for position tracking and analysis.
    ///
    /// # Parameters
    ///
    /// * `option` - The options contract details including type (call/put), strike price,
    ///   expiration date, underlying asset information, and other option parameters.
    ///
    /// * `premium` - The premium paid (for long positions) or received (for short positions)
    ///   per contract, represented as a positive value.
    ///
    /// * `date` - The timestamp when the position was opened, used for calculating time-based
    ///   metrics like days to expiration and position duration.
    ///
    /// * `open_fee` - The transaction costs paid to open the position per contract,
    ///   including broker commissions and exchange fees.
    ///
    /// * `close_fee` - The anticipated transaction costs to close the position per contract,
    ///   used for accurate profit/loss calculations.
    ///
    /// # Returns
    ///
    /// Returns a new `Position` instance containing the provided information.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use optionstratlib::{Options, pos, Side, OptionStyle};
    /// use chrono::Utc;
    /// use optionstratlib::model::Position;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    ///
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
    /// let position = Position::new(
    ///     option,
    ///     pos!(5.25),           // premium per contract
    ///     Utc::now(),           // position open date
    ///     pos!(0.65),           // opening fee per contract
    ///     pos!(0.65),           // closing fee per contract
    /// );
    /// ```
    pub fn new(
        option: Options,
        premium: Positive,
        date: DateTime<Utc>,
        open_fee: Positive,
        close_fee: Positive,
    ) -> Self {
        Position {
            option,
            premium,
            date,
            open_fee,
            close_fee,
        }
    }

    /// Updates a position with data from an `OptionData` instance, refreshing premium values
    /// and option details.
    ///
    /// This method handles the complete update of a position based on new market data,
    /// including:
    ///
    /// 1. Setting the position's timestamp to the current UTC time
    /// 2. Updating the underlying option details through the option's own update method
    /// 3. Setting the premium value based on the position's side (Long/Short) and option style (Call/Put)
    ///
    /// The premium is determined as follows:
    /// - For Long Call positions: Uses the call ask price (price to buy a call)
    /// - For Long Put positions: Uses the put ask price (price to buy a put)
    /// - For Short Call positions: Uses the call bid price (price to sell a call)
    /// - For Short Put positions: Uses the put bid price (price to sell a put)
    ///
    /// # Parameters
    ///
    /// * `option_data` - Reference to an `OptionData` struct containing current market data
    ///   for the relevant option, including bid/ask prices and option characteristics.
    ///
    /// # Panics
    ///
    /// This method will panic if the required premium value is `None` in the provided `option_data`.
    ///
    pub(crate) fn update_from_option_data(&mut self, option_data: &OptionData) {
        self.date = Utc::now();
        self.option.update_from_option_data(option_data);
        match (self.option.side, self.option.option_style) {
            (Side::Long, OptionStyle::Call) => {
                self.premium = option_data.call_ask.unwrap();
            }
            (Side::Long, OptionStyle::Put) => {
                self.premium = option_data.put_ask.unwrap();
            }
            (Side::Short, OptionStyle::Call) => {
                self.premium = option_data.call_bid.unwrap();
            }
            (Side::Short, OptionStyle::Put) => {
                self.premium = option_data.put_bid.unwrap();
            }
        }
        trace!("Updated position: {:#?}", self);
    }

    /// Calculates the total cost of the position based on the option's side and fees.
    ///
    /// Depending on whether the position is long or short, different components
    /// contribute to the total cost calculation:
    ///
    /// - For a long position, the total cost includes the premium, open fee, and close fee
    ///   multiplied by the option's quantity.
    /// - For a short position, the total cost includes only the open fee and close fee
    ///   multiplied by the option's quantity.
    ///
    /// # Returns
    ///
    /// A `f64` representing the total cost of the position. THE VALUE IS ALWAYS POSITIVE
    ///
    pub fn total_cost(&self) -> Result<Positive, PositionError> {
        let total_cost = match self.option.side {
            Side::Long => (self.premium + self.open_fee + self.close_fee) * self.option.quantity,
            Side::Short => self.fees()?,
        };

        Ok(total_cost)
    }

    /// Calculates the premium received from an options position.
    ///
    /// This method determines the premium amount received based on the position's side:
    /// - For long positions, it returns zero as the trader pays premium (doesn't receive any)
    /// - For short positions, it returns the total premium received (premium per contract × quantity)
    ///
    /// The result is always returned as a `Positive` value, ensuring non-negative amounts.
    ///
    /// # Returns
    ///
    /// - `Result<Positive, PositionError>` - A result containing the premium received as a `Positive`
    ///   value if successful, or a `PositionError` if any calculation errors occur.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use optionstratlib::{pos, Side, OptionStyle};
    /// use optionstratlib::model::Position;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use chrono::Utc;
    /// use tracing::info;
    ///
    /// // Create a short position
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
    /// let position = Position::new(
    ///     option,
    ///     pos!(5.25),  // premium per contract
    ///     Utc::now(),  // position open date
    ///     pos!(0.65),  // opening fee
    ///     pos!(0.65),  // closing fee
    /// );
    ///
    /// // Calculate premium received
    /// let received = position.premium_received().unwrap();
    /// info!("Premium received: {}", received);
    /// ```
    pub fn premium_received(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::ZERO),
            Side::Short => Ok(self.premium * self.option.quantity),
        }
    }

    /// Calculates the net premium received for the position.
    ///
    /// This method determines the premium amount received after accounting for costs,
    /// which is relevant primarily for short positions. For long positions, this always
    /// returns zero as premium is paid rather than received.
    ///
    /// For short positions, the method calculates the difference between the premium
    /// received and the total costs incurred. If this value is positive (meaning the
    /// premium exceeds the costs), it represents the maximum potential profit for the
    /// position. If negative, the position is considered invalid as it would represent
    /// a guaranteed loss.
    ///
    /// # Returns
    ///
    /// - `Ok(Positive)` - The net premium received as a non-negative value
    /// - `Err(PositionError)` - If the position is invalid because the premium received
    ///   is less than the costs, resulting in a guaranteed loss
    ///
    pub fn net_premium_received(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::ZERO),
            Side::Short => {
                // max profit is premium received - fees (cost)
                let premium = self.premium * self.option.quantity;
                let cost = -self.total_cost()?.to_dec();
                match premium > cost {
                    true => Ok(premium + cost),
                    false => Err(PositionError::ValidationError(
                        PositionValidationErrorKind::InvalidPosition {
                            reason: "Max profit is negative.".to_string(),
                        },
                    )),
                }
            }
        }
    }

    /// Calculates the profit and loss (PnL) at the option's expiration.
    ///
    /// This function determines the total profit or loss that would be realized
    /// when the option position expires, taking into account the intrinsic value
    /// at expiration, the cost to establish the position, and any premiums received.
    ///
    /// # Arguments
    ///
    /// * `price` - An optional reference to a positive decimal value representing
    ///   the underlying asset price at expiration. If None is provided, the calculation
    ///   will use the current underlying price stored in the option.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, Box<dyn Error>>` - The calculated profit or loss as a Decimal value,
    ///   or an error if the calculation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    ///
    /// // Assuming position is a properly initialized Position
    /// use chrono::Utc;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use optionstratlib::{pos, OptionStyle, Side};
    /// use optionstratlib::model::Position;
    ///
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
    /// let position = Position::new(
    ///     option,
    ///     pos!(5.25),  // premium per contract
    ///     Utc::now(),  // position open date
    ///     pos!(0.65),  // opening fee
    ///     pos!(0.65),  // closing fee
    /// );
    /// let current_price = pos!(105.0);
    ///
    /// // Calculate PnL at expiration with specified price
    /// let pnl_specific = position.pnl_at_expiration(&Some(&current_price)).unwrap();
    ///
    /// // Calculate PnL at expiration using the option's current underlying price
    /// let pnl_current = position.pnl_at_expiration(&None).unwrap();
    /// ```
    pub fn pnl_at_expiration(&self, price: &Option<&Positive>) -> Result<Decimal, Box<dyn Error>> {
        match price {
            None => Ok(self.option.intrinsic_value(self.option.underlying_price)?
                - self.total_cost()?
                + self.premium_received()?),
            Some(price) => Ok(self.option.intrinsic_value(**price)? - self.total_cost()?
                + self.premium_received()?),
        }
    }

    /// Calculates the unrealized profit and loss (PnL) for an options position at a given price.
    ///
    /// This method computes the current theoretical profit or loss of the position if it were
    /// to be closed at the specified price, taking into account the premium paid/received and
    /// all transaction fees (both opening and closing fees).
    ///
    /// The calculation differs based on the position side:
    /// - For long positions: (current_price - premium - open_fee - close_fee) * quantity
    /// - For short positions: (premium - current_price - open_fee - close_fee) * quantity
    ///
    /// # Parameters
    ///
    /// * `price` - A `Positive` value representing the current price of the option
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, PositionError>` - The calculated unrealized PnL as a `Decimal` if successful,
    ///   or a `PositionError` if the calculation fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use chrono::Utc;
    /// use tracing::info;
    /// use optionstratlib::model::Position;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use optionstratlib::{pos, OptionStyle, Side};
    /// let current_price = pos!(6.50);
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
    /// let position = Position::new(
    ///     option,
    ///     pos!(5.25),  // premium per contract
    ///     Utc::now(),  // position open date
    ///     pos!(0.65),  // opening fee
    ///     pos!(0.65),  // closing fee
    /// );
    /// let unrealized_pnl = position.unrealized_pnl(current_price).unwrap();
    /// info!("Current unrealized PnL: {}", unrealized_pnl);
    /// ```
    pub fn unrealized_pnl(&self, price: Positive) -> Result<Decimal, PositionError> {
        match self.option.side {
            Side::Long => Ok((price.to_dec()
                - self.premium.to_dec()
                - self.open_fee.to_dec()
                - self.close_fee.to_dec())
                * self.option.quantity),
            Side::Short => Ok((self.premium.to_dec()
                - price.to_dec()
                - self.open_fee.to_dec()
                - self.close_fee.to_dec())
                * self.option.quantity),
        }
    }

    /// Calculates the number of days the position has been held.
    ///
    /// This method computes the difference between the current UTC date and the
    /// position's opening date, returning the result as a `Positive` value.
    ///
    /// The calculation uses Chrono's `num_days` method to determine the precise
    /// number of whole days between the position's date and current time.
    ///
    /// # Returns
    ///
    /// * `Ok(Positive)` - The number of days the position has been held as a positive value
    /// * `Err(PositionError)` - If there's an error during the calculation or validation
    ///
    pub fn days_held(&self) -> Result<Positive, PositionError> {
        Ok(pos!((Utc::now() - self.date).num_days() as f64))
    }

    /// Calculates the number of days remaining until the option expires.
    ///
    /// This function determines the time to expiration in days based on the option's
    /// expiration date format. It handles both explicit day counts and datetime-based
    /// expiration dates.
    ///
    /// # Returns
    ///
    /// - `Ok(Positive)` - The number of days to expiration as a positive value
    /// - `Err(PositionError)` - If the calculation fails due to issues with the position data
    ///
    /// For datetime-based expirations, the function calculates the difference between
    /// the expiration date and the current date, converting the result to days.
    pub fn days_to_expiration(&self) -> Result<Positive, PositionError> {
        match self.option.expiration_date {
            ExpirationDate::Days(days) => Ok(days),
            ExpirationDate::DateTime(datetime) => Ok(pos!(
                datetime.signed_duration_since(Utc::now()).num_days() as f64
            )),
        }
    }

    /// Determines if the position is a long position.
    ///
    /// This method checks the side attribute of the option to determine the directionality
    /// of the position. Long positions profit when the underlying asset's price increases.
    ///
    /// # Returns
    ///
    /// * `true` if the position is long
    /// * `false` if the position is short
    ///
    pub fn is_long(&self) -> bool {
        match self.option.side {
            Side::Long => true,
            Side::Short => false,
        }
    }

    /// Determines if the position is a short position.
    ///
    /// This method checks the side attribute of the option to determine the directionality
    /// of the position. Short positions profit when the underlying asset's price decreases.
    ///
    /// # Returns
    ///
    /// * `true` if the position is short
    /// * `false` if the position is long
    ///
    pub fn is_short(&self) -> bool {
        match self.option.side {
            Side::Long => false,
            Side::Short => true,
        }
    }

    /// Calculates the net cost of the position based on the option's side and fees.
    ///
    /// This method calculates the net cost of a position by determining whether the position
    /// is long or short and then computing the respective costs:
    ///
    /// - For a long position, the net cost is equivalent to the `total_cost()` of the position.
    /// - For a short position, the net cost is calculated by subtracting the premium from the
    ///   sum of the open and close fees, and then multiplying the result by the option's quantity.
    ///
    /// # Returns
    ///
    /// A `Decimal` representing the net cost of the position.
    /// The value should be positive but if the fee is higher than the premium it will be negative
    /// in short positions
    pub fn net_cost(&self) -> Result<Decimal, PositionError> {
        match self.option.side {
            Side::Long => Ok(self.total_cost()?.to_dec()),
            Side::Short => {
                let fees = self.fees()?.to_dec();
                let premium = self.premium_received()?.to_dec();
                Ok(fees - premium)
            }
        }
    }

    /// Calculates the break-even price for an options position.
    ///
    /// This method determines the price of the underlying asset at which the position
    /// will neither make a profit nor a loss. The calculation varies based on both the
    /// side of the position (Long/Short) and the option style (Call/Put).
    ///
    /// The break-even price is an important reference point for options traders as it
    /// represents the threshold price that the underlying asset must cross for the
    /// position to become profitable, accounting for all costs associated with the position.
    ///
    /// # Formula by position type:
    /// - Long Call: Strike Price + Total Cost per Contract
    /// - Short Call: Strike Price + Premium - Total Cost per Contract
    /// - Long Put: Strike Price - Total Cost per Contract
    /// - Short Put: Strike Price - Premium + Total Cost per Contract
    ///
    /// # Returns
    ///
    /// - `Some(Positive)` containing the break-even price if the position has non-zero quantity
    /// - `None` if the position has zero quantity (no contracts)
    ///
    pub fn break_even(&self) -> Option<Positive> {
        if self.option.quantity == Positive::ZERO {
            return None;
        }
        let total_cost_per_contract = self.total_cost().unwrap() / self.option.quantity;
        match (&self.option.side, &self.option.option_style) {
            (Side::Long, OptionStyle::Call) => {
                Some(self.option.strike_price + total_cost_per_contract)
            }
            (Side::Short, OptionStyle::Call) => {
                Some(self.option.strike_price + self.premium - total_cost_per_contract)
            }
            (Side::Long, OptionStyle::Put) => {
                Some(self.option.strike_price - total_cost_per_contract)
            }
            (Side::Short, OptionStyle::Put) => {
                Some(self.option.strike_price - self.premium + total_cost_per_contract)
            }
        }
    }

    /// Calculates the maximum potential profit for an options position.
    ///
    /// This method determines the maximum possible profit based on the position's side:
    /// - For long positions (buying options), the profit potential is theoretically unlimited,
    ///   as the underlying asset could increase in value indefinitely.
    /// - For short positions (selling options), the maximum profit is capped at the net premium
    ///   received after accounting for all costs.
    ///
    /// # Returns
    ///
    /// - `Ok(Positive)` - The maximum potential profit as a positive value
    /// - `Err(PositionError)` - If there's an issue calculating the maximum profit
    #[allow(dead_code)]
    pub(crate) fn max_profit(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::INFINITY),
            Side::Short => self.net_premium_received(),
        }
    }

    /// Calculates the maximum potential loss for an options position.
    ///
    /// This method determines the maximum possible loss based on the position's side:
    /// - For long positions (buying options), the maximum loss is limited to the total cost
    ///   of entering the position (premium paid plus all fees).
    /// - For short positions (selling options), the loss potential is theoretically unlimited,
    ///   as the underlying asset could increase in value indefinitely.
    ///
    /// # Returns
    ///
    /// - `Ok(Positive)` - The maximum potential loss as a positive value
    /// - `Err(PositionError)` - If there's an issue calculating the maximum loss
    #[allow(dead_code)]
    pub(crate) fn max_loss(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => self.total_cost(),
            Side::Short => Ok(Positive::INFINITY),
        }
    }

    /// Calculates the total transaction fees for the position.
    ///
    /// This method computes the sum of opening and closing fees for the position,
    /// scaled by the quantity of options contracts. These fees typically include
    /// broker commissions, exchange fees, and other transaction costs.
    ///
    /// # Returns
    ///
    /// - `Ok(Positive)` - The total fees as a positive value
    /// - `Err(PositionError)` - If there's an issue calculating the fees
    pub fn fees(&self) -> Result<Positive, PositionError> {
        Ok((self.open_fee + self.close_fee) * self.option.quantity)
    }

    /// Validates the position to ensure it meets all necessary conditions for trading.
    ///
    /// This method performs a series of checks to determine if the position is valid:
    /// 1. For short positions, verifies that:
    ///    - Premium is greater than zero
    ///    - Premium exceeds the sum of opening and closing fees
    /// 2. Validates the underlying option parameters
    ///
    /// # Returns
    ///
    /// * `true` if the position is valid and meets all conditions
    /// * `false` otherwise, with specific failure reasons logged via debug messages
    ///
    /// # Examples
    ///
    /// ```rust
    /// use optionstratlib::model::{Position, Options};
    /// use optionstratlib::{pos, Side, OptionStyle};
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use chrono::Utc;
    ///
    /// // Create a valid position
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
    /// let position = Position::new(
    ///     option,
    ///     pos!(5.25),
    ///     Utc::now(),
    ///     pos!(0.65),
    ///     pos!(0.65)
    /// );
    ///
    /// assert!(position.validate());
    /// ```
    pub fn validate(&self) -> bool {
        if self.option.side == Side::Short && self.premium == Positive::ZERO {
            debug!("Premium must be greater than zero for short positions.");
            return false;
        }
        if !self.option.validate() {
            debug!("Option is not valid.");
            return false;
        }
        true
    }
}

impl Default for Position {
    fn default() -> Self {
        Position {
            option: Options::default(),
            premium: Positive::ZERO,
            date: Utc::now(),
            open_fee: Positive::ZERO,
            close_fee: Positive::ZERO,
        }
    }
}

/// Implementation of the `Greeks` trait for the `Position` struct.
///
/// This implementation allows a `Position` to calculate option Greeks (delta, gamma,
/// theta, vega, rho, etc.) by accessing its underlying option contract. The implementation
/// provides a way to expose the position's option for use in Greek calculations.
///
impl Greeks for Position {
    /// Returns a vector containing a reference to the option contract associated with this position.
    ///
    /// This method satisfies the `Greeks` trait requirement by providing access to the
    /// option contract that will be used for calculating various Greek values.
    ///
    /// # Returns
    ///
    /// - `Ok(Vec<&Options>)` - A vector containing a reference to the position's underlying option
    /// - `Err(GreeksError)` - If there is an error accessing the option data
    fn get_options(&self) -> Result<Vec<&Options>, GreeksError> {
        Ok(vec![&self.option])
    }
}

impl TransactionAble for Position {
    fn add_transaction(&mut self, _transaction: Transaction) -> Result<(), TransactionError> {
        todo!()
    }

    fn get_transactions(&self) -> Result<Vec<Transaction>, TransactionError> {
        todo!()
    }
}

impl TradeAble for Position {
    fn trade(&self) -> Trade {
        Trade {
            id: uuid::Uuid::new_v4(),
            action: Action::Buy,
            side: self.option.side,
            option_style: self.option.option_style,
            fee: self.open_fee + self.close_fee,
            symbol: None,
            strike: self.option.strike_price,
            expiry: self.option.expiration_date.get_date().unwrap(),
            timestamp: Utc::now().timestamp_nanos_opt().unwrap(),
            quantity: self.option.quantity,
            premium: self.premium,
            underlying_price: self.option.underlying_price,
            notes: None,
            status: TradeStatus::Other("Not yet initialized".to_string()),
        }
    }

    fn trade_ref(&self) -> &Trade {
        unimplemented!()
    }

    fn trade_mut(&mut self) -> &mut Trade {
        unimplemented!()
    }
}

impl TradeStatusAble for Position {
    fn open(&self) -> Trade {
        let mut trade = self.trade();
        trade.status = TradeStatus::Open;
        trade
    }

    fn close(&self) -> Trade {
        let mut trade = self.trade();
        if trade.premium <= pos!(0.01) {
            trade.premium = Positive::ZERO;
        }
        trade.status = TradeStatus::Closed;
        trade.action = Action::Sell;
        trade
    }

    fn expired(&self) -> Trade {
        let mut trade = self.trade();
        trade.status = TradeStatus::Expired;
        trade.action = Action::Sell;
        trade
    }

    fn exercised(&self) -> Trade {
        let mut trade = self.trade();
        trade.status = TradeStatus::Exercised;
        trade.action = Action::Sell;
        trade
    }

    fn assigned(&self) -> Trade {
        let mut trade = self.trade();
        trade.status = TradeStatus::Assigned;
        trade.action = Action::Other;
        trade
    }

    fn status_other(&self) -> Trade {
        let mut trade = self.trade();
        trade.status = TradeStatus::Other("Not yet initialized".to_string());
        trade.action = Action::Other;
        trade
    }
}

/// # Position Profit and Loss (PnL) Calculator
///
/// This trait implementation provides methods to calculate the profit and loss (PnL)
/// for option positions under different market scenarios.
///
/// The implementation offers two main calculations:
/// 1. Current PnL based on updated market conditions
/// 2. PnL at expiration based on a projected underlying price
///
/// These calculations are essential for risk management, position monitoring, and
/// strategy planning in options trading.
impl PnLCalculator for Position {
    /// Calculates the current unrealized profit and loss for an option position
    /// based on updated market conditions.
    ///
    /// This method computes the difference between the option's price at entry and its
    /// current theoretical price using the Black-Scholes model. It factors in changes to:
    /// - The underlying asset price
    /// - Time to expiration
    /// - Implied volatility
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The current price of the underlying asset
    /// * `expiration_date` - The updated expiration date for the calculation
    /// * `implied_volatility` - The current implied volatility of the option
    ///
    /// # Returns
    ///
    /// * `Result<PnL, Box<dyn Error>>` - A PnL object containing unrealized profit/loss and position cost details,
    ///   or an error if the calculation fails
    fn calculate_pnl(
        &self,
        underlying_price: &Positive,
        expiration_date: ExpirationDate,
        implied_volatility: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        let price_at_buy = self.option.calculate_price_black_scholes()?;
        let mut current_option = self.option.clone();
        current_option.expiration_date = expiration_date;
        current_option.underlying_price = *underlying_price;
        current_option.implied_volatility = *implied_volatility;
        let price_at_sell = current_option.calculate_price_black_scholes()?;
        let unrealized = price_at_sell - price_at_buy;
        let initial_cost = self.total_cost()?;
        let initial_income = self.premium_received()?;

        let realized = initial_income.to_dec() - initial_cost.to_dec();
        Ok(PnL::new(
            Some(realized),
            Some(unrealized),
            initial_cost,
            initial_income,
            self.date,
        ))
    }

    /// Calculates the expected profit and loss at option expiration for a given
    /// underlying price.
    ///
    /// This method determines the realized profit or loss that would occur if the option
    /// expires with the underlying at the specified price. It uses intrinsic value calculation
    /// at expiration rather than Black-Scholes pricing.
    ///
    /// # Arguments
    ///
    /// * `underlying_price` - The projected price of the underlying asset at expiration
    ///
    /// # Returns
    ///
    /// * `Result<PnL, Box<dyn Error>>` - A PnL object containing realized profit/loss and position cost details,
    ///   or an error if the calculation fails
    fn calculate_pnl_at_expiration(
        &self,
        underlying_price: &Positive,
    ) -> Result<PnL, Box<dyn Error>> {
        let initial_cost = self.total_cost()?;
        let initial_income = self.premium_received()?;
        let date_time = self.option.expiration_date.get_date()?;

        let realized = self.option.intrinsic_value(*underlying_price)? - initial_cost.to_dec()
            + initial_income.to_dec();
        Ok(PnL::new(
            Some(realized),
            Some(Decimal::ZERO),
            initial_cost,
            initial_income,
            date_time,
        ))
    }
}

/// Implementation of the Profit trait for the Position struct.
///
/// This allows calculating the profit of a position at a given price by using the
/// position's profit and loss (PnL) calculation at expiration.
///
impl Profit for Position {
    /// Calculates the profit of the position at a specific price.
    ///
    /// This method computes the profit or loss that would be realized if the position
    /// were to expire with the underlying asset at the specified price.
    ///
    /// # Parameters
    ///
    /// * `price` - The price at which to calculate the profit, represented as a Positive value.
    ///
    /// # Returns
    ///
    /// * `Result<Decimal, Box<dyn Error>>` - The calculated profit as a Decimal if successful,
    ///   or an error if the calculation fails.
    fn calculate_profit_at(&self, price: Positive) -> Result<Decimal, Box<dyn Error>> {
        self.pnl_at_expiration(&Some(&price))
    }
}

/// Implementation of the `Graph` trait for the `Position` struct, enabling graphical representation
/// of financial options positions.
///
/// This implementation provides methods to visualize the profit/loss (PnL) profile of an options position
/// across different price levels of the underlying asset. It handles the generation of appropriate title,
/// data values for plotting, and special chart elements like break-even points.
///
/// The visualization capabilities allow traders to analyze the potential outcomes of their options positions
/// at expiration across various price scenarios.
impl Graph for Position {
    /// Generates a title for the graph based on the option's characteristics.
    ///
    /// # Returns
    /// A `String` containing the formatted title that describes the position.
    fn title(&self) -> String {
        self.option.title()
    }

    /// Generates a vector of evenly spaced x-values for option pricing/plotting.
    ///
    /// This method creates a range of x-values (potential stock prices) centered around
    /// the strike price and spanning 5 standard deviations in each direction.
    /// The standard deviation is calculated as the product of strike price and implied volatility.
    ///
    /// # Returns
    ///
    /// A vector of `Positive` values representing potential stock prices, with 1000 total points
    /// (999 steps plus endpoints) evenly distributed across the range.
    ///
    /// # Implementation Details
    ///
    /// * The range extends 5 standard deviations above and below the strike price
    /// * Uses 1000 total points (steps + 1) for smooth visualization
    /// * All returned values are guaranteed positive through the use of the `pos!` macro
    ///
    fn get_x_values(&self) -> Vec<Positive> {
        self.option.get_x_values()
    }

    /// Calculates position profit/loss values at expiration for a range of underlying prices.
    ///
    /// This method transforms a slice of potential underlying prices into their corresponding
    /// profit/loss values at expiration for this position.
    ///
    /// # Parameters
    /// * `data` - A slice of `Positive` values representing potential prices of the underlying asset
    ///
    /// # Returns
    /// A `Vec<f64>` containing the calculated profit/loss values for each input price
    fn get_y_values(&self) -> Vec<f64> {
        let data = self.get_x_values();
        data.iter()
            .map(|&price| {
                self.pnl_at_expiration(&Some(&price))
                    .unwrap()
                    .to_f64()
                    .unwrap()
            })
            .collect()
    }

    /// Generates vertical lines for the graph to highlight significant price levels.
    ///
    /// This method creates vertical line indicators for important price points in the position analysis,
    /// specifically the break-even price level where the position transitions between profit and loss.
    ///
    /// # Returns
    /// A `Vec<ChartVerticalLine<f64, f64>>` containing vertical line definitions to be displayed on the chart
    fn get_vertical_lines(&self) -> Vec<ChartVerticalLine<f64, f64>> {
        match self.break_even() {
            Some(break_even) => {
                let vertical_lines = vec![ChartVerticalLine {
                    x_coordinate: break_even.into(),
                    y_range: (-50000.0, 50000.0),
                    label: "Break Even".to_string(),
                    label_offset: (5.0, 5.0),
                    line_color: BLACK,
                    label_color: BLACK,
                    line_style: ShapeStyle::from(&BLACK).stroke_width(1),
                    font_size: 18,
                }];
                vertical_lines
            }
            None => vec![],
        }
    }
}

#[cfg(test)]
mod tests_position {
    use super::*;
    use crate::constants::ZERO;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use chrono::Duration;
    use rust_decimal_macros::dec;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
        expiration_days: Positive,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days),
            implied_volatility: pos!(0.2),
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.01),
            option_style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]

    fn test_position_total_cost() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            7.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]

    fn test_position_total_cost_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            70.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]

    fn test_position_total_cost_short() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            2.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]

    fn test_position_total_cost_short_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.total_cost().unwrap(),
            20.0,
            "Total cost calculation is incorrect."
        );
    }

    #[test]

    fn test_position_check_negative_premium() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap(),
            dec!(3.0),
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_long_call_itm() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap(),
            dec!(3.0),
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_long_call_itm_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap(),
            dec!(30.0),
            "PNL at expiration for long call ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_short_call_itm() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -7.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_short_call_itm_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -70.0,
            "PNL at expiration for short call ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_long_put_itm() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            3.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_long_put_itm_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            30.0,
            "PNL at expiration for long put ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_short_put_itm() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -7.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_short_put_itm_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(90.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            -70.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_short_put_itm_winning() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(110.0),
            pos!(1.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            3.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]

    fn test_position_pnl_at_expiration_short_put_itm_quantity_winning() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(110.0),
            pos!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&None).unwrap().to_f64().unwrap(),
            30.0,
            "PNL at expiration for short put ITM is incorrect."
        );
    }

    #[test]

    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.pnl_at_expiration(&Some(&pos!(107.0))).unwrap(),
            Positive::ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]

    fn test_unrealized_pnl_long_call_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position
                .pnl_at_expiration(&Some(&pos!(107.0)))
                .unwrap()
                .to_f64()
                .unwrap(),
            ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]

    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position
                .unrealized_pnl(pos!(3.0))
                .unwrap()
                .to_f64()
                .unwrap(),
            ZERO,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]

    fn test_unrealized_pnl_short_call_bis() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position
                .unrealized_pnl(pos!(10.0))
                .unwrap()
                .to_f64()
                .unwrap(),
            -7.0,
            "Unrealized PNL for short call is incorrect."
        );
    }

    #[test]

    fn test_days_held() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let date = Utc::now() - Duration::days(10);
        let position = Position::new(option, pos!(5.0), date, Positive::ONE, Positive::ONE);
        assert_eq!(
            position.days_held().unwrap().to_f64(),
            10.0,
            "Days held calculation is incorrect."
        );
    }

    #[test]

    fn test_days_to_expiration() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(
            position.days_to_expiration().unwrap().to_f64(),
            30.0,
            "Days to expiration calculation is incorrect."
        );
    }

    #[test]

    fn test_is_long_position() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert!(
            position.is_long(),
            "is_long should return true for long positions."
        );
        assert!(
            !position.is_short(),
            "is_short should return false for long positions."
        );
    }

    #[test]

    fn test_is_short_position() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert!(
            position.is_short(),
            "is_short should return true for short positions."
        );
        assert!(
            !position.is_long(),
            "is_long should return false for short positions."
        );
    }
}

#[cfg(test)]
mod tests_valid_position {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;

    #[test]

    fn test_valid_position() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        assert!(position.validate());
    }

    #[test]

    fn test_zero_premium() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        position.premium = Positive::ZERO;
        assert!(!position.validate());
    }

    #[test]

    fn test_invalid_option() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        position.option.strike_price = Positive::ZERO; // This makes the option invalid
        assert!(!position.validate());
    }

    #[test]

    fn test_zero_fees() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        position.open_fee = Positive::ZERO;
        position.close_fee = Positive::ZERO;
        assert!(position.validate());
    }
}

#[cfg(test)]
mod tests_position_break_even {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use rust_decimal_macros::dec;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
        expiration_days: Positive,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days),
            implied_volatility: pos!(0.2),
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.01),
            option_style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]

    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]

    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]

    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]

    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]

    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]

    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]

    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 97.0);
    }

    #[test]

    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.break_even().unwrap(), 97.0);
    }
}

#[cfg(test)]
mod tests_position_max_loss_profit {
    use super::*;
    use crate::model::types::{OptionStyle, OptionType, Side};
    use crate::pos;
    use approx::assert_relative_eq;
    use rust_decimal_macros::dec;

    fn setup_option(
        side: Side,
        option_style: OptionStyle,
        strike_price: Positive,
        underlying_price: Positive,
        quantity: Positive,
        expiration_days: Positive,
    ) -> Options {
        Options {
            option_type: OptionType::European,
            side,
            underlying_symbol: "".to_string(),
            strike_price,
            expiration_date: ExpirationDate::Days(expiration_days),
            implied_volatility: pos!(0.2),
            quantity,
            underlying_price,
            risk_free_rate: dec!(0.01),
            option_style,
            dividend_yield: Positive::ZERO,
            exotic_params: None,
        }
    }

    #[test]

    fn test_unrealized_pnl_long_call() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 7.0, epsilon = 0.001);
        assert_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]

    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 70.0, epsilon = 0.001);
        assert_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]

    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            3.0,
            epsilon = 0.001
        );
    }

    #[test]

    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            30.0,
            epsilon = 0.001
        );
    }

    #[test]

    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]

    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit().unwrap(), Positive::INFINITY);
    }

    #[test]

    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(1.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            3.0,
            epsilon = 0.001
        );
    }

    #[test]

    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            pos!(100.0),
            pos!(105.0),
            pos!(10.0),
            pos!(30.0),
        );
        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_relative_eq!(position.max_loss().unwrap(), Positive::INFINITY);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            30.0,
            epsilon = 0.001
        );
    }
}

#[cfg(test)]
mod tests_update_from_option_data {
    use super::*;
    use crate::{pos, spos};
    use rust_decimal_macros::dec;

    fn create_test_option_data() -> OptionData {
        OptionData::new(
            pos!(110.0),
            spos!(9.5),
            spos!(10.0),
            spos!(8.5),
            spos!(9.0),
            spos!(0.25),
            Some(dec!(-0.3)),
            Some(dec!(0.3)),
            Some(dec!(0.3)),
            None,
            None,
        )
    }

    #[test]

    fn test_update_long_call() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.option.strike_price, pos!(110.0));
        assert_eq!(position.option.implied_volatility, 0.25);
        assert_eq!(position.premium, 10.0); // call_ask
    }

    #[test]

    fn test_update_short_call() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Call;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.5); // call_bid
    }

    #[test]

    fn test_update_long_put() {
        let mut position = Position::default();
        position.option.side = Side::Long;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 9.0); // put_ask
    }

    #[test]

    fn test_update_short_put() {
        let mut position = Position::default();
        position.option.side = Side::Short;
        position.option.option_style = OptionStyle::Put;

        let option_data = create_test_option_data();
        position.update_from_option_data(&option_data);

        assert_eq!(position.premium, 8.5); // put_bid
    }
}

#[cfg(test)]
mod tests_premium {
    use super::*;
    use crate::pos;

    fn setup_basic_position(side: Side) -> Position {
        let option = Options {
            side,
            quantity: pos!(1.0),
            ..Default::default()
        };

        Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE)
    }

    #[test]

    fn test_premium_received_long() {
        let position = setup_basic_position(Side::Long);
        assert_eq!(position.premium_received().unwrap(), Positive::ZERO);
    }

    #[test]

    fn test_premium_received_short() {
        let position = setup_basic_position(Side::Short);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }

    #[test]

    fn test_net_premium_received_long() {
        let position = setup_basic_position(Side::Long);
        assert_eq!(position.net_premium_received().unwrap(), 0.0);
    }

    #[test]

    fn test_net_premium_received_short() {
        let position = setup_basic_position(Side::Short);
        assert_eq!(position.net_premium_received().unwrap(), 3.0); // 5.0 - 2.0 (fees)
    }

    #[test]

    fn test_premium_received_with_quantity() {
        let side = Side::Short;
        let option = Options {
            side,
            quantity: pos!(10.0),
            ..Default::default()
        };

        let position = Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE);
        assert_eq!(position.premium_received().unwrap(), 50.0);
    }
}

#[cfg(test)]
mod tests_pnl_calculator {
    use super::*;
    use crate::{OptionType, assert_decimal_eq, pos};
    use rust_decimal_macros::dec;

    fn setup_test_position(side: Side, option_style: OptionStyle) -> Position {
        let option = Options::new(
            OptionType::European,
            side,
            "AAPL".to_string(),
            pos!(100.0),
            ExpirationDate::Days(pos!(30.0)),
            pos!(0.2),
            Positive::ONE,
            pos!(100.0),
            dec!(0.05),
            option_style,
            pos!(0.0),
            None,
        );

        Position::new(option, pos!(5.0), Utc::now(), Positive::ONE, Positive::ONE)
    }

    #[test]

    fn test_calculate_pnl_long_call_no_changes() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_eq!(pnl.unrealized.unwrap(), Decimal::ZERO); // 5.0 - 2.4933 - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 7.0);
        assert_eq!(position.premium_received().unwrap(), 0.0);
    }

    #[test]

    fn test_calculate_pnl_long_call_price_up() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(107.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(5.2150), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_long_call_vol_down() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.1))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.1352), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_long_call_date_closer() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(3.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.7494), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_short_call_no_changes() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_eq!(pnl.unrealized.unwrap(), Decimal::ZERO); // 5.0 - 2.4933 - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 2.0);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }

    #[test]

    fn test_calculate_pnl_short_call_price_up() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(107.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-5.2150), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_short_call_price_down() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(97.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.3069), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_short_call_vol_down() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.1))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.1352), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_short_call_vol_up() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(30.0)), &pos!(0.3))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-1.1386), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_short_call_date_closer() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(3.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(1.7494), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_short_call_date_further() {
        let position = setup_test_position(Side::Short, OptionStyle::Call);
        let pnl = position
            .calculate_pnl(&pos!(100.0), ExpirationDate::Days(pos!(40.0)), &pos!(0.2))
            .unwrap();

        assert_decimal_eq!(pnl.unrealized.unwrap(), dec!(-0.4224), dec!(0.0001));
    }

    #[test]

    fn test_calculate_pnl_at_expiration_long_call() {
        let position = setup_test_position(Side::Long, OptionStyle::Call);
        let pnl = position.calculate_pnl_at_expiration(&pos!(110.0)).unwrap();

        assert_eq!(pnl.realized.unwrap(), dec!(3.0)); // 10.0 - 7.0 (total cost)
        assert_eq!(position.total_cost().unwrap(), 7.0);
        assert_eq!(position.premium_received().unwrap(), 0.0);
    }

    #[test]

    fn test_calculate_pnl_at_expiration_short_put() {
        let position = setup_test_position(Side::Short, OptionStyle::Put);
        let pnl = position.calculate_pnl_at_expiration(&pos!(90.0)).unwrap();

        assert_eq!(pnl.realized.unwrap(), dec!(-7.0)); // -10.0 + 5.0 (premium) - 2.0 (fees)
        assert_eq!(position.total_cost().unwrap(), 2.0);
        assert_eq!(position.premium_received().unwrap(), 5.0);
    }
}

#[cfg(test)]
mod tests_graph {
    use super::*;

    #[test]

    fn test_title() {
        let position = Position::default();
        assert_eq!(position.title(), position.option.title());
    }

    #[test]

    fn test_get_values() {
        let position = Position::default();
        let values = position.get_y_values();
        assert_eq!(values.len(), 1000);
        assert!(!values.iter().any(|&x| x.is_nan()));
    }

    #[test]

    fn test_get_vertical_lines() {
        let position = Position::default();
        let lines = position.get_vertical_lines();
        assert_eq!(lines.len(), 0);
    }
}

#[cfg(test)]
mod tests_position_serde {
    use super::*;
    use crate::model::utils::create_sample_position;
    use crate::pos;
    use serde_json;
    use tracing::info;

    #[test]
    fn test_position_serialization() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string(&position).unwrap();

        // Verify the serialized string contains expected fields
        assert!(serialized.contains("\"option\""));
        assert!(serialized.contains("\"premium\""));
        assert!(serialized.contains("\"date\""));
        assert!(serialized.contains("\"open_fee\""));
        assert!(serialized.contains("\"close_fee\""));
        assert!(serialized.contains("AAPL"));
        assert!(serialized.contains("95"));
    }

    #[test]
    fn test_position_deserialization() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string(&position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(position, deserialized);
        assert_eq!(deserialized.option.underlying_symbol, "AAPL");
        assert_eq!(deserialized.option.strike_price, pos!(95.0));
        assert_eq!(deserialized.premium, pos!(5.0));
        assert_eq!(deserialized.open_fee, pos!(0.5));
        assert_eq!(deserialized.close_fee, pos!(0.5));
    }

    #[test]
    fn test_position_json_structure() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string_pretty(&position).unwrap();

        info!("Serialized Position:\n{}", serialized);

        let value: serde_json::Value = serde_json::from_str(&serialized).unwrap();

        // Test structure
        assert!(value.is_object());
        assert!(value.get("option").is_some());
        assert!(value.get("premium").is_some());
        assert!(value.get("date").is_some());
        assert!(value.get("open_fee").is_some());
        assert!(value.get("close_fee").is_some());
    }

    #[test]
    fn test_position_deserialize_invalid_json() {
        let invalid_json = r#"{
            "option": null,
            "premium": 5.0,
            "date": "2024-01-01T00:00:00Z",
            "open_fee": 1.0,
            "close_fee": 1.0
        }"#;

        let result: Result<Position, serde_json::Error> = serde_json::from_str(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_position_roundtrip() {
        let original = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();
        let reserialized = serde_json::to_string(&deserialized).unwrap();

        assert_eq!(serialized, reserialized);
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_position_with_different_option_types() {
        // Test with a Put option
        let put_position = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );

        let serialized = serde_json::to_string(&put_position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(put_position, deserialized);
        assert_eq!(deserialized.option.option_style, OptionStyle::Put);

        // Test with a Short position
        let short_position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos!(90.0),
            pos!(1.0),
            pos!(95.0),
            pos!(0.2),
        );

        let serialized = serde_json::to_string(&short_position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(short_position, deserialized);
        assert_eq!(deserialized.option.side, Side::Short);
    }
}
