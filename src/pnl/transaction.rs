use crate::error::TransactionError;
use crate::pnl::utils::PnL;
use crate::{OptionStyle, OptionType, Positive, Side};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// # Transaction Status
///
/// Represents the current state of an options transaction in the system.
///
/// This enum tracks the lifecycle status of option transactions as they move
/// through various states from creation to completion. Each status represents
/// a meaningful business state that affects how the transaction is processed,
/// displayed, and included in profit and loss calculations.
///
/// ## Status Values
///
/// * `Open` - The transaction is currently active and has not been closed or settled
/// * `Closed` - The transaction has been manually closed before expiration
/// * `Expired` - The transaction reached its expiration date without being exercised
/// * `Exercised` - The option was exercised, converting it to a position in the underlying asset
/// * `Assigned` - For short options, indicates the counterparty exercised the option
#[derive(Debug, Copy, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    /// * `open` - The transaction is open and active
    Open,

    /// * `closed` - The transaction has been closed
    Closed,

    /// * `expired` - The transaction has expired
    Expired,

    /// * `exercised` - The transaction has been exercised
    Exercised,

    /// * `assigned` - The transaction has been assigned
    Assigned,
}

/// # Transaction
///
/// Represents an options trading transaction with all relevant trade details.
///
/// This structure encapsulates all the information related to an options contract transaction,
/// including the type, style, pricing information, quantity, fees, and market conditions at the time
/// of execution.
///
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    /// * `status` - The current status of the transaction (e.g., open, closed, expired)
    status: TransactionStatus,
    /// * `date_time` - The date and time when the transaction occurred
    date_time: Option<DateTime<Utc>>,
    /// * `option_type` - The type of options contract (e.g., European, American)
    option_type: OptionType,
    /// * `side` - The directional exposure of the position (Long or Short)
    side: Side,
    /// * `option_style` - The exercise style of the option (e.g., Put, Call)
    option_style: OptionStyle,
    /// * `quantity` - The number of contracts involved in the transaction
    quantity: Positive,
    /// * `premium` - The premium paid or received in this transaction
    premium: Positive,
    /// * `fees` - Commissions and fees paid for this transaction
    fees: Positive,
    /// * `underlying_price` - The price of the underlying asset at the time of the transaction
    underlying_price: Option<Positive>,
    /// * `days_to_expiration` - The number of days remaining until the option expires
    days_to_expiration: Option<Positive>,
    /// * `implied_volatility` - The implied volatility at the time of the transaction
    implied_volatility: Option<Positive>,
}

impl Transaction {
    /// Creates a new Transaction with all parameters.
    ///
    /// # Parameters
    ///
    /// * `date_time` - The date and time when the transaction occurred
    /// * `option_type` - The type of options contract (e.g., European, American)
    /// * `side` - The directional exposure of the position (Long or Short)
    /// * `option_style` - The exercise style of the option (e.g., Put, Call)
    /// * `price` - The execution price of the transaction
    /// * `quantity` - The number of contracts involved in the transaction
    /// * `premium` - The premium paid or received in this transaction
    /// * `fees` - Commissions and fees paid for this transaction
    /// * `underlying_price` - The price of the underlying asset at the time of the transaction
    /// * `days_to_expiration` - The number of days remaining until the option expires
    /// * `implied_volatility` - The implied volatility at the time of the transaction
    ///
    /// # Returns
    ///
    /// A new `Transaction` instance
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        status: TransactionStatus,
        date_time: Option<DateTime<Utc>>,
        option_type: OptionType,
        side: Side,
        option_style: OptionStyle,
        quantity: Positive,
        premium: Positive,
        fees: Positive,
        underlying_price: Option<Positive>,
        days_to_expiration: Option<Positive>,
        implied_volatility: Option<Positive>,
    ) -> Self {
        Transaction {
            status,
            date_time,
            option_type,
            side,
            option_style,
            quantity,
            premium,
            fees,
            underlying_price,
            days_to_expiration,
            implied_volatility,
        }
    }
}

impl Transaction {
    // Getters

    /// Gets the date and time of the transaction.
    pub fn date_time(&self) -> Option<DateTime<Utc>> {
        self.date_time
    }

    /// Gets the option type.
    pub fn option_type(&self) -> OptionType {
        self.option_type.clone()
    }

    /// Gets the side (Long or Short).
    pub fn side(&self) -> Side {
        self.side
    }

    /// Gets the option style (Call or Put).
    pub fn option_style(&self) -> OptionStyle {
        self.option_style
    }

