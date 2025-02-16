/******************************************************************************
    Author: Joaquín Béjar García
    Email: jb@taunais.com 
    Date: 16/2/25
 ******************************************************************************/
use serde::{Deserialize, Serialize};
use crate::{Options, Positive};
use crate::error::StrategyError;
use crate::strategies::base::StrategyType;
use crate::strategies::{BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly, CustomStrategy, IronButterfly, IronCondor, LongButterflySpread, LongStraddle, LongStrangle, PoorMansCoveredCall, ShortButterflySpread, ShortStraddle, ShortStrangle, Strategies};


#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct OptionWithCosts {
    pub open_fee: Positive,
    pub close_fee: Positive,
    pub premium: Positive,
    pub option: Options,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct StrategyRequest {
    pub strategy_type: StrategyType,
    pub options: Vec<OptionWithCosts>,
}

pub trait StrategyConstructor: Strategies {
    fn get_strategy( _vec_options: &Vec<OptionWithCosts>) -> Result<Self, StrategyError> where Self: Sized {
        Err(StrategyError::NotImplemented)
    }
}

impl StrategyRequest {
    pub fn new(strategy_type: StrategyType, options: Vec<OptionWithCosts>) -> Self {
        Self {
            strategy_type,
            options,
        }
    }
    
    pub fn get_strategy(&self) -> Result<Box<dyn Strategies>, StrategyError> {
        match self.strategy_type {
            StrategyType::BullCallSpread => Ok(Box::new(BullCallSpread::get_strategy(&self.options)?)),
            StrategyType::BearCallSpread => Ok(Box::new(BearCallSpread::get_strategy(&self.options)?)),
            StrategyType::BullPutSpread => Ok(Box::new(BullPutSpread::get_strategy(&self.options)?)),
            StrategyType::BearPutSpread => Ok(Box::new(BearPutSpread::get_strategy(&self.options)?)),
            StrategyType::LongButterflySpread => Ok(Box::new(LongButterflySpread::get_strategy(&self.options)?)),
            StrategyType::ShortButterflySpread => Ok(Box::new(ShortButterflySpread::get_strategy(&self.options)?)),
            StrategyType::IronCondor => Ok(Box::new(IronCondor::get_strategy(&self.options)?)),
            StrategyType::IronButterfly => Ok(Box::new(IronButterfly::get_strategy(&self.options)?)),
            StrategyType::LongStraddle => Ok(Box::new(LongStraddle::get_strategy(&self.options)?)),
            StrategyType::ShortStraddle => Ok(Box::new(ShortStraddle::get_strategy(&self.options)?)),
            StrategyType::LongStrangle => Ok(Box::new(LongStrangle::get_strategy(&self.options)?)),
            StrategyType::ShortStrangle => Ok(Box::new(ShortStrangle::get_strategy(&self.options)?)),
            StrategyType::CoveredCall => Err(StrategyError::NotImplemented),
            StrategyType::ProtectivePut => Err(StrategyError::NotImplemented),
            StrategyType::Collar => Err(StrategyError::NotImplemented),
            StrategyType::LongCall => Err(StrategyError::NotImplemented),
            StrategyType::LongPut => Err(StrategyError::NotImplemented),
            StrategyType::ShortCall =>Err(StrategyError::NotImplemented),
            StrategyType::ShortPut => Err(StrategyError::NotImplemented),
            StrategyType::PoorMansCoveredCall => Ok(Box::new(PoorMansCoveredCall::get_strategy(&self.options)?)),
            StrategyType::CallButterfly => Ok(Box::new(CallButterfly::get_strategy(&self.options)?)),
            StrategyType::Custom => Ok(Box::new(CustomStrategy::get_strategy(&self.options)?)),
        }
    }
}


#[cfg(test)]
mod tests_serialization {
    use super::*;
    use chrono::{TimeZone, Utc, DateTime, NaiveDateTime};
    use rust_decimal_macros::dec;
    use serde_json;
    use crate::model::utils::create_sample_option_with_date;
    use crate::{pos, ExpirationDate, OptionStyle, OptionType, Side};
    

    fn sample_date() -> NaiveDateTime {
        DateTime::from_timestamp(1672531200, 0).unwrap().naive_utc()
    }

    #[test]
    fn test_options_serialization() {
        let option = create_sample_option_with_date(
            OptionStyle::Call,
            Side::Long,
            pos!(4600.0),
            pos!(1.0),
            pos!(4500.0),
            pos!(0.25),
            sample_date(),
        );

        let serialized = serde_json::to_string(&option).unwrap();
        assert!(serialized.contains("\"underlying_symbol\":\"AAPL\""));
        assert!(serialized.contains("\"option_style\":\"Call\""));
        assert!(serialized.contains("\"side\":\"Long\""));
    }

