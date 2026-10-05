/// Re-export of core financial types from the standalone `financial_types` crate.
///
/// This module re-exports fundamental trading enums (`Action`, `Side`,
/// `OptionStyle`, `UnderlyingAssetType`) and option contract type definitions
/// (`OptionType`, sub-enums, `OptionBasicType`) from their respective
/// external crates.
pub use financial_types::{Action, OptionStyle, Side, UnderlyingAssetType};
pub use option_type::{
    AsianAveragingType, BarrierType, BinaryType, LookbackType, OptionBasicType, OptionType,
    RainbowType,
};

use chrono::{DateTime, Utc};

mod datetime_format {
    use super::*;
    use serde::{self, Deserialize, Deserializer, Serializer};

    #[allow(dead_code)]
    pub fn serialize<S>(date: &DateTime<Utc>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = date.to_rfc3339();
        serializer.serialize_str(&s)
    }

    #[allow(dead_code)]
    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc3339(&s)
            .map(|dt| dt.with_timezone(&Utc))
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests_vec_collection {
    use positive::{Positive, pos_or_panic};

    #[test]
    fn test_collect_empty_iterator() {
        let empty_vec: Vec<Positive> = Vec::new();
        let collected: Vec<Positive> = empty_vec.into_iter().collect();
        assert!(collected.is_empty());
    }

    #[test]
    fn test_collect_single_value() {
        let values = vec![Positive::ONE];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 1);
        assert_eq!(collected[0], Positive::ONE);
    }

    #[test]
    fn test_collect_multiple_values() {
        let values = vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)];
        let collected: Vec<Positive> = values.into_iter().collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], Positive::ONE);
        assert_eq!(collected[1], Positive::TWO);
        assert_eq!(collected[2], pos_or_panic!(3.0));
    }

    #[test]
    fn test_collect_from_filter() {
        let values = vec![
            Positive::ONE,
            Positive::TWO,
            pos_or_panic!(3.0),
            pos_or_panic!(4.0),
        ];
        let collected: Vec<Positive> = values.into_iter().filter(|x| x.to_f64() > 2.0).collect();
        assert_eq!(collected.len(), 2);
        assert_eq!(collected[0], pos_or_panic!(3.0));
        assert_eq!(collected[1], pos_or_panic!(4.0));
    }

    #[test]
    fn test_collect_from_map() {
        let values = vec![Positive::ONE, Positive::TWO, pos_or_panic!(3.0)];
        let collected: Vec<Positive> = values
            .into_iter()
            .map(|x| pos_or_panic!(x.to_f64() * 2.0))
            .collect();
        assert_eq!(collected.len(), 3);
        assert_eq!(collected[0], Positive::TWO);
        assert_eq!(collected[1], pos_or_panic!(4.0));
        assert_eq!(collected[2], pos_or_panic!(6.0));
    }

    #[test]
    fn test_collect_from_chain() {
        let values1 = vec![Positive::ONE, Positive::TWO];
        let values2 = vec![pos_or_panic!(3.0), pos_or_panic!(4.0)];
        let collected: Vec<Positive> = values1.into_iter().chain(values2).collect();
        assert_eq!(collected.len(), 4);
        assert_eq!(collected[0], Positive::ONE);
        assert_eq!(collected[1], Positive::TWO);
        assert_eq!(collected[2], pos_or_panic!(3.0));
        assert_eq!(collected[3], pos_or_panic!(4.0));
    }
}
