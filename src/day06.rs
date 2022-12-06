use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day06.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 14 minutes 10 seconds
    let part_1_solution = calculate_last_index_of_first_non_repeating_char_sequence(INPUT, 4)
        .ok_or_else(|| {
            anyhow::anyhow!("Did not find a non repeating char sequence with length of 4")
        })?;
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 2 minutes 39 seconds
    let part_2_solution = calculate_last_index_of_first_non_repeating_char_sequence(INPUT, 14)
        .ok_or_else(|| {
            anyhow::anyhow!("Did not find a non repeating char sequence with length of 14")
        })?;
    println!("part_2_solution: {part_2_solution}");

    Ok(())
}

fn calculate_last_index_of_first_non_repeating_char_sequence(
    input: &str,
    sequence_length: usize,
) -> Option<usize> {
    let mut ring_buffer = Vec::with_capacity(sequence_length);
    for (index, char) in input.chars().enumerate() {
        if ring_buffer.len() == sequence_length {
            // @lint: Don't know how to do that and whether it really benefits.
            #[allow(clippy::pattern_type_mismatch)]
            let all_unequal = ring_buffer
                .iter()
                .enumerate()
                .cartesian_product(ring_buffer.iter().enumerate())
                .filter(|((index_a, _), (index_b, _))| index_a != index_b && index_a < index_b)
                .all(|((_, character_a), (_, character_b))| character_a != character_b);
            if all_unequal {
                return Some(index);
            }
            ring_buffer.remove(0);
        }
        ring_buffer.push(char);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_part_1_default() {
        // Arrange
        let test_inputs: [(&str, usize); 5] = [
            ("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 7),
            ("bvwbjplbgvbhsrlpgdmjqwftvncz", 5),
            ("nppdvjthqldpwncqszvftbrmjlhg", 6),
            ("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 10),
            ("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 11),
        ];

        for (test_input, test_target_output) in test_inputs {
            // Act
            let output = calculate_last_index_of_first_non_repeating_char_sequence(test_input, 4);

            // Assert
            assert_eq!(output, Some(test_target_output), "during {}", test_input);
        }
    }

    #[test]
    fn test_part_2_default() {
        // Arrange
        let test_inputs: [(&str, usize); 5] = [
            ("mjqjpqmgbljsphdztnvjfqwrcgsmlb", 19),
            ("bvwbjplbgvbhsrlpgdmjqwftvncz", 23),
            ("nppdvjthqldpwncqszvftbrmjlhg", 23),
            ("nznrnfrfntjfmvfwmzdfjlvtqnbhcprsg", 29),
            ("zcfzfwzzqfrljwzlrfnpqdbhtmscgvjw", 26),
        ];

        for (test_input, test_target_output) in test_inputs {
            // Act
            let output = calculate_last_index_of_first_non_repeating_char_sequence(test_input, 14);

            // Assert
            assert_eq!(output, Some(test_target_output), "during {}", test_input);
        }
    }
}
