use std::cmp::Ordering;
use std::fmt::Display;
use std::str::FromStr;

use anyhow::Context;

use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day13.input");

fn main() -> anyhow::Result<()> {
    // Part 1 - 2 hours 36 minutes 58 seconds
    let part_1_solution = calculate_sum_of_indices_of_pairs_in_right_order(INPUT)
        .context("Failed calculating part 1 solution.")?;
    println!("part_1_solution: {part_1_solution:?}");

    // Part 2 - 21 minutes 56 seconds
    let part_2_solution = calculate_decoder_key_for_distress_signal(INPUT)
        .context("Failed calculating part 2 solution.")?;
    println!("part_2_solution: {part_2_solution:?}");

    Ok(())
}

fn calculate_sum_of_indices_of_pairs_in_right_order(input: &str) -> anyhow::Result<u64> {
    Ok(PacketPairs::from_str(input)
        .context("Could not parse PacketPairs from string.")?
        .0
        .into_iter()
        .enumerate()
        .filter(|(_, packet_pair)| packet_pair.is_in_right_order())
        .map(|(index, _)| index + 1)
        .sum::<usize>() as u64)
}

fn calculate_decoder_key_for_distress_signal(input: &str) -> anyhow::Result<u64> {
    let divider_packets = vec![
        Packet(vec![PacketData::List(vec![PacketData::Integer(2)])]),
        Packet(vec![PacketData::List(vec![PacketData::Integer(6)])]),
    ];
    let mut packets = PacketPairs::from_str(input)
        .context("Failed parsing PacketPairs.")?
        .flatten();
    packets.extend_from_slice(divider_packets.as_slice());
    packets.sort_by(
        |left, right| match order_of_two_packet_data_slices(&left.0, &right.0) {
            PacketDataOrder::Correct => Ordering::Less,
            PacketDataOrder::Incorrect => Ordering::Greater,
            PacketDataOrder::Indecisive => Ordering::Equal,
        },
    );
    Ok(divider_packets
        .iter()
        .map(|divider_packet| {
            packets
                .iter()
                .find_position(|packet| *packet == divider_packet)
                .with_context(|| format!("Could not find position for {divider_packet:?}."))
                .map(|(index, _)| index)
        })
        .collect::<Result<Vec<usize>, _>>()?
        .into_iter()
        .map(|index| index + 1)
        .product::<usize>() as u64)
}

#[derive(Debug, Clone)]
struct PacketPairs(Vec<PacketPair>);

impl PacketPairs {
    fn flatten(self) -> Vec<Packet> {
        self.0
            .into_iter()
            .flat_map(|k| vec![k.left, k.right])
            .collect::<Vec<_>>()
    }
}

