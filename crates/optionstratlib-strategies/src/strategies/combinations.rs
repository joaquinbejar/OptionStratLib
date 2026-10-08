//! Combination search shared by the strategy optimisers.
//!
//! `process_n_times_iter` walks every combination of `n` legs and collects
//! what the caller builds from each one. The only consumer is
//! [`crate::strategies::custom`], so the helper is strategies-owned and
//! reports [`crate::error::StrategyError`] (ADR-0001 D2, amended: ownership follows the
//! responsibility, not the historical signature).
//!
//! `best_candidate` scores the candidates of a chain optimiser's
//! `find_optimal` on the rayon pool and picks the one the serial search
//! picked (#862).

use crate::error::StrategyError;
use itertools::Itertools;
use rayon::prelude::*;
use rust_decimal::Decimal;

/// Candidates [`best_candidate`] scores per round on the rayon pool. A round
/// is collected before it is scored, so the candidates held at once stay
/// bounded however many combinations the chain yields.
const OPTIMISER_ROUND: usize = 4096;

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
/// # fn main() -> Result<(), optionstratlib_strategies::error::StrategyError> {
/// use optionstratlib_strategies::strategies::process_n_times_iter;
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

/// The best-scoring candidate of a chain optimiser's search (#862).
///
/// `score` builds and scores one candidate, or returns `None` to skip it. It
/// runs once per candidate, on the rayon pool, in rounds of
/// [`OPTIMISER_ROUND`] candidates taken in order from `candidates`.
///
/// The result is the candidate with the highest score above `Decimal::MIN`;
/// among equal scores, the one `candidates` yields first. That is the
/// candidate the serial search picked: it started from `Decimal::MIN` and
/// replaced its best only on a strictly greater score. `None` when no
/// candidate scores above `Decimal::MIN`.
pub(crate) fn best_candidate<C, S, F>(candidates: impl Iterator<Item = C>, score: F) -> Option<S>
where
    C: Send,
    S: Send,
    F: Fn(C) -> Option<(Decimal, S)> + Sync,
{
    // The strategy rides through the reduction boxed: rayon carries the
    // partial result through every level of its split, and a strategy held
    // by value there overflowed a worker's stack in debug builds.
    let mut best: Option<(usize, Decimal, Box<S>)> = None;
    let mut indexed = candidates.enumerate();
    loop {
        let round: Vec<(usize, C)> = indexed.by_ref().take(OPTIMISER_ROUND).collect();
        if round.is_empty() {
            break;
        }
        let round_best = round
            .into_par_iter()
            .filter_map(|(index, candidate)| {
                let (value, strategy) = score(candidate)?;
                (value > Decimal::MIN).then(|| (index, value, Box::new(strategy)))
            })
            .reduce_with(better_candidate);
        best = match (best, round_best) {
            (Some(current), Some(challenger)) => Some(better_candidate(current, challenger)),
            (current, challenger) => current.or(challenger),
        };
    }
    best.map(|(_, _, strategy)| *strategy)
}

/// The higher-scoring of two scored candidates, the lower index on a tie.
///
/// Indices are distinct, so this is the maximum of a total order: the
/// reduction picks the same candidate however rayon splits the round.
fn better_candidate<S>(a: (usize, Decimal, S), b: (usize, Decimal, S)) -> (usize, Decimal, S) {
    if b.1 > a.1 || (b.1 == a.1 && b.0 < a.0) {
        b
    } else {
        a
    }
}

#[cfg(test)]
mod tests_best_candidate {
    use super::*;
    use rust_decimal_macros::dec;

    /// The serial search the optimisers ran before #862.
    fn serial(values: &[Option<Decimal>]) -> Option<usize> {
        let mut best_value = Decimal::MIN;
        let mut chosen = None;
        for (index, value) in values.iter().enumerate() {
            if let Some(value) = value
                && *value > best_value
            {
                best_value = *value;
                chosen = Some(index);
            }
        }
        chosen
    }

    fn parallel(values: &[Option<Decimal>]) -> Option<usize> {
        best_candidate(values.iter().enumerate(), |(index, value)| {
            value.map(|value| (value, index))
        })
    }

    #[test]
    fn test_ties_pick_the_first_candidate() {
        let values = vec![
            Some(dec!(1)),
            Some(dec!(3)),
            None,
            Some(dec!(3)),
            Some(dec!(2)),
        ];
        assert_eq!(parallel(&values), Some(1));
        assert_eq!(parallel(&values), serial(&values));
    }

    #[test]
    fn test_decimal_min_is_never_chosen() {
        let values = vec![Some(Decimal::MIN), None, Some(Decimal::MIN)];
        assert_eq!(parallel(&values), None);
        assert_eq!(serial(&values), None);
        let values = vec![Some(Decimal::MIN), Some(dec!(-5))];
        assert_eq!(parallel(&values), Some(1));
    }

    #[test]
    fn test_empty_and_unscorable() {
        assert_eq!(parallel(&[]), None);
        assert_eq!(parallel(&[None, None]), None);
    }

    #[test]
    fn test_matches_serial_across_rounds() {
        // Several rounds, many ties: scores cycle through 0..97, and the
        // maximum recurs in every round.
        let values: Vec<Option<Decimal>> = (0..3 * OPTIMISER_ROUND + 17)
            .map(|i| (i % 5 != 0).then(|| Decimal::from(i % 97)))
            .collect();
        assert_eq!(parallel(&values), serial(&values));
        // The maximum appears only in the last round.
        let mut late = values.clone();
        late.push(Some(dec!(1000)));
        assert_eq!(parallel(&late), Some(late.len() - 1));
        assert_eq!(parallel(&late), serial(&late));
    }
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
