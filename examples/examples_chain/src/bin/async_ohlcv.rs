use optionstratlib::chains::csv::read_ohlcv_from_zip_async;
use osl_example_support::setup_logger;
use std::error::Error;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    info!("--- Async OHLCV Reading ---");

    // Relative to the repository root, where the examples are run from.
    let zip_path = "examples/Data/cl-1m-sample.zip";

    info!("Reading OHLCV data from {} asynchronously...", zip_path);

    // We'll read without date filters first
    let candles = read_ohlcv_from_zip_async(zip_path.to_string(), None, None).await?;
    info!("Successfully read {} candles.", candles.len());
    if let Some(first) = candles.first() {
        info!("First candle: Date={}, Close={}", first.date, first.close);
    }

    info!("--- Async OHLCV Reading Completed ---");
    Ok(())
}
