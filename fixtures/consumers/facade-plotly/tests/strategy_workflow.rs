//! The 0.22 strategy-construction-to-chart workflow on the facade with
//! default features plus `plotly` (#552).
//!
//! A service receives a strategy as data, a `StrategyRequest` (a
//! `StrategyType` and its legs), and has two jobs: analyse it and chart it.
//!
//! - Analysis goes through `StrategyRequest::get_strategy`, which returns a
//!   `Box<dyn Strategable>`: one trait object for every strategy the
//!   builder knows, with the break-even points, the profit and loss bounds
//!   and the payoff at any price.
//! - Charting needs `Graph`, which is no longer a supertrait of the strategy
//!   contract (#658), so a `Box<dyn Strategable>` cannot be charted. A
//!   consumer that renders names the bound itself: it dispatches on the
//!   request's `strategy_type` to the concrete type, builds it with
//!   `StrategyConstructor::get_strategy` from the same legs, and charts it
//!   through a function generic over `S: StrategyConstructor + Graph`.
//!
//! The capability the old boxed-strategy chart call had is kept: every
//! strategy the builder can build is chartable, and the tests prove the
//! chart is the analysis, point for point, by comparing the payoff segments,
//! the current-price marker and the break-even labels the chart draws with
//! `calculate_profit_at` and `get_break_even_points` on the trait object.

use optionstratlib::error::StrategyError;
use optionstratlib::prelude::*;
use optionstratlib::strategies::StrategyRequest;
use optionstratlib::strategies::base::StrategyType;
use optionstratlib::visualization::{GraphData, OutputType, Series2D};
use std::error::Error;
use std::path::PathBuf;

/// Days to expiry of every leg.
const EXPIRY_DAYS: Decimal = dec!(30);

/// Implied volatility of every leg, as a fraction per year.
const IMPLIED_VOLATILITY: Decimal = dec!(0.2);

/// Risk-free rate, as a fraction per year.
const RISK_FREE_RATE: Decimal = dec!(0.05);

/// Open and close fee of every leg, in quote currency per contract.
const FEE: Decimal = dec!(0.5);

/// Name prefix of the payoff segments every strategy chart draws: one
/// series per run of prices with the same sign of profit.
const PAYOFF_SEGMENT: &str = "Segment ";

/// Name prefix of the break-even labels.
const BREAK_EVEN_LABEL: &str = "BE: ";

/// Name prefix of the marker at the current underlying price.
const CURRENT_PNL_MARKER: &str = "Current P/L: ";

/// Name of the zero-profit reference line.
const ZERO_LINE: &str = "Break Even";

/// Segment colours: profit green, loss red.
const PROFIT_COLOUR: &str = "#2ca02c";
const LOSS_COLOUR: &str = "#FF0000";

fn pos(value: Decimal) -> Result<Positive, Box<dyn Error>> {
    Ok(Positive::new_decimal(value)?)
}

/// One European leg on `TEST` with the underlying at 100.
fn leg(
    side: Side,
    style: OptionStyle,
    strike: Decimal,
    premium: Decimal,
) -> Result<Position, Box<dyn Error>> {
    let option = Options::new(
        OptionType::European,
        side,
        "TEST".to_string(),
        pos(strike)?,
        ExpirationDate::Days(pos(EXPIRY_DAYS)?),
        pos(IMPLIED_VOLATILITY)?,
        Positive::ONE,
        Positive::HUNDRED,
        RISK_FREE_RATE,
        style,
        Positive::ZERO,
        None,
    );
    Ok(Position::new(
        option,
        pos(premium)?,
        Utc::now(),
        pos(FEE)?,
        pos(FEE)?,
        None,
        None,
    ))
}

/// The requests a service would receive: a debit vertical, a credit
/// four-leg structure and a long-volatility pair.
fn requests() -> Result<Vec<StrategyRequest>, Box<dyn Error>> {
    use OptionStyle::{Call, Put};
    use Side::{Long, Short};
    Ok(vec![
        StrategyRequest::new(
            StrategyType::BullCallSpread,
            vec![
                leg(Long, Call, dec!(95), dec!(7.5))?,
                leg(Short, Call, dec!(105), dec!(2.5))?,
            ],
        ),
        StrategyRequest::new(
            StrategyType::IronCondor,
            vec![
                leg(Long, Put, dec!(85), dec!(0.6))?,
                leg(Short, Put, dec!(95), dec!(2.1))?,
                leg(Short, Call, dec!(105), dec!(2.4))?,
                leg(Long, Call, dec!(115), dec!(0.7))?,
            ],
        ),
        StrategyRequest::new(
            StrategyType::LongStraddle,
            vec![
                leg(Long, Call, dec!(100), dec!(4.5))?,
                leg(Long, Put, dec!(100), dec!(4.1))?,
            ],
        ),
    ])
}

/// What the rendering side keeps of a chart.
struct Chart {
    data: GraphData,
    title: String,
    figure_json: String,
}

