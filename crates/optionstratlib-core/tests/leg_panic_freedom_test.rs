//! Property-based tests for panic freedom in the leg, balance, payoff and
//! time helpers of the core crate (#788).
//!
//! Each driver below reached a `Decimal` or `Positive` operator, an integer
//! division or a `chrono` addition that aborted the caller before #788:
//!
//! - the spot, future and perpetual legs multiplied `pub` quantities, prices
//!   and margins with `Positive * Positive`, and divided by the quantity for
//!   the liquidation price (`PerpetualPosition::default()` divided by zero);
//! - `Balance` and `Portfolio` multiplied and summed premia with the raw
//!   operators and `Iterator::sum`;
//! - the Asian put payoff subtracted the average from the strike with
//!   `Positive - f64`, which aborts whenever the put is out of the money;
//! - `convert_time_frame` divided by the units per year of a custom frame
//!   that may be zero, and `get_x_days_formatted` added a day count the
//!   calendar cannot hold;
//! - `generate_price_points` divided by `num_points - 1`.
//!
//! The assertion is deliberately weak: whatever comes back, it must come
//! back.

use optionstratlib_core::model::leg::{
    Expirable, Fundable, FuturePosition, Leg, LegAble, Marginable, PerpetualPosition, SpotPosition,
};
use optionstratlib_core::model::payoff::{Payoff, PayoffInfo};
use optionstratlib_core::model::types::{
    AsianAveragingType, OptionStyle, OptionType, Side, UnderlyingAssetType,
};
use optionstratlib_core::model::utils::generate_price_points;
use optionstratlib_core::model::{Balance, ExpirationDate, Portfolio, Positive};
use optionstratlib_core::utils::TimeFrame;
use optionstratlib_core::utils::time::{convert_time_frame, get_x_days_formatted};
use proptest::prelude::*;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;

/// The smallest representable `Decimal`: a divisor at this scale turns an
/// ordinary per-unit division into an overflow.
const TINY: Decimal = Decimal::from_parts(1, 0, 0, false, 28);

/// A `Positive` from a `Decimal` literal that is non-negative by construction.
fn pos(value: Decimal) -> Positive {
    Positive::new_decimal(value).unwrap_or(Positive::ZERO)
}

/// Quantities, prices, margins and fees across the whole `Positive` range,
/// including the two ends that break the arithmetic.
fn extreme_positive() -> impl Strategy<Value = Positive> {
    prop_oneof![
        Just(Positive::ZERO),
        Just(pos(TINY)),
        Just(pos(dec!(0.01))),
        Just(Positive::ONE),
        Just(Positive::HUNDRED),
        Just(pos(dec!(1000000000000000))),
        Just(Positive::MAX),
    ]
}

/// Funding rates over the signed `Decimal` range.
fn extreme_decimal() -> impl Strategy<Value = Decimal> {
    prop_oneof![
        Just(Decimal::ZERO),
        Just(dec!(0.0001)),
        Just(dec!(-0.0001)),
        Just(dec!(1000000)),
        Just(Decimal::MAX),
        Just(Decimal::MIN),
    ]
}

fn extreme_side() -> impl Strategy<Value = Side> {
    prop_oneof![Just(Side::Long), Just(Side::Short)]
}

