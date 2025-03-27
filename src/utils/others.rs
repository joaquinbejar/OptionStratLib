/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/9/24
******************************************************************************/
use crate::constants::TOLERANCE;
use crate::error::DecimalError;
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive};
use rand::{Rng, rng};
use rayon::prelude::*;
use rust_decimal::Decimal;
use std::collections::BTreeSet;

/// Checks for approximate equality between two f64 values within a defined tolerance.
///
/// This function compares two floating-point numbers and returns `true` if the absolute
/// difference between them is less than the predefined `TOLERANCE` constant.  It is useful
/// for comparing floating-point values that may be subject to small rounding errors.
///
/// # Arguments
///
/// * `a` - The first f64 value to compare.
/// * `b` - The second f64 value to compare.
///
/// # Returns
///
/// `true` if the absolute difference between `a` and `b` is less than `TOLERANCE`, `false` otherwise.
///
/// # Example
///
/// ```
/// use optionstratlib::utils::others::approx_equal;
///
/// let x = 1.0;
/// let y = 1.00000001;
/// assert!(approx_equal(x, y)); // Returns true
///
/// let x = 1.0;
/// let y = 1.1;
/// assert!(!approx_equal(x, y)); // Returns false
/// ```
#[allow(dead_code)]
pub fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE.to_f64().unwrap()
}

/// Gets a random element from a BTreeSet.
///
/// This function returns a random element from the provided BTreeSet using a uniform distribution.
/// If the set is empty, it returns None.
///
/// # Type Parameters
///
/// * `T` - The type of elements in the BTreeSet
///
/// # Arguments
///
/// * `set` - A reference to a BTreeSet containing elements of type T
///
/// # Returns
///
/// * `Option<&T>` - A reference to a random element from the set, or None if the set is empty
///
pub fn get_random_element<T>(set: &BTreeSet<T>) -> Option<&T> {
    if set.is_empty() {
        return None;
    }
    let mut thread_rng = rng();
    let random_index = thread_rng.random_range(0..set.len());
    set.iter().nth(random_index)
}

/// Generates a random `Decimal` value using the provided random number generator.
///
/// This function takes a mutable reference to a random number generator (`rand::Rng`)
/// and uses it to generate a random `f64` value, which is then converted to a `Decimal`.
///
/// # Arguments
///
/// * `rng` - A mutable reference to a random number generator. This allows the function
///   to generate different random numbers on each call.
///
/// # Returns
///
/// A `Result` containing either the generated `Decimal` or a `DecimalError` if the
/// conversion from `f64` to `Decimal` fails.
///
/// # Errors
///
/// Returns a `DecimalError::ConversionError` if the `f64` value generated by the random
/// number generator cannot be converted to a `Decimal`. This can occur if the `f64`
/// value is NaN or infinite.
///
pub fn random_decimal(rng: &mut impl Rng) -> Result<Decimal, DecimalError> {
    Decimal::from_f64(rng.random::<f64>()).ok_or(DecimalError::ConversionError {
        // The source type being converted from
        from_type: "f64".to_string(),
        // The destination type being converted to
        to_type: "Decimal".to_string(),
        // Detailed explanation of why the conversion failed
        reason: "Failed to convert f64 to Decimal".to_string(),
    })
}

/// Processes combinations of elements from a slice in parallel.
///
/// This function takes a slice of elements, a combination size `n`, and a closure `process_combination`.
/// It generates all combinations with replacement of size `n` from the input slice and processes each combination
/// using the provided closure. The results from each combination processing are collected into a single vector.
///
/// The processing is done in parallel using Rayon's parallel iterators for improved performance.
///
/// # Arguments
///
/// * `positions` - A slice of elements to generate combinations from.
/// * `n` - The size of the combinations to generate.
/// * `process_combination` - A closure that takes a slice of references to elements from `positions`
///   and returns a vector of results.  This closure should implement `Send + Sync` since it's used in a multithreaded environment.
///
/// # Returns
///
/// * `Result<Vec<Y>, String>` - A `Result` containing a vector of the combined results from the closure
///   or an error string if the input slice is empty.
///
/// # Errors
///
/// Returns an error if the input `positions` slice is empty.
///
/// # Examples
///
/// ```
/// use optionstratlib::utils::others::process_n_times_iter;
///
/// let numbers = vec![1, 2, 3];
/// let n = 2;
/// let result = process_n_times_iter(&numbers, n, |combination| {
///     vec![combination[0] + combination[1]]
/// }).unwrap();
///
/// assert_eq!(result, vec![2, 3, 4, 4, 5, 6]);
/// ```
pub fn process_n_times_iter<T, Y, F>(
    positions: &[T],
    n: usize,
    process_combination: F,
) -> Result<Vec<Y>, String>
where
    F: FnMut(&[&T]) -> Vec<Y> + Send + Sync,
    T: Clone + Send + Sync,
    Y: Send,
{
    if positions.is_empty() {
        return Err("Vector empty".to_string());
    }

    let combinations: Vec<_> = positions.iter().combinations_with_replacement(n).collect();
    let process_combination = std::sync::Mutex::new(process_combination);

    Ok(combinations
        .par_iter()
        .flat_map(|combination| {
            let mut closure = process_combination.lock().unwrap();
            closure(combination)
        })
        .collect())
}

