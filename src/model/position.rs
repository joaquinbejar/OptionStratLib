/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 18/8/24
******************************************************************************/

use crate::error::position::PositionValidationErrorKind;
use crate::error::{PositionError, PricingError, TradeError};
use crate::model::decimal::{d_add, d_mul, d_sub};
use crate::model::expiration::resolve_expiration_date;
use crate::model::trade::TradeStatusAble;
use crate::model::types::{Action, OptionStyle, Side};
use crate::model::{Trade, TradeAble, TradeStatus};
use crate::{ExpirationDate, Options};
use chrono::{DateTime, Utc};
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use tracing::debug;
use utoipa::ToSchema;

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
/// # fn main() -> Result<(), optionstratlib::error::Error> {
/// use optionstratlib::{Options, Side, OptionStyle};
/// use positive::pos_or_panic;
/// use chrono::Utc;
/// use tracing::info;
/// use optionstratlib::model::Position;
/// use optionstratlib::model::utils::create_sample_option_simplest;
///
/// let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
/// let position = Position::new(
///     option,
///     pos_or_panic!(5.25),           // premium per contract
///     Utc::now(),           // position open date
///     pos_or_panic!(0.65),           // opening fee per contract
///     pos_or_panic!(0.65),           // closing fee per contract
///     None,
///     None,
/// );
///
/// let total_cost = position.total_cost()?;
/// info!("Total position cost: {}", total_cost);
/// # Ok(())
/// # }
/// ```
#[derive(Clone, PartialEq, Serialize, Deserialize, ToSchema)]
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

    /// Identifier for the position in an external system or platform
    pub epic: Option<String>,

    /// Additional custom data fields for the position stored as JSON
    pub extra_fields: Option<serde_json::Value>,
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
    /// use optionstratlib::{Options, Side, OptionStyle};
    /// use positive::pos_or_panic;
    /// use chrono::Utc;
    /// use optionstratlib::model::Position;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    ///
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
    /// let position = Position::new(
    ///     option,
    ///     pos_or_panic!(5.25),           // premium per contract
    ///     Utc::now(),           // position open date
    ///     pos_or_panic!(0.65),           // opening fee per contract
    ///     pos_or_panic!(0.65),           // closing fee per contract
    ///    None,                  // epic (optional)
    ///   None,                  // extra fields (optional)
    /// );
    /// ```
    #[must_use]
    pub fn new(
        option: Options,
        premium: Positive,
        date: DateTime<Utc>,
        open_fee: Positive,
        close_fee: Positive,
        epic: Option<String>,
        extra_fields: Option<serde_json::Value>,
    ) -> Self {
        Position {
            option,
            premium,
            date,
            open_fee,
            close_fee,
            epic,
            extra_fields,
        }
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
    /// A `Positive` representing the total cost of the position. THE VALUE IS ALWAYS POSITIVE
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PositiveError`] when accumulating the premium
    /// and the two fees, or scaling the accumulated per-contract cost by the
    /// contract quantity, overflows the `Positive` range. The raw `Positive`
    /// operators abort on overflow, so every step is taken through its
    /// checked counterpart.
    pub fn total_cost(&self) -> Result<Positive, PositionError> {
        let total_cost = match self.option.side {
            Side::Long => self
                .premium
                .checked_add(&self.open_fee)?
                .checked_add(&self.close_fee)?
                .checked_mul(&self.option.quantity)?,
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
    /// # fn main() -> Result<(), optionstratlib::error::Error> {
    /// use optionstratlib::{ Side, OptionStyle};
    /// use positive::pos_or_panic;
    /// use optionstratlib::model::Position;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use chrono::Utc;
    /// use tracing::info;
    ///
    /// // Create a short position
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
    /// let position = Position::new(
    ///     option,
    ///     pos_or_panic!(5.25),  // premium per contract
    ///     Utc::now(),  // position open date
    ///     pos_or_panic!(0.65),  // opening fee
    ///     pos_or_panic!(0.65),  // closing fee
    ///    None,        // epic (optional)
    ///   None,        // extra fields (optional)
    /// );
    ///
    /// // Calculate premium received
    /// let received = position.premium_received()?;
    /// info!("Premium received: {}", received);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PositiveError`] when the short-side
    /// `premium × quantity` product overflows the `Positive` range. Long
    /// positions receive no premium and are infallible.
    pub fn premium_received(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::ZERO),
            Side::Short => Ok(self.premium.checked_mul(&self.option.quantity)?),
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
    /// # Errors
    ///
    /// Propagates any `PositionError` raised by `total_cost()`, and returns
    /// [`PositionError::PositiveError`] when scaling the premium by the
    /// contract quantity overflows the `Positive` range.
    ///
    /// When the short-side net amount `premium − total_cost` is
    /// negative the function returns `Ok(Positive::ZERO)` (clamped)
    /// rather than an error, so a guaranteed-loss short position is
    /// reported as zero net received rather than as a failure.
    pub fn net_premium_received(&self) -> Result<Positive, PositionError> {
        match self.option.side {
            Side::Long => Ok(Positive::ZERO),
            Side::Short => {
                // max profit is premium received - fees (cost)
                let premium = self.premium.checked_mul(&self.option.quantity)?;
                let total_cost = self.total_cost()?;
                if premium >= total_cost {
                    Ok(premium.checked_sub(&total_cost)?)
                } else {
                    Ok(Positive::ZERO)
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
    /// * `Result<Decimal, PricingError>` - The calculated profit or loss as a Decimal value,
    ///   or an error if the calculation fails.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # fn main() -> Result<(), optionstratlib::error::Error> {
    /// // Assuming position is a properly initialized Position
    /// use chrono::Utc;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use optionstratlib::{OptionStyle, Side};
    /// use positive::pos_or_panic;
    /// use optionstratlib::model::Position;
    ///
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
    /// let position = Position::new(
    ///     option,
    ///     pos_or_panic!(5.25),  // premium per contract
    ///     Utc::now(),  // position open date
    ///     pos_or_panic!(0.65),  // opening fee
    ///     pos_or_panic!(0.65),  // closing fee
    ///    None,        // epic (optional)
    ///   None,        // extra fields (optional)
    /// );
    /// let current_price = pos_or_panic!(105.0);
    ///
    /// // Calculate PnL at expiration with specified price
    /// let pnl_specific = position.pnl_at_expiration(&Some(&current_price))?;
    ///
    /// // Calculate PnL at expiration using the option's current underlying price
    /// let pnl_current = position.pnl_at_expiration(&None)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Propagates any `OptionsError` returned by the underlying payoff
    /// evaluation ([`Options::intrinsic_value`] or [`Options::payoff`]),
    /// wrapped as `PricingError::OptionError`.
    pub fn pnl_at_expiration(&self, price: &Option<&Positive>) -> Result<Decimal, PricingError> {
        // P&L = intrinsic_value - total_cost + premium_received.
        // All three terms are monetary and the composition surfaces the
        // user-visible P&L, so the fused arithmetic goes through `d_add` /
        // `d_sub` to trip `DecimalError::Overflow` instead of wrapping on a
        // pathological cost basis or premium.
        let intrinsic = match price {
            None => self.option.intrinsic_value(self.option.underlying_price)?,
            Some(price) => self.option.intrinsic_value(**price)?,
        };
        let cost = self.total_cost()?.to_dec();
        let premium_recv = self.premium_received()?.to_dec();
        let net_after_cost = d_sub(intrinsic, cost, "position::pnl_at_expiration::net")?;
        d_add(
            net_after_cost,
            premium_recv,
            "position::pnl_at_expiration::total",
        )
        .map_err(PricingError::from)
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
    /// # fn main() -> Result<(), optionstratlib::error::Error> {
    /// use chrono::Utc;
    /// use tracing::info;
    /// use optionstratlib::model::Position;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use optionstratlib::{ OptionStyle, Side};
    /// use positive::pos_or_panic;
    /// let current_price = pos_or_panic!(6.50);
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Short);
    /// let position = Position::new(
    ///     option,
    ///     pos_or_panic!(5.25),  // premium per contract
    ///     Utc::now(),  // position open date
    ///     pos_or_panic!(0.65),  // opening fee
    ///     pos_or_panic!(0.65),  // closing fee
    ///    None,        // epic (optional)
    ///   None,        // extra fields (optional)
    /// );
    /// let unrealized_pnl = position.unrealized_pnl(current_price)?;
    /// info!("Current unrealized PnL: {}", unrealized_pnl);
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] wrapping any
    /// [`PositionValidationErrorKind`] surfaced by the internal Black–Scholes
    /// evaluation, or `PositionError::PricingError` when the
    /// implied-volatility recomputation at `price` fails.
    pub fn unrealized_pnl(&self, price: Positive) -> Result<Decimal, PositionError> {
        // Per-contract P&L (Long: price - premium - fees; Short: premium -
        // price - fees) then scaled by the contract quantity. Each step is a
        // monetary flow, so overflow surfaces a typed error rather than
        // wrapping silently on Decimal::MIN or Decimal::MAX-class inputs.
        let per_contract = match self.option.side {
            Side::Long => {
                let after_premium = d_sub(
                    price.to_dec(),
                    self.premium.to_dec(),
                    "position::unrealized_pnl::long::after_premium",
                )?;
                let after_open = d_sub(
                    after_premium,
                    self.open_fee.to_dec(),
                    "position::unrealized_pnl::long::after_open_fee",
                )?;
                d_sub(
                    after_open,
                    self.close_fee.to_dec(),
                    "position::unrealized_pnl::long::net",
                )?
            }
            Side::Short => {
                let after_price = d_sub(
                    self.premium.to_dec(),
                    price.to_dec(),
                    "position::unrealized_pnl::short::after_price",
                )?;
                let after_open = d_sub(
                    after_price,
                    self.open_fee.to_dec(),
                    "position::unrealized_pnl::short::after_open_fee",
                )?;
                d_sub(
                    after_open,
                    self.close_fee.to_dec(),
                    "position::unrealized_pnl::short::net",
                )?
            }
        };
        Ok(d_mul(
            per_contract,
            self.option.quantity.to_dec(),
            "position::unrealized_pnl::scaled",
        )?)
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
    /// # Errors
    ///
    /// Returns [`PositionError`] wrapping a
    /// `PositionValidationErrorKind::InvalidPositionSize` if the elapsed
    /// day-count is negative (future-dated open date) or cannot be
    /// represented as a `Positive`.
    pub fn days_held(&self) -> Result<Positive, PositionError> {
        let days = (Utc::now() - self.date).num_days() as f64;
        Positive::new(days).map_err(|e| {
            PositionError::ValidationError(PositionValidationErrorKind::InvalidPosition {
                reason: format!("failed to calculate days held: {}", e),
            })
        })
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
    ///
    /// # Errors
    ///
    /// Returns [`PositionError`] wrapping the underlying
    /// [`expiration_date::error::ExpirationDateError`] when the expiration
    /// cannot be converted (e.g. a past datetime that would produce a
    /// negative day count).
    pub fn days_to_expiration(&self) -> Result<Positive, PositionError> {
        match self.option.expiration_date {
            ExpirationDate::Days(days) => Ok(days),
            ExpirationDate::DateTime(datetime) => {
                let days = datetime.signed_duration_since(Utc::now()).num_days() as f64;
                Positive::new(days.max(0.0)).map_err(|e| {
                    PositionError::ValidationError(PositionValidationErrorKind::InvalidPosition {
                        reason: format!("failed to calculate days to expiration: {}", e),
                    })
                })
            }
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
    #[must_use]
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
    #[must_use]
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
    ///
    /// # Errors
    ///
    /// Propagates the [`PositionError::PositiveError`] raised by `total_cost`,
    /// `fees` and `premium_received` when the fee or premium accumulation, or
    /// its product with the contract quantity, overflows the `Positive` range.
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
    /// - `None` if the position has zero quantity (no contracts), if the position total
    ///   cost cannot be calculated, or if the break-even price is not representable as a
    ///   `Positive` — a cost per contract above the strike puts it below zero, and a cost
    ///   per contract near `Positive::MAX` overflows the sum
    ///
    #[must_use]
    pub fn break_even(&self) -> Option<Positive> {
        if self.option.quantity == Positive::ZERO {
            return None;
        }
        let position_total_cost = self.total_cost().ok()?;
        // The raw `Positive` operators abort on overflow and on a difference
        // that would go below zero; a long put whose cost per contract exceeds
        // its strike reaches the latter with entirely ordinary numbers.
        let total_cost_per_contract = position_total_cost
            .checked_div(&self.option.quantity)
            .ok()?;
        match (&self.option.side, &self.option.option_style) {
            (Side::Long, OptionStyle::Call) => self
                .option
                .strike_price
                .checked_add(&total_cost_per_contract)
                .ok(),
            (Side::Short, OptionStyle::Call) => self
                .option
                .strike_price
                .checked_add(&self.premium)
                .and_then(|total| total.checked_sub(&total_cost_per_contract))
                .ok(),
            (Side::Long, OptionStyle::Put) => self
                .option
                .strike_price
                .checked_sub(&total_cost_per_contract)
                .ok(),
            (Side::Short, OptionStyle::Put) => self
                .option
                .strike_price
                .checked_sub(&self.premium)
                .and_then(|net| net.checked_add(&total_cost_per_contract))
                .ok(),
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
            Side::Long => Ok(Positive::MAX),
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
            Side::Short => Ok(Positive::MAX),
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
    ///
    /// # Errors
    ///
    /// Returns [`PositionError::PositiveError`] when adding the two fees, or
    /// scaling their sum by the contract quantity, overflows the `Positive`
    /// range. The raw `Positive` operators abort on overflow, so both steps
    /// are taken through their checked counterparts.
    pub fn fees(&self) -> Result<Positive, PositionError> {
        Ok(self
            .open_fee
            .checked_add(&self.close_fee)?
            .checked_mul(&self.option.quantity)?)
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
    /// use optionstratlib::{Side, OptionStyle};
    /// use positive::pos_or_panic;
    /// use optionstratlib::model::utils::create_sample_option_simplest;
    /// use chrono::Utc;
    ///
    /// // Create a valid position
    /// let option = create_sample_option_simplest(OptionStyle::Call, Side::Long);
    /// let position = Position::new(
    ///     option,
    ///     pos_or_panic!(5.25),
    ///     Utc::now(),
    ///     pos_or_panic!(0.65),
    ///     pos_or_panic!(0.65),
    ///     None, // epic (optional)
    ///    None, // extra fields (optional)
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
            epic: None,
            extra_fields: None,
        }
    }
}

impl TradeAble for Position {
    fn trade(&self) -> Result<Trade, TradeError> {
        let fee = self
            .open_fee
            .checked_add(&self.close_fee)
            .map_err(|error| {
                TradeError::invalid_trade(&format!(
                    "open and close fees overflow the Positive range: {error}"
                ))
            })?;
        if let (Ok(expiry), Some(timestamp)) = (
            resolve_expiration_date(&self.option.expiration_date),
            Utc::now().timestamp_nanos_opt(),
        ) {
            Ok(Trade {
                id: uuid::Uuid::new_v4(),
                action: Action::Buy,
                side: self.option.side,
                option_style: self.option.option_style,
                fee,
                symbol: None,
                strike: self.option.strike_price,
                expiry,
                timestamp,
                quantity: self.option.quantity,
                premium: self.premium,
                underlying_price: self.option.underlying_price,
                notes: None,
                status: TradeStatus::Other("Not yet initialized".to_string()),
            })
        } else {
            Err(TradeError::invalid_trade(
                "Could not create trade from position",
            ))
        }
    }

    fn trade_ref(&self) -> Result<&Trade, TradeError> {
        Err(TradeError::invalid_trade(
            "trade_ref() is not implemented for Position",
        ))
    }

    fn trade_mut(&mut self) -> Result<&mut Trade, TradeError> {
        Err(TradeError::invalid_trade(
            "trade_mut() is not implemented for Position",
        ))
    }
}

impl TradeStatusAble for Position {
    fn open(&self) -> Result<Trade, TradeError> {
        let mut trade = self.trade()?;
        trade.status = TradeStatus::Open;
        Ok(trade)
    }

    fn close(&self) -> Result<Trade, TradeError> {
        let mut trade = self.trade()?;
        // Compare directly against the `Decimal` literal because
        // `trade.premium` already stores a `Decimal` internally. Reading
        // that value with `to_dec()` avoids an unnecessary conversion
        // through `Positive` just to check whether the premium should be
        // treated as effectively zero.
        if trade.premium.to_dec() <= rust_decimal_macros::dec!(0.01) {
            trade.premium = Positive::ZERO;
        }
        trade.status = TradeStatus::Closed;
        trade.action = Action::Sell;
        Ok(trade)
    }

    fn expired(&self) -> Result<Trade, TradeError> {
        let mut trade = self.trade()?;
        trade.status = TradeStatus::Expired;
        trade.action = Action::Sell;
        Ok(trade)
    }

    fn exercised(&self) -> Result<Trade, TradeError> {
        let mut trade = self.trade()?;
        trade.status = TradeStatus::Exercised;
        trade.action = Action::Sell;
        Ok(trade)
    }

    fn assigned(&self) -> Result<Trade, TradeError> {
        let mut trade = self.trade()?;
        trade.status = TradeStatus::Assigned;
        trade.action = Action::Other;
        Ok(trade)
    }

    fn status_other(&self) -> Result<Trade, TradeError> {
        let mut trade = self.trade()?;
        trade.status = TradeStatus::Other("Not yet initialized".to_string());
        trade.action = Action::Other;
        Ok(trade)
    }
}

#[cfg(test)]
mod tests_position {
    use super::*;

    use crate::constants::ZERO;
    use crate::model::types::{OptionStyle, OptionType, Side};

    use chrono::Duration;
    use num_traits::ToPrimitive;
    use positive::pos_or_panic;
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
            implied_volatility: pos_or_panic!(0.2),
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            Positive::ONE,
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            Positive::ONE,
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            Positive::ONE,
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::ONE,
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            pos_or_panic!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            Positive::ONE,
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(90.0),
            pos_or_panic!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            Positive::ONE,
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            pos_or_panic!(10.0),
            Positive::ZERO,
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(
            position
                .pnl_at_expiration(&Some(&pos_or_panic!(107.0)))
                .unwrap(),
            Positive::ZERO,
            "Unrealized PNL for long call is incorrect."
        );
    }

    #[test]
    fn test_unrealized_pnl_long_call_quantity() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(
            position
                .pnl_at_expiration(&Some(&pos_or_panic!(107.0)))
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(
            position
                .unrealized_pnl(pos_or_panic!(3.0))
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(
            position
                .unrealized_pnl(pos_or_panic!(10.0))
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let date = Utc::now() - Duration::days(10);
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            date,
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
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
    use positive::pos_or_panic;

    #[test]
    fn test_valid_position() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        assert!(position.validate());
    }

    #[test]
    fn test_zero_premium() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        position.premium = Positive::ZERO;
        assert!(!position.validate());
    }

    #[test]
    fn test_invalid_option() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        position.option.strike_price = Positive::ZERO; // This makes the option invalid
        assert!(!position.validate());
    }

    #[test]
    fn test_zero_fees() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
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

    use positive::pos_or_panic;
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
            implied_volatility: pos_or_panic!(0.2),
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 107.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_short_call_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 103.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 93.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 97.0);
    }

    #[test]
    fn test_unrealized_pnl_short_put_size() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.break_even().unwrap(), 97.0);
    }

    #[test]
    fn test_zero_quantity() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ZERO, // zero quantity
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert!(position.break_even().is_none());
    }
}

#[cfg(test)]
mod tests_position_max_loss_profit {
    use super::*;

    use crate::model::types::{OptionStyle, OptionType, Side};

    use approx::assert_relative_eq;
    use positive::pos_or_panic;
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
            implied_volatility: pos_or_panic!(0.2),
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 7.0, epsilon = 0.001);
        assert_eq!(position.max_profit().unwrap(), Positive::MAX);
    }

    #[test]
    fn test_unrealized_pnl_long_call_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Call,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 70.0, epsilon = 0.001);
        assert_eq!(position.max_profit().unwrap(), Positive::MAX);
    }

    #[test]
    fn test_unrealized_pnl_short_call() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Call,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap(), Positive::MAX);
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap(), Positive::MAX);
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 7.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit().unwrap(), Positive::MAX);
    }

    #[test]
    fn test_unrealized_pnl_long_put_size() {
        let option = setup_option(
            Side::Long,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap().to_f64(), 70.0, epsilon = 0.001);
        assert_relative_eq!(position.max_profit().unwrap(), Positive::MAX);
    }

    #[test]
    fn test_unrealized_pnl_short_put() {
        let option = setup_option(
            Side::Short,
            OptionStyle::Put,
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            Positive::ONE,
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap(), Positive::MAX);
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
            Positive::HUNDRED,
            pos_or_panic!(105.0),
            pos_or_panic!(10.0),
            pos_or_panic!(30.0),
        );
        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_relative_eq!(position.max_loss().unwrap(), Positive::MAX);
        assert_relative_eq!(
            position.max_profit().unwrap().to_f64(),
            30.0,
            epsilon = 0.001
        );
    }
}

