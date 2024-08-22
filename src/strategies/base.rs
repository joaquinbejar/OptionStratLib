/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 21/8/24
******************************************************************************/
use crate::constants::ZERO;
use crate::model::position::Position;
use crate::model::types::Side;
use crate::visualization::utils::Graph;
use plotters::prelude::*;
use std::error::Error;

/// This enum represents different types of trading strategies.
/// Each variant represents a specific strategy type.
#[derive(Clone, Debug)]
pub enum StrategyType {
    BullCallSpread,
    BearCallSpread,
    BullPutSpread,
    BearPutSpread,
    IronCondor,
    Straddle,
    Strangle,
    CoveredCall,
    ProtectivePut,
    Collar,
    LongCall,
    LongPut,
    ShortCall,
    ShortPut,
    Custom,
}

/// Represents a trading strategy.
///
/// A strategy consists of the following properties:
///
/// - `name`: The name of the strategy.
/// - `kind`: The type of the strategy.
/// - `description`: A description of the strategy.
/// - `legs`: A vector of positions that make up the strategy.
/// - `max_profit`: The maximum potential profit of the strategy (optional).
/// - `max_loss`: The maximum potential loss of the strategy (optional).
/// - `break_even_points`: A vector of break-even points for the strategy.
pub struct Strategy {
    pub name: String,
    pub kind: StrategyType,
    pub description: String,
    pub legs: Vec<Position>,
    pub max_profit: Option<f64>,
    pub max_loss: Option<f64>,
    pub break_even_points: Vec<f64>,
}

impl Strategy {
    pub fn new(name: String, kind: StrategyType, description: String) -> Self {
        Strategy {
            name,
            kind,
            description,
            legs: Vec::new(),
            max_profit: None,
            max_loss: None,
            break_even_points: Vec::new(),
        }
    }

    pub fn add_leg(&mut self, position: Position) {
        self.legs.push(position);
    }

    pub fn set_max_profit(&mut self, max_profit: f64) {
        self.max_profit = Some(max_profit);
    }

    pub fn set_max_loss(&mut self, max_loss: f64) {
        self.max_loss = Some(max_loss);
    }

    pub fn add_break_even_point(&mut self, point: f64) {
        self.break_even_points.push(point);
    }

    pub fn break_even(&self) -> f64 {
        match self.kind {
            StrategyType::BullCallSpread => {
                let long_call = self
                    .legs
                    .iter()
                    .find(|leg| leg.option.side == Side::Long)
                    .unwrap();
                let short_call = self
                    .legs
                    .iter()
                    .find(|leg| leg.option.side == Side::Short)
                    .unwrap();
                long_call.option.strike_price
                    + (long_call.total_cost() - short_call.net_premium_received())
                        / long_call.option.quantity as f64
            }
            StrategyType::BearCallSpread => ZERO,
            StrategyType::BullPutSpread => ZERO,
            StrategyType::BearPutSpread => ZERO,
            StrategyType::IronCondor => ZERO,
            StrategyType::Straddle => ZERO,
            StrategyType::Strangle => ZERO,
            StrategyType::CoveredCall => ZERO,
            StrategyType::ProtectivePut => ZERO,
            StrategyType::Collar => ZERO,
            StrategyType::LongCall => ZERO,
            StrategyType::LongPut => ZERO,
            StrategyType::ShortCall => ZERO,
            StrategyType::ShortPut => ZERO,
            StrategyType::Custom => ZERO,
        }
    }

    pub fn calculate_profit_at(&self, price: f64) -> f64 {
        self.legs
            .iter()
            .map(|leg| leg.pnl_at_expiration(Some(price)))
            .sum()
    }
}

