//! Asynchronous market I/O on the `optionstratlib` facade with only its
//! `async` feature (#552): a chain built through the `prelude` round-trips
//! through JSON and CSV files with the `*_async` wrappers, two loads run
//! concurrently on one runtime, the zipped OHLCV reader agrees with its
//! synchronous form, and each documented failure arrives as its typed error.
//! The facade paths are the items `optionstratlib-market` defines.

use optionstratlib::chains::csv::{read_ohlcv_from_zip, read_ohlcv_from_zip_async};
use optionstratlib::error::{ChainError, OhlcvError};
use optionstratlib::prelude::*;
use std::error::Error;
use std::path::PathBuf;

/// Nine one-minute crude-oil futures candles, cut from the repository's
/// `examples/Data/cl-1m-sample.zip`: the last two of 1 January 2008, five of
/// 2 January (the first three and the last two) and the first two of
/// 3 January. The file ships with the fixture, so a copy of it built outside
/// the repository against the packaged crates reads the same data.
const OHLCV_SAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ohlcv-sample.zip");

/// Compiles only when both arguments have the same type. Every function item
/// has its own type, so this proves a facade path re-exports the component's
/// function rather than wrapping it.
fn same_item<T>(_: T, _: T) {}

fn chain(symbol: &str) -> Result<OptionChain, Box<dyn Error>> {
    let params = OptionChainBuildParams::new(
        symbol.to_string(),
        None,
        5,
        spos!(5.0),
        dec!(-0.2),
        dec!(0.0),
        pos_or_panic!(0.01),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            spos!(0.02),
            Some(symbol.to_string()),
        ),
        pos_or_panic!(0.2),
    );
    Ok(OptionChain::build_chain(&params)?)
}

/// A fresh directory under cargo's per-target scratch directory.
fn scratch(name: &str) -> Result<String, Box<dyn Error>> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    Ok(dir.to_str().ok_or("utf-8 path")?.to_string())
}

fn assert_same_quotes(read: &OptionChain, written: &OptionChain) {
    assert_eq!(read.options.len(), written.options.len());
    for (read, written) in read.options.iter().zip(&written.options) {
        assert_eq!(read.strike_price, written.strike_price);
        assert_eq!(read.call_bid, written.call_bid);
        assert_eq!(read.put_ask, written.put_ask);
    }
}

#[tokio::test]
async fn test_chain_round_trips_through_json_asynchronously() -> Result<(), Box<dyn Error>> {
    let chain = chain("XYZ")?;
    let dir = scratch("facade-async-json")?;
    chain.save_to_json_async(&dir).await?;

    let file = format!("{dir}/{}.json", chain.get_title());
    let loaded = OptionChain::load_from_json_async(&file).await?;
    assert_eq!(loaded.symbol, "XYZ");
    assert_eq!(loaded.underlying_price, Positive::HUNDRED);
    assert_eq!(loaded.get_expiration_date(), chain.get_expiration_date());
    assert_same_quotes(&loaded, &chain);
    // The synchronous reader sees the same file the same way.
    assert_same_quotes(&OptionChain::load_from_json(&file)?, &chain);
    Ok(())
}

#[tokio::test]
async fn test_chain_round_trips_through_csv_asynchronously() -> Result<(), Box<dyn Error>> {
    let chain = chain("XYZ")?;
    let dir = scratch("facade-async-csv")?;
    chain.save_to_csv_async(&dir).await?;

    let file = format!("{dir}/{}.csv", chain.get_title());
    let loaded = OptionChain::load_from_csv_async(&file).await?;
    assert_eq!(loaded.symbol, "XYZ");
    assert_eq!(loaded.underlying_price, Positive::HUNDRED);
    assert_same_quotes(&loaded, &chain);
    Ok(())
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_two_chains_load_concurrently() -> Result<(), Box<dyn Error>> {
    let dir = scratch("facade-async-concurrent")?;
    let (spy, qqq) = (chain("SPY")?, chain("QQQ")?);
    spy.save_to_json(&dir)?;
    qqq.save_to_json(&dir)?;

    let spy_file = format!("{dir}/{}.json", spy.get_title());
    let qqq_file = format!("{dir}/{}.json", qqq.get_title());
    let (spy_loaded, qqq_loaded) = tokio::join!(
        OptionChain::load_from_json_async(&spy_file),
        OptionChain::load_from_json_async(&qqq_file),
    );
    assert_eq!(spy_loaded?.symbol, "SPY");
    assert_eq!(qqq_loaded?.symbol, "QQQ");
    Ok(())
}

#[tokio::test]
async fn test_ohlcv_async_reader_agrees_with_the_sync_one() -> Result<(), Box<dyn Error>> {
    let candles = read_ohlcv_from_zip_async(
        OHLCV_SAMPLE.to_string(),
        Some("02/01/2008".to_string()),
        Some("02/01/2008".to_string()),
    )
    .await?;
    // The sample holds five candles of 2 January 2008.
    assert_eq!(candles.len(), 5);
    let sync = read_ohlcv_from_zip(OHLCV_SAMPLE, Some("02/01/2008"), Some("02/01/2008"))?;
    assert_eq!(candles.len(), sync.len());
    for (a, s) in candles.iter().zip(&sync) {
        assert_eq!((a.date, &a.time, a.close), (s.date, &s.time, s.close));
    }
    let first = candles.first().ok_or("a first candle")?;
    assert_eq!(first.open, dec!(96.42));
    Ok(())
}

#[tokio::test]
async fn test_async_failures_arrive_as_typed_errors() {
    let missing = concat!(env!("CARGO_TARGET_TMPDIR"), "/no-such-chain.json");
    assert!(matches!(
        OptionChain::load_from_json_async(missing).await,
        Err(ChainError::FileError(_))
    ));
    let missing_zip = concat!(env!("CARGO_TARGET_TMPDIR"), "/no-such-candles.zip");
    assert!(matches!(
        read_ohlcv_from_zip_async(missing_zip.to_string(), None, None).await,
        Err(OhlcvError::IoError { .. })
    ));
}

#[test]
fn test_facade_paths_are_the_market_items() {
    same_item(
        optionstratlib::chains::csv::read_ohlcv_from_zip,
        optionstratlib_market::chains::csv::read_ohlcv_from_zip,
    );
    let chain: optionstratlib_market::chains::OptionChain = OptionChain::new(
        "XYZ",
        Positive::HUNDRED,
        "2030-01-18".to_string(),
        None,
        None,
    );
    let error: optionstratlib_market::error::ChainError =
        ChainError::invalid_parameters("probe", "reason");
    assert_eq!(chain.symbol, "XYZ");
    assert!(error.to_string().contains("probe"));
}