#[cfg(test)]
mod tests_premium {
    use super::*;
    use positive::pos_or_panic;

    fn setup_basic_position(side: Side) -> Position {
        let option = Options {
            side,
            quantity: Positive::ONE,
            ..Default::default()
        };

        Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        )
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
            quantity: pos_or_panic!(10.0),
            ..Default::default()
        };

        let position = Position::new(
            option,
            pos_or_panic!(5.0),
            Utc::now(),
            Positive::ONE,
            Positive::ONE,
            None,
            None,
        );
        assert_eq!(position.premium_received().unwrap(), 50.0);
    }
}

#[cfg(test)]
mod tests_position_serde {
    use super::*;

    use crate::model::utils::create_sample_position;

    use positive::pos_or_panic;
    use serde_json;
    use tracing::info;

    #[test]
    fn test_position_serialization() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
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
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        let serialized = serde_json::to_string(&position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(position, deserialized);
        assert_eq!(deserialized.option.underlying_symbol, "AAPL");
        assert_eq!(deserialized.option.strike_price, pos_or_panic!(95.0));
        assert_eq!(deserialized.premium, pos_or_panic!(5.0));
        assert_eq!(deserialized.open_fee, pos_or_panic!(0.5));
        assert_eq!(deserialized.close_fee, pos_or_panic!(0.5));
    }