impl Graph for Strategy {
    fn graph(&self, data: &[f64], file_path: &str) -> Result<(), Box<dyn Error>> {
        // Generate profit values for each price in the data vector
        let profit_values: Vec<f64> = data
            .iter()
            .map(|&price| self.calculate_profit_at(price))
            .collect();

        let dark_green = RGBColor(0, 150, 0);
        let dark_red = RGBColor(220, 0, 0);

        // Set up the drawing area with a 1200x800 pixel canvas
        let root = BitMapBackend::new(file_path, (1200, 800)).into_drawing_area();
        root.fill(&WHITE)?;

        // Determine the range for the X and Y axes
        let max_price = data.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_price = data.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_profit = profit_values
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        let min_profit = profit_values.iter().cloned().fold(f64::INFINITY, f64::min);
        let adjusted_max_profit = (max_profit * 1.2 - max_profit).abs();
        let adjusted_min_profit = (min_profit * 1.2 - min_profit).abs();
        let margin_value = std::cmp::max(adjusted_max_profit as i64, adjusted_min_profit as i64);
        let max_profit_value = max_profit + margin_value as f64;
        let min_profit_value = min_profit - margin_value as f64;

        let title = self.title();

        // Build the chart with specified margins and label sizes
        let mut chart = ChartBuilder::on(&root)
            .caption(title, ("sans-serif", 15))
            .margin(10)
            .top_x_label_area_size(40)
            .x_label_area_size(40)
            .y_label_area_size(60)
            .right_y_label_area_size(60)
            .build_cartesian_2d(min_price..max_price, min_profit_value..max_profit_value)?;

        // Configure and draw the mesh grid
        chart.configure_mesh().x_labels(20).y_labels(20).draw()?;

        // Draw a horizontal line at y = 0 to indicate break-even
        chart.draw_series(LineSeries::new(
            vec![(min_price, 0.0), (max_price, 0.0)],
            &BLACK,
        ))?;

        // Iterate through the data and profit values to draw the line segments
        let mut last_point = None;
        for (&price, &profit_value) in data.iter().zip(profit_values.iter()) {
            if let Some((last_price, last_profit)) = last_point {
                let color = if profit_value >= 0.0 {
                    &dark_green
                } else {
                    &dark_red
                };

                chart.draw_series(LineSeries::new(
                    vec![(last_price, last_profit), (price, profit_value)],
                    color,
                ))?;
            }
            last_point = Some((price, profit_value));
        }

        let break_even = self.break_even();
        // Draw a vertical line at the break-even price
        chart.draw_series(LineSeries::new(
            vec![
                (break_even, min_profit_value),
                (break_even, max_profit_value),
            ],
            &BLACK,
        ))?;

        let break_even_label_position = (10, 30);

        // Add a label at the top of the break-even line
        chart.draw_series(PointSeries::of_element(
            vec![(break_even, max_profit_value)],
            5,
            &BLACK,
            &|coord, _size, _style| {
                EmptyElement::at(coord)
                    + Text::new(
                        format!("Strike: {:.2}", break_even),
                        break_even_label_position, // Position the text just above the top of the line
                        ("sans-serif", 15).into_font(),
                    )
            },
        ))?;

        // Draw points on the graph with labels for the profit values
        for (i, (&price, &value)) in data.iter().zip(profit_values.iter()).enumerate() {
            let point_color = if value >= 0.0 { &dark_green } else { &dark_red };
            let label_offset = if value >= 0.0 { (20, 0) } else { (-20, -20) };
            let size = 3;

            chart.draw_series(PointSeries::of_element(
                vec![(price, value)],
                size,
                point_color,
                &|coord, size, style| {
                    let element =
                        EmptyElement::at(coord) + Circle::new((0, 0), size, style.filled());

                    if i % 10 == 0 {
                        element
                            + Text::new(
                                format!("{:.2}", value),
                                (label_offset.0, label_offset.1),
                                ("sans-serif", 15).into_font(),
                            )
                    } else {
                        EmptyElement::at(coord)
                            + Circle::new((0, 0), 0, style.filled())
                            + Text::new(
                                String::new(),
                                (label_offset.0, label_offset.1),
                                ("sans-serif", 15).into_font(),
                            )
                    }
                },
            ))?;
        }

        // Finalize and render the chart
        root.present()?;
        Ok(())
    }

    fn title(&self) -> String {
        let strategy_title = format!("Strategy: {} - {:?}", self.name, self.kind);
        let leg_titles: Vec<String> = self.legs.iter().map(|leg| leg.title()).collect();

        if leg_titles.is_empty() {
            strategy_title
        } else {
            format!("{}\n{}", strategy_title, leg_titles.join("\n"))
        }
    }
}
