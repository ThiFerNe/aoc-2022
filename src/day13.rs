use itertools::Itertools;
use std::fmt::{Display, Formatter};
use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day13.input");

fn main() {
    // Part 1 - 2 hours 36 minutes 58 seconds
    let part_1_solution = calculate_sum_of_indices_of_pairs_in_right_order(INPUT);
    println!("part_1_solution: {part_1_solution:?}");

    // Part 2 - 21 minutes 56 seconds
    let part_2_solution = calculate_decoder_key_for_distress_signal(INPUT);
    println!("part_2_solution: {part_2_solution:?}");
}

fn calculate_sum_of_indices_of_pairs_in_right_order(input: &str) -> u64 {
    let packet_pairs = PacketPairs::from_str(input).unwrap();
    packet_pairs
        .0
        .into_iter()
        .enumerate()
        .filter(|(_, a)| a.is_in_right_order())
        .map(|(index, _)| index + 1)
        .sum::<usize>() as u64
}

fn calculate_decoder_key_for_distress_signal(input: &str) -> u64 {
    let packet_pairs = PacketPairs::from_str(input).unwrap();
    let mut m = packet_pairs
        .0
        .into_iter()
        .flat_map(|k| vec![k.left, k.right])
        .collect::<Vec<_>>();
    let divider_packets = vec![
        Packet(vec![PacketData::List(vec![PacketData::Integer(2)])]),
        Packet(vec![PacketData::List(vec![PacketData::Integer(6)])]),
    ];
    m.extend_from_slice(divider_packets.as_slice());
    m.sort_by(
        |a, b| match list_of_packet_data_in_right_order(&a.0, &b.0) {
            Order::Correct => std::cmp::Ordering::Less,
            Order::Incorrect => std::cmp::Ordering::Greater,
            Order::Indecisive => std::cmp::Ordering::Equal,
        },
    );
    divider_packets
        .iter()
        .map(|divider_packet| m.iter().find_position(|a| *a == divider_packet).unwrap().0)
        .map(|k| k + 1)
        .product::<usize>() as u64
}

#[derive(Debug, Clone)]
struct PacketPairs(Vec<PacketPair>);

impl FromStr for PacketPairs {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.split("\n\n")
                .map(|packet_pair| PacketPair::from_str(packet_pair).unwrap())
                .collect::<Vec<_>>(),
        ))
    }
}

#[derive(Debug, Clone)]
struct PacketPair {
    left: Packet,
    right: Packet,
}

enum Order {
    Correct,
    Incorrect,
    Indecisive,
}

fn list_of_packet_data_in_right_order(left: &[PacketData], right: &[PacketData]) -> Order {
    for index in 0..(left.len().min(right.len())) {
        match packet_data_in_right_order(&left[index], &right[index]) {
            Order::Correct => return Order::Correct,
            Order::Incorrect => return Order::Incorrect,
            Order::Indecisive => {}
        }
    }
    if left.len() < right.len() {
        Order::Correct
    } else if left.len() > right.len() {
        Order::Incorrect
    } else {
        Order::Indecisive
    }
}

fn packet_data_in_right_order(left: &PacketData, right: &PacketData) -> Order {
    match (left, right) {
        (PacketData::Integer(a), PacketData::Integer(b)) => {
            if *a < *b {
                Order::Correct
            } else if *a > *b {
                Order::Incorrect
            } else {
                Order::Indecisive
            }
        }
        (PacketData::Integer(a), PacketData::List(b)) => {
            list_of_packet_data_in_right_order(&[PacketData::Integer(*a)], &b)
        }
        (PacketData::List(a), PacketData::Integer(b)) => {
            list_of_packet_data_in_right_order(&a, &[PacketData::Integer(*b)])
        }
        (PacketData::List(a), PacketData::List(b)) => list_of_packet_data_in_right_order(&a, &b),
    }
}

impl PacketPair {
    fn is_in_right_order(&self) -> bool {
        matches!(
            list_of_packet_data_in_right_order(&self.left.0, &self.right.0),
            Order::Correct
        )
    }
}

impl FromStr for PacketPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [line_a, line_b]: [Packet; 2] = s
            .lines()
            .map(|line| Packet::from_str(line).unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Ok(Self {
            left: line_a,
            right: line_b,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Packet(Vec<PacketData>);

impl FromStr for Packet {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pd = PacketData::from_str(s).unwrap();
        match pd {
            PacketData::List(list) => Ok(Self(list)),
            PacketData::Integer(_) => todo!(),
        }
    }
}

impl Display for Packet {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for (index, ll) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, ",")?;
            }
            write!(f, "{}", ll)?;
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
            let mut brackets_open = 0;
            let mut bracket_open: Option<usize> = None;
            let mut bracket_close: Option<usize> = None;
            let mut commas: Vec<usize> = Vec::new();
            for (index, character) in s.chars().enumerate() {
                match character {
                    '[' => {
                        if bracket_open.is_none() {
                            bracket_open = Some(index);
                        }
                        brackets_open += 1;
                    }
                    ']' => {
                        brackets_open -= 1;
                        if brackets_open == 0 {
                            if bracket_close.is_none() {
                                bracket_close = Some(index);
                            }
                        }
                    }
                    ',' => {
                        if brackets_open == 1 {
                            commas.push(index);
                        }
                    }
                    _ => {}
                }
            }
            if brackets_open != 0 {
                panic!("Uneven bracket number {brackets_open}");
            }
            let bracket_open = bracket_open.unwrap();
            let bracket_close = bracket_close.unwrap();
            let inner = s
                .chars()
                .skip(bracket_open + 1)
                .take(bracket_close - bracket_open - 1)
                .collect::<String>();

            if inner.is_empty() {
                Ok(Self::List(Vec::new()))
            } else {
                let mut parts = Vec::new();
                let mut last_offset = 0;
                for comma in commas {
                    parts.push(
                        inner
                            .chars()
                            .skip(last_offset)
                            .take(comma - last_offset - 1)
                            .collect::<String>(),
                    );
                    last_offset = comma;
                }
                parts.push(inner.chars().skip(last_offset).collect::<String>());

                Ok(Self::List(
                    parts
                        .into_iter()
                        .map(|part| Self::from_str(&part).unwrap())
                        .collect::<Vec<_>>(),
                ))
            }
        } else {
            Ok(Self::Integer(s.parse().unwrap()))
        }
    }
}

impl Display for PacketData {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PacketData::List(l) => {
                write!(f, "[")?;
                for (index, ll) in l.iter().enumerate() {
                    if index > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{}", ll)?;
                }
                write!(f, "]")
            }
            PacketData::Integer(i) => write!(f, "{}", i),
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
    fn test_part_1_default() {
        // Act
        let result = calculate_sum_of_indices_of_pairs_in_right_order(TEST_INPUT);

        // Assert
        assert_eq!(result, 13);
    }

    #[test]
    fn test_part_2_default() {
        // Act
        let result = calculate_decoder_key_for_distress_signal(TEST_INPUT);

        // Assert
        assert_eq!(result, 140);
    }
}