    #[test]
    fn test_position_json_structure() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
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
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );
        let serialized = serde_json::to_string(&original).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        // Compare objects, not strings (string comparison is fragile due to decimal formatting)
        assert_eq!(original, deserialized);

        // Verify roundtrip produces equivalent object
        let reserialized = serde_json::to_string(&deserialized).unwrap();
        let redeserialized: Position = serde_json::from_str(&reserialized).unwrap();
        assert_eq!(deserialized, redeserialized);
    }

    #[test]
    fn test_position_with_different_option_types() {
        // Test with a Put option
        let put_position = create_sample_position(
            OptionStyle::Put,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );

        let serialized = serde_json::to_string(&put_position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(put_position, deserialized);
        assert_eq!(deserialized.option.option_style, OptionStyle::Put);

        // Test with a Short position
        let short_position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );

        let serialized = serde_json::to_string(&short_position).unwrap();
        let deserialized: Position = serde_json::from_str(&serialized).unwrap();

        assert_eq!(short_position, deserialized);
        assert_eq!(deserialized.option.side, Side::Short);
    }
}

#[cfg(test)]
mod tests_position_tradeable_trait {
    use super::*;

    use crate::model::utils::create_sample_position;
    use positive::pos_or_panic;

    #[test]
    fn test_tradeable_trade_ref() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );

        let result = position.trade_ref();
        assert!(result.is_err());
    }

    #[test]
    fn test_tradeable_trade_mut() {
        let mut position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0),
            Positive::ONE,
            pos_or_panic!(95.0),
            pos_or_panic!(0.2),
        );

        let result = position.trade_mut();
        assert!(result.is_err());
    }
}