    #[test]
    fn test_options_deserialization() {
        let json_data = r#"{"option_type":"European","side":"Short","underlying_symbol":"AAPL","strike_price":150.0,"expiration_date":{"datetime":"2023-01-01T00:00:00Z"},"implied_volatility":0.2,"quantity":2.0,"underlying_price":155.0,"risk_free_rate":0.01,"option_style":"Put","dividend_yield":0.01,"exotic_params":null}"#;
        let deserialized: Options = serde_json::from_str(json_data).unwrap();

        assert_eq!(deserialized.option_type, OptionType::European);
        assert_eq!(deserialized.side, Side::Short);
        assert_eq!(deserialized.underlying_symbol, "AAPL");
        assert_eq!(deserialized.strike_price, pos!(150.0));
        assert_eq!(deserialized.expiration_date, ExpirationDate::DateTime(Utc.from_utc_datetime(&sample_date())));
        assert_eq!(deserialized.implied_volatility, pos!(0.2));
        assert_eq!(deserialized.quantity, pos!(2.0));
        assert_eq!(deserialized.underlying_price, pos!(155.0));
        assert_eq!(deserialized.risk_free_rate, dec!(0.01));
        assert_eq!(deserialized.option_style, OptionStyle::Put);
    }

    #[test]
    fn test_option_with_costs_serialization() {
        let option_with_costs = OptionWithCosts {
            open_fee: pos!(1.5),
            close_fee: pos!(2.0),
            premium: pos!(5.0),
            option: create_sample_option_with_date(
                OptionStyle::Call,
                Side::Long,
                pos!(2850.0),
                pos!(1.0),
                pos!(2800.0),
                pos!(0.3),
                sample_date(),
            ),
        };

        let serialized = serde_json::to_string(&option_with_costs).unwrap();
        assert!(serialized.contains("\"open_fee\":1.5"));
        assert!(serialized.contains("\"close_fee\":2"));
        assert!(serialized.contains("\"premium\":5"));
        assert!(serialized.contains("\"underlying_symbol\":\"AAPL\""));
    }

    #[test]
    fn test_strategy_request_serialization() {
        let strategy_request = StrategyRequest {
            strategy_type: StrategyType::ShortStrangle,
            options: vec![
                OptionWithCosts {
                    open_fee: pos!(1.0),
                    close_fee: pos!(1.2),
                    premium: pos!(4.5),
                    option: create_sample_option_with_date(
                        OptionStyle::Call,
                        Side::Short,
                        pos!(920.0),
                        pos!(1.0),
                        pos!(900.0),
                        pos!(0.35),
                        sample_date(),
                    ),
                }
            ],
        };

        let serialized = serde_json::to_string(&strategy_request).unwrap();
        assert!(serialized.contains("\"strategy_type\":\"ShortStrangle\""));
        assert!(serialized.contains("\"underlying_symbol\":\"AAPL\""));
    }

    #[test]
    fn test_strategy_request_deserialization() {
        let json_data = r#"{"strategy_type":"ShortStrangle","options":[{"open_fee":1.0,"close_fee":1.2,"premium":4.5,"option":{"option_type":"European","side":"Short","underlying_symbol":"AAPL","strike_price":900.0,"expiration_date":{"days":3},"implied_volatility":0.35,"quantity":1.0,"underlying_price":920.0,"risk_free_rate":0.02,"option_style":"Call","dividend_yield":0.01,"exotic_params":null}}]}"#;

        let deserialized: StrategyRequest = serde_json::from_str(json_data).unwrap();
        assert_eq!(deserialized.strategy_type, StrategyType::ShortStrangle);
        assert_eq!(deserialized.options.len(), 1);
        assert_eq!(deserialized.options[0].open_fee, pos!(1.0));
        assert_eq!(deserialized.options[0].option.underlying_symbol, "AAPL");
    }

    #[test]
    fn test_options_methods() {
        let option = create_sample_option_with_date(
            OptionStyle::Call,
            Side::Long,
            pos!(155.0),
            pos!(1.0),
            pos!(150.0),
            pos!(0.2),
            sample_date(),
        );

        assert!(option.is_long());
        assert!(!option.is_short());

        let intrinsic_value = option.intrinsic_value(pos!(155.0)).unwrap();
        assert_eq!(intrinsic_value, dec!(5.0));

        let payoff = option.payoff().unwrap();
        assert!(payoff > dec!(0.0));
    }
}