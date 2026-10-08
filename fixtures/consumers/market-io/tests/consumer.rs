//! Option chain and OHLCV file I/O from `optionstratlib-market` with only its
//! `io` feature (#552): a chain round-trips through JSON and CSV files, the
//! zipped OHLCV reader filters a date range, and each documented failure
//! arrives as its typed error.

use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::{pos_or_panic, spos};
use optionstratlib_market::chains::OptionChain;
use optionstratlib_market::chains::csv::read_ohlcv_from_zip;
use optionstratlib_market::chains::utils::{OptionChainBuildParams, OptionDataPriceParams};
use optionstratlib_market::error::{ChainError, OhlcvError};
use rust_decimal_macros::dec;
use std::error::Error;
use std::path::PathBuf;

/// Nine one-minute crude-oil futures candles, cut from the repository's
/// `examples/Data/cl-1m-sample.zip`: the last two of 1 January 2008, five of
/// 2 January (the first three and the last two) and the first two of
/// 3 January. The file ships with the fixture, so a copy of it built outside
/// the repository against the packaged crates reads the same data.
const OHLCV_SAMPLE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/ohlcv-sample.zip");

fn chain() -> Result<OptionChain, Box<dyn Error>> {
    let params = OptionChainBuildParams::new(
        "XYZ".to_string(),
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
            Some("XYZ".to_string()),
        ),
        pos_or_panic!(0.2),
    );
    Ok(OptionChain::build_chain(&params)?)
}

/// A fresh directory under cargo's per-target scratch directory.
fn scratch(name: &str) -> Result<PathBuf, Box<dyn Error>> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    std::fs::create_dir_all(&dir)?;
    Ok(dir)
}

#[test]
fn test_chain_round_trips_through_a_json_file() -> Result<(), Box<dyn Error>> {
    let chain = chain()?;
    let dir = scratch("market-io-json")?;
    let dir_str = dir.to_str().ok_or("utf-8 path")?;
    chain.save_to_json(dir_str)?;

    let file = dir.join(format!("{}.json", chain.get_title()));
    assert!(file.is_file());
    let loaded = OptionChain::load_from_json(file.to_str().ok_or("utf-8 path")?)?;
    assert_eq!(loaded.symbol, "XYZ");
    assert_eq!(loaded.underlying_price, Positive::HUNDRED);
    assert_eq!(loaded.get_expiration_date(), chain.get_expiration_date());
    assert_eq!(loaded.options.len(), chain.options.len());
    for (read, written) in loaded.options.iter().zip(&chain.options) {
        assert_eq!(read.strike_price, written.strike_price);
        assert_eq!(read.call_bid, written.call_bid);
        assert_eq!(read.put_ask, written.put_ask);
        assert_eq!(read.implied_volatility, written.implied_volatility);
    }
    Ok(())
}

#[test]
fn test_chain_round_trips_through_a_csv_file() -> Result<(), Box<dyn Error>> {
    let chain = chain()?;
    let dir = scratch("market-io-csv")?;
    chain.save_to_csv(dir.to_str().ok_or("utf-8 path")?)?;

    let file = dir.join(format!("{}.csv", chain.get_title()));
    let loaded = OptionChain::load_from_csv(file.to_str().ok_or("utf-8 path")?)?;
    // The CSV carries the quotes; the symbol and the price come back from the
    // file name the writer chose.
    assert_eq!(loaded.symbol, "XYZ");
    assert_eq!(loaded.underlying_price, Positive::HUNDRED);
    assert_eq!(loaded.options.len(), 11);
    let strikes: Vec<Positive> = loaded.options.iter().map(|o| o.strike_price).collect();
    let written: Vec<Positive> = chain.options.iter().map(|o| o.strike_price).collect();
    assert_eq!(strikes, written);
    for (read, written) in loaded.options.iter().zip(&chain.options) {
        assert_eq!(read.call_ask, written.call_ask);
        assert_eq!(read.put_bid, written.put_bid);
    }
    Ok(())
}

#[test]
fn test_a_missing_chain_file_is_a_file_error() {
    let missing = concat!(env!("CARGO_TARGET_TMPDIR"), "/no-such-chain.json");
    assert!(matches!(
        OptionChain::load_from_json(missing),
        Err(ChainError::FileError(_))
    ));
}

#[test]
fn test_ohlcv_candles_are_read_from_a_zip_for_one_day() -> Result<(), Box<dyn Error>> {
    let candles = read_ohlcv_from_zip(OHLCV_SAMPLE, Some("02/01/2008"), Some("02/01/2008"))?;
    // The sample holds five candles of 2 January 2008.
    assert_eq!(candles.len(), 5);
    let first = candles.first().ok_or("a first candle")?;
    assert_eq!(first.time, "00:01:00");
    assert_eq!(first.open, dec!(96.42));
    assert_eq!(first.volume, 1);
    let last = candles.last().ok_or("a last candle")?;
    assert_eq!(last.time, "23:58:00");
    assert_eq!(last.close, dec!(99.36));
    assert!(
        candles
            .iter()
            .all(|c| c.low <= c.open && c.low <= c.close && c.high >= c.open && c.high >= c.close),
        "every candle's range contains its open and close"
    );
    Ok(())
}

#[test]
fn test_ohlcv_date_bounds_are_optional_and_inclusive() -> Result<(), Box<dyn Error>> {
    assert_eq!(read_ohlcv_from_zip(OHLCV_SAMPLE, None, None)?.len(), 9);
    let from_third = read_ohlcv_from_zip(OHLCV_SAMPLE, Some("03/01/2008"), None)?;
    assert_eq!(from_third.len(), 2);
    assert_eq!(from_third.first().ok_or("a candle")?.volume, 14);
    let to_first = read_ohlcv_from_zip(OHLCV_SAMPLE, None, Some("01/01/2008"))?;
    assert_eq!(to_first.len(), 2);
    Ok(())
}

#[test]
fn test_an_inverted_ohlcv_range_is_an_invalid_parameter() {
    assert!(matches!(
        read_ohlcv_from_zip(OHLCV_SAMPLE, Some("03/01/2008"), Some("02/01/2008")),
        Err(OhlcvError::InvalidParameter { .. })
    ));
}
