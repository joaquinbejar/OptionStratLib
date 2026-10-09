/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 16/2/25
******************************************************************************/

use crate::error::StrategyError;
use crate::strategies::base::StrategyType;
use crate::strategies::custom::CustomStrategy;
use crate::strategies::{
    BearCallSpread, BearPutSpread, BullCallLadder, BullCallSpread, BullPutSpread, Collar,
    CoveredCall, IronButterfly, IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle,
    LongStrangle, PoorMansCoveredCall, ProtectivePut, ShortButterflySpread, ShortCall, ShortPut,
    ShortStraddle, ShortStrangle, Strategable, StrategyConstructor,
};
use optionstratlib_core::model::Position;
use optionstratlib_core::model::leg::SpotPosition;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "schema", derive(utoipa::ToSchema))]
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

    /// The share leg of a strategy that holds the underlying (`CoveredCall`,
    /// `ProtectivePut`, `Collar`); `None` for every other strategy (#831).
    /// Absent from the JSON when `None`, and read as `None` when missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spot_leg: Option<SpotPosition>,
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
    #[inline]
    #[must_use]
    pub fn new(strategy_type: StrategyType, positions: Vec<Position>) -> Self {
        Self {
            strategy_type,
            positions,
            spot_leg: None,
        }
    }

    /// Sets the share leg that `CoveredCall`, `ProtectivePut` and `Collar`
    /// hold besides their options (#831).
    #[inline]
    #[must_use]
    pub fn with_spot_leg(mut self, spot_leg: SpotPosition) -> Self {
        self.spot_leg = Some(spot_leg);
        self
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
    ///
    /// Every [`StrategyType`] is built (#831): `CoveredCall`, `ProtectivePut`
    /// and `Collar` from `spot_leg` and their options, every other type from
    /// `positions` alone.
    ///
    /// # Errors
    /// Returns the error of the strategy's
    /// [`StrategyConstructor::get_strategy_with_spot`]: an
    /// `OperationError` when the positions do not match the strategy's legs,
    /// when a covered strategy has no `spot_leg` or another strategy has one,
    /// and an `InvalidStrategy` when the legs fail its validation.
    #[inline(never)]
    pub fn get_strategy(&self) -> Result<Box<dyn Strategable>, StrategyError> {
        fn build<S: StrategyConstructor + Strategable + 'static>(
            request: &StrategyRequest,
        ) -> Result<Box<dyn Strategable>, StrategyError> {
            Ok(Box::new(S::get_strategy_with_spot(
                request.spot_leg.as_ref(),
                &request.positions,
            )?))
        }
        match self.strategy_type {
            StrategyType::BullCallSpread => build::<BullCallSpread>(self),
            StrategyType::BearCallSpread => build::<BearCallSpread>(self),
            StrategyType::BullPutSpread => build::<BullPutSpread>(self),
            StrategyType::BearPutSpread => build::<BearPutSpread>(self),
            StrategyType::LongButterflySpread => build::<LongButterflySpread>(self),
            StrategyType::ShortButterflySpread => build::<ShortButterflySpread>(self),
            StrategyType::IronCondor => build::<IronCondor>(self),
            StrategyType::IronButterfly => build::<IronButterfly>(self),
            StrategyType::LongStraddle => build::<LongStraddle>(self),
            StrategyType::ShortStraddle => build::<ShortStraddle>(self),
            StrategyType::LongStrangle => build::<LongStrangle>(self),
            StrategyType::ShortStrangle => build::<ShortStrangle>(self),
            StrategyType::CoveredCall => build::<CoveredCall>(self),
            StrategyType::ProtectivePut => build::<ProtectivePut>(self),
            StrategyType::Collar => build::<Collar>(self),
            StrategyType::LongCall => build::<LongCall>(self),
            StrategyType::LongPut => build::<LongPut>(self),
            StrategyType::ShortCall => build::<ShortCall>(self),
            StrategyType::ShortPut => build::<ShortPut>(self),
            StrategyType::PoorMansCoveredCall => build::<PoorMansCoveredCall>(self),
            StrategyType::BullCallLadder => build::<BullCallLadder>(self),
            StrategyType::Custom => build::<CustomStrategy>(self),
        }
    }
}

