# Issue #16: Improve `Positive` type safety and API

## Title
`refactor: Improve Positive type to eliminate panics and enhance type safety`

## Labels
- `refactor`
- `type-safety`
- `error-handling`
- `priority-high`

## Description

The `Positive` type in `src/model/positive.rs` is a wrapper around `Decimal` that guarantees non-negative values. However, the current implementation has several issues that can lead to runtime panics and violate the positivity invariant.

### Current State
- **50+ `.unwrap()` calls** in conversion methods that can panic
- **Arithmetic operations that panic** when results would be negative
- **`to_dec_ref_mut()` method** that allows breaking the invariant
- **`From` implementations that panic** instead of using `TryFrom`
- **`is_multiple()` uses `f64::EPSILON`** instead of `Decimal` precision

### Target State
- Zero panics in production code paths
- Safe arithmetic operations with `checked_*` and `saturating_*` variants
- Invariant cannot be violated through public API
- `TryFrom` for fallible conversions
- Proper `Decimal`-based precision for comparisons

## Tasks

### Phase 1: Add Safe Conversion Methods
- [ ] Add `to_f64_checked(&self) -> Option<f64>`
- [ ] Add `to_f64_lossy(&self) -> f64` (returns 0.0 on failure)
- [ ] Add `to_i64_checked(&self) -> Option<i64>`
- [ ] Add `to_u64_checked(&self) -> Option<u64>`
- [ ] Add `to_usize_checked(&self) -> Option<usize>`
- [ ] Add `#[must_use]` to all methods that return values
- [ ] Deprecate panicking versions with `#[deprecated]`

### Phase 2: Add Safe Arithmetic Operations
- [ ] Create `CheckedOps` trait with:
  - `checked_sub(&self, rhs: &Self) -> Result<Self, DecimalError>`
  - `checked_div(&self, rhs: &Self) -> Result<Self, DecimalError>`
- [ ] Create `SaturatingOps` trait with:
  - `saturating_sub(&self, rhs: &Self) -> Self` (returns ZERO if negative)
- [ ] Add `sqrt_checked(&self) -> Result<Positive, DecimalError>`
- [ ] Add `ln_checked(&self) -> Result<Decimal, DecimalError>` (ln can be negative)

### Phase 3: Fix Invariant Violations
- [ ] Remove or make `to_dec_ref_mut()` private (`pub(crate)`)
- [ ] Review all `impl From<T> for Positive` and convert to `TryFrom`
- [ ] Add `const unsafe fn new_unchecked(value: Decimal) -> Self` for performance-critical code
- [ ] Document safety requirements for `new_unchecked`

### Phase 4: Improve Precision
- [ ] Change `is_multiple(&self, other: f64)` to `is_multiple(&self, other: Positive)`
- [ ] Use `EPSILON` constant from `constants.rs` instead of `f64::EPSILON`

### Phase 5: Add NonZeroPositive Type (Optional)
- [ ] Create `NonZeroPositive` struct for values > 0 (not >= 0)
- [ ] Implement conversions between `Positive` and `NonZeroPositive`
- [ ] Use for division denominators and other cases requiring > 0

### Phase 6: Update Tests and Documentation
- [ ] Add tests for all new `_checked` methods
- [ ] Add tests for `saturating_*` operations
- [ ] Update documentation with panic conditions
- [ ] Add examples showing safe vs unsafe usage patterns
- [ ] Run `make lint-fix` and `make pre-push` to verify

## Acceptance Criteria

- [ ] No `.unwrap()` in non-test code paths (except where mathematically impossible to fail)
- [ ] All arithmetic operations have safe alternatives
- [ ] `to_dec_ref_mut()` is not publicly accessible
- [ ] All `From` implementations that can fail are converted to `TryFrom`
- [ ] All methods that return values have `#[must_use]`
- [ ] All existing tests pass
- [ ] New tests cover edge cases

## Technical Notes

### Safe Conversion Pattern

