use anyhow::Context;
use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day11.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 16 minutes 53 seconds
    let mut monkey_keep_away_part_1 = MonkeyKeepAway::from_str(INPUT)?;
    monkey_keep_away_part_1
        .run_for_rounds(20, WorryType::WithRelief)
        .context("Failed running for 20 rounds in part 1.")?;
    let part_1_solution = monkey_keep_away_part_1
        .calculate_level_of_monkey_business()
        .context("Failed calculating monkey business in part 2.")?;
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 1 hour 56 minutes 4 seconds + 2 hours 24 minutes 26 seconds + 27 minutes 29 seconds = 4 hours 47 minutes 59 seconds
    // third attempt with the help of https://github.com/schubart/AdventOfCode_2022_Rust/blob/c05c1f267566df54a94cf5364f6cbc5258756810/day11/src/lib.rs
    let mut monkey_keep_away_part_2 = MonkeyKeepAway::from_str(INPUT)?;
    monkey_keep_away_part_2
        .run_for_rounds(10_000, WorryType::NoRelief)
        .context("Failed running for 10_000 rounds in part 2.")?;
    let part_2_solution = monkey_keep_away_part_2
        .calculate_level_of_monkey_business()
        .context("Failed calculating monkey business in part 2.")?;
    println!("level_of_monkey_business: {part_2_solution}");

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MonkeyKeepAway {
    monkeys: Vec<Monkey>,
}

impl MonkeyKeepAway {
    fn run(&mut self, worry_type: WorryType) -> anyhow::Result<()> {
        let least_common_test_divisor: u64 = self
            .monkeys
            .iter()
            .map(|monkey| monkey.test.condition_divisible_by)
            .product();
        let count_of_monkeys = self.monkeys.len();
        for current_monkey_index in 0..count_of_monkeys {
            let current_monkey = self.monkeys.get_mut(current_monkey_index).ok_or_else(|| {
                anyhow::anyhow!(
                    "Did not find monkey at index {current_monkey_index} in the {} monkeys.",
                    count_of_monkeys
                )
            })?;
            let items: Vec<_> = current_monkey
                .items
                .drain(..)
                .map(|item| Ok::<_, anyhow::Error>(Item {
                    worry_level: current_monkey.operation.apply(item.worry_level)
                        .with_context(|| format!("Failed applying operation of monkey #{current_monkey_index} to {item:?}."))?,
                }))
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| format!("Could not drain current monkey's ({current_monkey_index}) items (A)."))?
                .into_iter()
                .map(|item| {
                    if matches!(worry_type, WorryType::WithRelief) {
                        Ok::<_, anyhow::Error>(Item {
                            worry_level: WorryLevel(
                                item.worry_level.0.checked_div(3)
                                    .ok_or_else(|| anyhow::anyhow!("Could not reduce worry level {} by dividing through 3.", item.worry_level.0))?
                            ),
                        })
                    } else {
                        Ok::<_, anyhow::Error>(Item {
                            worry_level: WorryLevel(
                                item.worry_level.0.checked_rem(least_common_test_divisor)
                                    .ok_or_else(|| anyhow::anyhow!("Could not calculate remainder of {} % {}(=lcd).", item.worry_level.0, least_common_test_divisor))?
                            ),
                        })
                    }
                })
                .collect::<Result<_, _>>()
                .with_context(|| format!("Could not drain current monkey's ({current_monkey_index}) items (B)."))?;

            current_monkey.count_of_item_inspections = current_monkey.count_of_item_inspections
                .checked_add(
                    u128::try_from(items.len())
                        .with_context(|| format!("There are too many items ({}) thrown by monkey #{current_monkey_index} which do not fit into u128.", items.len()))?
                )
                .ok_or_else(|| anyhow::anyhow!("Monkey #{current_monkey_index} has already thrown too many items, so that the count does not fit into u128."))?;

            let test = current_monkey.test;
            for item in items {
                let remainder = item.worry_level.0.checked_rem(test.condition_divisible_by)
                    .ok_or_else(|| anyhow::anyhow!("Monkey #{current_monkey_index} has an divisible check by {}, which fails.", test.condition_divisible_by))?;
                let target = if remainder == 0 {
                    test.target_if_true.0
                } else {
                    test.target_if_false.0
                };
                self.monkeys
                    .get_mut(target)
                    .ok_or_else(|| anyhow::anyhow!("Could not find monkey #{target} to which monkey #{current_monkey_index} throws something."))?
                    .items
                    .push(item);
            }
        }