#[cfg(test)]
mod tests_serialization {
    use super::*;
    use chrono::{DateTime, NaiveDateTime, Utc};
    use optionstratlib_core::model::OptionStyle;
    use optionstratlib_core::model::Positive;
    use optionstratlib_core::model::Side;
    use optionstratlib_core::model::utils::create_sample_option_with_date;
    use optionstratlib_core::pos_or_panic;
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
            spot_leg: None,
        };

        let serialized = serde_json::to_string(&strategy_request).unwrap();

        // Verify structure
        assert!(serialized.contains("\"strategy_type\":\"BearCallSpread\""));
        assert!(serialized.contains("\"positions\":["));
        assert!(serialized.contains("\"underlying_symbol\":\"AAPL\""));
        assert!(serialized.contains("\"premium\":\"4.5\""));
        assert!(serialized.contains("\"open_fee\":\"1\""));
        assert!(serialized.contains("\"close_fee\":\"1.2\""));
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
    use chrono::{DateTime, NaiveDateTime, Utc};
    use optionstratlib_core::assert_decimal_eq;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::OptionStyle;
    use optionstratlib_core::model::Positive;
    use optionstratlib_core::model::Side;
    use optionstratlib_core::model::utils::create_sample_option_with_date;
    use optionstratlib_core::pos_or_panic;
    use rust_decimal_macros::dec;
    use serde_json;

    fn sample_date() -> NaiveDateTime {
        // Tomorrow plus an hour buffer so the integer day count in
        // `Actual365Fixed` resolves to 1 day reliably even with
        // sub-second clock drift between `Utc::now()` calls.
        let future_timestamp = Utc::now().timestamp() + 86400 + 3600;
        DateTime::from_timestamp(future_timestamp, 0)
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
        assert!(serialized.contains("\"premium\":\"4.5\""));
        assert!(serialized.contains("\"open_fee\":\"1\""));
        assert!(serialized.contains("\"close_fee\":\"1.2\""));
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
        assert_decimal_eq!(
            greeks.delta,
            dec!(0.1581527925803475549715372372),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.gamma,
            dec!(-0.0083145207388161162095837985),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.theta,
            dec!(1.1661006475280848039640271479),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vega,
            dec!(-0.0674820170867640005374723859),
            dec!(1e-4)
        );
        assert_decimal_eq!(greeks.rho, dec!(0.0037641936907908711830937891), dec!(1e-4));
        assert_decimal_eq!(
            greeks.vanna,
            dec!(-0.0910934609396018233276697490),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vomma,
            dec!(0.2162187229901720187343681980),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.veta,
            dec!(0.0000581489888845782729032393),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.charm,
            dec!(0.0167839745441953523259637898),
            dec!(1e-5)
        );
        assert_decimal_eq!(
            greeks.color,
            dec!(0.0088091718729949492796589761),
            dec!(1e-6)
        );
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
        let greeks = strategy
            .greeks()
            .unwrap_or_else(|e| panic!("greeks err: {e:?}"));
        assert_decimal_eq!(
            greeks.delta,
            dec!(-0.1581527925803475549715372372),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.gamma,
            dec!(0.0083145207388161162095837985),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.theta,
            dec!(-1.1661006475280848039640271479),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vega,
            dec!(0.0674820170867640005374723859),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.rho,
            dec!(-0.0037641936907908711830937891),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vanna,
            dec!(0.0910934609396018233276697490),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vomma,
            dec!(-0.2162187229901720187343681980),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.veta,
            dec!(-0.0000581489888845782729032393),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.charm,
            dec!(-0.0167839745441953523259637898),
            dec!(1e-5)
        );
        assert_decimal_eq!(
            greeks.color,
            dec!(-0.0088091718729949492796589761),
            dec!(1e-6)
        );
    }

    #[test]
    fn test_strategy_bear_put_spread() {
        // Textbook legs since #696: long the 910 put, short the 900 put. The
        // request used to carry the inverted legs, so every pinned Greek is
        // the old value negated (same two options with their sides flipped).
        let strategy_request = StrategyRequest::new(
            StrategyType::BearPutSpread,
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
                    pos_or_panic!(3.5),
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
                    pos_or_panic!(4.5),
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
        assert_decimal_eq!(
            greeks.delta,
            dec!(-0.1581527925803475549715372372),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.gamma,
            dec!(0.0083145207388161162095837985),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.theta,
            dec!(-1.1647309721540014524854918129),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vega,
            dec!(0.0674820170867640005374723859),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.rho,
            dec!(-0.0040381287656075429636946826),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vanna,
            dec!(0.0910934609396018233276697490),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vomma,
            dec!(-0.2162187229901720187343681980),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.veta,
            dec!(-0.0000581489888845782729032393),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.charm,
            dec!(-0.0167839745441953523341827428),
            dec!(1e-5)
        );
        assert_decimal_eq!(
            greeks.color,
            dec!(-0.0088091718729949492796589761),
            dec!(1e-6)
        );
    }

    #[test]
    fn test_strategy_bull_put_spread() {
        // Textbook legs since #696: long the 900 put, short the 910 put. The
        // request used to carry the inverted legs, so every pinned Greek is
        // the old value negated (same two options with their sides flipped).
        let strategy_request = StrategyRequest::new(
            StrategyType::BullPutSpread,
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
                    pos_or_panic!(3.5),
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
                    pos_or_panic!(4.5),
                    Utc::now(),
                    Positive::ONE,
                    pos_or_panic!(1.2),
                    None,
                    None,
                ),
            ],
        );

        let strategy = strategy_request.get_strategy().unwrap();
        let greeks = strategy
            .greeks()
            .unwrap_or_else(|e| panic!("greeks err: {e:?}"));
        assert_decimal_eq!(
            greeks.delta,
            dec!(0.1581527925803475549715372372),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.gamma,
            dec!(-0.0083145207388161162095837985),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.theta,
            dec!(1.1647309721540014524854918129),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vega,
            dec!(-0.0674820170867640005374723859),
            dec!(1e-4)
        );
        assert_decimal_eq!(greeks.rho, dec!(0.0040381287656075429636946826), dec!(1e-4));
        assert_decimal_eq!(
            greeks.vanna,
            dec!(-0.0910934609396018233276697490),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.vomma,
            dec!(0.2162187229901720187343681980),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.veta,
            dec!(0.0000581489888845782729032393),
            dec!(1e-4)
        );
        assert_decimal_eq!(
            greeks.charm,
            dec!(0.0167839745441953523341827428),
            dec!(1e-5)
        );
        assert_decimal_eq!(
            greeks.color,
            dec!(0.0088091718729949492796589761),
            dec!(1e-5)
        );
    }

    /// Every strategy built by `new` comes back from a request carrying its
    /// own legs with the same type and break-even points (#831).
    fn assert_round_trip(original: &dyn Strategable, request: StrategyRequest) {
        let rebuilt = match request.get_strategy() {
            Ok(strategy) => strategy,
            Err(e) => panic!("{:?}: {e}", request.strategy_type),
        };
        assert_eq!(rebuilt.get_title(), original.get_title());
        assert_eq!(
            rebuilt.get_break_even_points().unwrap(),
            original.get_break_even_points().unwrap()
        );
    }

    fn option_legs(strategy: &dyn Strategable) -> Vec<Position> {
        strategy
            .get_positions()
            .unwrap()
            .into_iter()
            .cloned()
            .collect()
    }

    fn expiry() -> ExpirationDate {
        ExpirationDate::Days(pos_or_panic!(30.0))
    }

    #[test]
    fn test_strategy_single_legs_round_trip() {
        let long_call = LongCall::new(
            "TEST".to_string(),
            pos_or_panic!(105.0),
            expiry(),
            pos_or_panic!(0.2),
            Positive::ONE,
            Positive::HUNDRED,
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(3.0),
            Positive::ONE,
            Positive::ONE,
        )
        .unwrap();
        let legs = option_legs(&long_call);
        assert_round_trip(
            &long_call,
            StrategyRequest::new(StrategyType::LongCall, legs),
        );

        // The other three single legs carry their one position through.
        for (kind, style, side) in [
            (StrategyType::LongPut, OptionStyle::Put, Side::Long),
            (StrategyType::ShortCall, OptionStyle::Call, Side::Short),
            (StrategyType::ShortPut, OptionStyle::Put, Side::Short),
        ] {
            let leg = Position::new(
                create_sample_option_with_date(
                    style,
                    side,
                    Positive::HUNDRED,
                    Positive::ONE,
                    pos_or_panic!(105.0),
                    pos_or_panic!(0.2),
                    sample_date(),
                ),
                pos_or_panic!(3.0),
                Utc::now(),
                Positive::ONE,
                Positive::ONE,
                None,
                None,
            );
            let strategy =
                match StrategyRequest::new(kind.clone(), vec![leg.clone()]).get_strategy() {
                    Ok(strategy) => strategy,
                    Err(e) => panic!("{kind:?}: {e}"),
                };
            assert_eq!(option_legs(strategy.as_ref()), vec![leg]);
            assert_eq!(strategy.get_break_even_points().unwrap().len(), 1);
        }
    }

    #[test]
    fn test_strategy_covered_round_trip() {
        let covered_call = CoveredCall::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            expiry(),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(100.0),
            pos_or_panic!(2.0),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
        )
        .unwrap();
        let request = StrategyRequest::new(StrategyType::CoveredCall, option_legs(&covered_call))
            .with_spot_leg(covered_call.spot_leg.clone());
        assert_round_trip(&covered_call, request);

        let protective_put = ProtectivePut::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            expiry(),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(100.0),
            pos_or_panic!(1.5),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
        )
        .unwrap();
        let request =
            StrategyRequest::new(StrategyType::ProtectivePut, option_legs(&protective_put))
                .with_spot_leg(protective_put.spot_leg.clone());
        assert_round_trip(&protective_put, request);

        let collar = Collar::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(95.0),
            pos_or_panic!(110.0),
            expiry(),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(100.0),
            pos_or_panic!(1.5),
            pos_or_panic!(2.0),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
        )
        .unwrap();
        let request = StrategyRequest::new(StrategyType::Collar, option_legs(&collar))
            .with_spot_leg(collar.spot_leg.clone());
        assert_round_trip(&collar, request.clone());

        // The share leg survives JSON, and a request without one omits it.
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"spot_leg\""));
        let back: StrategyRequest = serde_json::from_str(&json).unwrap();
        assert!(back == request);
        let no_spot = StrategyRequest::new(StrategyType::Collar, vec![]);
        let json = serde_json::to_string(&no_spot).unwrap();
        assert!(!json.contains("spot_leg"));
        let back: StrategyRequest = serde_json::from_str(&json).unwrap();
        assert!(back.spot_leg.is_none());
    }

    #[test]
    fn test_strategy_request_rejects_mismatched_legs() {
        let invalid = |request: StrategyRequest| {
            assert!(
                matches!(
                    request.get_strategy(),
                    Err(StrategyError::OperationError(_))
                ),
                "{:?} should be rejected",
                request.strategy_type
            );
        };
        let covered_call = CoveredCall::new(
            "TEST".to_string(),
            Positive::HUNDRED,
            pos_or_panic!(110.0),
            expiry(),
            pos_or_panic!(0.2),
            dec!(0.05),
            Positive::ZERO,
            pos_or_panic!(100.0),
            pos_or_panic!(2.0),
            Positive::ONE,
            Positive::ONE,
            pos_or_panic!(0.01),
            pos_or_panic!(0.01),
        )
        .unwrap();
        let short_call = option_legs(&covered_call);
        let spot = covered_call.spot_leg.clone();

        // Every type rejects an empty request.
        for kind in [
            StrategyType::CoveredCall,
            StrategyType::ProtectivePut,
            StrategyType::Collar,
            StrategyType::LongCall,
            StrategyType::LongPut,
            StrategyType::ShortCall,
            StrategyType::ShortPut,
        ] {
            invalid(StrategyRequest::new(kind, vec![]));
        }
        // A covered strategy needs its share leg.
        invalid(StrategyRequest::new(
            StrategyType::CoveredCall,
            short_call.clone(),
        ));
        // A short call is not a long call.
        invalid(StrategyRequest::new(
            StrategyType::LongCall,
            short_call.clone(),
        ));
        // A strategy without a share leg rejects one.
        invalid(
            StrategyRequest::new(StrategyType::ShortCall, short_call.clone())
                .with_spot_leg(spot.clone()),
        );
        // The share leg must be long and on the options' underlying.
        let mut short_spot = spot.clone();
        short_spot.side = Side::Short;
        invalid(
            StrategyRequest::new(StrategyType::CoveredCall, short_call.clone())
                .with_spot_leg(short_spot),
        );
        let mut other_symbol = spot;
        other_symbol.symbol = "OTHER".to_string();
        invalid(
            StrategyRequest::new(StrategyType::CoveredCall, short_call).with_spot_leg(other_symbol),
        );
    }
}