    /// Gets the quantity of contracts.
    pub fn quantity(&self) -> Positive {
        self.quantity
    }

    /// Gets the premium.
    pub fn premium(&self) -> Positive {
        self.premium
    }

    /// Gets the fees.
    pub fn fees(&self) -> Positive {
        self.fees
    }

    /// Gets the underlying price, if available.
    pub fn underlying_price(&self) -> Option<Positive> {
        self.underlying_price
    }

    /// Gets the days to expiration, if available.
    pub fn days_to_expiration(&self) -> Option<Positive> {
        self.days_to_expiration
    }

    /// Gets the implied volatility, if available.
    pub fn implied_volatility(&self) -> Option<Positive> {
        self.implied_volatility
    }
}

impl Transaction {
    /// Updates the implied volatility for this transaction.
    ///
    /// # Parameters
    ///
    /// * `iv` - The new implied volatility value
    pub fn update_implied_volatility(&mut self, iv: Positive) {
        self.implied_volatility = Some(iv);
    }

    /// Updates the underlying price for this transaction.
    ///
    /// # Parameters
    ///
    /// * `price` - The new underlying price value
    pub fn update_underlying_price(&mut self, price: Positive) {
        self.underlying_price = Some(price);
    }

    /// Updates the days to expiration for this transaction.
    ///
    /// # Parameters
    ///
    /// * `days` - The new days to expiration value
    pub fn update_days_to_expiration(&mut self, days: Positive) {
        self.days_to_expiration = Some(days);
    }
}

impl Transaction {
    /// Calculates the profit and loss for this transaction based on its current status.
    ///
    /// # Parameters
    ///
    /// * `current_price` - The current price of the underlying asset
    ///
    /// # Returns
    ///
    /// A Result containing the PnL or an error
    pub fn pnl(&self) -> Result<PnL, TransactionError> {
        match self.status {
            TransactionStatus::Open => self.calculate_open_pnl(),
            TransactionStatus::Closed
            | TransactionStatus::Expired
            | TransactionStatus::Exercised
            | TransactionStatus::Assigned => self.calculate_closed_pnl(),
        }
    }

    /// Calculates PnL for an open position.
    ///
    /// # Parameters
    ///
    /// * `current_price` - The current price of the underlying asset
    ///
    /// # Returns
    ///
    /// A Result containing the PnL with unrealized values or an error
    fn calculate_open_pnl(&self) -> Result<PnL, TransactionError> {
        if self.option_type != OptionType::European {
            return Err(TransactionError {
                message: "Unsupported option type in Transaction".to_string(),
            });
        }

        let realized = match self.side {
            Side::Long => -(self.premium + self.fees).to_dec() * self.quantity,
            Side::Short => (self.premium - self.fees).to_dec() * self.quantity,
        };

        Ok(PnL::new(
            Some(realized),
            None,
            self.premium,
            self.fees,
            Utc::now(),
        ))
    }

