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

use crate::chains::utils::calculate_optimal_price_range;
use crate::model::{Options, Position};
use crate::strategies::base::BasicAble;
use crate::visualization::{
    ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Series2D, TraceMode,
};
use num_traits::FromPrimitive;
use positive::Positive;
use rust_decimal::Decimal;

impl Graph for Options {
    fn graph_data(&self) -> GraphData {
        let range = calculate_optimal_price_range(
            self.underlying_price,
            self.strike_price,
            self.implied_volatility,
            self.expiration_date,
        )
        .unwrap_or_else(|_| {
            // Fallback to a reasonable default range based on strike price
            let lower = self.strike_price * Positive::new(0.5).unwrap_or(Positive::ONE);
            let upper = self.strike_price * Positive::new(1.5).unwrap_or(Positive::ONE);
            (lower, upper)
        });

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
            let profit = self
                .payoff_at_price(&Positive::new(i as f64).unwrap_or(Positive::ONE))
                .unwrap_or_default();
            match profit {
                p if p == Decimal::ZERO => {
                    positive_series
                        .x
                        .push(Decimal::from_u64(i).unwrap_or_default());
                    positive_series.y.push(profit);
                    negative_series
                        .x
                        .push(Decimal::from_u64(i).unwrap_or_default());
                    negative_series.y.push(profit);
                }
                p if p > Decimal::ZERO => {
                    positive_series
                        .x
                        .push(Decimal::from_u64(i).unwrap_or_default());
                    positive_series.y.push(profit);
                }
                _ => {
                    negative_series
                        .x
                        .push(Decimal::from_u64(i).unwrap_or_default());
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