#[cfg(test)]
mod tests_position_tradestatusable_trait {
    use super::*;

    use crate::model::utils::create_sample_position;
    use positive::pos_or_panic;

    #[test]
    fn test_tradestatusable_expired() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.2),  // implied volatility
        );

        let result = position.expired();
        assert!(result.is_ok());
    }

    #[test]
    fn test_tradestatusable_exercised() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.2),  // implied volatility
        );

        let result = position.exercised();
        assert!(result.is_ok());
    }

    #[test]
    fn test_tradestatusable_assigned() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.2),  // implied volatility
        );

        let result = position.assigned();
        assert!(result.is_ok());
    }

    #[test]
    fn test_tradestatusable_status_other() {
        let position = create_sample_position(
            OptionStyle::Call,
            Side::Short,
            pos_or_panic!(90.0), // underlying price
            Positive::ONE,       // quantity
            pos_or_panic!(95.0), // strike price
            pos_or_panic!(0.2),  // implied volatility
        );

        let result = position.status_other();
        assert!(result.is_ok());
    }

    /// Regression for issue #331: `Position::close()` must zero out the
    /// trade premium when it is at or below the 0.01 cutoff and preserve
    /// it otherwise. This is the only observable effect of the rewritten
    /// threshold comparison (previously backed by an `unsafe`
    /// `Positive::new_unchecked` block).
    #[test]
    fn test_tradestatusable_close_zeros_tiny_premium() {
        use crate::model::utils::create_sample_option_simplest;

        let build = |premium: Positive| {
            Position::new(
                create_sample_option_simplest(OptionStyle::Call, Side::Long),
                premium,
                Utc::now(),
                pos_or_panic!(0.01),
                pos_or_panic!(0.01),
                None,
                None,
            )
        };

        // Just above the cutoff: premium is preserved.
        let position = build(pos_or_panic!(0.02));
        let trade = position.close().expect("close() should succeed");
        assert_eq!(trade.premium, pos_or_panic!(0.02));

        // Exactly at the cutoff: premium is zeroed.
        let position = build(pos_or_panic!(0.01));
        let trade = position.close().expect("close() should succeed");
        assert_eq!(trade.premium, Positive::ZERO);

        // Below the cutoff: premium is zeroed.
        let position = build(pos_or_panic!(0.005));
        let trade = position.close().expect("close() should succeed");
        assert_eq!(trade.premium, Positive::ZERO);
    }
}
