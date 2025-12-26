# OptionStratLib Refactoring Issues

This document describes the GitHub issues to be created for the refactoring of OptionStratLib.
Each issue is designed to be self-contained and can be worked on independently.

---

## Issue #1: Reduce `.unwrap()` usage in `chains/chain.rs`

### Title
`refactor: Replace unwrap() calls with proper error handling in chains/chain.rs`

### Labels
`refactor`, `error-handling`, `priority-high`

### Description
The file `src/chains/chain.rs` contains 257 occurrences of `.unwrap()` which can cause panics in production. This issue focuses on replacing these with proper error handling.

### Tasks
- [ ] Audit all `.unwrap()` calls in `chains/chain.rs`
- [ ] Replace with `?` operator where the function returns `Result`
- [ ] Replace with `unwrap_or_default()` or `unwrap_or_else()` where appropriate
- [ ] Add proper error types to `error/chains.rs` if needed
- [ ] Update tests to verify error handling behavior
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- No `.unwrap()` calls remain in production code (only in tests)
- All existing tests pass
- New error cases are properly documented

### Estimated Effort
Medium (4-6 hours)

---

## Issue #2: Reduce `.unwrap()` usage in `greeks/equations.rs`

### Title
`refactor: Replace unwrap() calls with proper error handling in greeks/equations.rs`

### Labels
`refactor`, `error-handling`, `priority-high`

### Description
The file `src/greeks/equations.rs` contains 156 occurrences of `.unwrap()`. Greeks calculations are critical for options pricing and should never panic.

### Tasks
- [ ] Audit all `.unwrap()` calls in `greeks/equations.rs`
- [ ] Replace with `?` operator where the function returns `Result`
- [ ] Use `GreeksError` for domain-specific errors
- [ ] Add `#[must_use]` annotations where appropriate
- [ ] Update tests to verify error handling behavior
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- No `.unwrap()` calls remain in production code
- All existing tests pass
- Error messages are clear and actionable

### Estimated Effort
Medium (3-5 hours)

---

## Issue #3: Reduce `.unwrap()` usage in `model/option.rs`

### Title
`refactor: Replace unwrap() calls with proper error handling in model/option.rs`

### Labels
`refactor`, `error-handling`, `priority-high`

### Description
The file `src/model/option.rs` contains 134 occurrences of `.unwrap()`. The Options struct is fundamental to the library and should handle errors gracefully.

### Tasks
- [ ] Audit all `.unwrap()` calls in `model/option.rs`
- [ ] Replace with proper error handling using `OptionsError`
- [ ] Ensure all public methods return `Result` where failure is possible
- [ ] Update documentation with error conditions
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- No `.unwrap()` calls remain in production code
- All existing tests pass
- API documentation includes error conditions

### Estimated Effort
Medium (4-6 hours)

---

## Issue #4: Resolve TODOs in `pricing/black_scholes_model.rs`

### Title
`fix: Resolve 14 TODO/FIXME items in black_scholes_model.rs`

### Labels
`bug`, `pricing`, `priority-high`

### Description
The file `src/pricing/black_scholes_model.rs` contains 14 TODO/FIXME comments that need to be addressed. These are critical for correct options pricing.

### Tasks
- [ ] Review each TODO/FIXME comment
- [ ] Implement missing functionality
- [ ] Add tests for edge cases mentioned in TODOs
- [ ] Remove TODO comments once resolved
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- All TODO/FIXME comments are resolved or converted to tracked issues
- Pricing accuracy is maintained or improved
- New tests cover the resolved items

### Estimated Effort
High (6-8 hours)

---

## Issue #5: Resolve TODOs in `model/position.rs`

### Title
`fix: Resolve 4 TODO/FIXME items in model/position.rs`

### Labels
`bug`, `model`, `priority-medium`

### Description
The file `src/model/position.rs` contains 4 TODO/FIXME comments that need to be addressed.

### Tasks
- [ ] Review each TODO/FIXME comment
- [ ] Implement missing functionality
- [ ] Add tests for resolved items
- [ ] Remove TODO comments once resolved
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- All TODO/FIXME comments are resolved
- Position management works correctly
- Tests verify the resolved functionality

### Estimated Effort
Low (2-3 hours)

---

## Issue #6: Extract common logic from strategy files

### Title
`refactor: Extract common strategy logic to reduce file sizes`

### Labels
`refactor`, `strategies`, `priority-medium`

### Description
Several strategy files are excessively large due to duplicated logic:
- `short_strangle.rs` (129KB)
- `iron_condor.rs` (126KB)
- `iron_butterfly.rs` (113KB)
- `long_butterfly_spread.rs` (112KB)

### Tasks
- [ ] Identify common patterns across strategy implementations
- [ ] Create shared traits for strategy categories:
  - `SpreadStrategy` for vertical spreads
  - `ButterflyStrategy` for butterfly patterns
  - `CondorStrategy` for condor patterns
