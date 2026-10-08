/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 19/9/25
******************************************************************************/

//! [`Graph`] implementations for the core model types.
//!
//! An `Options` contract plots its payoff at expiry across an optimal price
//! range around the strike, split into a positive and a negative series; a
//! `Position` delegates to its underlying contract. The implementations live
//! in the visualization layer (local trait, core-owned type) so the core
//! model never references chart types.

use crate::visualization::{
    ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Series2D, TraceMode,
};
use optionstratlib_core::model::Positive;
use optionstratlib_core::model::{Options, Position};
use optionstratlib_market::chains::utils::calculate_optimal_price_range;
use optionstratlib_strategies::strategies::base::BasicAble;
use rust_decimal::Decimal;

impl Graph for Options {
    fn graph_data(&self) -> GraphData {
        let range = match calculate_optimal_price_range(
            self.underlying_price,
            self.strike_price,
            self.implied_volatility,
            self.expiration_date,
        ) {
            Ok(range) => range,
            Err(_) => {
                // Fallback to a reasonable default range based on strike price
                let lower = self.strike_price.checked_mul_dec(Decimal::new(5, 1));
                let upper = self.strike_price.checked_mul_dec(Decimal::new(15, 1));
                match (lower, upper) {
                    (Ok(lower), Ok(upper)) => (lower, upper),
                    // A strike this close to `Positive::MAX` has no range
                    // around it to chart.
                    _ => return GraphData::Series(Series2D::default()),
                }
            }
        };

        let mut positive_series = Series2D {
            x: vec![],
            y: vec![],
            name: "Positive Payoff".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#2ca02c".to_string()),
            line_width: Some(2.0),
        };
        let mut negative_series = Series2D {
            x: vec![],
            y: vec![],
            name: "Negative Payoff".to_string(),
            mode: TraceMode::Lines,
            line_color: Some("#FF0000".to_string()),
            line_width: Some(2.0),
        };

        let (range_start, range_end) = (
            range.0.to_u64_checked().unwrap_or_default(),
            range.1.to_u64_checked().unwrap_or_default(),
        );
        for i in range_start..range_end {
            // Every `u64` is a valid `Positive`; the `else` is never taken.
            let Ok(price) = Positive::try_from(i) else {
                continue;
            };
            // A price whose payoff cannot be computed is left out of the
            // chart rather than drawn at zero.
            let Ok(profit) = self.payoff_at_price(&price) else {
                continue;
            };
            match profit {
                p if p == Decimal::ZERO => {
                    positive_series.x.push(Decimal::from(i));
                    positive_series.y.push(profit);
                    negative_series.x.push(Decimal::from(i));
                    negative_series.y.push(profit);
                }
                p if p > Decimal::ZERO => {
                    positive_series.x.push(Decimal::from(i));
                    positive_series.y.push(profit);
                }
                _ => {
                    negative_series.x.push(Decimal::from(i));
                    negative_series.y.push(profit);
                }
            }
        }
        let multi_series_2d = vec![positive_series, negative_series];
        GraphData::MultiSeries(multi_series_2d)
    }

    fn graph_config(&self) -> GraphConfig {
        let title = self.get_title();
        let legend = Some(vec![title.clone()]);
        GraphConfig {
            title,
            width: 1600,
            height: 900,
            x_label: Some("Underlying Price".to_string()),
            y_label: Some("Profit/Loss".to_string()),
            z_label: None,
            line_style: LineStyle::Solid,
            color_scheme: ColorScheme::Default,
            legend,
            show_legend: false,
        }
    }
}

/// Implementation of the `Graph` trait for the `Position` struct, enabling graphical representation
/// of financial options positions.
///
/// This implementation provides methods to visualize the profit/loss (PnL) profile of an options position
/// across different price levels of the underlying asset. It handles the generation of appropriate title,
/// data values for plotting, and special chart elements like break-even points.
///
/// The visualization capabilities allow traders to analyze the potential outcomes of their options positions
/// at expiration across various price scenarios.
impl Graph for Position {
    fn graph_data(&self) -> GraphData {
        self.option.graph_data()
    }

    fn graph_config(&self) -> GraphConfig {
        self.option.graph_config()
    }
}
