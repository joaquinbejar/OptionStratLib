//! Combination search shared by the strategy optimisers.
//!
//! `process_n_times_iter` walks every combination of `n` legs and collects
//! what the caller builds from each one. The only consumer is
//! [`crate::strategies::custom`], so the helper is strategies-owned and
//! reports [`StrategyError`] (ADR-0001 D2, amended: ownership follows the
//! responsibility, not the historical signature).

use crate::error::StrategyError;
use itertools::Itertools;
use rayon::prelude::*;

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
/// * `Result<Vec<Y>, StrategyError>` - A `Result` containing a vector of the combined results from the closure
///   or an error if the input slice is empty.
///
/// # Errors
///
/// Returns an error if the input `positions` slice is empty.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), optionstratlib::error::StrategyError> {
/// use optionstratlib::strategies::process_n_times_iter;
///
/// let numbers = vec![1, 2, 3];
/// let n = 2;
/// let result = process_n_times_iter(&numbers, n, |combination| {
///     vec![combination[0] + combination[1]]
/// })?;
///
/// assert_eq!(result, vec![2, 3, 4, 4, 5, 6]);
/// # Ok(())
/// # }
/// ```
pub fn process_n_times_iter<T, Y, F>(
    positions: &[T],
    n: usize,
    process_combination: F,
) -> Result<Vec<Y>, StrategyError>
where
    F: FnMut(&[&T]) -> Vec<Y> + Send + Sync,
    T: Clone + Send + Sync,
    Y: Send,
{
    if positions.is_empty() {
        return Err(StrategyError::empty_collection("positions"));
    }

    let combinations: Vec<_> = positions.iter().combinations_with_replacement(n).collect();
    let process_combination = std::sync::Mutex::new(process_combination);

    Ok(combinations
        .par_iter()
        .flat_map(|combination| {
            // Mutex-poison recovery: a panic in one closure invocation
            // shouldn't poison the entire combination scan.
            let mut closure = process_combination
                .lock()
                .unwrap_or_else(|e| e.into_inner());
            closure(combination)
        })
        .collect())
}

#[cfg(test)]
mod tests_process_n_times_iter {
    use super::*;

    #[test]
    fn test_empty_vector() {
        let empty_vec: Vec<i32> = vec![];
        let result = process_n_times_iter(&empty_vec, 1, |_| vec![42]);
        // The helper now reports the strategies error of its own layer.
        match result {
            Err(StrategyError::EmptyCollection { context }) => assert_eq!(context, "positions"),
            other => panic!("expected StrategyError::EmptyCollection, got {other:?}"),
        }
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
