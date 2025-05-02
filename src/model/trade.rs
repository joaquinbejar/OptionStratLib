use std::fmt;
use crate::model::types::Action;
use crate::{OptionStyle, Positive, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

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
    pub strike: Positive,       // 180.0
    /// * `expiry` - The expiration date of the Option, represented as a `DateTime<Utc>` (e.g., 2025-06-20T00:00:00Z).
    pub expiry: DateTime<Utc>,  // 2025-06-20T00:00:00Z
    /// * `timestamp` - The trade execution time in nanoseconds since the Unix epoch 
    ///   (1970-01-01 00:00:00 UTC). Serialized using nanoseconds with `serde`.
    #[serde(with = "ts_ns")]
    pub timestamp: i64,
    /// * `quantity` - The number of contracts traded, represented as a positive value.
    pub quantity: Positive, // contracts traded
    /// * `premium` - The premium per contract, represented as a `Decimal` value.
    pub premium: Decimal,   // premium per contract
    /// * `underlying_price` - The price of the underlying asset, represented as a positive value.
    pub underlying_price: Positive,
    /// * `notes` - Optional free-form notes associated with the trade.
    pub notes: Option<String>,
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
        premium: Decimal,
        underlying_price: Positive,
        notes: Option<String>,
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
        }
    }

    /// Convert back to `DateTime<Utc>` when you need it for pretty printing.
    pub fn datetime(&self) -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_nanos(self.timestamp)
    }
}

impl fmt::Display for Trade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let string = serde_json::to_string(self)
            .unwrap_or_else(|e| format!(r#"{{"error":"{}"}}"#, e));
        f.write_str(&string)
    }
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
    use rust_decimal::Decimal;
    use uuid::Uuid;

    /// Helper: build a deterministic sample Trade we can reuse.
    fn sample_trade() -> Trade {
        Trade {
            id: Uuid::nil(),
            action: Action::Buy,
            side: Side::Long,
            option_style: OptionStyle::Call,
            fee: Positive(Decimal::new(15, 2)),            // 0.15
            symbol: Some("AAPL".to_string()),
            strike: Positive(Decimal::new(1800, 1)),       // 180.0
            expiry: Utc.with_ymd_and_hms(2025, 6, 20, 0, 0, 0).unwrap(),
            timestamp: 1_700_000_000_000_000_000,          // arbitrary nanos
            quantity: Positive(Decimal::from(3u32)),
            premium: Decimal::new(250, 2),                 // 2.50
            underlying_price: Positive(Decimal::new(1850, 1)), // 185.0
            notes: Some("unit-test".to_string()),
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
            Positive(Decimal::new(2000, 1)),                 // 200.0
            Utc::now(),
            Positive(Decimal::from(1u32)),
            Decimal::new(300, 2),                            // 3.00
            Positive(Decimal::new(1900, 1)),                 // 190.0
            None,
        );
        let now_after = Utc::now().timestamp_nanos_opt().unwrap();
        assert!(trade.timestamp >= now_before - FIVE_SECS_NS);
        assert!(trade.timestamp <= now_after   + FIVE_SECS_NS);
    }

    #[test]
    fn timestamp_field_is_json_number() {
        let trade = sample_trade();
        let v = serde_json::to_value(&trade).unwrap();
        assert!(v["timestamp"].is_number());
    }
}