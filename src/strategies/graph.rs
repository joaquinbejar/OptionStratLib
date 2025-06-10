use crate::Positive;
use crate::pricing::Profit;
use crate::strategies::{
    BasicAble, BearCallSpread, BearPutSpread, BullCallSpread, BullPutSpread, CallButterfly,
    IronButterfly, IronCondor, LongButterflySpread, LongCall, LongPut, LongStraddle, LongStrangle,
    PoorMansCoveredCall, ShortButterflySpread, ShortCall, ShortPut, ShortStraddle, ShortStrangle,
    Strategies,
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
                    let underlying_price = self.get_underlying_price();
                    let pay_off_at_underlying_price = self.calculate_profit_at(&underlying_price).unwrap_or(Decimal::ZERO);
                    let range = match self.get_best_range_to_show(Positive::ONE){
                        Ok(range) => range,
                        Err(_) => return GraphData::Series(Series2D::default()),
                    };
                    
                    // Create a single continuous series for the profit/loss chart
                    let mut profit_series = Series2D {
                        x: vec![],
                        y: vec![],
                        name: "Profit/Loss".to_string(),
                        mode: TraceMode::Lines,
                        line_color: Some("#2ca02c".to_string()),
                        line_width: Some(2.0),
                    };
                    
                    // Add a zero line for reference
                    let mut zero_line = Series2D {
                        x: vec![],
                        y: vec![],
                        name: "Break Even".to_string(),
                        mode: TraceMode::Lines,
                        line_color: Some("#000000".to_string()),
                        line_width: Some(1.0),
                    };
                    
                    // Add a vertical line at the current underlying price
                    let mut current_price_line = Series2D {
                        x: vec![],
                        y: vec![],
                        name: "Current Price".to_string(),
                        mode: TraceMode::Lines,
                        line_color: Some("#0000FF".to_string()), // Color azul
                        line_width: Some(1.0),
                    };
                    
                    // Add a point at current price with its payoff value
                    // Color depends on payoff value: green (positive), blue (zero), red (negative)
                    let point_color = if pay_off_at_underlying_price > Decimal::ZERO {
                        "#2ca02c".to_string() // Verde para valores positivos
                    } else if pay_off_at_underlying_price < Decimal::ZERO {
                        "#FF0000".to_string() // Rojo para valores negativos
                    } else {
                        "#0000FF".to_string() // Azul para cero
                    };
                    
                    // Format the payoff value with 2 decimals for the label
                    let formatted_payoff = format!("{:.2}", pay_off_at_underlying_price);
                    
                    // Create a point to mark the current position with a larger size
                    // and use the appropriate color based on the payoff value
                    let current_point = Series2D {
                        x: vec![underlying_price.to_dec()],
                        y: vec![pay_off_at_underlying_price],
                        // La etiqueta aparecerá en la leyenda y al pasar el cursor sobre el punto
                        name: format!("Current P/L: {}", formatted_payoff),
                        // Usar LinesMarkers para que se muestre el punto
                        mode: TraceMode::Markers,
                        line_color: Some(point_color.clone()),
                        line_width: Some(10.0), // Punto más grande para mejor visibilidad
                    };
                    
                    // Create an additional series for the visible label on the graph
                    // This series will have a single point with a visible text label
                    // Calculate a vertical offset for the label based on the graph range
                    // so that it appears just above the point
                    let y_offset = if pay_off_at_underlying_price >= Decimal::ZERO {
                        pay_off_at_underlying_price + Decimal::new(100, 0) // Larger positive offset
                    } else {
                        pay_off_at_underlying_price - Decimal::new(100, 0) // Larger negative offset
                    };
                    
                    // Create a series for the label with visible text
                    let label_series = Series2D {
                        x: vec![underlying_price.to_dec()],
                        y: vec![y_offset],
                        // The label will show the payoff value with the corresponding sign
                        name: formatted_payoff,
                        // Use TextLabels mode to display as text
                        mode: TraceMode::TextLabels,
                        // Use the same color as the point for visual consistency
                        line_color: Some(point_color),
                        // Make the point invisible so only the text is visible
                        line_width: Some(0.0),
                    };
                    
                    // Get min and max values for reference lines
                    if !range.is_empty() {
                        let min_price = range.first().unwrap().to_dec();
                        let max_price = range.last().unwrap().to_dec();
                        
                        // Add zero line points (horizontal line at y=0)
                        zero_line.x.push(min_price);
                        zero_line.y.push(Decimal::ZERO);
                        zero_line.x.push(max_price);
                        zero_line.y.push(Decimal::ZERO);
                        
                        // Calculate min and max profit values to determine the vertical range
                        let mut min_profit = Decimal::MAX;
                        let mut max_profit = Decimal::MIN;
                        
                        for price in range.iter() {
                            if let Ok(profit) = self.calculate_profit_at(price) {
                                if profit < min_profit {
                                    min_profit = profit;
                                }
                                if profit > max_profit {
                                    max_profit = profit;
                                }
                            }
                        }
                        
                        // Add vertical line at current underlying price
                        // Extend it slightly beyond min/max profit for visibility
                        let padding = (max_profit - min_profit) * Decimal::new(5, 2); // 5% padding
                        current_price_line.x.push(underlying_price.to_dec());
                        current_price_line.y.push(min_profit - padding);
                        current_price_line.x.push(underlying_price.to_dec());
                        current_price_line.y.push(max_profit + padding);
                    }
                    
                    // Calculate profit at each price point and add to the series
                    for price in range {
                        let profit = self
                            .calculate_profit_at(&price)
                            .unwrap();
                        
                        profit_series.x.push(price.to_dec());
                        profit_series.y.push(profit);
                    }
                    
                    // Crear series separadas para segmentos positivos y negativos, pero asegurando continuidad
                    let mut segments = Vec::new();
                    let mut current_segment = Vec::new();
                    let mut current_sign: Option<i8> = None;
                    
                    // Procesar los puntos para crear segmentos continuos con el mismo signo
                    for (i, price) in profit_series.x.iter().enumerate() {
                        let profit = profit_series.y[i];
                        let sign = if profit > Decimal::ZERO {
                            1
                        } else if profit < Decimal::ZERO {
                            -1
                        } else {
                            0
                        };
                        
                        // Si cambia el signo o es el primer punto
                        if current_sign.is_none() || (sign != 0 && current_sign.unwrap() != sign) {
                            // Si ya hay puntos en el segmento actual, guardarlo
                            if !current_segment.is_empty() {
                                segments.push((current_segment, current_sign.unwrap()));
                                current_segment = Vec::new();
                            }
                            current_sign = Some(sign);
                        }
                        
                        // Añadir el punto al segmento actual
                        current_segment.push((*price, profit));
                    }
                    
                    // Añadir el último segmento si no está vacío
                    if !current_segment.is_empty() && current_sign.is_some() {
                        segments.push((current_segment, current_sign.unwrap()));
                    }
                    
                    // Crear series para cada segmento
                    let mut series_list = Vec::new();
                    for (i, (segment, sign)) in segments.iter().enumerate() {
                        let color = if *sign > 0 {
                            "#2ca02c".to_string() // Verde para valores positivos
                        } else if *sign < 0 {
                            "#FF0000".to_string() // Rojo para valores negativos
                        } else {
                            "#000000".to_string() // Negro para cero
                        };
                        
                        let series = Series2D {
                            x: segment.iter().map(|(x, _)| *x).collect(),
                            y: segment.iter().map(|(_, y)| *y).collect(),
                            name: format!("Segment {}", i + 1),
                            mode: TraceMode::Lines,
                            line_color: Some(color),
                            line_width: Some(2.0),
                        };
                        
                        series_list.push(series);
                    }
                    
                    // Eliminar las líneas verticales para cada precio, ya que no son necesarias
                    // y estaban causando errores por usar variables no definidas
                    
                    // Añadir las líneas de referencia, el punto actual y la etiqueta
                    series_list.push(zero_line);
                    series_list.push(current_price_line);
                    series_list.push(current_point);
                    series_list.push(label_series);
                    
                    GraphData::MultiSeries(series_list)
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
    CallButterfly
);
