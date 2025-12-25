/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/

use crate::error::StrategyError;
use crate::model::Position;
use crate::strategies::base::StrategyType;
use crate::strategies::custom::CustomStrategy;
use crate::strategies::{
    BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, IronButterfly,
    IronCondor, LongButterflySpread, LongStraddle, LongStrangle, PoorMansCoveredCall,
    ShortButterflySpread, ShortStraddle, ShortStrangle, Strategable, StrategyConstructor,
};
use positive::{Positive, pos_or_panic};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// A request structure for creating and analyzing options trading strategies.
///
/// This structure encapsulates all necessary information to construct and evaluate
/// a specific options trading strategy. It contains the strategy type (such as
/// Bull Call Spread, Iron Condor, etc.) and the collection of financial positions
/// that make up the strategy.
///
/// `StrategyRequest` is typically used as an input to strategy analysis services
/// or functions that construct, validate, and evaluate option strategies based
/// on their positions.
///
#[derive(Clone, PartialEq, Serialize, Deserialize, ToSchema)]
pub struct StrategyRequest {
    /// The type of options trading strategy to construct or analyze.
    /// This determines the expected structure and validation rules
    /// for the provided positions.
    pub strategy_type: StrategyType,

    /// A collection of financial positions that make up the strategy.
    /// These positions typically include various options contracts
    /// (calls and puts) with different strike prices and expiration dates,
    /// arranged according to the selected strategy type.
    pub positions: Vec<Position>,
}

/// Request handler for options trading strategies.
///
/// This implementation provides functionality to create new strategy requests
/// and instantiate concrete strategy objects based on the specified strategy type
/// and positions.
impl StrategyRequest {
    /// Creates a new strategy request with the specified strategy type and positions.
    ///
    /// # Parameters
    /// * `strategy_type` - The type of options trading strategy to construct.
    /// * `positions` - A collection of financial positions that make up the strategy.
    ///
    /// # Returns
    /// A new `StrategyRequest` instance containing the provided strategy type and positions.
    pub fn new(strategy_type: StrategyType, positions: Vec<Position>) -> Self {
        Self {
            strategy_type,
            positions,
        }
    }

