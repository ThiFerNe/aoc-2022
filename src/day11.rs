use std::str::FromStr;
use std::time::Instant;

const INPUT: &str = include_str!("../inputs/day11.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 16 minutes 53 seconds
    let mut monkey_keep_away = MonkeyKeepAway::from_str(INPUT)?;
    monkey_keep_away.run_for_rounds(20, WorryType::WithRelief);
    let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business();
    println!("level_of_monkey_business: {level_of_monkey_business}");

    // PART 2 - 1 hour 56 minutes 4 seconds + 2 hours 24 minutes 26 seconds + 27 minutes 29 seconds = 4 hours 47 minutes 59 seconds
    // third attempt with the help of https://github.com/schubart/AdventOfCode_2022_Rust/blob/c05c1f267566df54a94cf5364f6cbc5258756810/day11/src/lib.rs
    let mut monkey_keep_away = MonkeyKeepAway::from_str(INPUT)?;
    monkey_keep_away.run_for_rounds(10_000, WorryType::NoRelief);
    let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business();
    println!("level_of_monkey_business: {level_of_monkey_business}");

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MonkeyKeepAway {
    monkeys: Vec<Monkey>,
}

impl MonkeyKeepAway {
    fn run(&mut self, worry_type: WorryType) {
        let least_common_test_divisor: u64 = self
            .monkeys
            .iter()
            .map(|monkey| monkey.test.condition_divisible_by)
            .product();
        for current_monkey_index in 0..self.monkeys.len() {
            let current_monkey = &mut self.monkeys[current_monkey_index];
            let items: Vec<_> = current_monkey
                .items
                .drain(..)
                .map(|item| Item {
                    worry_level: current_monkey.operation.apply(item.worry_level),
                })
                .map(|item| {
                    if matches!(worry_type, WorryType::WithRelief) {
                        Item {
                            worry_level: WorryLevel(item.worry_level.0 / 3),
                        }
                    } else {
                        Item {
                            worry_level: WorryLevel(item.worry_level.0 % least_common_test_divisor),
                        }
                    }
                })
                .collect();
            current_monkey.count_of_item_inspections += u128::try_from(items.len()).unwrap();
            let test = current_monkey.test;
            for item in items {
                let target = if item.worry_level.0 % test.condition_divisible_by == 0 {
                    test.target_if_true.0
                } else {
                    test.target_if_false.0
                };
                self.monkeys[target].items.push(item);
            }
        }
    }

    fn run_for_rounds(&mut self, count_of_rounds: u128, worry_type: WorryType) {
        for round in 0..count_of_rounds {
            let start = Instant::now();
            println!(
                "<{:?}> {round}/{count_of_rounds}={:.2}%",
                start,
                100. * round as f64 / count_of_rounds as f64
            );
            self.run(worry_type);
            let end = Instant::now();
            println!("<{:?}={:?}>", end, end - start);
        }
    }

    fn calculate_level_of_monkey_business(&self) -> u128 {
        let mut monkeys_with_their_inspection_counts = self
            .monkeys
            .iter()
            .map(|monkey| (monkey.index, monkey.count_of_item_inspections))
            .collect::<Vec<_>>();
        monkeys_with_their_inspection_counts.sort_by(|(_, a), (_, b)| b.cmp(a));
        println!("monkeys_with_their_inspection_counts: {monkeys_with_their_inspection_counts:?}");
        monkeys_with_their_inspection_counts[0].1 * monkeys_with_their_inspection_counts[1].1
    }
}

impl FromStr for MonkeyKeepAway {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            monkeys: s
                .split("\n\n")
                .enumerate()
                .map(|(part_index, monkey_string)| {
                    let monkey_lines: [&str; 6] = monkey_string
                        .lines()
                        .collect::<Vec<_>>()
                        .try_into()
                        .map_err(|vec: Vec<_>| {
                            anyhow::anyhow!(
                                "Part #{part_index} does not have exactly 6 but has {} lines.",
                                vec.len()
                            )
                        })?;

                    let monkey_index = monkey_lines[0]
                        .strip_prefix("Monkey ")
                        .unwrap()
                        .strip_suffix(":")
                        .unwrap()
                        .parse()
                        .unwrap();

                    let items = monkey_lines[1]
                        .strip_prefix("  Starting items: ")
                        .unwrap()
                        .split(", ")
                        .map(|starting_item_worry_level_str| {
                            starting_item_worry_level_str
                                .parse()
                                .map(|worry_level| Item {
                                    worry_level: WorryLevel(worry_level),
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let operation_str = monkey_lines[2].strip_prefix("  Operation: ").unwrap();
                    let operation = Operation::from_str(operation_str).unwrap();

                    let test_predicament_str = monkey_lines[3].strip_prefix("  Test: ").unwrap();
                    let condition_divisible_by = test_predicament_str
                        .strip_prefix("divisible by ")
                        .unwrap()
                        .parse()
                        .unwrap();

                    let test_if_true = monkey_lines[4]
                        .strip_prefix("    If true: throw to monkey ")
                        .unwrap()
                        .parse()
                        .unwrap();
                    let test_if_false = monkey_lines[5]
                        .strip_prefix("    If false: throw to monkey ")
                        .unwrap()
                        .parse()
                        .unwrap();

                    Ok::<_, anyhow::Error>(Monkey {
                        index: MonkeyIndex(monkey_index),
                        items,
                        operation,
                        test: Test {
                            condition_divisible_by,
                            target_if_true: MonkeyIndex(test_if_true),
                            target_if_false: MonkeyIndex(test_if_false),
                        },
                        count_of_item_inspections: 0,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum WorryType {
    NoRelief,
    WithRelief,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Monkey {
    index: MonkeyIndex,
    items: Vec<Item>,
    operation: Operation,
    test: Test,
    count_of_item_inspections: u128,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct MonkeyIndex(usize);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Item {
    worry_level: WorryLevel,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct WorryLevel(u64);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Operation {
    Product { factor: u64 },
    ProductByFactorOld,
    Sum { summand: u64 },
}

impl Operation {
    fn apply(&self, worry_level: WorryLevel) -> WorryLevel {
        match self {
            Operation::Product { factor } => WorryLevel(worry_level.0 * *factor),
            Operation::ProductByFactorOld => WorryLevel(worry_level.0 * worry_level.0),
            Operation::Sum { summand } => WorryLevel(worry_level.0 + *summand),
        }
    }
}

impl FromStr for Operation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let suffix = s.strip_prefix("new = old ").unwrap();
        let operand = suffix.chars().next().unwrap();
        let factor = suffix.chars().skip(2).collect::<String>();
        match (operand, factor.as_str()) {
            ('*', "old") => Ok(Self::ProductByFactorOld),
            ('*', _) => Ok(Self::Product {
                factor: factor.parse().unwrap(),
            }),
            ('+', _) => Ok(Self::Sum {
                summand: factor.parse().unwrap(),
            }),
            _ => panic!(),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Test {
    condition_divisible_by: u64,
    target_if_true: MonkeyIndex,
    target_if_false: MonkeyIndex,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "Monkey 0:
  Starting items: 79, 98
  Operation: new = old * 19
  Test: divisible by 23
    If true: throw to monkey 2
    If false: throw to monkey 3

Monkey 1:
  Starting items: 54, 65, 75, 74
  Operation: new = old + 6
  Test: divisible by 19
    If true: throw to monkey 2
    If false: throw to monkey 0

Monkey 2:
  Starting items: 79, 60, 97
  Operation: new = old * old
  Test: divisible by 13
    If true: throw to monkey 1
    If false: throw to monkey 3

Monkey 3:
  Starting items: 74
  Operation: new = old + 3
  Test: divisible by 17
    If true: throw to monkey 0
    If false: throw to monkey 1";

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Arrange
        let mut monkey_keep_away = MonkeyKeepAway::from_str(TEST_INPUT)?;

        // Act
        monkey_keep_away.run_for_rounds(20, WorryType::WithRelief);
        let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business();

        // Assert
        assert_eq!(level_of_monkey_business, 10605);

        Ok(())
    }

    #[test]
    fn test_monkey_keep_away_from_str() -> anyhow::Result<()> {
        // Act
        let monkey_keep_away = MonkeyKeepAway::from_str(TEST_INPUT)?;

        // Assert
        #[rustfmt::skip]
        assert_eq!(
            monkey_keep_away,
            MonkeyKeepAway {
                monkeys: vec![
                    Monkey {
                        index: MonkeyIndex(0),
                        items: vec![
                            Item { worry_level: WorryLevel(79) },
                            Item { worry_level: WorryLevel(98) },
                        ],
                        operation: Operation::Product { factor: 19 },
                        test: Test {
                            condition_divisible_by: 23,
                            target_if_true: MonkeyIndex(2),
                            target_if_false: MonkeyIndex(3)
                        },
                        count_of_item_inspections: 0,
                    },
                    Monkey {
                        index: MonkeyIndex(1),
                        items: vec![
                            Item { worry_level: WorryLevel(54) },
                            Item { worry_level: WorryLevel(65) },
                            Item { worry_level: WorryLevel(75) },
                            Item { worry_level: WorryLevel(74) },
                        ],
                        operation: Operation::Sum { summand: 6 },
                        test: Test {
                            condition_divisible_by: 19,
                            target_if_true: MonkeyIndex(2),
                            target_if_false: MonkeyIndex(0)
                        },
                        count_of_item_inspections: 0,
                    },
                    Monkey {
                        index: MonkeyIndex(2),
                        items: vec![
                            Item { worry_level: WorryLevel(79) },
                            Item { worry_level: WorryLevel(60) },
                            Item { worry_level: WorryLevel(97) },
                        ],
                        operation: Operation::ProductByFactorOld,
                        test: Test {
                            condition_divisible_by: 13,
                            target_if_true: MonkeyIndex(1),
                            target_if_false: MonkeyIndex(3)
                        },
                        count_of_item_inspections: 0,
                    },
                    Monkey {
                        index: MonkeyIndex(3),
                        items: vec![
                            Item { worry_level: WorryLevel(74) },
                        ],
                        operation: Operation::Sum { summand: 3 },
                        test: Test {
                            condition_divisible_by: 17,
                            target_if_true: MonkeyIndex(0),
                            target_if_false: MonkeyIndex(1)
                        },
                        count_of_item_inspections: 0,
                    },
                ]
            }
        );

        Ok(())
    }

    #[test]
    fn test_monkey_keep_away_round() -> anyhow::Result<()> {
        // Arrange
        let mut monkey_keep_away = MonkeyKeepAway::from_str(TEST_INPUT)?;

        // Act
        monkey_keep_away.run(WorryType::WithRelief);

        // Assert
        let items_per_monkey = monkey_keep_away
            .monkeys
            .into_iter()
            .map(|monkey| monkey.items)
            .collect::<Vec<Vec<_>>>();
        assert_eq!(
            items_per_monkey,
            vec![
                vec![
                    Item {
                        worry_level: WorryLevel(20)
                    },
                    Item {
                        worry_level: WorryLevel(23)
                    },
                    Item {
                        worry_level: WorryLevel(27)
                    },
                    Item {
                        worry_level: WorryLevel(26)
                    }
                ],
                vec![
                    Item {
                        worry_level: WorryLevel(2080)
                    },
                    Item {
                        worry_level: WorryLevel(25)
                    },
                    Item {
                        worry_level: WorryLevel(167)
                    },
                    Item {
                        worry_level: WorryLevel(207)
                    },
                    Item {
                        worry_level: WorryLevel(401)
                    },
                    Item {
                        worry_level: WorryLevel(1046)
                    }
                ],
                vec![],
                vec![],
            ]
        );

        Ok(())
    }

    #[test]
    fn test_part_3_default() -> anyhow::Result<()> {
        // Arrange
        let mut monkey_keep_away = MonkeyKeepAway::from_str(TEST_INPUT)?;

        // Act
        monkey_keep_away.run_for_rounds(10_000, WorryType::NoRelief);
        let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business();

        // Assert
        assert_eq!(level_of_monkey_business, 2_713_310_158);

        Ok(())
    }
}
