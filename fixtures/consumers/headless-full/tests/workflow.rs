//! End-to-end 0.22 workflows on the facade defaults (#552): every capability
//! is on, no rendering backend is.
//!
//! - Pricing through generic code and through `dyn Greeks` trait objects
//!   that mix an option and a strategy.
//! - The simulation-backed chain generator the default `synthetic` feature
//!   routes to `chains::generator_optionchain`.
//! - A backtest whose statistics are read as data and rendered as the
//!   terminal report `visualization::terminal` owns.

use optionstratlib::backtesting::SimulationStatsResult;
use optionstratlib::chains::generator_optionchain;
use optionstratlib::prelude::*;
use optionstratlib::simulation::generator_positive;
use optionstratlib::visualization::terminal::{ChainReport, SimulationReport};
use std::error::Error;

/// A walker that keeps every default: a `Historical` walk replays its prices.
#[derive(Clone)]
struct Replay;

impl WalkTypeAble<Positive, Positive> for Replay {}
impl WalkTypeAble<Positive, OptionChain> for Replay {}

/// Long 100 call, 30 days, premium 5, open and close fees 0.5 each.
fn long_call() -> Result<LongCall, Box<dyn Error>> {
    Ok(LongCall::new(
        "TEST".to_string(),
        Positive::HUNDRED,
        ExpirationDate::Days(pos_or_panic!(30.0)),
        pos_or_panic!(0.20),
        Positive::ONE,
        Positive::HUNDRED,
        dec!(0.05),
        Positive::ZERO,
        pos_or_panic!(5.0),
        pos_or_panic!(0.5),
        pos_or_panic!(0.5),
    )?)
}

/// The delta of anything with Greeks: the generic form.
fn delta_of<G: Greeks>(item: &G) -> Result<Decimal, Box<dyn Error>> {
    Ok(item.delta()?)
}

/// The delta of a mixed book: the trait-object form.
fn book_delta(book: &[&dyn Greeks]) -> Result<Decimal, Box<dyn Error>> {
    book.iter().try_fold(Decimal::ZERO, |total, item| {
        let delta = item.delta()?;
        total
            .checked_add(delta)
            .ok_or_else(|| "delta overflow".into())
    })
}

#[test]
fn test_greeks_through_generics_and_trait_objects() -> Result<(), Box<dyn Error>> {
    let strategy = long_call()?;
    let positions = strategy.get_positions()?;
    let option = &positions.first().ok_or("a long call has one leg")?.option;

    // A long at-the-money call: delta a little above one half.
    let option_delta = delta_of(option)?;
    assert!(
        option_delta > dec!(0.5) && option_delta < dec!(0.6),
        "{option_delta}"
    );
    // The strategy's delta is its one leg's.
    assert_eq!(delta_of(&strategy)?, option_delta);
    // A book holding both is twice that, through `dyn Greeks`.
    let book: [&dyn Greeks; 2] = [option, &strategy];
    assert_eq!(book_delta(&book)?, option_delta * dec!(2));
    Ok(())
}

fn seed_chain() -> Result<OptionChain, Box<dyn Error>> {
    let params = OptionChainBuildParams::new(
        "XYZ".to_string(),
        None,
        10,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(60.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some("XYZ".to_string()),
        ),
        pos_or_panic!(0.25),
    );
    Ok(OptionChain::build_chain(&params)?)
}

fn chain_walk(
    size: usize,
    prices: Vec<Positive>,
) -> Result<WalkParams<Positive, OptionChain>, Box<dyn Error>> {
    Ok(WalkParams {
        size,
        init_step: Step {
            x: Xstep::new(
                Positive::ONE,
                TimeFrame::Day,
                ExpirationDate::Days(pos_or_panic!(60.0)),
            ),
            y: Ystep::new(0, seed_chain()?),
        },
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices,
            symbol: None,
        },
        walker: Box::new(Replay),
        seed: None,
    })
}

#[test]
fn test_synthetic_chains_follow_a_replayed_path() -> Result<(), Box<dyn Error>> {
    let prices = vec![
        Positive::HUNDRED,
        pos_or_panic!(104.0),
        pos_or_panic!(98.0),
        pos_or_panic!(103.0),
    ];
    let steps = generator_optionchain(&chain_walk(4, prices.clone())?)?;
    assert_eq!(steps.len(), 4);
    for (step, price) in steps.iter().zip(&prices).skip(1) {
        let chain = step.y.value();
        assert_eq!(chain.underlying_price, *price);
        // Each generated chain is a full chain, rendered as the terminal
        // table the visualization crate owns.
        assert!(chain.options.len() > 10);
        assert!(chain.render_table().contains("XYZ"));
    }
    Ok(())
}

fn backtest(prices: &[f64]) -> Result<SimulationStatsResult, Box<dyn Error>> {
    let params = WalkParams {
        size: prices.len(),
        init_step: Step::new(
            Positive::ONE,
            TimeFrame::Day,
            ExpirationDate::Days(pos_or_panic!(30.0)),
            Positive::HUNDRED,
        ),
        walker: Box::new(Replay),
        walk_type: WalkType::Historical {
            timeframe: TimeFrame::Day,
            prices: prices
                .iter()
                .map(|price| Positive::new(*price))
                .collect::<Result<_, _>>()?,
            symbol: Some("TEST".to_string()),
        },
        seed: None,
    };
    let simulator = Simulator::new("fixture".to_string(), 2, &params, generator_positive)?;
    Ok(long_call()?.simulate(&simulator, ExitPolicy::Expiration)?)
}

#[test]
fn test_a_backtest_is_read_as_data_and_rendered_as_a_report() -> Result<(), Box<dyn Error>> {
    // In the money at expiry: 20 - premium 5 - fees 1 = 14 per walk.
    let stats = backtest(&[100.0, 105.0, 110.0, 115.0, 120.0])?;
    assert_eq!(stats.total_simulations, 2);
    assert_eq!(stats.average_pnl, dec!(14));
    assert_eq!(stats.win_rate, dec!(100));

    let summary = stats.render_summary()?;
    assert!(summary.contains("SIMULATION SUMMARY"));
    assert!(
        summary.contains("│ Total Simulations │ 2     │"),
        "{summary}"
    );
    assert!(
        summary.contains("│ Total P&L             │ $28.00 │"),
        "{summary}"
    );
    assert!(
        summary.contains("│ Average P&L per Trade │ $14.00 │"),
        "{summary}"
    );
    assert!(!summary.contains('\u{1b}'), "plain text, no ANSI escapes");

    let runs = stats.render_individual_results();
    assert_eq!(runs.matches("$14.00").count(), 2, "{runs}");
    Ok(())
}
