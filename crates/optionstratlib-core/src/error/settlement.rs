/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 5/11/26
******************************************************************************/

//! Target crate (ADR-0001 D6): **core**. Owns `SettlementError`.

use crate::model::types::{OptionType, Side};
use crate::model::{SettlementEvent, SettlementType};
use chrono::{DateTime, Utc};
use thiserror::Error;

/// Reasons why [`settlement_obligation`](crate::model::settlement_obligation)
/// cannot compute what an option position owes or receives.
#[derive(Error, Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SettlementError {
    /// The contract's option type has no settlement rule. Only European and
    /// American options are supported.
    #[error(
        "unsupported option type for settlement: {option_type} (supported: European, American)"
    )]
    UnsupportedOptionType {
        /// The rejected option type.
        option_type: OptionType,
    },

    /// The event needs a settlement price and none was supplied.
    ///
    /// An expiry needs one to decide moneyness and a cash settlement needs one
    /// to size its cashflow. A physical explicit exercise or assignment needs
    /// none.
    #[error("missing settlement price for a {settlement_type:?} {event:?} event")]
    MissingPrice {
        /// The settlement type of the event.
        settlement_type: SettlementType,
        /// The event that needs the price.
        event: SettlementEvent,
    },

    /// The settlement price was observed after the cutoff.
    #[error("settlement price observed at {observed_at} is after the cutoff {cutoff}")]
    PriceAfterCutoff {
        /// When the price was observed.
        observed_at: DateTime<Utc>,
        /// The latest instant a price may be observed at.
        cutoff: DateTime<Utc>,
    },

    /// The event cannot happen to a position on this side: only a long
    /// position exercises and only a short position is assigned.
    #[error("settlement event {event:?} is incompatible with a {side} position")]
    EventSideMismatch {
        /// The requested event.
        event: SettlementEvent,
        /// The side of the position.
        side: Side,
    },

    /// A figure of the obligation left the representable range.
    #[error("settlement arithmetic overflow in {operation}: {reason}")]
    ArithmeticOverflow {
        /// The computation that overflowed, for example `"physical cash"`.
        operation: &'static str,
        /// The underlying arithmetic error, with its operands.
        reason: String,
    },
}

#[cfg(test)]
mod tests_settlement_error {
    use super::*;
    use chrono::TimeZone;

    #[test]
    fn test_settlement_error_unsupported_option_type_names_the_type() {
        let error = SettlementError::UnsupportedOptionType {
            option_type: OptionType::Cliquet {
                reset_dates: Vec::new(),
            },
        };
        assert_eq!(
            error.to_string(),
            "unsupported option type for settlement: Cliquet Option (Reset Dates: []) (supported: European, American)"
        );
    }

    #[test]
    fn test_settlement_error_missing_price_names_the_event() {
        let error = SettlementError::MissingPrice {
            settlement_type: SettlementType::Physical,
            event: SettlementEvent::Expiry,
        };
        assert_eq!(
            error.to_string(),
            "missing settlement price for a Physical Expiry event"
        );
    }

    #[test]
    fn test_settlement_error_price_after_cutoff_names_both_instants() {
        let cutoff = Utc.timestamp_opt(1_000, 0).single();
        let observed_at = Utc.timestamp_opt(2_000, 0).single();
        assert!(cutoff.is_some() && observed_at.is_some());
        if let (Some(cutoff), Some(observed_at)) = (cutoff, observed_at) {
            let text = SettlementError::PriceAfterCutoff {
                observed_at,
                cutoff,
            }
            .to_string();
            assert!(text.contains(&observed_at.to_string()));
            assert!(text.contains(&cutoff.to_string()));
        }
    }

    #[test]
    fn test_settlement_error_event_side_mismatch_names_event_and_side() {
        let error = SettlementError::EventSideMismatch {
            event: SettlementEvent::Assignment,
            side: Side::Long,
        };
        assert_eq!(
            error.to_string(),
            "settlement event Assignment is incompatible with a Long position"
        );
    }

    #[test]
    fn test_settlement_error_overflow_names_the_operation() {
        let error = SettlementError::ArithmeticOverflow {
            operation: "physical cash",
            reason: "out of range".to_string(),
        };
        assert_eq!(
            error.to_string(),
            "settlement arithmetic overflow in physical cash: out of range"
        );
    }
}
