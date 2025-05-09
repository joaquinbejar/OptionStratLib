use crate::Positive;
use crate::pricing::Profit;
use crate::strategies::{
    BasicAble, BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly,
    CustomStrategy, IronButterfly, IronCondor, LongButterflySpread, LongCall, LongPut,
    LongStraddle, LongStrangle, PoorMansCoveredCall, ShortButterflySpread, ShortCall, ShortPut,
    ShortStraddle, ShortStrangle, Strategies,
};
use crate::visualization::{
    ColorScheme, Graph, GraphConfig, GraphData, LineStyle, Series2D, TraceMode,
};
use rust_decimal::Decimal;


/// Macro `impl_graph_for_payoff_strategy` generates implementations of the `Graph` trait
/// for one or more given types. This is specifically designed for types that represent
/// payoff strategies, enabling them to produce graph data and configurations for financial
/// visualizations, such as profit/loss graphs based on an underlying price range.
/// 
/// # Usage
/// 
/// Use the macro to implement the `Graph` trait for multiple types. For example:
/// 

/// This will generate implementations of the `Graph` trait for `TypeA` and `TypeB`.
/// 
/// # Generated Trait Implementation
/// 
/// ## `graph_data`
/// 
/// - Generates `GraphData` for the associated type by calculating the profit/loss data for
///   a price range obtained from the `get_best_range_to_show` method.
/// - The result is displayed as two series:
///   1. A "Positive Payoff" series (green) for profits.
///   2. A "Negative Payoff" series (red) for losses.
/// - Handles profits, losses, and break-even points separately:
///   - Break-even points (`0`) are shared between positive and negative series.
///   - Prices where profit is positive are included in the positive series only.
///   - Prices where profit is negative are included in the negative series only.
/// - If `get_best_range_to_show` fails, a default empty `GraphData::Series` is returned.
/// 
/// ## `graph_config`
/// 
/// - Produces a `GraphConfig` structure for styling and configuring the graph:
///   - Title: Based on the strategy's title via `get_title` method.
///   - Width and height: Predefined dimensions (1600x900).
///   - Labels: X-axis represents "Underlying Price", and Y-axis represents "Profit/Loss".
///   - Other visual configuration properties include:
///     - `line_style`: Set to `LineStyle::Solid`.
///     - `color_scheme`: Set to default.
///     - `legend`: Title is added to the legend, but `show_legend` is disabled by default.
/// 
/// # Parameters
/// 
/// - `$t`: One or more types for which the `Graph` trait should be implemented.
/// 
/// 
/// This allows the `MyStrategy` type to generate graph data and configurations for visual representation
/// of its payoff strategy.
/// 
/// # Notes
/// 
/// - The macro assumes that the types implemented provide specific methods (`get_best_range_to_show`,
///   `calculate_profit_at`, and `get_title`) necessary for the `Graph` trait.
/// - Predefined values like colors, line widths, and dimensions can be adjusted within the macro if needed.
/// 
#[macro_export]
macro_rules! impl_graph_for_payoff_strategy {
    ($($t:ty),*) => {
        $(
            impl Graph for $t {
                fn graph_data(&self) -> GraphData {
                    let range = match self.get_best_range_to_show(Positive::ONE){
                        Ok(range) => range,
                        Err(_) => return GraphData::Series(Series2D::default()),
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


                    for price in range {
                        let profit = self
                            .calculate_profit_at(&price)
                            .unwrap();
                        match profit {
                            p if p == Decimal::ZERO => {
                                positive_series.x.push(price.to_dec());
                                positive_series.y.push(profit);
                                negative_series.x.push(price.to_dec());
                                negative_series.y.push(profit);
                            }
                            p if p > Decimal::ZERO => {
                                positive_series.x.push(price.to_dec());
                                positive_series.y.push(profit);
                            }
                            _ => {
                                negative_series.x.push(price.to_dec());
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
        )*
    };
}

// Uso
impl_graph_for_payoff_strategy!(
    BullCallSpread,
    BearCallSpread,
    BullPutSpread,
    BearPutSpread,
    LongButterflySpread,
    ShortButterflySpread,
    IronCondor,
    IronButterfly,
    LongStraddle,
    ShortStraddle,
    LongStrangle,
    ShortStrangle,
    LongCall,
    LongPut,
    ShortCall,
    ShortPut,
    PoorMansCoveredCall,
    CallButterfly,
    CustomStrategy
);