impl FromStr for PacketPairs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split("\n\n")
                .map(|packet_pair| {
                    PacketPair::from_str(packet_pair).with_context(|| {
                        format!("Failed parsing PacketPair from \"{packet_pair}\".")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

#[derive(Debug, Clone)]
struct PacketPair {
    left: Packet,
    right: Packet,
}

enum PacketDataOrder {
    Correct,
    Incorrect,
    Indecisive,
}

fn order_of_two_packet_data_slices(left: &[PacketData], right: &[PacketData]) -> PacketDataOrder {
    for (left_packet_data, right_packet_data) in left.iter().zip(right.iter()) {
        match order_of_two_packet_data(left_packet_data, right_packet_data) {
            PacketDataOrder::Correct => return PacketDataOrder::Correct,
            PacketDataOrder::Incorrect => return PacketDataOrder::Incorrect,
            PacketDataOrder::Indecisive => {}
        }
    }
    match left.len().cmp(&right.len()) {
        Ordering::Less => PacketDataOrder::Correct,
        Ordering::Greater => PacketDataOrder::Incorrect,
        Ordering::Equal => PacketDataOrder::Indecisive,
    }
}

fn order_of_two_packet_data(left: &PacketData, right: &PacketData) -> PacketDataOrder {
    match (left, right) {
        (PacketData::Integer(left_integer), PacketData::Integer(right_integer)) => {
            match left_integer.cmp(right_integer) {
                Ordering::Less => PacketDataOrder::Correct,
                Ordering::Greater => PacketDataOrder::Incorrect,
                Ordering::Equal => PacketDataOrder::Indecisive,
            }
        }
        (PacketData::Integer(left_integer), PacketData::List(right_list)) => {
            order_of_two_packet_data_slices(&[PacketData::Integer(*left_integer)], right_list)
        }
        (PacketData::List(left_list), PacketData::Integer(right_integer)) => {
            order_of_two_packet_data_slices(left_list, &[PacketData::Integer(*right_integer)])
        }
        (PacketData::List(left_list), PacketData::List(right_list)) => {
            order_of_two_packet_data_slices(left_list, right_list)
        }
    }
}

impl PacketPair {
    fn is_in_right_order(&self) -> bool {
        matches!(
            order_of_two_packet_data_slices(&self.left.0, &self.right.0),
            PacketDataOrder::Correct
        )
    }
}

impl FromStr for PacketPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [left, right]: [Packet; 2] = s
            .lines()
            .map(|line| {
                Packet::from_str(line)
                    .with_context(|| format!("Could not parse Packet from string \"{line}\"."))
            })
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|vec: Vec<_>| {
                anyhow::anyhow!(
                    "Could not transform Vec<_> into [_; 2], because it has {} elements.",
                    vec.len()
                )
            })?;
        Ok(Self { left, right })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Packet(Vec<PacketData>);

impl FromStr for Packet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let packet_data =
            PacketData::from_str(s).context("Could not parse PacketData from string.")?;
        match packet_data {
            PacketData::List(packet_data_vec) => Ok(Self(packet_data_vec)),
            PacketData::Integer(_) => unimplemented!(),
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, packet_data) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", packet_data)?;
        }
        write!(f, "]")
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum PacketData {
    List(Vec<PacketData>),
    Integer(u8),
}

impl FromStr for PacketData {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('[') {
            let mut opened_brackets = 0;
            let mut optional_main_bracket_open_index: Option<usize> = None;
            let mut optional_main_bracket_close_index: Option<usize> = None;
            let mut indices_of_commas: Vec<usize> = Vec::new();
            for (index, character) in s.chars().enumerate() {
                match character {
                    '[' => {
                        if optional_main_bracket_open_index.is_none() {
                            optional_main_bracket_open_index = Some(index);
                        }
                        opened_brackets += 1;
                    }
                    ']' => {
                        opened_brackets -= 1;
                        if opened_brackets == 0 && optional_main_bracket_close_index.is_none() {
                            optional_main_bracket_close_index = Some(index);
                        }
                    }
                    ',' => {
                        if opened_brackets == 1 {
                            indices_of_commas.push(index);
                        }
                    }
                    _ => {}
                }
            }
            if opened_brackets != 0 {
                panic!("Uneven bracket number {opened_brackets}");
            }
            let main_bracket_open_index = optional_main_bracket_open_index
                .context("Could not find main bracket open index.")?;
            let main_bracket_close_index = optional_main_bracket_close_index
                .context("Could not find main bracket close index.")?;
            let text_between_main_brackets = s
                .chars()
                .skip(main_bracket_open_index + 1)
                .take(main_bracket_close_index - main_bracket_open_index - 1)
                .collect::<String>();

            if text_between_main_brackets.is_empty() {
                Ok(Self::List(Vec::new()))
            } else {
                let mut parts = Vec::new();
                let mut last_offset = 0;
                for comma in indices_of_commas {
                    parts.push(
                        text_between_main_brackets
                            .chars()
                            .skip(last_offset)
                            .take(comma - last_offset - 1)
                            .collect::<String>(),
                    );
                    last_offset = comma;
                }
                parts.push(
                    text_between_main_brackets
                        .chars()
                        .skip(last_offset)
                        .collect::<String>(),
                );

                Ok(Self::List(
                    parts
                        .into_iter()
                        .map(|part| {
                            Self::from_str(&part).with_context(|| {
                                format!(
                                    "Failed parsing PacketData::List part from part \"{part}\"."
                                )
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                ))
            }
        } else {
            Ok(Self::Integer(s.parse().with_context(|| {
                format!("Failed parsing PacketData::Integer from \"{s}\".")
            })?))
        }
    }
}

impl Display for PacketData {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketData::List(packet_data_vec) => {
                write!(f, "[")?;
                for (index, packet_data) in packet_data_vec.iter().enumerate() {
                    if index > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", packet_data)?;
                }
                write!(f, "]")
            }
            PacketData::Integer(integer) => write!(f, "{}", integer),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "[1,1,3,1,1]
[1,1,5,1,1]

[[1],[2,3,4]]
[[1],4]

[9]
[[8,7,6]]

[[4,4],4,4]
[[4,4],4,4,4]

[7,7,7,7]
[7,7,7]

[]
[3]

[[[]]]
[[]]

[1,[2,[3,[4,[5,6,7]]]],8,9]
[1,[2,[3,[4,[5,6,0]]]],8,9]";

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Act
        let result = calculate_sum_of_indices_of_pairs_in_right_order(TEST_INPUT)?;

        // Assert
        assert_eq!(result, 13);

        Ok(())
    }

    #[test]
    fn test_part_2_default() -> anyhow::Result<()> {
        // Act
        let result = calculate_decoder_key_for_distress_signal(TEST_INPUT)?;

        // Assert
        assert_eq!(result, 140);

        Ok(())
    }
}
