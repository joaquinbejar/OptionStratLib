use crate::error::TradeError;
use crate::model::types::Action;
use crate::pnl::PnL;
use crate::{OptionStyle, Side};
use chrono::{DateTime, Utc};
use positive::Positive;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::{fmt, io};
use utoipa::ToSchema;

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
#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, ToSchema)]
pub enum TradeStatus {
    /// * `open` - The transaction is open and active
    #[default]
    Open,

    /// * `closed` - The transaction has been closed
    Closed,

    /// * `expired` - The transaction has expired
    Expired,

    /// * `exercised` - The transaction has been exercised
    Exercised,

    /// * `assigned` - The transaction has been assigned
    Assigned,

    /// This enum represents different categories or classifications, including the "Other" category.
    ///
    /// Use the `Other` case to represent items or instances that do not fall into predefined categories.
    Other(String),
}

/// A trait representing the status management of a trade.
///
/// This trait provides methods for transitioning a trade into various predefined statuses.
/// Implementations of this trait should define how a trade moves between these statuses.
///
/// Each method returns a `Trade` instance representing the trade in its updated status.
pub trait TradeStatusAble {
    /// - `open`: Return a `Trade` instance representing the trade in its open status or a
    ///   TradeError if the transition is invalid.
    fn open(&self) -> Result<Trade, TradeError>;
    /// - `closed`: Return a `Trade` instance representing the trade in its closed status or a
    ///   TradeError if the transition is invalid.
    fn close(&self) -> Result<Trade, TradeError>;
    /// - `expired`: Return a `Trade` instance representing the trade in its expired status or a
    ///   TradeError if the transition is invalid.
    fn expired(&self) -> Result<Trade, TradeError>;
    /// - `exercised`: Return a `Trade` instance representing the trade in its exercised status or a
    ///   TradeError if the transition is invalid.
    fn exercised(&self) -> Result<Trade, TradeError>;
    /// - `assigned`: Return a `Trade` instance representing the trade in its assigned status or a
    ///   TradeError if the transition is invalid.
    fn assigned(&self) -> Result<Trade, TradeError>;
    /// - `status_other`: Return a `Trade` instance representing undeclared status or a
    ///   TradeError if the transition is invalid.
    fn status_other(&self) -> Result<Trade, TradeError>;
}

/// Represents a trade with detailed information such as action, side, option style,
/// associated fees, and various metadata.
///
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Trade {
    /// * `id` - A universally unique identifier (`UUID`) for the trade.
    pub id: uuid::Uuid,
    /// * `action` - The action associated with the trade (e.g., Buy or Sell).
    pub action: Action,
    /// * `side` - Indicates whether the trade is on the Long or Short side.
    pub side: Side,
    /// Specifies the style of an options contract Call or Put.
    pub option_style: OptionStyle,
    /// * `fee` - The transaction fee associated with the trade, represented as a positive value.
    pub fee: Positive,
    /// * `symbol` - An optional ticker symbol for the trade (e.g., "AAPL" for Apple Inc.).
    pub symbol: Option<String>, // “AAPL”
    /// * `strike` - The strike price of the option, represented as a positive value (e.g., 180.0).
    pub strike: Positive, // 180.0
    /// * `expiry` - The expiration date of the Option, represented as a `DateTime<Utc>` (e.g., 2025-06-20T00:00:00Z).
    pub expiry: DateTime<Utc>, // 2025-06-20T00:00:00Z
    /// * `timestamp` - The trade execution time in nanoseconds since the Unix epoch
    ///   (1970-01-01 00:00:00 UTC). Serialized using nanoseconds with `serde`.
    #[serde(with = "ts_ns")]
    pub timestamp: i64,
    /// * `quantity` - The number of contracts traded, represented as a positive value.
    pub quantity: Positive, // contracts traded
    /// * `premium` - The premium per contract, represented as a `Decimal` value.
    pub premium: Positive, // premium per contract
    /// * `underlying_price` - The price of the underlying asset, represented as a positive value.
    pub underlying_price: Positive,
    /// * `notes` - Optional free-form notes associated with the trade.
    pub notes: Option<String>,

    /// Represents the current status of a trade.
    ///
    /// This `status` field indicates the state of a trade during its lifecycle.
    /// It uses the `TradeStatus` enum to define the possible statuses.
    ///
    /// This field is essential for tracking and managing the lifecycle of a trade.
    pub status: TradeStatus,
}

