use optionstratlib::error::GraphError;
use optionstratlib::visualization::{Graph, GraphData, Series2D, TraceMode};
use rust_decimal_macros::dec;

fn main() -> Result<(), GraphError> {
    // Creamos nuestra serie de datos
    let series = Series2D {
        x: vec![dec!(1.0), dec!(2.0), dec!(3.0)],
        y: vec![dec!(4.0), dec!(5.0), dec!(6.0)],
        name: "Series 1".to_string(),
        mode: TraceMode::Lines,
        line_color: Some("#1f77b4".to_string()),
        line_width: Some(2.0),
    };

    let chart = {
        struct AnonymousChart(Series2D);

        impl Graph for AnonymousChart {
            fn graph_data(&self) -> GraphData {
                GraphData::Series(self.0.clone())
            }
        }

        AnonymousChart(series)
    };
    let path: &std::path::Path = "Draws/Visualization/interactive_chart.html".as_ref();
    chart.to_interactive_html(path)?;

    println!("Interactive HTML chart created successfully!");
    Ok(())
}
