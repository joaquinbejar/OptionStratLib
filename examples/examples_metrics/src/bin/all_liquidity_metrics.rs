/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # All Liquidity Metrics Example
//!
//! This comprehensive example demonstrates all liquidity metrics available in
//! OptionStratLib:
//! - Bid-Ask Spread Curve (by strike)
//! - Volume Profile Curve (by strike) and Surface (strike vs time)
//! - Open Interest Distribution Curve (by strike)
//!
//! ## Output
//! All graphs are saved to `./Draws/Metrics/`

use optionstratlib::chains::chain::OptionChain;
use optionstratlib::chains::OptionData;
use optionstratlib::error::CurveError;
use optionstratlib::metrics::{
    BidAskSpreadCurve, OpenInterestCurve, VolumeProfileCurve, VolumeProfileSurface,
};
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    tracing::info!("=== OptionStratLib Liquidity Metrics Demo ===\n");

    // Build a synthetic option chain with full liquidity data
    let mut chain = OptionChain::new(
        "SPY",
        pos_or_panic!(450.0),
        "2024-12-31".to_string(),
        None,
        None,
    );

    // Add options with comprehensive liquidity data
    let strikes_data = [
        // (strike, call_bid, call_ask, put_bid, put_ask, volume, open_interest)
        (
            pos_or_panic!(380.0),
            pos_or_panic!(71.0),
            pos_or_panic!(73.0),
            pos_or_panic!(0.5),
            Positive::ONE,
            pos_or_panic!(500.0),
            2500u64,
        ),
        (
            pos_or_panic!(400.0),
            pos_or_panic!(52.0),
            pos_or_panic!(53.5),
            pos_or_panic!(1.5),
            Positive::TWO,
            pos_or_panic!(1500.0),
            15000u64,
        ),
        (
            pos_or_panic!(420.0),
            pos_or_panic!(33.0),
            pos_or_panic!(33.8),
            pos_or_panic!(3.5),
            pos_or_panic!(4.0),
            pos_or_panic!(3500.0),
            8000u64,
        ),
        (
            pos_or_panic!(430.0),
            pos_or_panic!(24.0),
            pos_or_panic!(24.5),
            pos_or_panic!(5.5),
            pos_or_panic!(6.0),
            pos_or_panic!(5000.0),
            6000u64,
        ),
        (
            pos_or_panic!(440.0),
            pos_or_panic!(16.0),
            pos_or_panic!(16.3),
            pos_or_panic!(9.0),
            pos_or_panic!(9.3),
            pos_or_panic!(8000.0),
            12000u64,
        ),
        (
            pos_or_panic!(445.0),
            pos_or_panic!(12.5),
            pos_or_panic!(12.7),
            pos_or_panic!(11.5),
            pos_or_panic!(11.7),
            pos_or_panic!(12000.0),
            9000u64,
        ),
        (
            pos_or_panic!(450.0),
            pos_or_panic!(9.5),
            pos_or_panic!(9.6),
            pos_or_panic!(14.5),
            pos_or_panic!(14.6),
            pos_or_panic!(18000.0),
            45000u64,
        ),
        (
            pos_or_panic!(455.0),
            pos_or_panic!(7.0),
            pos_or_panic!(7.2),
            pos_or_panic!(18.0),
            pos_or_panic!(18.2),
            pos_or_panic!(14000.0),
            11000u64,
        ),
        (
            pos_or_panic!(460.0),
            pos_or_panic!(5.0),
            pos_or_panic!(5.2),
            pos_or_panic!(22.0),
            pos_or_panic!(22.3),
            pos_or_panic!(9000.0),
            14000u64,
        ),
        (
            pos_or_panic!(470.0),
            pos_or_panic!(2.5),
            pos_or_panic!(2.8),
            pos_or_panic!(30.0),
            pos_or_panic!(30.5),
            pos_or_panic!(5500.0),
            7500u64,
        ),
        (
            pos_or_panic!(480.0),
            pos_or_panic!(1.2),
            pos_or_panic!(1.5),
            pos_or_panic!(40.0),
            pos_or_panic!(41.0),
            pos_or_panic!(3000.0),
            10000u64,
        ),
        (
            pos_or_panic!(500.0),
            pos_or_panic!(0.3),
            pos_or_panic!(0.6),
            pos_or_panic!(58.0),
            pos_or_panic!(60.0),
            pos_or_panic!(1200.0),
            25000u64,
        ),
        (
            pos_or_panic!(520.0),
            pos_or_panic!(0.1),
            pos_or_panic!(0.4),
            pos_or_panic!(75.0),
            pos_or_panic!(78.0),
            pos_or_panic!(400.0),
            5000u64,
        ),
    ];

    for (strike, call_bid, call_ask, put_bid, put_ask, volume, oi) in strikes_data {
        let option_data = OptionData::new(
            strike,
            spos!(call_bid.to_f64()),
            spos!(call_ask.to_f64()),
            spos!(put_bid.to_f64()),
            spos!(put_ask.to_f64()),
            pos_or_panic!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(volume.to_f64()),
            Some(oi),
            Some("SPY".to_string()),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(Box::new(pos_or_panic!(450.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);
    }

    tracing::info!("Option Chain Information:");
    tracing::info!("  Symbol: {}", chain.symbol);
    tracing::info!("  Underlying Price: {}", chain.underlying_price);
    tracing::info!("  Number of strikes: {}", chain.options.len());
    tracing::info!("");

    // ========================================
    // 1. Bid-Ask Spread Curve
    // ========================================
    tracing::info!("1. BID-ASK SPREAD CURVE");
    tracing::info!("   Shows liquidity across strikes (tighter = more liquid)");

    let spread_curve = chain.bid_ask_spread_curve()?;
    tracing::info!("   Generated {} curve points", spread_curve.points.len());

    let points: Vec<_> = spread_curve.points.iter().collect();
    if let Some(min) = points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "   Tightest spread: {:.2}% at strike {}",
            min.y * dec!(100),
            min.x
        );
    }

    spread_curve
        .plot()
        .title("Bid-Ask Spread Curve (SPY)")
        .x_label("Strike Price")
        .y_label("Relative Spread (%)")
        .save("./Draws/Metrics/bid_ask_spread_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    spread_curve
        .write_html("./Draws/Metrics/bid_ask_spread_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: bid_ask_spread_curve.png/html\n");

    // ========================================
    // 2. Volume Profile Curve
    // ========================================
    tracing::info!("2. VOLUME PROFILE CURVE");
    tracing::info!("   Shows trading activity distribution by strike");

    let volume_curve = chain.volume_profile_curve()?;
    tracing::info!("   Generated {} curve points", volume_curve.points.len());

    let points: Vec<_> = volume_curve.points.iter().collect();
    if let Some(max) = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!("   Highest volume: {} contracts at strike {}", max.y, max.x);
    }

    volume_curve
        .plot()
        .title("Volume Profile Curve (SPY)")
        .x_label("Strike Price")
        .y_label("Volume (contracts)")
        .save("./Draws/Metrics/volume_profile_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    volume_curve
        .write_html("./Draws/Metrics/volume_profile_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: volume_profile_curve.png/html\n");

    // ========================================
    // 3. Volume Profile Surface
    // ========================================
    tracing::info!("3. VOLUME PROFILE SURFACE");
    tracing::info!("   Shows volume evolution across strike and time");

    let days = vec![
        Positive::ONE,
        pos_or_panic!(5.0),
        pos_or_panic!(10.0),
        pos_or_panic!(15.0),
        pos_or_panic!(20.0),
        pos_or_panic!(25.0),
        pos_or_panic!(30.0),
    ];

    let volume_surface = chain
        .volume_profile_surface(days)
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!(
        "   Generated {} surface points",
        volume_surface.points.len()
    );

    volume_surface
        .plot()
        .title("Volume Profile Surface (SPY)")
        .x_label("Strike Price")
        .y_label("Days to Expiration")
        .z_label("Volume")
        .dimensions(1600, 1200)
        .save("./Draws/Metrics/volume_profile_surface.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    volume_surface
        .write_html("./Draws/Metrics/volume_profile_surface.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: volume_profile_surface.png/html\n");

    // ========================================
    // 4. Open Interest Distribution Curve
    // ========================================
    tracing::info!("4. OPEN INTEREST DISTRIBUTION CURVE");
    tracing::info!("   Shows outstanding contracts by strike (max pain indicator)");

    let oi_curve = chain.open_interest_curve()?;
    tracing::info!("   Generated {} curve points", oi_curve.points.len());

    let points: Vec<_> = oi_curve.points.iter().collect();
    if let Some(max) = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "   Maximum OI: {} contracts at strike {} (potential max pain)",
            max.y,
            max.x
        );
    }

    oi_curve
        .plot()
        .title("Open Interest Distribution (SPY)")
        .x_label("Strike Price")
        .y_label("Open Interest (contracts)")
        .save("./Draws/Metrics/open_interest_curve.png")
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    oi_curve
        .write_html("./Draws/Metrics/open_interest_curve.html".as_ref())
        .map_err(|e| CurveError::ConstructionError(e.to_string()))?;

    tracing::info!("   Saved: open_interest_curve.png/html\n");

    // ========================================
    // Summary
    // ========================================
    tracing::info!("=== Summary ===");
    tracing::info!("All liquidity metrics generated successfully!");
    tracing::info!("Output directory: ./Draws/Metrics/");
    tracing::info!("");
    tracing::info!("Files created:");
    tracing::info!("  - bid_ask_spread_curve.png/html");
    tracing::info!("  - volume_profile_curve.png/html");
    tracing::info!("  - volume_profile_surface.png/html");
    tracing::info!("  - open_interest_curve.png/html");

    Ok(())
}