- [ ] Extract common calculation methods to `strategies/utils.rs`
- [ ] Refactor strategies to use shared traits and utilities
- [ ] Ensure all tests pass after refactoring
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Each strategy file is under 50KB
- No functionality is lost
- Code duplication is significantly reduced
- All tests pass

### Estimated Effort
High (8-12 hours)

---

## Issue #7: Reduce unnecessary `.clone()` calls

### Title
`perf: Reduce unnecessary clone() calls across the codebase`

### Labels
`performance`, `refactor`, `priority-medium`

### Description
The codebase contains 795 occurrences of `.clone()`. Many of these could be avoided with better use of references, `Cow<T>`, or `Arc<T>`.

### Tasks
- [ ] Audit `.clone()` calls in high-frequency code paths:
  - `chains/chain.rs` (55 occurrences)
  - `strategies/short_strangle.rs` (49 occurrences)
  - `strategies/iron_condor.rs` (46 occurrences)
- [ ] Replace with references where possible
- [ ] Use `Cow<'_, T>` for data that is rarely modified
- [ ] Use `Arc<T>` for shared data in multi-threaded contexts
- [ ] Benchmark critical paths before and after
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Clone count reduced by at least 50%
- No performance regression (verified by benchmarks)
- All tests pass

### Estimated Effort
Medium (4-6 hours)

---

## Issue #8: Implement Collar strategy

### Title
`feat: Complete implementation of Collar strategy`

### Labels
`enhancement`, `strategies`, `priority-medium`

### Description
The file `src/strategies/collar.rs` is almost empty (388 bytes). The Collar strategy is an important protective strategy that should be fully implemented.

### Tasks
- [ ] Implement `Collar` struct with all required fields
- [ ] Implement `Strategable` trait
- [ ] Implement `StrategyConstructor` trait
- [ ] Implement `Profit` calculations
- [ ] Implement `Greeks` calculations
- [ ] Implement `DeltaNeutrality` trait
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Collar strategy is fully functional
- All trait implementations are complete
- Tests cover edge cases
- Documentation includes usage examples

### Estimated Effort
High (6-8 hours)

---

## Issue #9: Implement Protective Put strategy

### Title
`feat: Complete implementation of Protective Put strategy`

### Labels
`enhancement`, `strategies`, `priority-medium`

### Description
The file `src/strategies/protective_put.rs` is almost empty (358 bytes). The Protective Put is a fundamental hedging strategy that should be fully implemented.

### Tasks
- [ ] Implement `ProtectivePut` struct with all required fields
- [ ] Implement `Strategable` trait
- [ ] Implement `StrategyConstructor` trait
- [ ] Implement `Profit` calculations
- [ ] Implement `Greeks` calculations
- [ ] Implement `DeltaNeutrality` trait
- [ ] Add comprehensive tests
- [ ] Add documentation with examples
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Protective Put strategy is fully functional
- All trait implementations are complete
- Tests cover edge cases
- Documentation includes usage examples

### Estimated Effort
Medium (4-6 hours)

---

## Issue #10: Improve error context with anyhow

### Title
`refactor: Add error context using anyhow at API boundaries`

### Labels
`refactor`, `error-handling`, `priority-low`

### Description
While the project uses `thiserror` for typed errors, adding `anyhow::Context` at API boundaries would improve error messages for users.

### Tasks
- [ ] Add `anyhow` as a dependency
- [ ] Identify public API boundaries
- [ ] Add `.context()` calls to provide meaningful error messages
- [ ] Update error documentation
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Error messages include context about what operation failed
- Existing error types are preserved
- No breaking changes to public API

### Estimated Effort
Low (2-3 hours)

---

## Issue #11: Add benchmarks for critical paths

### Title
`test: Add comprehensive benchmarks for critical code paths`

### Labels
`testing`, `performance`, `priority-low`

### Description
Currently there is only one benchmark file. Adding more benchmarks will help identify performance regressions.

### Tasks
- [ ] Add benchmarks for Greeks calculations
- [ ] Add benchmarks for Black-Scholes pricing
- [ ] Add benchmarks for option chain construction
- [ ] Add benchmarks for strategy profit calculations
- [ ] Document baseline performance metrics
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Benchmarks cover all critical code paths
- Baseline metrics are documented
- CI can detect performance regressions

### Estimated Effort
Medium (4-6 hours)

---

## Issue #12: Add property-based testing

### Title
`test: Add property-based testing for mathematical invariants`

### Labels
`testing`, `quality`, `priority-low`

### Description
Add property-based testing using `proptest` to validate mathematical invariants like Put-Call Parity and Greeks bounds.

