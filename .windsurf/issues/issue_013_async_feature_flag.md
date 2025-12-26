# Issue #13: Add async feature flag for I/O operations

## Title
`feat: Add async feature flag for asynchronous I/O operations`

## Labels
- `enhancement`
- `feature`
- `priority-low`

## Description

Add an optional `async` feature flag that enables asynchronous I/O operations for loading option chains and market data.

### Current State
- All I/O operations are synchronous
- May block the main thread during data loading
- Not suitable for async applications

### Target State
- Async feature is optional and doesn't affect sync users
- Async API mirrors sync API
- Better integration with async runtimes

## Tasks

- [ ] Add `tokio` as an optional dependency in `Cargo.toml`
- [ ] Create `async` feature flag in `Cargo.toml`
- [ ] Add async versions of file loading functions:
  - `load_chain_async`
  - `read_ohlcv_async`
- [ ] Add async versions of data fetching functions (if applicable)
- [ ] Ensure feature doesn't affect sync users
- [ ] Add documentation for async usage
- [ ] Add async examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Async feature is optional and doesn't affect sync users
- [ ] Async API mirrors sync API
- [ ] Documentation explains when to use async
- [ ] Examples demonstrate async usage
- [ ] No performance regression for sync users

## Technical Notes

### Cargo.toml Changes

```toml
[features]
default = []
plotly = ["dep:plotly"]
async = ["tokio"]

[dependencies]
tokio = { version = "1", features = ["fs", "io-util"], optional = true }
```

### Async API Pattern

```rust
// src/chains/chain.rs

impl OptionChain {
    /// Loads an option chain from a JSON file (sync version)
    pub fn load_from_json(path: &str) -> Result<Self, ChainError> {
        let data = std::fs::read_to_string(path)?;
        serde_json::from_str(&data).map_err(ChainError::from)
    }
    
    /// Loads an option chain from a JSON file (async version)
    #[cfg(feature = "async")]
    pub async fn load_from_json_async(path: &str) -> Result<Self, ChainError> {
        let data = tokio::fs::read_to_string(path).await?;
        serde_json::from_str(&data).map_err(ChainError::from)
    }
}
```

### Example Usage

```rust
// examples/async_chain_loading.rs

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let chain = OptionChain::load_from_json_async("data/options.json").await?;
    println!("Loaded {} options", chain.options.len());
    Ok(())
}
```

### Files to Update
- `Cargo.toml` (add feature and dependency)
- `src/chains/chain.rs` (add async methods)
- `src/utils/csv.rs` (add async methods)
- `examples/` (add async examples)

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

None
