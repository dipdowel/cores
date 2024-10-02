use std::collections::HashSet;
use std::str::FromStr;
/// Parses a string with a set of CPU core numbers.
/// The valid format of the input matches the one of `/sys/devices/system/cpu/online` on Linux
/// - Single value: `X`
/// - Consecutive values (aka ranges): `X-Y`
/// - Non-consecutive values: `X,Y,Z`
/// - Mixed values: `X-Y,A,B-Z`
///
/// **Valid input examples:** `0`, `0-5`, `0-3,5`, `0-1,3,5`, `0-2,4-5`
pub fn parse(input: &str) -> HashSet<usize> {
    let mut result = HashSet::new();

    // Remove all whitespace characters from the input.
    let input: String = input.chars().filter(|c| !c.is_whitespace()).collect();

    // Split the input by commas to handle separate components.
    for part in input.split(',') {
        if let Some(hyphen_position) = part.find('-') {
            // If there's a hyphen, it's a range.
            let start = usize::from_str(&part[..hyphen_position]).unwrap();
            let end = usize::from_str(&part[hyphen_position + 1..]).unwrap();
            // Insert all values from the range start to end (inclusive).
            for value in start..=end {
                result.insert(value);
            }
        } else {
            // Otherwise, it's a single value.
            result.insert(usize::from_str(part).unwrap());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    /// Helper function to create a HashSet from a list of usize values
    fn to_set(values: &[usize]) -> HashSet<usize> {
        values.iter().cloned().collect()
    }

    #[test]
    fn test_single_value() {
        let input = "0";
        let expected = to_set(&[0]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_range() {
        let input = "0-3";
        let expected = to_set(&[0, 1, 2, 3]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_non_consecutive_values() {
        let input = "0,2,4";
        let expected = to_set(&[0, 2, 4]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_mixed_values() {
        let input = "0-2,4,6-7";
        let expected = to_set(&[0, 1, 2, 4, 6, 7]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_complex_mixed_values() {
        let input = "0-1,3,5-6,8";
        let expected = to_set(&[0, 1, 3, 5, 6, 8]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_single_value_with_whitespace() {
        let input = " 0 ";
        let expected = to_set(&[0]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_range_with_whitespace() {
        let input = " 0 - 2 ";
        let expected = to_set(&[0, 1, 2]);
        let result = parse(input);
        assert_eq!(result, expected);
    }

    #[test]
    fn test_invalid_input() {
        let input = "a-b"; // Invalid input, will panic
        let result = std::panic::catch_unwind(|| parse(input));
        assert!(result.is_err());
    }
}