/// Every frame, with custom frames that have no unit or the whole range in
/// a year.
fn extreme_time_frame() -> impl Strategy<Value = TimeFrame> {
    prop_oneof![
        Just(TimeFrame::Microsecond),
        Just(TimeFrame::Second),
        Just(TimeFrame::Day),
        Just(TimeFrame::Week),
        Just(TimeFrame::Year),
        Just(TimeFrame::Custom(Positive::ZERO)),
        Just(TimeFrame::Custom(pos(TINY))),
        Just(TimeFrame::Custom(Positive::MAX)),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// Every figure the spot leg exposes.
    #[test]
    fn test_spot_leg_never_panics(
        quantity in extreme_positive(),
        cost_basis in extreme_positive(),
        open_fee in extreme_positive(),
        close_fee in extreme_positive(),
        price in extreme_positive(),
        side in extreme_side(),
    ) {
        let spot = SpotPosition::new(
            "SPOT".to_string(),
            quantity,
            cost_basis,
            side,
            chrono::Utc::now(),
            open_fee,
            close_fee,
        );
        let _ = spot.initial_value();
        let _ = spot.market_value(price);
        let _ = spot.percentage_return(price);
        let _ = spot.break_even_price();
        let _ = spot.to_string();
        let _ = spot.notional_value(price);
        let _ = Leg::spot(spot).notional_value(price);
    }

    /// Every figure the future leg exposes, including the margin and
    /// liquidation figures that divide by the quantity.
    #[test]
    fn test_future_leg_never_panics(
        quantity in extreme_positive(),
        entry_price in extreme_positive(),
        contract_size in extreme_positive(),
        initial_margin in extreme_positive(),
        maintenance_margin in extreme_positive(),
        price in extreme_positive(),
        side in extreme_side(),
    ) {
        let future = FuturePosition::new(
            "FUT".to_string(),
            quantity,
            entry_price,
            side,
            ExpirationDate::Days(pos(dec!(30))),
            contract_size,
            initial_margin,
            maintenance_margin,
            chrono::Utc::now(),
            Positive::ZERO,
        );
        let _ = future.notional_value_at_entry();
        let _ = future.notional_value_at_price(price);
        let _ = future.tick_value(price);
        let _ = future.total_margin_required();
        let _ = future.basis(price);
        let _ = future.implied_leverage();
        let _ = future.initial_margin();
        let _ = future.maintenance_margin();
        let _ = future.leverage();
        let _ = future.liquidation_price(price);
        let _ = future.is_liquidation_risk(price, Decimal::ZERO);
        let _ = future.days_to_expiration();
        let _ = future.time_to_expiration_years();
        let _ = future.expiration_timestamp();
        let _ = future.is_expired();
        let _ = FuturePosition::long(
            "FUT".to_string(),
            quantity,
            entry_price,
            ExpirationDate::Days(Positive::ONE),
            contract_size,
            initial_margin,
        );
    }

    /// Every figure the perpetual leg exposes, including the funding
    /// payments and the liquidation price of a zero quantity.
    #[test]
    fn test_perpetual_leg_never_panics(
        quantity in extreme_positive(),
        entry_price in extreme_positive(),
        margin in extreme_positive(),
        funding_rate in extreme_decimal(),
        price in extreme_positive(),
        side in extreme_side(),
    ) {
        let perpetual = PerpetualPosition {
            quantity,
            entry_price,
            margin,
            funding_rate,
            side,
            ..PerpetualPosition::default()
        };
        let _ = perpetual.notional_value_at_entry();
        let _ = perpetual.notional_value_at_price(price);
        let _ = perpetual.maintenance_margin();
        let _ = perpetual.liquidation_price(price);
        let _ = perpetual.is_liquidation_risk(price, Decimal::ZERO);
        let _ = perpetual.funding_payment(price);
        let _ = perpetual.annualized_funding(price);
        let _ = perpetual.roe_percentage(price);
        let _ = perpetual.margin_ratio(price);
        let _ = perpetual.effective_leverage(price);
    }

    /// The balance and portfolio figures over quantities and premia whose
    /// products and sums leave the `Decimal` range.
    #[test]
    fn test_balance_and_portfolio_never_panic(
        balances in prop::collection::vec(
            (extreme_positive(), extreme_positive(), extreme_positive()),
            0..4,
        ),
    ) {
        let mut portfolio = Portfolio::new("PROP".to_string());
        for (quantity, average, current) in balances {
            let balance = Balance::new(
                "BAL".to_string(),
                quantity,
                average,
                Some(current),
                "EX".to_string(),
                UnderlyingAssetType::Stock,
                None,
            );
            let _ = balance.get_total_value();
            let _ = balance.get_unrealized_pnl();
            let _ = balance.is_profitable();
            let _ = balance.get_cost_basis();
            let _ = balance.get_percentage_return();
            portfolio.add_balance(balance);
        }
        let _ = portfolio.get_total_value();
        let _ = portfolio.get_total_unrealized_pnl();
        let _ = portfolio.has_profitable_positions();
    }

    /// The Asian payoff for both styles and averages, over strikes below,
    /// at and above the average.
    #[test]
    fn test_asian_payoff_never_panics(
        strike in extreme_positive(),
        prices in prop::collection::vec(extreme_positive(), 0..5),
        geometric in any::<bool>(),
        put in any::<bool>(),
        side in extreme_side(),
    ) {
        let averaging_type = if geometric {
            AsianAveragingType::Geometric
        } else {
            AsianAveragingType::Arithmetic
        };
        let info = PayoffInfo {
            spot: Positive::HUNDRED,
            strike,
            style: if put { OptionStyle::Put } else { OptionStyle::Call },
            side,
            spot_prices: Some(prices),
            spot_min: None,
            spot_max: None,
        };
        let _ = OptionType::Asian { averaging_type }.payoff(&info);
    }

    /// Conversions between every pair of frames, including custom frames
    /// with no unit in a year.
    #[test]
    fn test_convert_time_frame_never_panics(
        value in extreme_positive(),
        from in extreme_time_frame(),
        to in extreme_time_frame(),
    ) {
        let _ = convert_time_frame(value, &from, &to);
    }

    /// Day offsets across the whole `i64` range.
    #[test]
    fn test_get_x_days_formatted_never_panics(days in any::<i64>()) {
        let _ = get_x_days_formatted(days);
    }

    /// Price grids of every size, including the empty and single-point ones.
    #[test]
    fn test_generate_price_points_never_panics(
        low in extreme_decimal(),
        high in extreme_decimal(),
        num_points in 0usize..6,
    ) {
        let _ = generate_price_points(low, high, num_points);
    }
}