/// Charts any strategy that implements the graph contract: the explicit
/// bound a rendering consumer writes.
fn chart<S: StrategyConstructor + Graph>(positions: &[Position]) -> Result<Chart, Box<dyn Error>> {
    let strategy = S::get_strategy(positions)?;
    Ok(Chart {
        data: strategy.graph_data(),
        title: strategy.graph_config().title,
        figure_json: strategy.to_plot().to_json(),
    })
}

/// Charts the strategy a request names: the dispatch that replaces the old
/// `Graph` supertrait on the boxed strategy.
fn chart_request(request: &StrategyRequest) -> Result<Chart, Box<dyn Error>> {
    let positions = request.positions.as_slice();
    match request.strategy_type {
        StrategyType::BullCallSpread => chart::<BullCallSpread>(positions),
        StrategyType::BearCallSpread => chart::<BearCallSpread>(positions),
        StrategyType::BullPutSpread => chart::<BullPutSpread>(positions),
        StrategyType::BearPutSpread => chart::<BearPutSpread>(positions),
        StrategyType::LongButterflySpread => chart::<LongButterflySpread>(positions),
        StrategyType::ShortButterflySpread => chart::<ShortButterflySpread>(positions),
        StrategyType::IronCondor => chart::<IronCondor>(positions),
        StrategyType::IronButterfly => chart::<IronButterfly>(positions),
        StrategyType::LongStraddle => chart::<LongStraddle>(positions),
        StrategyType::ShortStraddle => chart::<ShortStraddle>(positions),
        StrategyType::LongStrangle => chart::<LongStrangle>(positions),
        StrategyType::ShortStrangle => chart::<ShortStrangle>(positions),
        StrategyType::PoorMansCoveredCall => chart::<PoorMansCoveredCall>(positions),
        StrategyType::BullCallLadder => chart::<BullCallLadder>(positions),
        StrategyType::Custom => chart::<CustomStrategy>(positions),
        // No builder: `StrategyRequest::get_strategy` reports the same. Named
        // one by one, so a strategy that gains a builder fails to compile
        // here until it is chartable.
        StrategyType::CoveredCall
        | StrategyType::ProtectivePut
        | StrategyType::Collar
        | StrategyType::LongCall
        | StrategyType::LongPut
        | StrategyType::ShortCall
        | StrategyType::ShortPut => Err(Box::new(StrategyError::NotImplemented)),
    }
}

/// The series of a strategy chart.
fn series(data: &GraphData) -> Result<&[Series2D], Box<dyn Error>> {
    match data {
        GraphData::MultiSeries(series) => Ok(series),
        _ => Err("a strategy chart is a multi-series chart".into()),
    }
}

#[test]
fn test_requests_are_analysed_through_the_trait_object() -> Result<(), Box<dyn Error>> {
    let strategies: Vec<Box<dyn Strategable>> = requests()?
        .iter()
        .map(StrategyRequest::get_strategy)
        .collect::<Result<_, _>>()?;
    assert_eq!(strategies.len(), 3);

    // Bull call spread 95/105 for a 5.00 debit and 2.00 of fees: break-even
    // 102, profit capped at 3, loss capped at 7.
    let spread = &strategies[0];
    assert_eq!(spread.get_break_even_points()?, &vec![pos(dec!(102))?]);
    assert_eq!(spread.get_max_profit()?, pos(dec!(3))?);
    assert_eq!(spread.get_max_loss()?, pos(dec!(7))?);
    assert_eq!(spread.calculate_profit_at(&pos(dec!(120))?)?, dec!(3));
    assert_eq!(spread.calculate_profit_at(&pos(dec!(80))?)?, dec!(-7));

    // Iron condor 85/95/105/115 for a 3.20 credit less 4.00 of fees: the
    // wings cap the loss at 10.80, the body earns -0.80 at 100.
    let condor = &strategies[1];
    assert_eq!(condor.calculate_profit_at(&Positive::HUNDRED)?, dec!(-0.8));
    assert_eq!(condor.calculate_profit_at(&pos(dec!(70))?)?, dec!(-10.8));
    assert_eq!(condor.calculate_profit_at(&pos(dec!(130))?)?, dec!(-10.8));

    // Long straddle at 100 for an 8.60 debit and 2.00 of fees: two
    // break-evens 10.60 either side of the strike, loss capped at 10.60.
    let straddle = &strategies[2];
    assert_eq!(
        straddle.get_break_even_points()?,
        &vec![pos(dec!(89.4))?, pos(dec!(110.6))?]
    );
    assert_eq!(straddle.get_max_loss()?, pos(dec!(10.6))?);
    assert_eq!(
        straddle.calculate_profit_at(&Positive::HUNDRED)?,
        dec!(-10.6)
    );

    // Generic code over the trait object: the fees of the whole book.
    let fees = strategies
        .iter()
        .map(|strategy| strategy.get_fees())
        .try_fold(Positive::ZERO, |total, fees| fees.map(|fees| total + fees))?;
    assert_eq!(fees, pos(dec!(8))?);
    Ok(())
}

