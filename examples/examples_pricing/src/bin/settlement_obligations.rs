/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 2026-11-05
******************************************************************************/

//! Settlement obligations example.
//!
//! Computes what expiry, exercise and assignment oblige an option position to
//! pay, receive or deliver: a physically settled equity option held on both
//! sides, a cash-settled index option, and the typed errors a caller handles.
//! The function is pure, so the example books nothing.

use chrono::{DateTime, TimeZone, Utc};
use optionstratlib::error::SettlementError;
use optionstratlib::model::types::{OptionStyle, OptionType, Side};
use optionstratlib::model::{
    CashRounding, SettlementEvent, SettlementObligation, SettlementPrice, SettlementTerms,
    SettlementType, settlement_obligation,
};
use optionstratlib::{ExpirationDate, Options};
use positive::{Positive, pos_or_panic};
use rust_decimal_macros::dec;
use tracing::info;

/// A contract with `quantity` contracts of `multiplier` units each.
fn contract(side: Side, style: OptionStyle, quantity: f64, multiplier: f64) -> Options {
    Options::new(
        OptionType::European,
        side,
        "XYZ".to_string(),
        pos_or_panic!(100.0),
        ExpirationDate::Days(pos_or_panic!(1.0)),
        pos_or_panic!(0.2),
        pos_or_panic!(quantity),
        pos_or_panic!(100.0),
        dec!(0.05),
        style,
        Positive::ZERO,
        None,
    )
    .with_contract_size(pos_or_panic!(multiplier))
}

fn terms(
    settlement_type: SettlementType,
    event: SettlementEvent,
    price: Option<f64>,
    cutoff: DateTime<Utc>,
) -> SettlementTerms {
    SettlementTerms {
        settlement_type,
        event,
        price: price.map(|price| SettlementPrice {
            price: pos_or_panic!(price),
            source: "exchange closing auction".to_string(),
            observed_at: cutoff,
        }),
        cutoff,
        rounding: CashRounding { scale: 2 },
    }
}

fn report(label: &str, obligation: &SettlementObligation) {
    info!(
        extinguished_contracts = %obligation.extinguished_quantity,
        cash = %obligation.cash,
        underlying = %obligation.underlying,
        "{label}"
    );
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .without_time()
        .init();

    let cutoff = Utc
        .with_ymd_and_hms(2026, 12, 18, 21, 0, 0)
        .single()
        .ok_or("the cutoff is not a valid instant")?;

    info!("=== Physical settlement at expiry: strike 100, settlement 110 ===");
    let expiry = terms(
        SettlementType::Physical,
        SettlementEvent::Expiry,
        Some(110.0),
        cutoff,
    );
    for (label, side, style) in [
        ("long call", Side::Long, OptionStyle::Call),
        ("short call", Side::Short, OptionStyle::Call),
        (
            "short put (out of the money)",
            Side::Short,
            OptionStyle::Put,
        ),
    ] {
        let option = contract(side, style, 2.0, 100.0);
        report(label, &settlement_obligation(&option, &expiry)?);
    }

    info!("=== Explicit physical events need no settlement price ===");
    let exercise = terms(
        SettlementType::Physical,
        SettlementEvent::Exercise,
        None,
        cutoff,
    );
    let long_put = contract(Side::Long, OptionStyle::Put, 2.0, 100.0);
    report(
        "long put, exercised",
        &settlement_obligation(&long_put, &exercise)?,
    );

    info!("=== Cash settlement: multiplier 50, settlement 103.333 rounded to cents ===");
    let cash = terms(
        SettlementType::Cash,
        SettlementEvent::Expiry,
        Some(103.333),
        cutoff,
    );
    let long_index_call = contract(Side::Long, OptionStyle::Call, 1.0, 50.0);
    report(
        "long index call",
        &settlement_obligation(&long_index_call, &cash)?,
    );

    info!("=== Typed errors ===");
    let no_price = terms(SettlementType::Cash, SettlementEvent::Expiry, None, cutoff);
    match settlement_obligation(&long_index_call, &no_price) {
        Err(error @ SettlementError::MissingPrice { .. }) => info!(%error, "as expected"),
        other => return Err(format!("expected a missing price, got {other:?}").into()),
    }
    let short_call = contract(Side::Short, OptionStyle::Call, 1.0, 100.0);
    match settlement_obligation(&short_call, &exercise) {
        Err(error @ SettlementError::EventSideMismatch { .. }) => info!(%error, "as expected"),
        other => return Err(format!("expected an event mismatch, got {other:?}").into()),
    }

    Ok(())
}