```rust
impl Positive {
    /// Converts to f64, returning None if conversion fails.
    #[must_use]
    pub fn to_f64_checked(&self) -> Option<f64> {
        self.0.to_f64()
    }

    /// Converts to f64 with lossy conversion (returns 0.0 on failure).
    #[must_use]
    pub fn to_f64_lossy(&self) -> f64 {
        self.0.to_f64().unwrap_or(0.0)
    }

    /// Converts to f64.
    /// 
    /// # Panics
    /// 
    /// Panics if the value cannot be represented as f64.
    /// Consider using `to_f64_checked()` or `to_f64_lossy()` instead.
    #[deprecated(since = "0.15.0", note = "Use to_f64_checked() or to_f64_lossy() instead")]
    pub fn to_f64(&self) -> f64 {
        self.0.to_f64().unwrap()
    }
}
```

### Checked Arithmetic Pattern

```rust
/// Trait for checked arithmetic operations on Positive values.
pub trait CheckedOps {
    type Error;
    
    /// Checked subtraction that returns Result instead of panicking.
    fn checked_sub(&self, rhs: &Self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl CheckedOps for Positive {
    type Error = DecimalError;
    
    fn checked_sub(&self, rhs: &Self) -> Result<Self, DecimalError> {
        let result = self.0 - rhs.0;
        Positive::new_decimal(result)
    }
}

/// Trait for saturating arithmetic operations on Positive values.
pub trait SaturatingOps {
    /// Saturating subtraction that returns ZERO instead of negative.
    fn saturating_sub(&self, rhs: &Self) -> Self
    where
        Self: Sized;
}

impl SaturatingOps for Positive {
    fn saturating_sub(&self, rhs: &Self) -> Self {
        if self.0 > rhs.0 {
            Positive(self.0 - rhs.0)
        } else {
            Positive::ZERO
        }
    }
}
```

### TryFrom Pattern

```rust
// Replace this:
impl From<f64> for Positive {
    fn from(value: f64) -> Self {
        Positive::new(value).expect("Value must be positive")
    }
}

// With this:
impl TryFrom<f64> for Positive {
    type Error = DecimalError;
    
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Positive::new(value)
    }
}
```

### NonZeroPositive Type (Optional)

```rust
/// A wrapper type that represents a strictly positive decimal value (> 0).
/// 
/// Unlike `Positive` which allows zero, `NonZeroPositive` guarantees
/// the value is always greater than zero. This is useful for:
/// - Division denominators
/// - Volatility values
/// - Time to expiration
#[derive(PartialEq, Clone, Copy, Hash, ToSchema)]
pub struct NonZeroPositive(Decimal);

impl NonZeroPositive {
    pub fn new(value: f64) -> Result<Self, DecimalError> {
        let dec = Decimal::from_f64(value);
        match dec {
            Some(value) if value > Decimal::ZERO => Ok(NonZeroPositive(value)),
            Some(_) => Err(DecimalError::OutOfBounds {
                value,
                min: f64::MIN_POSITIVE,
                max: f64::MAX,
            }),
            None => Err(DecimalError::ConversionError { ... }),
        }
    }
    
    /// Converts to Positive (always succeeds).
    #[must_use]
    pub fn to_positive(self) -> Positive {
        Positive(self.0)
    }
}

impl From<NonZeroPositive> for Positive {
    fn from(value: NonZeroPositive) -> Self {
        Positive(value.0)
    }
}
```

## Files to Update

- `src/model/positive.rs` (primary)
- `src/error/decimal.rs` (add new error variants if needed)
- `src/constants.rs` (verify EPSILON usage)
- `src/prelude.rs` (export new traits)
- Tests throughout the codebase that use `Positive`

## Estimated Effort

**High (8-12 hours)**

## Dependencies

None

## Related Issues

- Issue #1: Reduce unwrap() in chains/chain.rs
- Issue #2: Reduce unwrap() in greeks/equations.rs
- Issue #3: Reduce unwrap() in model/option.rs

## Breaking Changes

This issue introduces breaking changes:

1. **`From<f64>` → `TryFrom<f64>`**: Code using `.into()` will need to use `try_into()` or `Positive::new()`
2. **`to_dec_ref_mut()` removal**: Code modifying the inner value will need refactoring
3. **Deprecation warnings**: Existing code using `to_f64()`, `to_i64()`, etc. will show warnings

### Migration Guide

```rust
// Before
let p: Positive = 5.0.into();
let f = p.to_f64();

// After
let p = Positive::new(5.0)?;  // or use pos! macro
let f = p.to_f64_lossy();     // or to_f64_checked().unwrap_or(0.0)
```

---

*Generated on: 2024-12-25*
