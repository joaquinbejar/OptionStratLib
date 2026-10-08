//! Benchmarks of the `optionstratlib-market` file I/O paths (#789), behind
//! the `io` feature.
//!
//! CSV and JSON round-trips of a chain through a temporary directory, the
//! repository's real SP500 chain fixture, and the OHLCV ZIP reader on the
//! 74 000-candle sample. Disk I/O is part of what a caller pays, so it is in
//! the measurement; the page cache keeps it warm after the first iteration.
//!
//! Run with `cargo bench -p optionstratlib-market --features io --bench chains_io`.

use criterion::measurement::WallTime;
use criterion::{BenchmarkGroup, Criterion, Throughput, criterion_group, criterion_main};
use optionstratlib_core::model::{ExpirationDate, Positive};
use optionstratlib_core::pos_or_panic;
use optionstratlib_market::chains::csv::read_ohlcv_from_zip;
use optionstratlib_market::chains::utils::OptionDataPriceParams;
use optionstratlib_market::chains::{OptionChain, OptionChainBuildParams};
use rust_decimal_macros::dec;
use std::fmt::Debug;
use std::hint::black_box;

/// The real 45-strike chain the strategy optimiser tests read.
const SP500_JSON: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/Chains/SP500-18-oct-2024-5781.88.json"
);

/// One minute bars of crude oil, 74 061 rows.
const OHLCV_ZIP: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/Data/cl-1m-sample.zip"
);

/// Checks that `f` succeeds on the fixture, then measures it.
///
/// A path that fails fast would otherwise be timed as its error branch and
/// report a misleadingly cheap number.
fn bench_ok<T, E: Debug>(
    group: &mut BenchmarkGroup<'_, WallTime>,
    name: &str,
    mut f: impl FnMut() -> Result<T, E>,
) {
    if let Err(e) = f() {
        panic!("bench `{name}`: the fixture returned an error: {e:?}");
    }
    group.bench_function(name, |bench| bench.iter(|| black_box(f())));
}

fn chain(half_width: usize) -> OptionChain {
    let params = OptionChainBuildParams::new(
        "BENCH".to_string(),
        None,
        half_width,
        None,
        dec!(-0.2),
        dec!(0.1),
        pos_or_panic!(0.02),
        2,
        OptionDataPriceParams::new(
            Some(Box::new(Positive::HUNDRED)),
            Some(ExpirationDate::Days(pos_or_panic!(30.0))),
            Some(dec!(0.05)),
            Some(pos_or_panic!(0.01)),
            Some("BENCH".to_string()),
        ),
        pos_or_panic!(0.2),
    );
    OptionChain::build_chain(&params).expect("the fixture chain builds")
}

fn bench_chain_files(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/chain_io");
    let dir = tempfile::tempdir().expect("a temporary directory");
    let dir_path = dir.path().to_str().expect("the temporary path is UTF-8");

    for half_width in [10, 50] {
        let chain = chain(half_width);
        let strikes = chain.options.len();
        let csv_path = format!("{dir_path}/{}.csv", chain.get_title());
        let json_path = format!("{dir_path}/{}.json", chain.get_title());
        group.throughput(Throughput::Elements(strikes as u64));
        bench_ok(&mut group, &format!("save_to_csv/{strikes}"), || {
            black_box(&chain).save_to_csv(dir_path)
        });
        bench_ok(&mut group, &format!("load_from_csv/{strikes}"), || {
            OptionChain::load_from_csv(black_box(&csv_path))
        });
        bench_ok(&mut group, &format!("save_to_json/{strikes}"), || {
            black_box(&chain).save_to_json(dir_path)
        });
        bench_ok(&mut group, &format!("load_from_json/{strikes}"), || {
            OptionChain::load_from_json(black_box(&json_path))
        });
    }

    bench_ok(&mut group, "load_from_json/sp500_fixture", || {
        OptionChain::load_from_json(black_box(SP500_JSON))
    });
    group.finish();
}

fn bench_ohlcv(c: &mut Criterion) {
    let mut group = c.benchmark_group("market/ohlcv_zip");
    group.sample_size(10);
    bench_ok(&mut group, "read_ohlcv_from_zip/all_74061", || {
        read_ohlcv_from_zip(black_box(OHLCV_ZIP), None, None)
    });
    // A one-day window still decompresses and scans the whole archive.
    bench_ok(&mut group, "read_ohlcv_from_zip/one_day", || {
        read_ohlcv_from_zip(black_box(OHLCV_ZIP), Some("02/01/2008"), Some("02/01/2008"))
    });
    group.finish();
}

criterion_group!(benches, bench_chain_files, bench_ohlcv);
criterion_main!(benches);
