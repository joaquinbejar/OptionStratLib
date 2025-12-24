/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 24/12/25
******************************************************************************/

//! # Bid-Ask Spread Curve Example
//!
//! This example demonstrates how to generate and visualize a bid-ask spread
//! curve from an option chain. The curve shows liquidity across different strikes.
//!
//! ## Output
//! - PNG image: `./Draws/Metrics/bid_ask_spread_curve.png`
//! - HTML interactive: `./Draws/Metrics/bid_ask_spread_curve.html`

use optionstratlib::chains::OptionData;
use optionstratlib::chains::chain::OptionChain;
use optionstratlib::error::CurveError;
use optionstratlib::metrics::BidAskSpreadCurve;
use optionstratlib::model::ExpirationDate;
use optionstratlib::prelude::*;
use rust_decimal_macros::dec;

fn main() -> Result<(), CurveError> {
    setup_logger();

    // Build a synthetic option chain with bid/ask data
    let mut chain = OptionChain::new("SPY", pos!(450.0), "2024-12-31".to_string(), None, None);

    // Add options with realistic bid/ask spreads
    // Spreads are tighter near ATM and wider for OTM options
    let strikes_data = [
        // (strike, call_bid, call_ask, put_bid, put_ask)
        (pos!(380.0), pos!(71.0), pos!(73.0), pos!(0.5), pos!(1.0)),
        (pos!(400.0), pos!(52.0), pos!(53.5), pos!(1.5), pos!(2.0)),
        (pos!(420.0), pos!(33.0), pos!(33.8), pos!(3.5), pos!(4.0)),
        (pos!(430.0), pos!(24.0), pos!(24.5), pos!(5.5), pos!(6.0)),
        (pos!(440.0), pos!(16.0), pos!(16.3), pos!(9.0), pos!(9.3)),
        (pos!(445.0), pos!(12.5), pos!(12.7), pos!(11.5), pos!(11.7)),
        (pos!(450.0), pos!(9.5), pos!(9.6), pos!(14.5), pos!(14.6)), // ATM - tightest
        (pos!(455.0), pos!(7.0), pos!(7.2), pos!(18.0), pos!(18.2)),
        (pos!(460.0), pos!(5.0), pos!(5.2), pos!(22.0), pos!(22.3)),
        (pos!(470.0), pos!(2.5), pos!(2.8), pos!(30.0), pos!(30.5)),
        (pos!(480.0), pos!(1.2), pos!(1.5), pos!(40.0), pos!(41.0)),
        (pos!(500.0), pos!(0.3), pos!(0.6), pos!(58.0), pos!(60.0)),
        (pos!(520.0), pos!(0.1), pos!(0.4), pos!(75.0), pos!(78.0)),
    ];

    for (strike, call_bid, call_ask, put_bid, put_ask) in strikes_data {
        let option_data = OptionData::new(
            strike,
            spos!(call_bid.to_f64()),
            spos!(call_ask.to_f64()),
            spos!(put_bid.to_f64()),
            spos!(put_ask.to_f64()),
            pos!(0.20),
            Some(dec!(0.5)),
            Some(dec!(-0.5)),
            Some(dec!(0.05)),
            spos!(1000.0),
            Some(5000),
            Some("SPY".to_string()),
            Some(ExpirationDate::Days(pos!(30.0))),
            Some(Box::new(pos!(450.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            None,
            None,
        );
        chain.options.insert(option_data);
    }

    tracing::info!(
        "Built option chain for {} with {} options",
        chain.symbol,
        chain.options.len()
    );
    tracing::info!("Underlying price: {}", chain.underlying_price);

    // Generate the bid-ask spread curve
    tracing::info!("Generating Bid-Ask Spread Curve...");

    let spread_curve = chain.bid_ask_spread_curve()?;

    tracing::info!(
        "Bid-Ask Spread Curve generated with {} points",
        spread_curve.points.len()
    );

    // Analyze spread pattern
    let points: Vec<_> = spread_curve.points.iter().collect();

    // Find tightest spread (most liquid)
    if let Some(min_spread) = points.iter().min_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "Tightest spread: {:.2}% at strike {}",
            min_spread.y * dec!(100),
            min_spread.x
        );
    }

    // Find widest spread (least liquid)
    if let Some(max_spread) = points.iter().max_by(|a, b| a.y.partial_cmp(&b.y).unwrap()) {
        tracing::info!(
            "Widest spread: {:.2}% at strike {}",
            max_spread.y * dec!(100),
            max_spread.x
        );
    }

    // Plot and save the curve
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

    tracing::info!("Bid-Ask Spread curve saved to ./Draws/Metrics/bid_ask_spread_curve.png");
    tracing::info!("Interactive HTML saved to ./Draws/Metrics/bid_ask_spread_curve.html");

    Ok(())
}
