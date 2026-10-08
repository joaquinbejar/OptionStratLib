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

/// Most prices the single-contract payoff chart evaluates.
///
/// Up to this many, the chart steps one unit of the underlying at a time
/// across the range, as it always has. A wider range (a strike in the
/// millions or beyond, whose range is millions of units wide) is sampled at
/// this many evenly spaced prices instead, so the chart stays bounded rather
/// than looping once per unit (#797). Ten thousand points is several times
/// the 1600-pixel width the chart is drawn at.
const MAX_OPTION_CHART_POINTS: u64 = 10_000;

/// The prices the payoff chart evaluates over `[start, end)`, in units of
/// the underlying: every integer while there are at most
/// [`MAX_OPTION_CHART_POINTS`] of them, otherwise that many evenly spaced
/// prices from `start`. An empty or inverted range has none.
fn chart_prices(start: u64, end: u64) -> Vec<Decimal> {
    let Some(width) = end.checked_sub(start) else {
        return Vec::new();
    };
    if width <= MAX_OPTION_CHART_POINTS {
        return (start..end).map(Decimal::from).collect();
    }
    let first = Decimal::from(start);
    let Some(step) = Decimal::from(width).checked_div(Decimal::from(MAX_OPTION_CHART_POINTS))
    else {
        return Vec::new();
    };
    (0..MAX_OPTION_CHART_POINTS)
        .filter_map(|index| first.checked_add(step.checked_mul(Decimal::from(index))?))
        .collect()
}

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
        for x in chart_prices(range_start, range_end) {
            // Every price in the range is non-negative; the `else` is never
            // taken.
            let Ok(price) = Positive::new_decimal(x) else {
                continue;
            };
            // A price whose payoff cannot be computed is left out of the
            // chart rather than drawn at zero.
            let Ok(profit) = self.payoff_at_price(&price) else {
                continue;
            };
            match profit {
                p if p == Decimal::ZERO => {
                    positive_series.x.push(x);
                    positive_series.y.push(profit);
                    negative_series.x.push(x);
                    negative_series.y.push(profit);
                }
                p if p > Decimal::ZERO => {
                    positive_series.x.push(x);
                    positive_series.y.push(profit);
                }
                _ => {
                    negative_series.x.push(x);
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

#[cfg(test)]
mod tests_option_chart_sampling {
    use super::*;
    use optionstratlib_core::model::ExpirationDate;
    use optionstratlib_core::model::types::{OptionStyle, OptionType, Side};
    use rust_decimal_macros::dec;

    #[test]
    fn test_chart_prices_steps_one_unit_up_to_the_cap() {
        let prices = chart_prices(70, 130);
        assert_eq!(prices, (70..130).map(Decimal::from).collect::<Vec<_>>());

        let at_cap = chart_prices(0, MAX_OPTION_CHART_POINTS);
        assert_eq!(at_cap.len(), MAX_OPTION_CHART_POINTS as usize);
        assert_eq!(
            at_cap.last(),
            Some(&Decimal::from(MAX_OPTION_CHART_POINTS - 1))
        );
    }

    #[test]
    fn test_chart_prices_samples_a_wide_range_evenly() {
        let start = 700_000_000_000_000_u64;
        let end = 1_300_000_000_000_000_u64;
        let prices = chart_prices(start, end);
        assert_eq!(prices.len(), MAX_OPTION_CHART_POINTS as usize);
        assert_eq!(prices.first(), Some(&Decimal::from(start)));
        assert!(
            prices
                .windows(2)
                .all(|pair| matches!(pair, [a, b] if a < b))
        );
        assert!(prices.iter().all(|price| *price < Decimal::from(end)));
    }

    #[test]
    fn test_chart_prices_empty_or_inverted_range_has_none() {
        assert!(chart_prices(10, 10).is_empty());
        assert!(chart_prices(10, 5).is_empty());
    }

    /// A strike of 1e15 used to loop once per unit across a range about
    /// 6e14 units wide (#797). It now returns a bounded series.
    #[test]
    fn test_option_graph_data_with_a_huge_strike_is_bounded() {
        let huge = Positive::new_decimal(dec!(1000000000000000)).unwrap();
        let option = Options::new(
            OptionType::European,
            Side::Long,
            "HUGE".to_string(),
            huge,
            ExpirationDate::Days(Positive::new(30.0).unwrap()),
            Positive::new(0.2).unwrap(),
            Positive::ONE,
            huge,
            dec!(0.05),
            OptionStyle::Call,
            Positive::ZERO,
            None,
        );

        let started = std::time::Instant::now();
        let GraphData::MultiSeries(series) = option.graph_data() else {
            panic!("expected the positive and negative payoff series");
        };
        assert!(started.elapsed() < std::time::Duration::from_secs(30));

        let points: usize = series.iter().map(|s| s.x.len()).sum();
        assert!(points > 0);
        // A zero payoff is drawn in both series, so the bound is twice the cap.
        assert!(points <= 2 * MAX_OPTION_CHART_POINTS as usize);
    }
}
