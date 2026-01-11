# Issue #257: Update dependencies - zip 7.0, plotly 0.14, positive 0.3.0

## Summary

Update the following dependencies to their latest versions:

| Dependency | Current | Target | Notes |
|------------|---------|--------|-------|
| `zip` | 6.0 | 7.0 | Major version update |
| `plotly` | 0.13 | 0.14 | Minor version update |
| `positive` | 0.2.1 | 0.3.0 | Minor version with new features |

## Changes Required

### 1. Update Cargo.toml

```toml
# Before
zip = "6.0"
plotly = { version = "0.13", default-features = false, features = ["static_export_default"] }
positive = { version = "0.2.1", features = ["utoipa"] }

# After
zip = "7.0"
plotly = { version = "0.14", default-features = false, features = ["static_export_default"] }
positive = { version = "0.3", features = ["utoipa"] }
```

### 2. Leverage New `positive` Constants

The `positive 0.3.0` crate introduces many new constants that can replace `pos_or_panic!` calls:

#### New Constants Available

| Constant | Value | Replaces |
|----------|-------|----------|
| `Positive::THREE` | 3.0 | `pos_or_panic!(3.0)` |
| `Positive::FOUR` | 4.0 | `pos_or_panic!(4.0)` |
| `Positive::FIVE` | 5.0 | `pos_or_panic!(5.0)` |
| `Positive::SIX` | 6.0 | `pos_or_panic!(6.0)` |
| `Positive::SEVEN` | 7.0 | `pos_or_panic!(7.0)` |
| `Positive::EIGHT` | 8.0 | `pos_or_panic!(8.0)` |
| `Positive::NINE` | 9.0 | `pos_or_panic!(9.0)` |
| `Positive::FIFTEEN` | 15.0 | `pos_or_panic!(15.0)` |
| `Positive::TWENTY` | 20.0 | `pos_or_panic!(20.0)` |
| `Positive::TWENTY_FIVE` | 25.0 | `pos_or_panic!(25.0)` |
| `Positive::THIRTY` | 30.0 | `pos_or_panic!(30.0)` |
| `Positive::THIRTY_FIVE` | 35.0 | `pos_or_panic!(35.0)` |
| `Positive::FORTY` | 40.0 | `pos_or_panic!(40.0)` |
| `Positive::FORTY_FIVE` | 45.0 | `pos_or_panic!(45.0)` |
| `Positive::FIFTY` | 50.0 | `pos_or_panic!(50.0)` |
| `Positive::FIFTY_FIVE` | 55.0 | `pos_or_panic!(55.0)` |
| `Positive::SIXTY` | 60.0 | `pos_or_panic!(60.0)` |
| `Positive::SIXTY_FIVE` | 65.0 | `pos_or_panic!(65.0)` |
| `Positive::SEVENTY` | 70.0 | `pos_or_panic!(70.0)` |
| `Positive::SEVENTY_FIVE` | 75.0 | `pos_or_panic!(75.0)` |
| `Positive::EIGHTY` | 80.0 | `pos_or_panic!(80.0)` |
| `Positive::EIGHTY_FIVE` | 85.0 | `pos_or_panic!(85.0)` |
| `Positive::NINETY` | 90.0 | `pos_or_panic!(90.0)` |
| `Positive::NINETY_FIVE` | 95.0 | `pos_or_panic!(95.0)` |
| `Positive::TWO_HUNDRED` | 200.0 | `pos_or_panic!(200.0)` |
| `Positive::THREE_HUNDRED` | 300.0 | `pos_or_panic!(300.0)` |
| `Positive::FOUR_HUNDRED` | 400.0 | `pos_or_panic!(400.0)` |
| `Positive::FIVE_HUNDRED` | 500.0 | `pos_or_panic!(500.0)` |
| `Positive::THOUSAND` | 1000.0 | `pos_or_panic!(1000.0)` |
| `Positive::PI` | π | `pos_or_panic!(3.14159...)` |
| `Positive::E` | e | `pos_or_panic!(2.71828...)` |
| `Positive::INFINITY` | ∞ | Max positive value |

### 3. Leverage New `positive` Methods

New methods that can improve error handling and code quality:

#### Checked/Safe Conversion Methods

- `to_f64_checked()` - Returns `Option<f64>` instead of panicking
- `to_f64_lossy()` - Returns 0.0 on failure instead of panicking
- `to_i64_checked()` - Returns `Option<i64>`
- `to_u64_checked()` - Returns `Option<u64>`
- `to_usize_checked()` - Returns `Option<usize>`

#### Checked Arithmetic

- `checked_sub(&self, rhs: &Self) -> Result<Self, PositiveError>` - Safe subtraction
- `saturating_sub(&self, rhs: &Self) -> Self` - Returns ZERO instead of negative
- `checked_div(&self, rhs: &Self) -> Result<Self, PositiveError>` - Safe division
- `sqrt_checked(&self) -> Result<Positive, PositiveError>` - Safe square root

#### Utility Methods

- `sub_or_zero(&self, other: &Decimal) -> Positive` - Subtraction with zero floor
- `sub_or_none(&self, other: &Decimal) -> Option<Positive>` - Subtraction returning None if negative
- `is_multiple_of(&self, other: &Positive) -> bool` - Check if value is multiple
- `clamp(&self, min: Positive, max: Positive) -> Positive` - Clamp value to range
- `round_to_nice_number(&self) -> Positive` - Round to aesthetically pleasing number

## Implementation Tasks

### Phase 1: Update Dependencies

- [ ] Update `Cargo.toml` with new versions
- [ ] Run `cargo build` to check for breaking changes
- [ ] Fix any compilation errors from API changes

### Phase 2: Replace Constants

- [ ] Search for `pos_or_panic!(5.0)` and replace with `Positive::FIVE`
- [ ] Search for `pos_or_panic!(30.0)` and replace with `Positive::THIRTY`
- [ ] Search for other numeric constants that match new `Positive` constants
- [ ] Focus on production code first, then tests

### Phase 3: Adopt New Methods

- [ ] Replace `.to_f64().unwrap()` patterns with `.to_f64_lossy()` where appropriate
- [ ] Use `checked_sub` and `saturating_sub` for safer arithmetic
- [ ] Use `checked_div` to avoid division by zero panics

### Phase 4: Testing

- [ ] Run full test suite
- [ ] Verify `make lint-fix` passes
- [ ] Verify `make pre-push` passes

## Acceptance Criteria

- [ ] All dependencies updated to target versions
- [ ] No compilation errors
- [ ] All tests pass
- [ ] `pos_or_panic!` calls replaced with constants where applicable
- [ ] New safe methods adopted where beneficial
- [ ] `make lint-fix` passes
- [ ] `make pre-push` passes

## Related

- Continues the work from #227 (unwrap replacement)
- Improves type safety and reduces panic potential
