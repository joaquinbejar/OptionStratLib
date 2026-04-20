use optionstratlib::prelude::setup_logger;
use optionstratlib::utils::read_ohlcv_from_zip_async;
use std::error::Error;
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    setup_logger();
    info!("--- Async OHLCV Reading ---");

    let zip_path = "../../examples/Data/cl-1m-sample.zip"; // Path relative to the example run dir

    info!("Reading OHLCV data from {} asynchronously...", zip_path);

    // We'll read without date filters first
    let result: Result<Vec<optionstratlib::utils::OhlcvCandle>, optionstratlib::error::OhlcvError> =
        read_ohlcv_from_zip_async(zip_path.to_string(), None, None).await;

    match result {
        Ok(candles) => {
            info!("Successfully read {} candles.", candles.len());
            if let Some(first) = candles.first() {
                info!("First candle: Date={}, Close={}", first.date, first.close);
            }
        }
        Err(e) => {
            error!("Error reading OHLCV: {}", e);
            // Don't fail the example if file is not found, just report it
        }
    }

    info!("--- Async OHLCV Reading Completed ---");
    Ok(())
}
