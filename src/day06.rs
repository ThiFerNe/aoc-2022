const INPUT: &str = include_str!("../inputs/day06.input");

fn main() {
    // PART 1 - 14 minutes 10 seconds
    let part_1_solution = calculate_count_of_characters_to_process_until_start_of_packet(INPUT);
    println!("part_1_solution: {part_1_solution:?}");
}

fn calculate_count_of_characters_to_process_until_start_of_packet(input: &str) -> Option<usize> {
    let mut ring_buffer = Vec::with_capacity(4);
    for (index, char) in input.chars().enumerate() {
        if ring_buffer.len() == 4 {
            let no_one_equal = (0..ring_buffer.len()).all(|index_a| {
                ((index_a + 1)..ring_buffer.len())
                    .all(|index_b| ring_buffer[index_a] != ring_buffer[index_b])
            });
            if no_one_equal {
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
            let output = calculate_count_of_characters_to_process_until_start_of_packet(test_input);

            // Assert
            assert_eq!(output, Some(test_target_output), "during {}", test_input);
        }
    }
}