        Ok(())
    }

    fn run_for_rounds(
        &mut self,
        count_of_rounds: u128,
        worry_type: WorryType,
    ) -> anyhow::Result<()> {
        for round in 0..count_of_rounds {
            self.run(worry_type)
                .with_context(|| format!("Run #{round} failed."))?;
        }
        Ok(())
    }

    fn calculate_level_of_monkey_business(&self) -> anyhow::Result<u128> {
        let mut monkeys_with_their_inspection_counts = self
            .monkeys
            .iter()
            .map(|monkey| (monkey.index, monkey.count_of_item_inspections))
            .collect::<Vec<_>>();
        monkeys_with_their_inspection_counts.sort_by(|&(_, ref a), &(_, ref b)| b.cmp(a));
        let most_inspection_count = monkeys_with_their_inspection_counts
            .get(0)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "There might be no monkeys, because cannot get first monkey inspection count."
                )
            })?
            .1;
        let second_most_inspection_count = monkeys_with_their_inspection_counts.get(1)
            .ok_or_else(|| anyhow::anyhow!("There are not enough monkeys, because cannot get second monkey inspection count."))?
            .1;
        most_inspection_count.checked_mul(second_most_inspection_count)
            .ok_or_else(|| anyhow::anyhow!("The two most inspection counts are too large to multiply ({most_inspection_count} and {second_most_inspection_count})."))
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

                    let monkey_index: usize = monkey_lines[0]
                        .strip_prefix("Monkey ")
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} first line does not start with \"Monkey \"."))?
                        .strip_suffix(':')
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} first line does not end with \":\"."))?
                        .parse()
                        .with_context(|| format!("Part #{part_index} monkey index is no valid number."))?;

                    let items = monkey_lines[1]
                        .strip_prefix("  Starting items: ")
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} second line does not start with \"  Starting items: \"."))?
                        .split(", ")
                        .map(|starting_item_worry_level_str| {
                            starting_item_worry_level_str
                                .parse()
                                .map(|worry_level| Item {
                                    worry_level: WorryLevel(worry_level),
                                })
                        })
                        .collect::<Result<Vec<_>, _>>()
                        .with_context(|| format!("Part #{part_index} starting items contain an invalid number."))?;

                    let operation_str = monkey_lines[2].strip_prefix("  Operation: ")
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} third line does not start with \"  Operation: \"."))?;
                    let operation = Operation::from_str(operation_str)
                        .with_context(|| format!("Part #{part_index} operation could not be parsed."))?;

                    let condition_divisible_by = monkey_lines[3].strip_prefix("  Test: divisible by ")
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} fourth line does not start with \"  Test: divisible by \"."))?
                        .parse()
                        .with_context(|| format!("Part #{part_index} divisible by test number could not be parsed."))?;

                    let test_if_true = monkey_lines[4]
                        .strip_prefix("    If true: throw to monkey ")
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} fifth line does not start with \"    If true: throw to monkey \"."))?
                        .parse()
                        .with_context(|| format!("Part #{part_index} test if true target monkey index could not be parsed"))?;
                    let test_if_false = monkey_lines[5]
                        .strip_prefix("    If false: throw to monkey ")
                        .ok_or_else(|| anyhow::anyhow!("Part #{part_index} fifth line does not start with \"    If false: throw to monkey \"."))?
                        .parse()
                        .with_context(|| format!("Part #{part_index} test if false target monkey index could not be parsed"))?;

                    Ok(Monkey {
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
                .collect::<Result<Vec<_>, anyhow::Error>>()?,
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
    fn apply(&self, worry_level: WorryLevel) -> anyhow::Result<WorryLevel> {
        let applied = match *self {
            Operation::Product { factor } => worry_level.0.checked_mul(factor),
            Operation::ProductByFactorOld => worry_level.0.checked_mul(worry_level.0),
            Operation::Sum { summand } => worry_level.0.checked_add(summand),
        };
        applied
            .map(WorryLevel)
            .ok_or_else(|| anyhow::anyhow!("Could not apply {self:?} to {worry_level:?}."))
    }
}

impl FromStr for Operation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let suffix = s
            .strip_prefix("new = old ")
            .ok_or_else(|| anyhow::anyhow!("Value does not start with required \"new = old \"."))?;

        let parts: [&str; 2] =
            suffix
                .split(' ')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|vec: Vec<_>| {
                    anyhow::anyhow!(
                        "Value suffix after \"new = old \" are not two elements, but {} ({:?}).",
                        vec.len(),
                        vec
                    )
                })?;

        match (parts[0], parts[1]) {
            ("*", "old") => Ok(Self::ProductByFactorOld),
            ("*", _) => Ok(Self::Product {
                factor: parts[1]
                    .parse()
                    .context("Could not parse product factor.")?,
            }),
            ("+", _) => Ok(Self::Sum {
                summand: parts[1].parse().context("Could not parse sum summand.")?,
            }),
            _ => Err(anyhow::anyhow!(
                "Unexpected operator ({}) and operand ({}).",
                parts[0],
                parts[1]
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Test {
    condition_divisible_by: u64,
    target_if_true: MonkeyIndex,
    target_if_false: MonkeyIndex,
}

#[allow(clippy::panic_in_result_fn)]
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
        monkey_keep_away.run_for_rounds(20, WorryType::WithRelief)?;
        let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business()?;

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
        monkey_keep_away.run(WorryType::WithRelief)?;

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
        monkey_keep_away.run_for_rounds(10_000, WorryType::NoRelief)?;
        let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business()?;

        // Assert
        assert_eq!(level_of_monkey_business, 2_713_310_158);

        Ok(())
    }
}