#[cfg(test)]
mod tests_approx_equal {
    use super::*;

    #[test]

    fn test_approx_equal_exact_values() {
        assert!(approx_equal(1.0, 1.0));
    }

    #[test]

    fn test_approx_equal_within_tolerance() {
        let a = 1.00000001;
        let b = 1.0;
        assert!(approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_outside_tolerance() {
        let a = 1.0001;
        let b = 1.0;
        assert!(!approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_negative_values() {
        let a = -1.00000001;
        let b = -1.0;
        assert!(approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_large_values_within_tolerance() {
        let a = 1000000.000000001;
        let b = 1000000.0;
        assert!(approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_large_values_outside_tolerance() {
        let a = 1000000.1;
        let b = 1000000.0;
        assert!(!approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_zero() {
        let a = 0.0;
        let b = 0.0;
        assert!(approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_zero_with_small_value() {
        let a = 0.000000001;
        let b = 0.0;
        assert!(approx_equal(a, b));
    }

    #[test]

    fn test_approx_equal_zero_outside_tolerance() {
        let a = 0.01;
        let b = 0.0;
        assert!(!approx_equal(a, b));
    }
}

#[cfg(test)]
mod tests_get_random_element {
    use super::*;
    use crate::chains::OptionData;
    use crate::pos;
    use std::collections::BTreeSet;

    #[test]

    fn test_get_random_element_empty_set() {
        let set: BTreeSet<i32> = BTreeSet::new();
        assert!(get_random_element(&set).is_none());
    }

    #[test]

    fn test_get_random_element_single_element() {
        let mut set = BTreeSet::new();
        set.insert(42);
        assert_eq!(get_random_element(&set), Some(&42));
    }

    #[test]

    fn test_get_random_element_multiple_elements() {
        let mut set = BTreeSet::new();
        for i in 0..5 {
            set.insert(i);
        }
        let random_element = get_random_element(&set);
        assert!(random_element.is_some());
        assert!((0..5).contains(random_element.unwrap()));
    }

    #[test]

    fn test_get_random_element_with_option_data() {
        let mut set = BTreeSet::new();
        for i in 0..5 {
            let option_data = OptionData::new(
                pos!(100.0 + i as f64), // strike_price
                None,                   // call_bid
                None,                   // call_ask
                None,                   // put_bid
                None,                   // put_ask
                None,                   // implied_volatility
                None,                   // delta
                None,                   // volume
                None,                   // open_interest
                None,
                None,
            );
            set.insert(option_data);
        }

        let random_option = get_random_element(&set);
        assert!(random_option.is_some());

        let strike = random_option.unwrap().strike_price;
        assert!(strike >= pos!(100.0) && strike <= pos!(104.0));
    }

    #[test]

    fn test_get_random_element_distribution() {
        // Test that the distribution is somewhat uniform
        let mut set = BTreeSet::new();
        for i in 0..3 {
            set.insert(i);
        }

        let mut counts = vec![0; 3];
        for _ in 0..1000 {
            if let Some(&value) = get_random_element(&set) {
                counts[value as usize] += 1;
            }
        }

        // Check that each element was selected at least some times
        // (allowing for some random variation)
        for count in counts {
            assert!(count > 200); // Should be around 333 for uniform distribution
        }
    }
}

#[cfg(test)]
mod tests_process_n_times_iter {
    use super::*;

    #[test]

    fn test_empty_vector() {
        let empty_vec: Vec<i32> = vec![];
        let result = process_n_times_iter(&empty_vec, 1, |_| vec![42]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vector empty".to_string());
    }

    #[test]

    fn test_single_element_single_combination() {
        let vec = vec![1];
        let result = process_n_times_iter(&vec, 1, |combination| vec![*combination[0] * 2]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![2]);
    }

    #[test]

    fn test_multiple_elements_single_output() {
        let vec = vec![1, 2, 3];
        let result =
            process_n_times_iter(&vec, 2, |combination| vec![combination[0] + combination[1]]);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.len(), 6);
        assert!(result.contains(&2)); // 1 + 1
        assert!(result.contains(&3)); // 1 + 2
        assert!(result.contains(&4)); // 2 + 2
    }

    #[test]

    fn test_type_conversion() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, 1, |combination| vec![combination[0].to_string()]);
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, vec!["1", "2"]);
    }

    #[test]

    fn test_multiple_outputs_per_combination() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, 1, |combination| {
            vec![combination[0] * 2, combination[0] * 3]
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result, vec![2, 3, 4, 6]);
    }

    #[test]

    fn test_empty_output() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, 1, |_| Vec::<i32>::new());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }

    #[test]

    fn test_with_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct TestStruct {
            value: i32,
        }

        let vec = vec![TestStruct { value: 1 }, TestStruct { value: 2 }];

        let result = process_n_times_iter(&vec, 2, |combination| {
            vec![TestStruct {
                value: combination[0].value + combination[1].value,
            }]
        });

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains(&TestStruct { value: 2 })); // 1 + 1
        assert!(result.contains(&TestStruct { value: 3 })); // 1 + 2
        assert!(result.contains(&TestStruct { value: 4 })); // 2 + 2
    }

    #[test]

    fn test_combination_size_larger_than_input() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, 3, |combination| {
            let sum = combination.iter().copied().sum::<i32>();
            vec![sum]
        });

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(!result.is_empty());

        let expected_sums = vec![3, 4, 5, 6]; // 1+1+1, 1+1+2, 1+2+2, 2+2+2
        for sum in expected_sums {
            assert!(result.contains(&sum));
        }
    }

    #[test]

    fn test_mutable_state() {
        let vec = vec![1, 2];
        let mut sum = 0;
        let result = process_n_times_iter(&vec, 1, |combination| {
            sum += combination[0];
            vec![sum]
        });
        assert!(result.is_ok());
    }

    #[test]

    fn test_filter_combinations() {
        let vec = vec![1, 2, 3, 4];
        let result = process_n_times_iter(&vec, 2, |combination| {
            if combination[0] + combination[1] > 5 {
                vec![combination[0] + combination[1]]
            } else {
                vec![]
            }
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.iter().all(|&x| x > 5));
    }
}

#[cfg(test)]
mod tests_random_decimal {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;
    use rand::rngs::mock::StepRng;

    use tracing::info;

    #[test]
    fn test_random_decimal_generates_valid_value() {
        // Create a deterministic RNG for testing
        let mut t_rng = StepRng::new(42, 1);

        // Call the function
        let result = random_decimal(&mut t_rng);

        // Check that it succeeded
        assert!(result.is_ok());

        // Verify the value is within expected range [0.0, 1.0)
        let decimal = result.unwrap();
        assert!(decimal >= Decimal::ZERO);
        assert!(decimal < Decimal::ONE);
    }

    #[test]
    fn test_random_decimal_different_calls_different_values() {
        // Use SmallRng instead of StepRng to ensure different values
        let mut t_rng = SmallRng::seed_from_u64(42);

        // Generate two decimal values
        let decimal1 = random_decimal(&mut t_rng).unwrap();
        let decimal2 = random_decimal(&mut t_rng).unwrap();

        // Values should be different
        assert_ne!(decimal1, decimal2);
    }

    #[test]
    fn test_random_decimal_reproduces_expected_values() {
        // Using a known seed should produce predictable outputs
        let mut t_rng = SmallRng::seed_from_u64(12345);

        // Generate the value once
        let decimal = random_decimal(&mut t_rng).unwrap();

        // Store this value for future runs
        info!("Generated decimal: {}", decimal);

        // Reset RNG with same seed
        let mut rng2 = SmallRng::seed_from_u64(12345);

        // The second generation should match the first
        let decimal2 = random_decimal(&mut rng2).unwrap();
        assert_eq!(decimal, decimal2);
    }

    #[test]
    fn test_random_decimal_with_multiple_rng_types() {
        // Test with different RNG implementations

        // Mock RNG
        {
            let mut t_rng = StepRng::new(1, 1);
            assert!(random_decimal(&mut t_rng).is_ok());
        }

        // Thread-local RNG
        {
            let mut t_rng = rng();
            assert!(random_decimal(&mut t_rng).is_ok());
        }

        // Small RNG with seed
        {
            let mut t_rng = SmallRng::seed_from_u64(42);
            assert!(random_decimal(&mut t_rng).is_ok());
        }
    }

    // This test verifies we can create multiple random decimals
    #[test]
    fn test_multiple_random_decimals() {
        let mut t_rng = SmallRng::seed_from_u64(42);
        let decimals: Vec<Decimal> = (0..10)
            .map(|_| random_decimal(&mut t_rng).unwrap())
            .collect();

        // Check we have 10 values
        assert_eq!(decimals.len(), 10);

        // Check they're all different
        for i in 0..9 {
            assert_ne!(decimals[i], decimals[i + 1]);
        }
    }
}
