# OptionStratLib Refactoring Issues

This directory contains individual issue files for the refactoring of OptionStratLib.
Each file describes a self-contained issue that can be worked on independently.

## Issue Index

### Priority High (Critical Fixes)

| Issue | Title | Effort | File |
|-------|-------|--------|------|
| #1 | Reduce unwrap() in chains/chain.rs | Medium | [issue_001](./issue_001_unwrap_chains_chain.md) |
| #2 | Reduce unwrap() in greeks/equations.rs | Medium | [issue_002](./issue_002_unwrap_greeks_equations.md) |
| #3 | Reduce unwrap() in model/option.rs | Medium | [issue_003](./issue_003_unwrap_model_option.md) |
| #4 | Resolve TODOs in black_scholes_model.rs | High | [issue_004](./issue_004_todos_black_scholes.md) |
| #16 | Improve Positive type safety and API | High | [issue_016](./issue_016_improve_positive_type.md) |
| #17 | Reduce unwrap() in src/model (remaining files) | High | [issue_017](./issue_017_unwrap_model_remaining.md) |

### Priority Medium (Code Quality)

| Issue | Title | Effort | File |
|-------|-------|--------|------|
| #5 | Resolve TODOs in model/position.rs | Low | [issue_005](./issue_005_todos_position.md) |
| #6 | Extract common strategy logic | High | [issue_006](./issue_006_extract_strategy_logic.md) |
| #7 | Reduce unnecessary clone() calls | Medium | [issue_007](./issue_007_reduce_clone_calls.md) |
| #8 | Implement Collar strategy | High | [issue_008](./issue_008_implement_collar.md) |
| #9 | Implement Protective Put strategy | Medium | [issue_009](./issue_009_implement_protective_put.md) |
| #15 | Reduce expect() usage | Medium | [issue_015](./issue_015_reduce_expect_usage.md) |

### Priority Low (Polish)

| Issue | Title | Effort | File |
|-------|-------|--------|------|
| #10 | Improve error context with anyhow | Low | [issue_010](./issue_010_error_context_anyhow.md) |
| #11 | Add benchmarks for critical paths | Medium | [issue_011](./issue_011_add_benchmarks.md) |
| #12 | Add property-based testing | Medium | [issue_012](./issue_012_property_based_testing.md) |
| #13 | Add async feature flag | Medium | [issue_013](./issue_013_async_feature_flag.md) |
| #14 | Improve metrics documentation | Low | [issue_014](./issue_014_improve_metrics_docs.md) |

## Recommended Execution Order

### Phase 1: Critical Fixes (3-4 weeks)
1. Issue #16 - Improve Positive type safety and API
2. Issue #4 - Resolve TODOs in black_scholes_model.rs
3. Issue #1 - Reduce unwrap() in chains/chain.rs
4. Issue #2 - Reduce unwrap() in greeks/equations.rs
5. Issue #3 - Reduce unwrap() in model/option.rs
6. Issue #17 - Reduce unwrap() in src/model (remaining files)

### Phase 2: Code Quality (2 weeks)
7. Issue #5 - Resolve TODOs in model/position.rs
8. Issue #15 - Reduce expect() usage
9. Issue #6 - Extract common strategy logic
10. Issue #7 - Reduce unnecessary clone() calls

### Phase 3: Feature Completion (1-2 weeks)
11. Issue #8 - Implement Collar strategy
12. Issue #9 - Implement Protective Put strategy

### Phase 4: Polish (2 weeks)
13. Issue #11 - Add benchmarks for critical paths
14. Issue #12 - Add property-based testing
15. Issue #10 - Improve error context with anyhow
16. Issue #13 - Add async feature flag
17. Issue #14 - Improve metrics documentation

## Labels Reference

| Label | Description |
|-------|-------------|
| `priority-high` | Critical issues that should be addressed first |
| `priority-medium` | Important but not blocking |
| `priority-low` | Nice-to-have improvements |
| `refactor` | Code refactoring without changing behavior |
| `bug` | Something isn't working correctly |
| `enhancement` | New feature or request |
| `error-handling` | Related to error handling improvements |
| `performance` | Performance optimization |
| `testing` | Testing improvements |
| `documentation` | Documentation improvements |
| `strategies` | Related to trading strategies |
| `pricing` | Related to options pricing |
| `model` | Related to core data models |

## Effort Estimates

| Effort | Hours |
|--------|-------|
| Low | 2-3 hours |
| Medium | 4-6 hours |
| High | 6-12 hours |

## Total Estimated Effort

- **Phase 1**: 35-51 hours (includes Issue #16: 8-12h, Issue #17: 10-14h)
- **Phase 2**: 17-26 hours
- **Phase 3**: 10-14 hours
- **Phase 4**: 12-19 hours
- **Total**: 74-110 hours

---

*Updated on: 2024-12-25*
