/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/

use crate::error::StrategyError;
use crate::model::Position;
use crate::strategies::base::StrategyType;
use crate::strategies::{
    BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, CustomStrategy,
    IronButterfly, IronCondor, LongButterflySpread, LongStraddle, LongStrangle,
    PoorMansCoveredCall, ShortButterflySpread, ShortStraddle, ShortStrangle, Strategable,
    StrategyConstructor,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyRequest {
    pub strategy_type: StrategyType,
    pub positions: Vec<Position>,
}

impl StrategyRequest {
    pub fn new(strategy_type: StrategyType, positions: Vec<Position>) -> Self {
        Self {
            strategy_type,
            positions,
        }
    }

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
    use crate::{OptionStyle, Side, pos};
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
                        pos!(920.0),
                        pos!(1.0),
                        pos!(900.0),
                        pos!(0.35),
                        sample_date(),
                    ),
                    pos!(4.5),
                    Utc::now(),
                    pos!(1.0),
                    pos!(1.2),
                ),
                Position::new(
                    create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Long,
                        pos!(920.0),
                        pos!(1.0),
                        pos!(910.0),
                        pos!(0.35),
                        sample_date(),
                    ),
                    pos!(3.5),
                    Utc::now(),
                    pos!(1.0),
                    pos!(1.2),
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
        assert_eq!(short_call.option.strike_price, pos!(900.0));
        assert_eq!(short_call.premium, pos!(4.5));
        assert_eq!(short_call.open_fee, pos!(1.0));
        assert_eq!(short_call.close_fee, pos!(1.2));

        // Verify second option (Long Call)
        let long_call = &deserialized.positions[1];
        assert_eq!(long_call.option.side, Side::Long);
        assert_eq!(long_call.option.strike_price, pos!(910.0));
        assert_eq!(long_call.premium, pos!(3.5));
        assert_eq!(long_call.open_fee, pos!(1.0));
        assert_eq!(long_call.close_fee, pos!(1.2));
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