impl Trade {
    /// Creates a new instance of the struct with the provided parameters.
    ///
    /// # Parameters
    /// - `id` (`uuid::Uuid`): The unique identifier for the entity.
    /// - `action` (`Action`): The action being performed (e.g., buy, sell).
    /// - `side` (`Side`): The side of the transaction (e.g., long, short).
    /// - `option_style` (`OptionStyle`): The style of the option (e.g., American, European).
    /// - `fee` (`Positive`): The fee associated with the transaction. It must be a positive value.
    /// - `symbol` (`Option<String>`): The symbol of the underlying asset, if applicable.
    /// - `strike` (`Positive`): The strike price of the option. This must be a positive value.
    /// - `expiry` (`DateTime<Utc>`): The expiration date and time of the option in UTC.
    /// - `quantity` (`Positive`): The quantity involved in the transaction. This must be a positive value.
    /// - `premium` (`Decimal`): The premium value associated with the transaction.
    /// - `underlying_price` (`Positive`): The price of the underlying asset. This must be a positive value.
    /// - `notes` (`Option<String>`): Any additional notes or metadata for the transaction.
    ///
    /// # Returns
    /// An instance of the struct initialized with the provided parameters, along with a timestamp
    /// (`i64`) in nanoseconds representing the moment of creation.
    ///
    /// # Panics
    /// The method will panic if obtaining the current timestamp (`Utc::now()`) in nanoseconds fails.
    ///
    /// # Remarks
    /// - Ensure all `Positive` values are validated and created using the appropriate constructors.
    /// - The `symbol` and `notes` parameters are optional and can be set to `None` if not applicable.
    ///
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: uuid::Uuid,
        action: Action,
        side: Side,
        option_style: OptionStyle,
        fee: Positive,
        symbol: Option<String>,
        strike: Positive,
        expiry: DateTime<Utc>,
        quantity: Positive,
        premium: Positive,
        underlying_price: Positive,
        notes: Option<String>,
        status: TradeStatus,
    ) -> Self {
        let timestamp = Utc::now()
            .timestamp_nanos_opt()
            .expect("system time outside chrono range");
        Self {
            id,
            action,
            side,
            option_style,
            fee,
            symbol,
            strike,
            expiry,
            timestamp,
            quantity,
            premium,
            underlying_price,
            notes,
            status,
        }
    }

    /// Convert back to `DateTime<Utc>` when you need it for pretty printing.
    pub fn datetime(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(self.timestamp)
    }

    /// Sets the timestamp for the current instance using the provided `DateTime<Utc>` value.
    ///
    /// # Parameters
    /// - `datetime`: A `DateTime<Utc>` object representing the new timestamp to be set.
    ///
    /// # Panics
    /// This function will panic if the calculated nanosecond representation of the provided
    /// `datetime` falls outside the valid range that can be represented by the `chrono` library.
    ///
    pub fn set_timestamp(&mut self, datetime: DateTime<Utc>) {
        self.timestamp = datetime
            .timestamp_nanos_opt()
            .expect("system time outside chrono range");
    }

    /// Calculates the total cost associated with a transaction based on the action,
    /// side, fee, and premium.
    ///
    /// # Returns
    /// A `Positive` type, which represents the total cost of the transaction.
    ///
    /// # Logic
    /// - The total cost is determined by the transaction's `action` (Buy/Sell),
    ///   `side` (Long/Short), `fee`, and `premium`, all adjusted by the `quantity`.
    /// - The `fee` and `premium` are multiplied by the `quantity` to determine their
    ///   respective costs.
    /// - Depending on the combination of `action` and `side`, the following rules are applied:
    ///   - `(Action::Buy, Side::Long)` or `(Action::Sell, Side::Short)`:
    ///     The cost includes both `fees` and `premium`.
    ///   - `(Action::Buy, Side::Short)` or `(Action::Sell, Side::Long)`:
    ///     The cost includes only `fees`.
    ///
    /// # Assumptions
    /// - It assumes that `fee`, `premium`, and `quantity` are positive values.
    /// - The `Positive` type enforces that the resulting cost is non-negative.
    pub fn cost(&self) -> Positive {
        let fees = self.fee * self.quantity;
        let premium = self.premium * self.quantity;
        match (self.action, self.side) {
            (Action::Buy, Side::Long) | (Action::Sell, Side::Short) => premium + fees,
            (Action::Buy, Side::Short) | (Action::Sell, Side::Long) => fees,
            _ => Positive::ZERO,
        }
    }

    /// Computes the income generated based on the action and side of a trade.
    ///
    /// # Description
    /// This function calculates the income by determining the premium associated
    /// with the current object's `quantity` and `premium` values. The resulting
    /// income depends on the combination of the `action` and `side` values:
    ///
    /// - If the action is `Buy` and the side is `Long`, or if the action is `Sell`
    ///   and the side is `Short`, the income is `0`.
    /// - If the action is `Buy` and the side is `Short`, or if the action is `Sell`
    ///   and the side is `Long`, the function returns the computed `premium`.
    ///
    /// # Returns
    /// - `Positive::ZERO`: If the `action` and `side` combination does not result
    ///   in an income.
    /// - A `Positive` value (equal to the computed `premium`): For combinations
    ///   where income is generated.
    ///
    /// # Panics
    /// This function does not explicitly handle invalid cases, so incorrect usage
    /// (e.g., uninitialized fields or invalid state) may lead to runtime panics.
    ///
    /// # Dependencies
    /// This method relies on:
    /// - `Action` enum (expected values: `Buy`, `Sell`)
    /// - `Side` enum (expected values: `Long`, `Short`)
    /// - `Positive` type for representing non-negative values.
    pub fn income(&self) -> Positive {
        let premium = self.quantity * self.premium;
        match (self.action, self.side) {
            (Action::Buy, Side::Long) | (Action::Sell, Side::Short) => Positive::ZERO,
            (Action::Buy, Side::Short) | (Action::Sell, Side::Long) => premium,
            _ => Positive::ZERO,
        }
    }

    /// Calculates the net value by subtracting the cost from the income.
    ///
    /// # Returns
    ///
    /// A `Decimal` value representing the net result of income minus cost.
    ///
    /// # Behavior
    /// - The `income()` method is called to obtain the income value, which is then converted to a `Decimal` using `to_dec()`.
    /// - The `cost()` method is called to obtain the cost value, which is also converted to a `Decimal` using `to_dec()`.
    /// - The net value is computed as: `income.to_dec() - cost.to_dec()`.
    ///
    pub fn net(&self) -> Decimal {
        self.income().to_dec() - self.cost().to_dec()
    }

    /// Checks if the current trade status is `Open`.
    ///
    /// # Returns
    ///
    /// * `true` - If the trade's status is `Open`.
    /// * `false` - If the trade's status is not `Open`.
    ///
    /// This method is commonly used to determine whether a trade is still active and can be interacted with.
    pub fn is_open(&self) -> bool {
        self.status == TradeStatus::Open
    }

    /// Computes and returns the Profit and Loss (PnL) for the current object.
    ///
    /// # Returns
    /// * `PnL` - A value representing the calculated Profit and Loss based on the current object's state.
    ///
    /// # Implementation Details
    /// This function leverages the `Into` trait to convert the current object (`self`) into a `PnL` instance.
    ///
    pub fn pnl(&self) -> PnL {
        self.into()
    }

    /// Determines whether the trade is closed.
    ///
    /// This method checks the status of a trade and returns `true`
    /// if the trade’s status is `TradeStatus::Closed`, otherwise returns `false`.
    ///
    /// # Returns
    /// * `bool` - `true` if the trade's status is `TradeStatus::Closed`, `false` otherwise.
    ///
    pub fn is_closed(&self) -> bool {
        self.status == TradeStatus::Closed
    }

    /// Checks if the trade has expired.
    ///
    /// This method compares the current `status` of the trade with `TradeStatus::Expired`
    /// and returns `true` if the trade is expired, otherwise `false`.
    ///
    /// # Returns
    /// * `true` - If the trade's status is `TradeStatus::Expired`.
    /// * `false` - If the trade's status is not `TradeStatus::Expired`.
    ///
    pub fn is_expired(&self) -> bool {
        self.status == TradeStatus::Expired
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string =
            serde_json::to_string(self).unwrap_or_else(|e| format!(r#"{{"error":"{e}"}}"#));
        f.write_str(&string)
    }
}

impl TradeAble for Trade {
    fn trade(&self) -> Result<Trade, TradeError> {
        Ok(self.clone())
    }

    fn trade_ref(&self) -> Result<&Trade, TradeError> {
        Ok(self)
    }

    fn trade_mut(&mut self) -> Result<&mut Trade, TradeError> {
        Ok(self)
    }
}

/// Saves a list of trades to a file in JSON format.
///
/// # Parameters
/// - `trades`: A slice containing trade data to be saved. Each trade must implement serialization.
/// - `file_path`: The path to the file where the trade data will be saved. If the file already exists, its contents will be overwritten.
///
/// # Returns
/// - `Ok(())` if the trades are successfully saved to the file.
/// - `Err(io::Error)` if an I/O error occurs during file creation or writing, or if serialization fails.
///
/// # JSON Formatting
/// The trades are serialized to JSON using compact formatting (without pretty printing).
///
/// # Errors
/// This function will return an error if:
/// - Serialization of the `trades` slice to JSON fails.
/// - The file cannot be created or opened at the specified `file_path`.
/// - Writing to the file fails.
///
pub fn save_trades(trades: &[Trade], file_path: &str) -> io::Result<()> {
    // Serialize to compact JSON without pretty formatting
    let json =
        serde_json::to_string(trades).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Create or open the file for writing
    let mut file = File::create(file_path)?;

    // Write the JSON string to the file
    file.write_all(json.as_bytes())?;

    Ok(())
}

/// A trait that provides functionality for accessing and modifying trade-related data.
///
/// Implementors of this trait should provide mechanisms to retrieve both immutable
/// and mutable references to a `Trade` object.
///
pub trait TradeAble {
    /// Retrieves a reference to a `Trade` instance associated with the current object.
    ///
    /// # Returns
    ///
    /// A `Trade` object, or a `TradeError` if the trade cannot be accessed.
    ///
    /// This method allows access to the `Trade` data within the context of the implementing object.
    /// It ensures that the `Trade` is available for viewing or interaction.
    ///
    /// Note: This method returns an owned `Trade` instance, allowing modification.
    fn trade(&self) -> Result<Trade, TradeError>;

    /// Returns a reference to the `Trade` associated with the current instance.
    ///
    /// # Returns
    /// A reference to the `Trade` object tied to this instance, or a `TradeError` if the trade
    /// cannot be accessed.
    ///
    /// # Note
    /// The returned reference has the same lifetime as the instance it is called on.
    fn trade_ref(&self) -> Result<&Trade, TradeError>;

    /// Provides a mutable reference to the `Trade` instance contained within the current structure.
    ///
    /// # Returns
    ///
    /// A mutable reference to the internal `Trade` instance, or a `TradeError` if the trade
    /// cannot be accessed.
    /// This allows the caller to modify the `Trade` directly.
    ///
    /// # Notes
    ///
    /// - Since this method provides a mutable reference, it enforces Rust's borrow rules.
    ///   Only one mutable reference to the `Trade` is allowed at a time.
    /// - Ensure that concurrent access to the structure is properly managed to avoid runtime issues.
    fn trade_mut(&mut self) -> Result<&mut Trade, TradeError>;
}

mod ts_ns {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(nanos: &i64, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        s.serialize_i64(*nanos)
    }

    pub fn deserialize<'de, D>(d: D) -> Result<i64, D::Error>
    where
        D: Deserializer<'de>,
    {
        i64::deserialize(d) // read it straight back as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use chrono::{TimeZone, Utc};
    use positive::pos_or_panic;
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    use uuid::Uuid;

    /// Helper: build a deterministic sample Trade we can reuse.
    fn sample_trade() -> Trade {
        Trade {
            id: Uuid::nil(),
            action: Action::Buy,
            side: Side::Long,
            option_style: OptionStyle::Call,
            fee: Positive(Decimal::new(15, 2)), // 0.15
            symbol: Some("AAPL".to_string()),
            strike: Positive(Decimal::new(1800, 1)), // 180.0
            expiry: Utc.with_ymd_and_hms(2025, 6, 20, 0, 0, 0).unwrap(),
            timestamp: 1_700_000_000_000_000_000, // arbitrary nanos
            quantity: Positive(Decimal::from(3u32)),
            premium: pos_or_panic!(2.5),                       // 2.50
            underlying_price: Positive(Decimal::new(1850, 1)), // 185.0
            notes: Some("unit-test".to_string()),
            status: TradeStatus::Open,
        }
    }

    #[test]
    fn display_matches_serde_json() {
        let trade = sample_trade();
        let expect = serde_json::to_string(&trade).unwrap();
        assert_eq!(expect, trade.to_string());
    }

    #[test]
    fn serde_roundtrip() {
        let trade = sample_trade();
        let json = serde_json::to_string(&trade).unwrap();
        let back: Trade = serde_json::from_str(&json).unwrap();
        assert_eq!(trade, back);
    }

    #[test]
    fn datetime_conversion_roundtrip() {
        let trade = sample_trade();
        let dt = trade.datetime();
        assert_eq!(dt.timestamp_nanos_opt().unwrap(), trade.timestamp);
    }

    #[test]
    fn new_sets_reasonable_timestamp() {
        // Allow up to 5 s drift between now() in ctor and now() here.
        const FIVE_SECS_NS: i64 = 5_000_000_000;
        let now_before = Utc::now().timestamp_nanos_opt().unwrap();
        let trade = Trade::new(
            Uuid::new_v4(),
            Action::Sell,
            Side::Short,
            OptionStyle::Put,
            Positive(Decimal::new(25, 2)),
            None,
            Positive(Decimal::new(2000, 1)), // 200.0
            Utc::now(),
            Positive(Decimal::from(1u32)),
            pos_or_panic!(3.0),              // 3.00
            Positive(Decimal::new(1900, 1)), // 190.0
            None,
            TradeStatus::Open,
        );
        let now_after = Utc::now().timestamp_nanos_opt().unwrap();
        assert!(trade.timestamp >= now_before - FIVE_SECS_NS);
        assert!(trade.timestamp <= now_after + FIVE_SECS_NS);
    }

    #[test]
    fn timestamp_field_is_json_number() {
        let trade = sample_trade();
        let v = serde_json::to_value(&trade).unwrap();
        assert!(v["timestamp"].is_number());
    }

    /// Build a reproducible sample trade so tests stay deterministic.
    fn sample_trade_bis(action: Action, side: Side, status: TradeStatus) -> Trade {
        Trade::new(
            Uuid::nil(), // id
            action,
            side,
            OptionStyle::Call,        // option_style
            pos_or_panic!(0.15),      // fee   = 0.15
            Some("AAPL".to_string()), // symbol
            pos_or_panic!(180.0),     // strike = 180
            Utc.with_ymd_and_hms(2025, 6, 20, 0, 0, 0).unwrap(),
            pos_or_panic!(3.0),       // quantity = 3
            pos_or_panic!(2.50),      // premium  = 2.50
            pos_or_panic!(185.0),     // underlying_price
            Some("unit-test".into()), // notes
            status,
        )
    }

    #[test]
    fn new_sets_current_timestamp() {
        let before = Utc::now().timestamp_nanos_opt().unwrap();
        let tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let after = Utc::now().timestamp_nanos_opt().unwrap();
        assert!(tr.timestamp >= before && tr.timestamp <= after);
    }

    #[test]
    fn datetime_roundtrip() {
        let tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let dt = tr.datetime();
        assert_eq!(dt.timestamp_nanos_opt().unwrap(), tr.timestamp);
    }

    #[test]
    fn set_timestamp_overwrites_value() {
        let mut tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let new_dt = Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0).unwrap();
        tr.set_timestamp(new_dt);
        assert_eq!(tr.datetime(), new_dt);
    }

    /* ---------- cost / income / net math ---------- */

    fn expect_cost_income(action: Action, side: Side, exp_cost: Decimal, exp_income: Decimal) {
        let tr = sample_trade_bis(action, side, TradeStatus::Open);

        assert_eq!(tr.cost().to_dec(), exp_cost);
        assert_eq!(tr.income().to_dec(), exp_income);
        assert_eq!(tr.net(), exp_income - exp_cost);
    }

    #[test]
    fn cash_flows_buy_long() {
        // cost  = (premium + fee) * qty  = (2.50 + 0.15) * 3 = 7.95
        // income = 0
        expect_cost_income(Action::Buy, Side::Long, dec!(7.95), dec!(0));
    }

    #[test]
    fn cash_flows_sell_long() {
        // cost   = fee * qty            = 0.15 * 3 = 0.45
        // income = premium * qty        = 2.50 * 3 = 7.50
        expect_cost_income(Action::Sell, Side::Long, dec!(0.45), dec!(7.50));
    }

    #[test]
    fn cash_flows_buy_short() {
        // cost   = fee * qty            = 0.45
        // income = premium * qty        = 7.50
        expect_cost_income(Action::Buy, Side::Short, dec!(0.45), dec!(7.50));
    }

    #[test]
    fn cash_flows_sell_short() {
        // cost   = (premium + fee) * qty = 7.95
        // income = 0
        expect_cost_income(Action::Sell, Side::Short, dec!(7.95), dec!(0));
    }

    /* ---------- status helpers ---------- */

    #[test]
    fn status_helpers_work() {
        let open = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let closed = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Closed);
        let exp = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Expired);

        assert!(open.is_open() && !open.is_closed() && !open.is_expired());
        assert!(closed.is_closed() && !closed.is_open() && !closed.is_expired());
        assert!(exp.is_expired() && !exp.is_open() && !exp.is_closed());
    }

    /* ---------- Display ---------- */

    #[test]
    fn display_outputs_json() {
        let tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let json = serde_json::to_string(&tr).unwrap();
        assert_eq!(json, tr.to_string());
    }

    /* ---------- TradeAble trait ---------- */

    #[test]
    fn tradeable_returns_same_ref() {
        let tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let trait_obj: &dyn TradeAble = &tr;

        // The reference handed back by trade() must be the very same object.
        assert!(std::ptr::eq(
            trait_obj.trade_ref().unwrap() as *const Trade,
            &tr as *const Trade
        ));
    }

    #[test]
    fn tradeable_mut_allows_mutation() {
        let mut tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        {
            let trait_obj: &mut dyn TradeAble = &mut tr;
            // Flip the status via the mutable reference returned by trade_mut()
            let trade_ref = trait_obj.trade_mut().unwrap();
            trade_ref.status = TradeStatus::Closed;
        }
        // Change is visible on the original value
        assert!(tr.is_closed());
    }

    #[test]
    fn tradeable_returns_cloned_trade() {
        let tr = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let trait_obj: &dyn TradeAble = &tr;

        // Get a Trade cloned object
        let tr_cloned = trait_obj
            .trade()
            .expect("trade() should return a cloned Trade object");

        // Cloned object and original reference have the same values
        assert_eq!(tr_cloned, tr);

        // Must not be the same allocation
        assert!(!std::ptr::eq(
            &tr_cloned as *const Trade,
            &tr as *const Trade
        ));

        // Mutating the clone does not affect the original reference
        let mut tr_cloned = tr_cloned;
        tr_cloned.status = TradeStatus::Closed;

        assert!(tr_cloned.is_closed());
        assert!(!tr.is_closed());
    }

    /* ---------- TradeStatusAble ---------- */

    /// helper: assert that two trades are identical except for the `status` field
    fn assert_same_except_status(a: &Trade, b: &Trade) {
        let mut aa = a.clone();
        aa.status = b.status.clone();
        assert_eq!(aa, *b);
    }

    impl TradeStatusAble for Trade {
        fn open(&self) -> Result<Trade, TradeError> {
            let mut tr = self.clone();
            tr.status = TradeStatus::Open;
            Ok(tr)
        }

        fn close(&self) -> Result<Trade, TradeError> {
            let mut tr = self.clone();
            tr.status = TradeStatus::Closed;
            Ok(tr)
        }

        fn expired(&self) -> Result<Trade, TradeError> {
            let mut tr = self.clone();
            tr.status = TradeStatus::Expired;
            Ok(tr)
        }

        fn exercised(&self) -> Result<Trade, TradeError> {
            let mut tr = self.clone();
            tr.status = TradeStatus::Exercised;
            Ok(tr)
        }

        fn assigned(&self) -> Result<Trade, TradeError> {
            let mut tr = self.clone();
            tr.status = TradeStatus::Assigned;
            Ok(tr)
        }

        fn status_other(&self) -> Result<Trade, TradeError> {
            let mut tr = self.clone();
            tr.status = TradeStatus::Other("other".to_string());
            Ok(tr)
        }
    }

    #[test]
    fn status_transitions_return_new_trade() {
        let base = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);

        // Closed
        let closed = base.close().unwrap();
        assert_eq!(closed.status, TradeStatus::Closed);
        assert_same_except_status(&base, &closed);
        assert_eq!(base.status, TradeStatus::Open); // original untouched

        // Expired
        let expired = base.expired().unwrap();
        assert_eq!(expired.status, TradeStatus::Expired);
        assert_same_except_status(&base, &expired);

        // Exercised
        let exercised = base.exercised().unwrap();
        assert_eq!(exercised.status, TradeStatus::Exercised);
        assert_same_except_status(&base, &exercised);

        // Assigned
        let assigned = base.assigned().unwrap();
        assert_eq!(assigned.status, TradeStatus::Assigned);
        assert_same_except_status(&base, &assigned);

        // Open (idempotent transition)
        let reopened = closed.open().unwrap();
        assert_eq!(reopened.status, TradeStatus::Open);
        assert_same_except_status(&base, &reopened);
    }

    #[test]
    fn status_other_sets_custom_string() {
        let base = sample_trade_bis(Action::Buy, Side::Long, TradeStatus::Open);
        let other = base.status_other().unwrap();
        if let TradeStatus::Other(s) = &other.status {
            // default impl should put some non-empty tag
            assert!(!s.is_empty(), "status_other() must fill the string");
        } else {
            panic!("status_other() did not return TradeStatus::Other");
        }
        assert_same_except_status(&base, &other);
    }
}
