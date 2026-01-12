# Issue #223: Add async feature flag for asynchronous I/O operations

## Goal
Implement an `async` feature flag in `OptionStratLib` to support asynchronous I/O operations, specifically for chain data fetching and CSV processing.

## Phases

### Phase 1: Planning & Setup
- [ ] Research required dependencies for async support (e.g., `tokio`, `async-trait`, `reqwest`).
- [ ] Create a new branch `feat/issue-223-async-feature`.
- [ ] Update `Cargo.toml` with the `async` feature and optional dependencies.

### Phase 2: Implementation - Core Async Trait & Methods
- [ ] Implement async versions of methods in `src/chains/chain.rs`.
- [ ] Implement async versions of methods in `src/utils/csv.rs`.
- [ ] Ensure backward compatibility with sync operations.

### Phase 3: Examples & Documentation
- [ ] Create async examples in the `examples/` directory.
- [ ] Update documentation to explain how to use the `async` feature.

### Phase 4: Verification
- [ ] Run `cargo test --features async` to verify async implementations.
- [ ] Run `make lint-fix pre-push` for code quality.
- [ ] Create a Pull Request.
