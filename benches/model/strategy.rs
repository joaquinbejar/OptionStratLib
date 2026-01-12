/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 12/1/26
******************************************************************************/

use criterion::Criterion;
use optionstratlib::ExpirationDate;
use optionstratlib::strategies::Strategies;
use optionstratlib::strategies::bull_call_spread::BullCallSpread;
use optionstratlib::strategies::iron_butterfly::IronButterfly;
use optionstratlib::strategies::iron_condor::IronCondor;
use optionstratlib::strategies::long_call::LongCall;
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;
use std::hint::black_box;

fn create_long_call() -> LongCall {
    LongCall::new(
        "AAPL".to_string(),
        pos_or_panic!(155.0),                      // long_call_strike
        ExpirationDate::Days(pos_or_panic!(30.0)), // long_call_expiration
        pos_or_panic!(0.2),                        // implied_volatility
        Positive::ONE,                             // quantity
        pos_or_panic!(150.0),                      // underlying_price
        dec!(0.01),                                // risk_free_rate
        pos_or_panic!(0.02),                       // dividend_yield
        pos_or_panic!(5.0),                        // premium_long_call
        pos_or_panic!(0.5),                        // open_fee_long_call
        pos_or_panic!(0.5),                        // close_fee_long_call
    )
}

fn create_bull_call_spread() -> BullCallSpread {
    BullCallSpread::new(
        "AAPL".to_string(),
        pos_or_panic!(150.0),                      // underlying_price
        pos_or_panic!(145.0),                      // long_strike
        pos_or_panic!(155.0),                      // short_strike
        ExpirationDate::Days(pos_or_panic!(30.0)), // expiration
        pos_or_panic!(0.2),                        // implied_volatility
        dec!(0.01),                                // risk_free_rate
        pos_or_panic!(0.02),                       // dividend_yield
        Positive::ONE,                             // quantity
        pos_or_panic!(8.0),                        // premium_long_call
        pos_or_panic!(3.0),                        // premium_short_call
        pos_or_panic!(0.5),                        // open_fee_long_call
        pos_or_panic!(0.5),                        // close_fee_long_call
        pos_or_panic!(0.5),                        // open_fee_short_call
        pos_or_panic!(0.5),                        // close_fee_short_call
    )
}

fn create_iron_condor() -> IronCondor {
    IronCondor::new(
        "AAPL".to_string(),
        pos_or_panic!(150.0),                      // underlying_price
        pos_or_panic!(155.0),                      // short_call_strike
        pos_or_panic!(145.0),                      // short_put_strike
        pos_or_panic!(160.0),                      // long_call_strike
        pos_or_panic!(140.0),                      // long_put_strike
        ExpirationDate::Days(pos_or_panic!(30.0)), // expiration
        pos_or_panic!(0.2),                        // implied_volatility
        dec!(0.01),                                // risk_free_rate
        pos_or_panic!(0.02),                       // dividend_yield
        Positive::ONE,                             // quantity
        pos_or_panic!(1.5),                        // premium_short_call
        pos_or_panic!(1.5),                        // premium_short_put
        pos_or_panic!(1.0),                        // premium_long_call
        pos_or_panic!(1.0),                        // premium_long_put
        pos_or_panic!(0.5),                        // open_fee
        pos_or_panic!(0.5),                        // close_fee
    )
}

fn create_iron_butterfly() -> IronButterfly {
    IronButterfly::new(
        "AAPL".to_string(),
        pos_or_panic!(150.0),                      // underlying_price
        pos_or_panic!(150.0),                      // short_strike
        pos_or_panic!(155.0),                      // long_call_strike
        pos_or_panic!(145.0),                      // long_put_strike
        ExpirationDate::Days(pos_or_panic!(30.0)), // expiration
        pos_or_panic!(0.2),                        // implied_volatility
        dec!(0.01),                                // risk_free_rate
        pos_or_panic!(0.02),                       // dividend_yield
        Positive::ONE,                             // quantity
        pos_or_panic!(2.5),                        // premium_short_call
        pos_or_panic!(2.5),                        // premium_short_put
        pos_or_panic!(1.0),                        // premium_long_call
        pos_or_panic!(1.0),                        // premium_long_put
        pos_or_panic!(0.5),                        // open_fee
        pos_or_panic!(0.5),                        // close_fee
    )
}

pub(crate) fn benchmark_strategies(c: &mut Criterion) {
    let mut group = c.benchmark_group("Strategies");
    let long_call = create_long_call();
    let bull_call_spread = create_bull_call_spread();
    let iron_condor = create_iron_condor();
    let iron_butterfly = create_iron_butterfly();

    group.bench_function("long_call_max_profit", |b| {
        b.iter(|| black_box(long_call.get_max_profit()))
    });
    group.bench_function("long_call_max_loss", |b| {
        b.iter(|| black_box(long_call.get_max_loss()))
    });

    group.bench_function("bull_call_spread_max_profit", |b| {
        b.iter(|| black_box(bull_call_spread.get_max_profit()))
    });
    group.bench_function("bull_call_spread_max_loss", |b| {
        b.iter(|| black_box(bull_call_spread.get_max_loss()))
    });

    group.bench_function("iron_condor_max_profit", |b| {
        b.iter(|| black_box(iron_condor.get_max_profit()))
    });
    group.bench_function("iron_condor_max_loss", |b| {
        b.iter(|| black_box(iron_condor.get_max_loss()))
    });

    group.bench_function("iron_butterfly_max_profit", |b| {
        b.iter(|| black_box(iron_butterfly.get_max_profit()))
    });
    group.bench_function("iron_butterfly_max_loss", |b| {
        b.iter(|| black_box(iron_butterfly.get_max_loss()))
    });

    group.finish();
}