    /// Creates and returns a concrete strategy instance based on the strategy type
    /// and positions specified in this request.
    ///
    /// This method acts as a factory that constructs the appropriate strategy object
    /// by delegating to the corresponding strategy implementation's `get_strategy` method.
    ///
    /// # Returns
    /// * `Ok(Box<dyn Strategable>)` - A boxed trait object implementing the `Strategable`
    ///   trait if the strategy creation was successful.
    /// * `Err(StrategyError)` - An error indicating why the strategy could not be created.
    ///   Returns `StrategyError::NotImplemented` for strategies that are not yet implemented.
    ///
    /// # Errors
    /// This method can return errors from the underlying strategy constructors or
    /// `StrategyError::NotImplemented` for strategies that are defined but not yet implemented.
    pub fn get_strategy(&self) -> Result<Box<dyn Strategable>, StrategyError> {
        match self.strategy_type {
            StrategyType::BullCallSpread => {
                Ok(Box::new(BullCallSpread::get_strategy(&self.positions)?))
            }
            StrategyType::BearCallSpread => {
                Ok(Box::new(BearCallSpread::get_strategy(&self.positions)?))
            }
            StrategyType::BullPutSpread => {
                Ok(Box::new(BullPutSpread::get_strategy(&self.positions)?))
            }
            StrategyType::BearPutSpread => {
                Ok(Box::new(BearPutSpread::get_strategy(&self.positions)?))
            }
            StrategyType::LongButterflySpread => Ok(Box::new(LongButterflySpread::get_strategy(
                &self.positions,
            )?)),
            StrategyType::ShortButterflySpread => Ok(Box::new(ShortButterflySpread::get_strategy(
                &self.positions,
            )?)),
            StrategyType::IronCondor => Ok(Box::new(IronCondor::get_strategy(&self.positions)?)),
            StrategyType::IronButterfly => {
                Ok(Box::new(IronButterfly::get_strategy(&self.positions)?))
            }
            StrategyType::LongStraddle => {
                Ok(Box::new(LongStraddle::get_strategy(&self.positions)?))
            }
            StrategyType::ShortStraddle => {
                Ok(Box::new(ShortStraddle::get_strategy(&self.positions)?))
            }
            StrategyType::LongStrangle => {
                Ok(Box::new(LongStrangle::get_strategy(&self.positions)?))
            }
            StrategyType::ShortStrangle => {
                Ok(Box::new(ShortStrangle::get_strategy(&self.positions)?))
            }
            StrategyType::CoveredCall => Err(StrategyError::NotImplemented),
            StrategyType::ProtectivePut => Err(StrategyError::NotImplemented),
            StrategyType::Collar => Err(StrategyError::NotImplemented),
            StrategyType::LongCall => Err(StrategyError::NotImplemented),
            StrategyType::LongPut => Err(StrategyError::NotImplemented),
            StrategyType::ShortCall => Err(StrategyError::NotImplemented),
            StrategyType::ShortPut => Err(StrategyError::NotImplemented),
            StrategyType::PoorMansCoveredCall => Ok(Box::new(PoorMansCoveredCall::get_strategy(
                &self.positions,
            )?)),
            StrategyType::CallButterfly => {
                Ok(Box::new(CallButterfly::get_strategy(&self.positions)?))
            }
            StrategyType::Custom => Ok(Box::new(CustomStrategy::get_strategy(&self.positions)?)),
        }
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use crate::model::utils::create_sample_option_with_date;
    use crate::{OptionStyle, Side};
    use chrono::{DateTime, NaiveDateTime, Utc};
    use serde_json;

    fn sample_date() -> NaiveDateTime {
        DateTime::from_timestamp(1672531200, 0).unwrap().naive_utc()
    }

    #[test]
    fn test_strategy_request_serialization() {
        let strategy_request = StrategyRequest {
            strategy_type: StrategyType::BearCallSpread,
            positions: vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(900.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(910.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(3.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        };

        let serialized = serde_json::to_string(&strategy_request).unwrap();

        // Verify structure
        assert!(serialized.contains("\"strategy_type\":\"BearCallSpread\""));
        assert!(serialized.contains("\"positions\":["));
        assert!(serialized.contains("\"underlying_symbol\":\"AAPL\""));
        assert!(serialized.contains("\"premium\":4.5"));
        assert!(serialized.contains("\"open_fee\":1"));
        assert!(serialized.contains("\"close_fee\":1.2"));
    }

    #[test]
    fn test_strategy_request_deserialization() {
        let json_data = r#"{
            "strategy_type": "BearCallSpread",
            "positions": [
                {
                    "option": {
                        "option_type": "European",
                        "side": "Short",
                        "underlying_symbol": "AAPL",
                        "strike_price": 900.0,
                        "expiration_date": {"days": 30},
                        "implied_volatility": 0.35,
                        "quantity": 1.0,
                        "underlying_price": 920.0,
                        "risk_free_rate": 0.02,
                        "option_style": "Call",
                        "dividend_yield": 0.01,
                        "exotic_params": null
                    },
                    "premium": 4.5,
                    "date": "2024-01-01T00:00:00Z",
                    "open_fee": 1.0,
                    "close_fee": 1.2
                },
                {
                    "option": {
                        "option_type": "European",
                        "side": "Long",
                        "underlying_symbol": "AAPL",
                        "strike_price": 910.0,
                        "expiration_date": {"days": 30},
                        "implied_volatility": 0.35,
                        "quantity": 1.0,
                        "underlying_price": 920.0,
                        "risk_free_rate": 0.02,
                        "option_style": "Call",
                        "dividend_yield": 0.01,
                        "exotic_params": null
                    },
                    "premium": 3.5,
                    "date": "2024-01-01T00:00:00Z",
                    "open_fee": 1.0,
                    "close_fee": 1.2
                }
            ]
        }"#;

        let deserialized: StrategyRequest = serde_json::from_str(json_data).unwrap();

        // Verify deserialized data
        assert_eq!(deserialized.strategy_type, StrategyType::BearCallSpread);
        assert_eq!(deserialized.positions.len(), 2);

        // Verify first option (Short Call)
        let short_call = &deserialized.positions[0];
        assert_eq!(short_call.option.side, Side::Short);
        assert_eq!(short_call.option.strike_price, pos_or_panic!(900.0));
        assert_eq!(short_call.premium, pos_or_panic!(4.5));
        assert_eq!(short_call.open_fee, Positive::ONE);
        assert_eq!(short_call.close_fee, pos_or_panic!(1.2));

        // Verify second option (Long Call)
        let long_call = &deserialized.positions[1];
        assert_eq!(long_call.option.side, Side::Long);
        assert_eq!(long_call.option.strike_price, pos_or_panic!(910.0));
        assert_eq!(long_call.premium, pos_or_panic!(3.5));
        assert_eq!(long_call.open_fee, Positive::ONE);
        assert_eq!(long_call.close_fee, pos_or_panic!(1.2));
    }

    #[test]
    fn test_strategy_request_invalid_json() {
        let invalid_json = r#"{
            "strategy_type": "InvalidStrategy",
            "positions": []
        }"#;

        let result = serde_json::from_str::<StrategyRequest>(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_request_empty_options() {
        let json_data = r#"{
            "strategy_type": "BearCallSpread",
            "positions": []
        }"#;

        let deserialized: StrategyRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(deserialized.strategy_type, StrategyType::BearCallSpread);
        assert!(deserialized.positions.is_empty());
    }
}

#[cfg(test)]
mod tests_strategies_build_model {
    use super::*;
    use crate::model::utils::create_sample_option_with_date;
    use crate::{OptionStyle, Side, assert_decimal_eq};
    use chrono::{DateTime, NaiveDateTime, Utc};
    use rust_decimal_macros::dec;
    use serde_json;

    fn sample_date() -> NaiveDateTime {
        let tomorrow_timestamp = Utc::now().timestamp() + 86400;
        DateTime::from_timestamp(tomorrow_timestamp, 0)
            .unwrap()
            .naive_utc()
    }

    #[test]
    fn test_strategy_request() {
        let strategy_request = StrategyRequest::new(
            StrategyType::BearCallSpread,
            vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(900.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(910.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(3.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        );

        let serialized = serde_json::to_string(&strategy_request).unwrap();

        // Verify structure
        assert!(serialized.contains("\"strategy_type\":\"BearCallSpread\""));
        assert!(serialized.contains("\"positions\":["));
        assert!(serialized.contains("\"underlying_symbol\":\"AAPL\""));
        assert!(serialized.contains("\"premium\":4.5"));
        assert!(serialized.contains("\"open_fee\":1"));
        assert!(serialized.contains("\"close_fee\":1.2"));
    }

    #[test]
    fn test_strategy_bull_call_spread() {
        let strategy_request = StrategyRequest::new(
            StrategyType::BullCallSpread,
            vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(900.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(910.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(3.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        );

        let strategy = strategy_request.get_strategy().unwrap();
        let greeks_result = strategy.greeks();
        assert!(greeks_result.is_ok());
        let greeks = greeks_result.unwrap();
        assert_decimal_eq!(greeks.delta, dec!(0.1579), dec!(1e-4));
        assert_decimal_eq!(greeks.gamma, dec!(0.0309), dec!(1e-4));
        assert_decimal_eq!(greeks.theta, dec!(-4.5486), dec!(1e-4));
        assert_decimal_eq!(greeks.vega, dec!(0.2508), dec!(1e-4));
        assert_decimal_eq!(greeks.rho, dec!(0.0398), dec!(1e-4));
        assert_decimal_eq!(greeks.vanna, dec!(-1.2135), dec!(1e-4));
        assert_decimal_eq!(greeks.vomma, dec!(0.5476), dec!(1e-4));
        assert_decimal_eq!(greeks.veta, dec!(0.0031), dec!(1e-4));
        assert_decimal_eq!(greeks.charm, dec!(0.209293), dec!(1e-4));
        assert_decimal_eq!(greeks.color, dec!(-0.003801), dec!(1e-6));
    }

    #[test]
    fn test_strategy_bear_call_spread() {
        let strategy_request = StrategyRequest::new(
            StrategyType::BearCallSpread,
            vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(900.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(910.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(3.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        );

        let strategy = strategy_request.get_strategy().unwrap();
        let greeks_result = strategy.greeks();
        assert!(greeks_result.is_ok());
        let greeks = greeks_result.unwrap();
        assert_decimal_eq!(greeks.delta, dec!(-0.1579), dec!(1e-4));
        assert_decimal_eq!(greeks.gamma, dec!(0.0309), dec!(1e-4));
        assert_decimal_eq!(greeks.theta, dec!(-4.5486), dec!(1e-4));
        assert_decimal_eq!(greeks.vega, dec!(0.2508), dec!(1e-4));
        assert_decimal_eq!(greeks.rho, dec!(0.0398), dec!(1e-4));
        assert_decimal_eq!(greeks.vanna, dec!(-1.2135), dec!(1e-4));
        assert_decimal_eq!(greeks.vomma, dec!(0.5476), dec!(1e-4));
        assert_decimal_eq!(greeks.veta, dec!(0.0031), dec!(1e-4));
        assert_decimal_eq!(greeks.charm, dec!(0.209293), dec!(1e-4));
        assert_decimal_eq!(greeks.color, dec!(-0.003801), dec!(1e-6));
    }

    #[test]
    fn test_strategy_bear_put_spread() {
        let strategy_request = StrategyRequest::new(
            StrategyType::BearPutSpread,
            vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Put,
                        Side::Long,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(900.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Put,
                        Side::Short,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(910.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(3.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        );

        let strategy = strategy_request.get_strategy().unwrap();
        let greeks_result = strategy.greeks();
        assert!(greeks_result.is_ok());
        let greeks = greeks_result.unwrap();
        assert_decimal_eq!(greeks.delta, dec!(0.1579), dec!(1e-4));
        assert_decimal_eq!(greeks.gamma, dec!(0.0309), dec!(1e-4));
        assert_decimal_eq!(greeks.theta, dec!(-4.3511), dec!(1e-4));
        assert_decimal_eq!(greeks.vega, dec!(0.2508), dec!(1e-4));
        assert_decimal_eq!(greeks.rho, dec!(-0.0097), dec!(1e-4));
        assert_decimal_eq!(greeks.vanna, dec!(-1.2135), dec!(1e-4));
        assert_decimal_eq!(greeks.vomma, dec!(0.5476), dec!(1e-4));
        assert_decimal_eq!(greeks.veta, dec!(0.0031), dec!(1e-4));
        assert_decimal_eq!(greeks.charm, dec!(0.209239), dec!(1e-4));
        assert_decimal_eq!(greeks.color, dec!(-0.003801), dec!(1e-6));
    }

    #[test]
    fn test_strategy_bull_put_spread() {
        let strategy_request = StrategyRequest::new(
            StrategyType::BullPutSpread,
            vec![
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Put,
                        Side::Short,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(900.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Put,
                        Side::Long,
                        pos_or_panic!(920.0),
                        Positive::ONE,
                        pos_or_panic!(910.0),
                        pos_or_panic!(0.35),
                        sample_date(),
                    ),
                    pos_or_panic!(3.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        );

        let strategy = strategy_request.get_strategy().unwrap();
        let greeks_result = strategy.greeks();
        assert!(greeks_result.is_ok());
        let greeks = greeks_result.unwrap();
        assert_decimal_eq!(greeks.delta, dec!(-0.1579), dec!(1e-4));
        assert_decimal_eq!(greeks.gamma, dec!(0.0309), dec!(1e-4));
        assert_decimal_eq!(greeks.theta, dec!(-4.3511), dec!(1e-4));
        assert_decimal_eq!(greeks.vega, dec!(0.2508), dec!(1e-4));
        assert_decimal_eq!(greeks.rho, dec!(-0.0097), dec!(1e-4));
        assert_decimal_eq!(greeks.vanna, dec!(-1.2135), dec!(1e-4));
        assert_decimal_eq!(greeks.vomma, dec!(0.5476), dec!(1e-4));
        assert_decimal_eq!(greeks.veta, dec!(0.0031), dec!(1e-4));
        assert_decimal_eq!(greeks.charm, dec!(0.209239), dec!(1e-6));
        assert_decimal_eq!(greeks.color, dec!(-0.003801), dec!(1e-6));
    }

    #[test]
    fn test_strategy_covered_call() {
        let strategy_request = StrategyRequest::new(StrategyType::CoveredCall, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_protective_put() {
        let strategy_request = StrategyRequest::new(StrategyType::ProtectivePut, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_collar() {
        let strategy_request = StrategyRequest::new(StrategyType::Collar, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_long_call() {
        let strategy_request = StrategyRequest::new(StrategyType::LongCall, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_long_put() {
        let strategy_request = StrategyRequest::new(StrategyType::LongPut, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_short_call() {
        let strategy_request = StrategyRequest::new(StrategyType::ShortCall, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }

    #[test]
    fn test_strategy_short_put() {
        let strategy_request = StrategyRequest::new(StrategyType::ShortPut, vec![]);
        let result = strategy_request.get_strategy();
        assert!(result.is_err());
    }
}