#[test]
fn test_every_request_charts_what_the_analysis_computes() -> Result<(), Box<dyn Error>> {
    for request in requests()? {
        let analysed = request.get_strategy()?;
        let chart = chart_request(&request)?;
        let series = series(&chart.data)?;
        let title = &chart.title;
        assert_eq!(*title, analysed.get_title());

        // Every point of every payoff segment is the trait object's profit,
        // and the segment colour is the sign of that profit.
        let segments: Vec<&Series2D> = series
            .iter()
            .filter(|series| series.name.starts_with(PAYOFF_SEGMENT))
            .collect();
        assert!(!segments.is_empty(), "{title}: a payoff curve");
        let mut points = 0_usize;
        for segment in segments {
            assert_eq!(segment.x.len(), segment.y.len());
            for (price, profit) in segment.x.iter().zip(&segment.y) {
                assert_eq!(
                    analysed.calculate_profit_at(&pos(*price)?)?,
                    *profit,
                    "{title}: the chart at {price} is the analysis"
                );
                points += 1;
            }
            let colour = segment.line_color.as_deref();
            if segment.y.iter().any(|profit| *profit > Decimal::ZERO) {
                assert_eq!(colour, Some(PROFIT_COLOUR), "{title}: {}", segment.name);
            } else if segment.y.iter().any(|profit| *profit < Decimal::ZERO) {
                assert_eq!(colour, Some(LOSS_COLOUR), "{title}: {}", segment.name);
            }
        }
        assert!(points > 10, "{title}: {points} payoff points");

        // The marker at the underlying price reads the analysed profit there.
        let at_spot = analysed.calculate_profit_at(&Positive::HUNDRED)?;
        let marker = series
            .iter()
            .find(|series| series.name.starts_with(CURRENT_PNL_MARKER))
            .ok_or("a current-price marker")?;
        assert_eq!(marker.x, vec![dec!(100)]);
        assert_eq!(marker.y, vec![at_spot]);
        assert_eq!(marker.name, format!("{CURRENT_PNL_MARKER}{at_spot:.2}"));

        // One label per analysed break-even point.
        let labels: Vec<Decimal> = series
            .iter()
            .filter(|series| series.name.starts_with(BREAK_EVEN_LABEL))
            .flat_map(|series| series.x.iter().copied())
            .collect();
        let break_evens: Vec<Decimal> = analysed
            .get_break_even_points()?
            .iter()
            .map(|point| point.to_dec())
            .collect();
        assert_eq!(labels, break_evens, "{title}: break-even labels");

        // The Plotly figure draws the same chart: `to_plot` names the first
        // trace after the legend (the title) and keeps the others' names.
        assert!(
            chart.figure_json.contains(&marker.name),
            "{title}: the Plotly figure carries the current-price marker"
        );
        assert!(chart.figure_json.contains(ZERO_LINE), "{title}");
    }
    Ok(())
}

#[test]
fn test_a_request_chart_is_written_as_html() -> Result<(), Box<dyn Error>> {
    let requests = requests()?;
    let request = requests.first().ok_or("one request")?;
    let strategy = BullCallSpread::get_strategy(&request.positions)?;
    let path = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("strategy-workflow.html");
    strategy.render(OutputType::Html(&path))?;
    let html = std::fs::read_to_string(&path)?;
    assert!(html.contains("plotly"), "an interactive Plotly page");
    assert!(html.contains(ZERO_LINE));
    Ok(())
}

#[test]
fn test_a_request_without_a_builder_is_not_implemented() -> Result<(), Box<dyn Error>> {
    // Single legs have constructors, not builders, on both sides.
    let request = StrategyRequest::new(
        StrategyType::LongCall,
        vec![leg(Side::Long, OptionStyle::Call, dec!(100), dec!(4.5))?],
    );
    assert!(matches!(
        request.get_strategy(),
        Err(StrategyError::NotImplemented)
    ));
    let error = chart_request(&request)
        .err()
        .ok_or("a long call has no builder")?;
    assert!(matches!(
        error.downcast_ref::<StrategyError>(),
        Some(StrategyError::NotImplemented)
    ));
    Ok(())
}

#[test]
fn test_a_request_with_the_wrong_legs_is_rejected_by_both_sides() -> Result<(), Box<dyn Error>> {
    // An iron condor needs four legs.
    let request = StrategyRequest::new(
        StrategyType::IronCondor,
        vec![leg(Side::Long, OptionStyle::Put, dec!(85), dec!(0.6))?],
    );
    assert!(request.get_strategy().is_err());
    let error = chart_request(&request)
        .err()
        .ok_or("one leg is not an iron condor")?;
    assert!(error.downcast_ref::<StrategyError>().is_some(), "{error}");
    Ok(())
}
