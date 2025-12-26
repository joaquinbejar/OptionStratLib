# Issue #11: Add benchmarks for critical paths

## Title
`test: Add comprehensive benchmarks for critical code paths`

## Labels
- `testing`
- `performance`
- `priority-low`

## Description

Currently there is only one benchmark file. Adding more benchmarks will help identify performance regressions and guide optimization efforts.

### Current State
- Limited benchmark coverage
- No baseline performance metrics documented
- Difficult to detect performance regressions

### Target State
- Comprehensive benchmarks for all critical paths
- Baseline metrics documented
- CI can detect performance regressions

## Tasks

- [ ] Add benchmarks for Greeks calculations:
  - Delta, Gamma, Theta, Vega, Rho
  - Individual and batch calculations
- [ ] Add benchmarks for Black-Scholes pricing:
  - Call and put pricing
  - With and without Greeks
- [ ] Add benchmarks for option chain construction:
  - Chain building from parameters
  - Chain loading from JSON/CSV
- [ ] Add benchmarks for strategy profit calculations:
  - Single point P&L
  - P&L curve generation
- [ ] Add benchmarks for optimization routines:
  - Strategy optimization
  - Delta neutrality adjustments
- [ ] Document baseline performance metrics
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] Benchmarks cover all critical code paths
- [ ] Baseline metrics are documented in README or docs
- [ ] Benchmarks run in CI (optional but recommended)
- [ ] No significant performance regressions detected

## Technical Notes

### Benchmark Structure

```rust
// benches/greeks_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use optionstratlib::prelude::*;

fn delta_benchmark(c: &mut Criterion) {
    let option = create_test_option();
    
    c.bench_function("delta_calculation", |b| {
        b.iter(|| {
            black_box(option.delta())
        })
    });
}

fn greeks_batch_benchmark(c: &mut Criterion) {
    let options: Vec<Options> = create_test_options(1000);
    
    c.bench_function("greeks_batch_1000", |b| {
        b.iter(|| {
            for opt in &options {
                black_box(opt.greeks());
            }
        })
    });
}

criterion_group!(benches, delta_benchmark, greeks_batch_benchmark);
criterion_main!(benches);
```

### Benchmark Categories

| Category | Benchmarks |
|----------|------------|
| Greeks | delta, gamma, theta, vega, rho, all_greeks |
| Pricing | bs_call, bs_put, binomial, monte_carlo |
| Chains | build_chain, load_json, filter_chain |
| Strategies | profit_at_price, profit_curve, break_even |
| Optimization | find_optimal, delta_adjust |

### Files to Create
- `benches/greeks_bench.rs`
- `benches/pricing_bench.rs`
- `benches/chain_bench.rs`
- `benches/strategy_bench.rs`
- `benches/mod.rs` (update)

## Estimated Effort

**Medium (4-6 hours)**

## Dependencies

None

## Related Issues

- Issue #7: Reduce unnecessary clone() calls (benchmarks will help verify)
