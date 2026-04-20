use optionstratlib::prelude::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("--- Async Option Chain Operations ---");

    let chain = OptionChain::new(
        "AAPL",
        pos_or_panic!(150.0),
        "2024-12-20".to_string(),
        None,
        None,
    );

    // `save_to_json_async` / `save_to_csv_async` take a *directory* and append
    // `{title}.{ext}` from the chain's metadata. Use a scratch subdir under
    // the OS temp dir so the example never collides with other processes.
    let dir = std::env::temp_dir().join("optionstratlib-async-chain-ops");
    std::fs::create_dir_all(&dir)?;
    let dir_str = dir.to_string_lossy().to_string();
    let filename_json = format!("{}.json", chain.get_title());
    let filename_csv = format!("{}.csv", chain.get_title());
    let json_path = dir.join(&filename_json);
    let csv_path = dir.join(&filename_csv);

    println!("Saving chain to {} asynchronously...", json_path.display());
    chain.save_to_json_async(&dir_str).await?;
    println!("Successfully saved to JSON.");

    println!(
        "Loading chain from {} asynchronously...",
        json_path.display()
    );
    let loaded_json = OptionChain::load_from_json_async(json_path.to_str().unwrap()).await?;
    println!(
        "Successfully loaded from JSON. Symbol: {}",
        loaded_json.symbol
    );

    println!("Saving chain to {} asynchronously...", csv_path.display());
    chain.save_to_csv_async(&dir_str).await?;
    println!("Successfully saved to CSV.");

    println!(
        "Loading chain from {} asynchronously...",
        csv_path.display()
    );
    let loaded_csv = OptionChain::load_from_csv_async(csv_path.to_str().unwrap()).await?;
    println!(
        "Successfully loaded from CSV. Symbol: {}",
        loaded_csv.symbol
    );

    let _ = std::fs::remove_file(&json_path);
    let _ = std::fs::remove_file(&csv_path);

    println!("--- Async Chain Operations Completed ---");
    Ok(())
}