### Tasks
- [ ] Add `proptest` as a dev dependency
- [ ] Add property tests for Put-Call Parity
- [ ] Add property tests for Greeks bounds (0 ≤ delta ≤ 1 for calls)
- [ ] Add property tests for arbitrage-free pricing
- [ ] Add property tests for Positive type invariants
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Property tests cover key mathematical invariants
- Tests run as part of the test suite
- No false positives in normal operation

### Estimated Effort
Medium (4-6 hours)

---

## Issue #13: Add async feature flag for I/O operations

### Title
`feat: Add async feature flag for asynchronous I/O operations`

### Labels
`enhancement`, `feature`, `priority-low`

### Description
Add an optional `async` feature flag that enables asynchronous I/O operations for loading option chains and market data.

### Tasks
- [ ] Add `tokio` as an optional dependency
- [ ] Create `async` feature flag in `Cargo.toml`
- [ ] Add async versions of file loading functions
- [ ] Add async versions of data fetching functions
- [ ] Add documentation for async usage
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- Async feature is optional and doesn't affect sync users
- Async API mirrors sync API
- Documentation explains when to use async

### Estimated Effort
Medium (4-6 hours)

---

## Issue #14: Improve documentation for new metrics modules

### Title
`docs: Add comprehensive documentation for metrics modules`

### Labels
`documentation`, `priority-low`

### Description
The new metrics modules (`composite/`, `liquidity/`, `stress/`, `temporal/`) need more comprehensive documentation with examples.

### Tasks
- [ ] Add module-level documentation with overview
- [ ] Add examples for each trait implementation
- [ ] Add mathematical background where appropriate
- [ ] Add usage examples in doc comments
- [ ] Verify examples compile with `cargo test --doc`
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- All public items have documentation
- Examples are runnable
- Mathematical formulas are explained

### Estimated Effort
Low (2-4 hours)

---

## Issue #15: Reduce `.expect()` usage in production code

### Title
`refactor: Replace expect() calls with proper error handling`

### Labels
`refactor`, `error-handling`, `priority-medium`

### Description
The codebase contains 163 occurrences of `.expect()`. While better than `.unwrap()`, these should still be replaced with proper error handling in production code.

### Tasks
- [ ] Audit `.expect()` calls in production code
- [ ] Focus on high-impact files:
  - `pnl/metrics.rs` (37 occurrences)
  - `model/format.rs` (14 occurrences)
- [ ] Replace with `?` operator or proper error handling
- [ ] Keep `.expect()` only in tests and initialization code
- [ ] Run `make lint-fix` and `make pre-push` to verify

### Acceptance Criteria
- `.expect()` only used in tests and initialization
- All existing tests pass
- Error messages are preserved or improved

### Estimated Effort
Medium (3-5 hours)

---

## Summary Table

| Issue | Title | Priority | Effort | Labels |
|-------|-------|----------|--------|--------|
| #1 | Reduce unwrap() in chains/chain.rs | High | Medium | refactor, error-handling |
| #2 | Reduce unwrap() in greeks/equations.rs | High | Medium | refactor, error-handling |
| #3 | Reduce unwrap() in model/option.rs | High | Medium | refactor, error-handling |
| #4 | Resolve TODOs in black_scholes_model.rs | High | High | bug, pricing |
| #5 | Resolve TODOs in model/position.rs | Medium | Low | bug, model |
| #6 | Extract common strategy logic | Medium | High | refactor, strategies |
| #7 | Reduce unnecessary clone() calls | Medium | Medium | performance, refactor |
| #8 | Implement Collar strategy | Medium | High | enhancement, strategies |
| #9 | Implement Protective Put strategy | Medium | Medium | enhancement, strategies |
| #10 | Improve error context with anyhow | Low | Low | refactor, error-handling |
| #11 | Add benchmarks for critical paths | Low | Medium | testing, performance |
| #12 | Add property-based testing | Low | Medium | testing, quality |
| #13 | Add async feature flag | Low | Medium | enhancement, feature |
| #14 | Improve metrics documentation | Low | Low | documentation |
| #15 | Reduce expect() usage | Medium | Medium | refactor, error-handling |

---

## Recommended Order of Execution

1. **Phase 1 - Critical Fixes** (Issues #1-#4)
   - Focus on error handling and pricing correctness
   - Estimated: 2-3 weeks

2. **Phase 2 - Code Quality** (Issues #5-#7, #15)
   - Reduce technical debt and improve performance
   - Estimated: 2 weeks

3. **Phase 3 - Feature Completion** (Issues #8-#9)
   - Complete missing strategies
   - Estimated: 1-2 weeks

4. **Phase 4 - Polish** (Issues #10-#14)
   - Improve testing, documentation, and developer experience
   - Estimated: 2 weeks

---

*Generated on: 2024-12-24*
*Total Estimated Effort: 60-90 hours*