    /// Calculates PnL for a closed position.
    ///
    /// # Parameters
    ///
    /// * `current_price` - The current price of the underlying asset
    ///
    /// # Returns
    ///
    /// A Result containing the PnL with realized values or an error
    fn calculate_closed_pnl(&self) -> Result<PnL, TransactionError> {
        if self.option_type != OptionType::European {
            return Err(TransactionError {
                message: "Unsupported option type in Transaction".to_string(),
            });
        }

        let realized = match self.side {
            Side::Short => -(self.premium + self.fees).to_dec() * self.quantity,
            Side::Long => (self.premium - self.fees).to_dec() * self.quantity,
        };

        Ok(PnL::new(
            Some(realized),
            None,
            self.premium,
            self.fees,
            Utc::now(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pos;
    use chrono::Utc;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;

    #[test]
    fn test_long_call_position_profitable_close() {
        // 1. Create an open long call position
        let open_date = Utc::now();
        let mut long_call = Transaction::new(
            TransactionStatus::Open, // Initial status is Open
            Some(open_date),         // Transaction date
            OptionType::European,    // European option
            Side::Long,              // Long position
            OptionStyle::Call,       // Call option
            pos!(1.0),               // 1 contract
            pos!(5.0),               // Premium paid: $5.00
            pos!(1.0),               // Fees: $1.00
            Some(pos!(100.0)),       // Underlying price at open: $100.00
            Some(pos!(30.0)),        // 30 days to expiration
            Some(pos!(0.2)),         // IV: 20%
        );

        // 2. Calculate initial PnL (should be negative as we paid premium + fees)
        let initial_pnl = long_call.pnl().unwrap();
        assert!(initial_pnl.realized.unwrap() < Decimal::ZERO);
        assert_eq!(initial_pnl.realized.unwrap(), dec!(-6.0)); // Premium + fees = -$6.00

        // 3. Now let's simulate price movement and close the position
        // Price has increased from $100 to $110
        long_call.update_underlying_price(pos!(110.0));

        // Close the position
        let closed_date = Utc::now();
        let closed_call = Transaction::new(
            TransactionStatus::Closed, // Status is now Closed
            Some(closed_date),         // Closing date
            OptionType::European,      // European option
            Side::Long,                // Long position
            OptionStyle::Call,         // Call option
            pos!(1.0),                 // 1 contract
            pos!(12.0),                // Closing premium: $12.00 (higher due to price increase)
            pos!(1.0),                 // Closing fees: $1.00
            Some(pos!(110.0)),         // Underlying price at close: $110.00
            Some(pos!(20.0)),          // 20 days to expiration (10 days elapsed)
            Some(pos!(0.22)),          // IV: 22%
        );

        // 4. Calculate closing PnL (should be positive as closing premium > initial premium + fees)
        let closing_pnl = closed_call.pnl().unwrap();
        assert!(closing_pnl.realized.unwrap() > Decimal::ZERO);
        assert_eq!(closing_pnl.realized.unwrap(), dec!(11.0)); // Selling premium - fees = $12.00 - $1.00 = $11.00

        // 5. Verify total profit by comparing the two PnLs
        let total_profit = closing_pnl.realized.unwrap() + initial_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(5.0)); // Net profit of $5.00 ($11.00 - $6.00)
    }

    #[test]
    fn test_long_call_position_unprofitable_close() {
        // Create an open long call position
        let open_date = Utc::now();
        let mut long_call = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            pos!(1.0),
            pos!(5.0), // Premium paid: $5.00
            pos!(1.0), // Fees: $1.00
            Some(pos!(100.0)),
            Some(pos!(30.0)),
            Some(pos!(0.2)),
        );

        // Calculate initial PnL
        let initial_pnl = long_call.pnl().unwrap();
        assert_eq!(initial_pnl.realized.unwrap(), dec!(-6.0));

        // Price has decreased from $100 to $95
        long_call.update_underlying_price(pos!(95.0));

        // Close the position for less than we paid
        let closed_date = Utc::now();
        let closed_call = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Long,
            OptionStyle::Call,
            pos!(1.0),
            pos!(2.0), // Closing premium: $2.00 (lower due to price decrease)
            pos!(1.0), // Closing fees: $1.00
            Some(pos!(95.0)),
            Some(pos!(20.0)),
            Some(pos!(0.18)),
        );

        // Calculate closing PnL
        let closing_pnl = closed_call.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(1.0)); // Selling premium - fees = $2.00 - $1.00 = $1.00

        // Verify total loss
        let total_profit = closing_pnl.realized.unwrap() + initial_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(-5.0)); // Net loss of $5.00 ($1.00 - $6.00)
    }

    #[test]
    fn test_short_call_position_profitable_close() {
        // 1. Create an open short call position
        let open_date = Utc::now();
        let mut short_call = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Short,       // Short position
            OptionStyle::Call, // Call option
            pos!(1.0),         // 1 contract
            pos!(5.0),         // Premium received: $5.00
            pos!(1.0),         // Fees: $1.00
            Some(pos!(100.0)), // Underlying price at open: $100.00
            Some(pos!(30.0)),  // 30 days to expiration
            Some(pos!(0.2)),   // IV: 20%
        );

        // 2. Calculate initial PnL (should be positive as we receive premium - fees)
        let initial_pnl = short_call.pnl().unwrap();
        assert!(initial_pnl.realized.unwrap() > Decimal::ZERO);
        assert_eq!(initial_pnl.realized.unwrap(), dec!(4.0)); // Premium - fees = $5.00 - $1.00 = $4.00

        // 3. Simulate price decrease from $100 to $95 (favorable for short call)
        short_call.update_underlying_price(pos!(95.0));

        // 4. Close the position by buying it back for less than we received
        let closed_date = Utc::now();
        let closed_call = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Short,
            OptionStyle::Call,
            pos!(1.0),
            pos!(2.0), // Closing premium: $2.00 (lower due to price decrease)
            pos!(1.0), // Closing fees: $1.00
            Some(pos!(95.0)),
            Some(pos!(20.0)),
            Some(pos!(0.18)),
        );

        // 5. Calculate closing PnL (should be negative as we're paying to close)
        let closing_pnl = closed_call.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(-3.0)); // Premium paid + fees = $2.00 + $1.00 = $3.00

        // 6. Verify total profit
        let total_profit = initial_pnl.realized.unwrap() + closing_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(1.0)); // Net profit of $1.00 ($4.00 - $3.00)
    }

    #[test]
    fn test_short_call_position_unprofitable_close() {
        // 1. Create an open short call position
        let open_date = Utc::now();
        let mut short_call = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Short,
            OptionStyle::Call,
            pos!(1.0),
            pos!(5.0), // Premium received: $5.00
            pos!(1.0), // Fees: $1.00
            Some(pos!(100.0)),
            Some(pos!(30.0)),
            Some(pos!(0.2)),
        );

        // 2. Calculate initial PnL
        let initial_pnl = short_call.pnl().unwrap();
        assert_eq!(initial_pnl.realized.unwrap(), dec!(4.0)); // Premium - fees = $4.00

        // 3. Simulate price increase from $100 to $110 (unfavorable for short call)
        short_call.update_underlying_price(pos!(110.0));

        // 4. Close the position by buying it back for more than we received
        let closed_date = Utc::now();
        let closed_call = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Short,
            OptionStyle::Call,
            pos!(1.0),
            pos!(12.0), // Closing premium: $12.00 (higher due to price increase)
            pos!(1.0),  // Closing fees: $1.00
            Some(pos!(110.0)),
            Some(pos!(20.0)),
            Some(pos!(0.22)),
        );

        // 5. Calculate closing PnL
        let closing_pnl = closed_call.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(-13.0)); // Premium paid + fees = $12.00 + $1.00 = $13.00

        // 6. Verify total loss
        let total_profit = initial_pnl.realized.unwrap() + closing_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(-9.0)); // Net loss of $7.00 ($4.00 - $13.00)
    }

    #[test]
    fn test_long_put_position_profitable_close() {
        // 1. Create an open long put position
        let open_date = Utc::now();
        let mut long_put = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Long,        // Long position
            OptionStyle::Put,  // Put option
            pos!(1.0),         // 1 contract
            pos!(4.0),         // Premium paid: $4.00
            pos!(1.0),         // Fees: $1.00
            Some(pos!(100.0)), // Underlying price at open: $100.00
            Some(pos!(30.0)),  // 30 days to expiration
            Some(pos!(0.2)),   // IV: 20%
        );

        // 2. Calculate initial PnL (should be negative as we paid premium + fees)
        let initial_pnl = long_put.pnl().unwrap();
        assert!(initial_pnl.realized.unwrap() < Decimal::ZERO);
        assert_eq!(initial_pnl.realized.unwrap(), dec!(-5.0)); // Premium + fees = -$5.00

        // 3. Simulate price decrease from $100 to $90 (favorable for long put)
        long_put.update_underlying_price(pos!(90.0));

        // 4. Close the position by selling it for more than we paid
        let closed_date = Utc::now();
        let closed_put = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Long,
            OptionStyle::Put,
            pos!(1.0),
            pos!(10.0), // Closing premium: $10.00 (higher due to price decrease)
            pos!(1.0),  // Closing fees: $1.00
            Some(pos!(90.0)),
            Some(pos!(20.0)),
            Some(pos!(0.25)),
        );

        // 5. Calculate closing PnL
        let closing_pnl = closed_put.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(9.0)); // Selling premium - fees = $10.00 - $1.00 = $9.00

        // 6. Verify total profit
        let total_profit = closing_pnl.realized.unwrap() + initial_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(4.0)); // Net profit of $4.00 ($9.00 - $5.00)
    }

    #[test]
    fn test_long_put_position_unprofitable_close() {
        // 1. Create an open long put position
        let open_date = Utc::now();
        let mut long_put = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Long,
            OptionStyle::Put,
            pos!(1.0),
            pos!(4.0), // Premium paid: $4.00
            pos!(1.0), // Fees: $1.00
            Some(pos!(100.0)),
            Some(pos!(30.0)),
            Some(pos!(0.2)),
        );

        // 2. Calculate initial PnL
        let initial_pnl = long_put.pnl().unwrap();
        assert_eq!(initial_pnl.realized.unwrap(), dec!(-5.0)); // Premium + fees = -$5.00

        // 3. Simulate price increase from $100 to $105 (unfavorable for long put)
        long_put.update_underlying_price(pos!(105.0));

        // 4. Close the position by selling it for less than we paid
        let closed_date = Utc::now();
        let closed_put = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Long,
            OptionStyle::Put,
            pos!(1.0),
            pos!(2.0), // Closing premium: $2.00 (lower due to price increase)
            pos!(1.0), // Closing fees: $1.00
            Some(pos!(105.0)),
            Some(pos!(20.0)),
            Some(pos!(0.18)),
        );

        // 5. Calculate closing PnL
        let closing_pnl = closed_put.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(1.0)); // Selling premium - fees = $2.00 - $1.00 = $1.00

        // 6. Verify total loss
        let total_profit = closing_pnl.realized.unwrap() + initial_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(-4.0)); // Net loss of $4.00 ($1.00 - $5.00)
    }

    #[test]
    fn test_short_put_position_profitable_close() {
        // 1. Create an open short put position
        let open_date = Utc::now();
        let mut short_put = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Short,       // Short position
            OptionStyle::Put,  // Put option
            pos!(1.0),         // 1 contract
            pos!(4.0),         // Premium received: $4.00
            pos!(1.0),         // Fees: $1.00
            Some(pos!(100.0)), // Underlying price at open: $100.00
            Some(pos!(30.0)),  // 30 days to expiration
            Some(pos!(0.2)),   // IV: 20%
        );

        // 2. Calculate initial PnL (should be positive as we receive premium - fees)
        let initial_pnl = short_put.pnl().unwrap();
        assert!(initial_pnl.realized.unwrap() > Decimal::ZERO);
        assert_eq!(initial_pnl.realized.unwrap(), dec!(3.0)); // Premium - fees = $4.00 - $1.00 = $3.00

        // 3. Simulate price increase from $100 to $105 (favorable for short put)
        short_put.update_underlying_price(pos!(105.0));

        // 4. Close the position by buying it back for less than we received
        let closed_date = Utc::now();
        let closed_put = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Short,
            OptionStyle::Put,
            pos!(1.0),
            pos!(1.5), // Closing premium: $1.50 (lower due to price increase)
            pos!(1.0), // Closing fees: $1.00
            Some(pos!(105.0)),
            Some(pos!(20.0)),
            Some(pos!(0.15)),
        );

        // 5. Calculate closing PnL
        let closing_pnl = closed_put.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(-2.5)); // Premium paid + fees = $1.50 + $1.00 = $2.50

        // 6. Verify total profit
        let total_profit = initial_pnl.realized.unwrap() + closing_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(0.5)); // Net profit of $2.50 ($3.00 - $2.50)
    }

    #[test]
    fn test_short_put_position_unprofitable_close() {
        // 1. Create an open short put position
        let open_date = Utc::now();
        let mut short_put = Transaction::new(
            TransactionStatus::Open,
            Some(open_date),
            OptionType::European,
            Side::Short,
            OptionStyle::Put,
            pos!(1.0),
            pos!(4.0), // Premium received: $4.00
            pos!(1.0), // Fees: $1.00
            Some(pos!(100.0)),
            Some(pos!(30.0)),
            Some(pos!(0.2)),
        );

        // 2. Calculate initial PnL
        let initial_pnl = short_put.pnl().unwrap();
        assert_eq!(initial_pnl.realized.unwrap(), dec!(3.0)); // Premium - fees = $3.00

        // 3. Simulate price decrease from $100 to $90 (unfavorable for short put)
        short_put.update_underlying_price(pos!(90.0));

        // 4. Close the position by buying it back for more than we received
        let closed_date = Utc::now();
        let closed_put = Transaction::new(
            TransactionStatus::Closed,
            Some(closed_date),
            OptionType::European,
            Side::Short,
            OptionStyle::Put,
            pos!(1.0),
            pos!(10.0), // Closing premium: $10.00 (higher due to price decrease)
            pos!(1.0),  // Closing fees: $1.00
            Some(pos!(90.0)),
            Some(pos!(20.0)),
            Some(pos!(0.25)),
        );

        // 5. Calculate closing PnL
        let closing_pnl = closed_put.pnl().unwrap();
        assert_eq!(closing_pnl.realized.unwrap(), dec!(-11.0)); // Premium paid + fees = $10.00 + $1.00 = $11.00

        // 6. Verify total loss
        let total_profit = initial_pnl.realized.unwrap() + closing_pnl.realized.unwrap();
        assert_eq!(total_profit, dec!(-8.0)); // Net loss of $6.00 ($3.00 - $11.00)
    }
}
