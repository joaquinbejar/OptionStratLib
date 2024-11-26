/******************************************************************************
   Author: Joaquín Béjar García
   Email: jb@taunais.com
   Date: 27/9/24
******************************************************************************/
use crate::constants::TOLERANCE;
use itertools::Itertools;
use std::collections::BTreeSet;

#[allow(dead_code)]
pub(crate) fn approx_equal(a: f64, b: f64) -> bool {
    (a - b).abs() < TOLERANCE
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

    let random_index = rand::random::<usize>() % set.len();
    set.iter().nth(random_index)
}

/// Processes combinations from a vector of elements using a specified function.
///
/// This function takes a slice of elements `positions` and a processing function `process_combination`.
/// It generates all combinations with replacement of the length equal to the length of `positions`,
/// applies the processing function to each combination, flattens the result, and collects it into a new vector.
///
/// # Type Parameters:
/// - `T`: The type of elements in the `positions` slice. The type must implement the `Clone` trait.
/// - `F`: The type of the processing function. It must be a function that takes a slice of references to elements and returns a vector of elements.
///
/// # Parameters:
/// - `positions`: A slice of elements. The function generates combinations from this slice.
/// - `process_combination`: A function that processes each combination. The function takes a slice of references to elements and returns a vector of elements.
///
/// # Returns:
/// - `Ok(Vec<T>)`: A vector containing the processed elements if the input slice is not empty.
/// - `Err(String)`: An error message if the input slice is empty.
///
/// # Errors:
/// This function will return an error if the `positions` slice is empty.
///
pub fn process_n_times_iter<T, F>(positions: &[T], process_combination: F) -> Result<Vec<T>, String>
where
    F: Fn(&[&T]) -> Vec<T>,
    T: Clone,
{
    if positions.is_empty() {
        return Err("Vector empty".to_string());
    }

    Ok(positions
        .iter()
        .combinations_with_replacement(positions.len())
        .flat_map(|combination| process_combination(&combination))
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
    use crate::chains::chain::OptionData;
    use crate::model::types::PositiveF64;
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
        let result = process_n_times_iter(&empty_vec, |_| vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vector empty".to_string());
    }

    #[test]
    fn test_single_element() {
        let vec = vec![1];
        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().cloned().cloned().collect()
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1]);
    }

    #[test]
    fn test_multiple_elements() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().cloned().cloned().collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        // Para 2 elementos, esperamos todas las combinaciones posibles con repetición
        assert_eq!(result, vec![1, 1, 1, 2, 2, 2]);
    }

    #[test]
    fn test_with_strings() {
        let vec = vec!["a".to_string(), "b".to_string()];
        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().map(|&x| x.clone()).collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"b".to_string()));
    }

    #[test]
    fn test_custom_processing() {
        let vec = vec![1, 2, 3];
        let result = process_n_times_iter(&vec, |combination| {
            // Multiplicar cada elemento por 2
            combination.iter().map(|&&x| x * 2).collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        // Verificar que todos los elementos son pares
        assert!(result.iter().all(|&x| x % 2 == 0));
    }

    #[test]
    fn test_with_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct TestStruct {
            value: i32,
        }

        let vec = vec![TestStruct { value: 1 }, TestStruct { value: 2 }];

        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().map(|&x| x.clone()).collect()
        });

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains(&TestStruct { value: 1 }));
        assert!(result.contains(&TestStruct { value: 2 }));
    }

    #[test]
    fn test_filter_and_process() {
        let vec = vec![1, 2, 3, 4];
        let result = process_n_times_iter(&vec, |combination| {
            combination
                .iter()
                .filter(|&&x| x % 2 == 0)
                .map(|&&x| x)
                .collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.iter().all(|&x| x % 2 == 0));
    }

    #[test]
    fn test_empty_result_from_callback() {
        let vec = vec![1, 2, 3];
        let result = process_n_times_iter(&vec, |_| Vec::new());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}

#[cfg(test)]
mod tests_process_n_times_iter_bis {
    use super::*;

    #[test]
    fn test_empty_vector() {
        let empty_vec: Vec<i32> = vec![];
        let result = process_n_times_iter(&empty_vec, |_| vec![]);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Vector empty".to_string());
    }

    #[test]
    fn test_single_element() {
        let vec = vec![1];
        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().cloned().cloned().collect()
        });
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), vec![1]);
    }

    #[test]
    fn test_multiple_elements() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().cloned().cloned().collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        // Para 2 elementos, esperamos todas las combinaciones posibles con repetición
        assert_eq!(result, vec![1, 1, 1, 2, 2, 2]);
    }

    #[test]
    fn test_with_strings() {
        let vec = vec!["a".to_string(), "b".to_string()];
        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().map(|&x| x.clone()).collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains(&"a".to_string()));
        assert!(result.contains(&"b".to_string()));
    }

    #[test]
    fn test_custom_processing() {
        let vec = vec![1, 2, 3];
        let result = process_n_times_iter(&vec, |combination| {
            // Multiplicar cada elemento por 2
            combination.iter().map(|&&x| x * 2).collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        // Verificar que todos los elementos son pares
        assert!(result.iter().all(|&x| x % 2 == 0));
    }

    #[test]
    fn test_with_custom_struct() {
        #[derive(Clone, Debug, PartialEq)]
        struct TestStruct {
            value: i32,
        }

        let vec = vec![TestStruct { value: 1 }, TestStruct { value: 2 }];

        let result = process_n_times_iter(&vec, |combination| {
            combination.iter().map(|&x| x.clone()).collect()
        });

        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains(&TestStruct { value: 1 }));
        assert!(result.contains(&TestStruct { value: 2 }));
    }

    #[test]
    fn test_modified_elements() {
        let vec = vec![1, 2];
        let result = process_n_times_iter(&vec, |combination| {
            // Multiplicar cada elemento por 10
            combination.iter().map(|&&x| x * 10).collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.contains(&10));
        assert!(result.contains(&20));
    }

    #[test]
    fn test_filter_and_process() {
        let vec = vec![1, 2, 3, 4];
        let result = process_n_times_iter(&vec, |combination| {
            // Solo procesar números pares
            combination
                .iter()
                .filter(|&&x| x % 2 == 0)
                .map(|&&x| x)
                .collect()
        });
        assert!(result.is_ok());
        let result = result.unwrap();
        assert!(result.iter().all(|&x| x % 2 == 0));
    }

    #[test]
    fn test_empty_result_from_callback() {
        let vec = vec![1, 2, 3];
        let result = process_n_times_iter(&vec, |_| Vec::new());
        assert!(result.is_ok());
        assert!(result.unwrap().is_empty());
    }
}
