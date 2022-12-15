use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day11.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 16 minutes 53 seconds
    let mut monkey_keep_away = MonkeyKeepAway::from_str(INPUT)?;
    monkey_keep_away.run_for_rounds(20);
    let level_of_monkey_business = monkey_keep_away.calculate_level_of_monkey_business();
    println!("level_of_monkey_business: {level_of_monkey_business}");
    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct MonkeyKeepAway {
    monkeys: Vec<Monkey>,
}

impl MonkeyKeepAway {
    fn run(&mut self) {
        let count_of_monkey = self.monkeys.len();
        for current_monkey_index in 0..count_of_monkey {
            let mut throw_queue = Vec::new();
            let monkey_items = self.monkeys[current_monkey_index].take_items();
            let current_monkey = &self.monkeys[current_monkey_index];
            println!("Monkey {}:", current_monkey.index.0);
            for mut item in monkey_items {
                println!(
                    " Monkey inspects an item with a worry level of {}.",
                    item.worry_level
                );
                item.worry_level = current_monkey.operation.apply(item.worry_level);
                match &current_monkey.operation {
                    Operation::Product { factor } => println!(
                        "  Worry level is multiplied by {factor} to {}.",
                        item.worry_level
                    ),
                    Operation::ProductByFactorOld => println!(
                        "  Worry level is multiplied by itself to {}.",
                        item.worry_level
                    ),
                    Operation::Sum { summand } => println!(
                        "  Worry level increases by {summand} to {}.",
                        item.worry_level
                    ),
                }
                item.worry_level = item.worry_level / 3;
                println!(
                    "  Monkey gets bored with item. Worry level is divided by 3 to {}.",
                    item.worry_level
                );
                let predicament_result = current_monkey.test.predicament.check(item.worry_level);
                match &current_monkey.test.predicament {
                    Predicament::Divisible { by } => println!(
                        "  Current worry level is{} divisible by {by}.",
                        if predicament_result { "" } else { " not" }
                    ),
                }
                let target = if predicament_result {
                    current_monkey.test.on_true_target
                } else {
                    current_monkey.test.on_false_target
                };
                println!(
                    "  Item with worry level {} is thrown to monkey {}.",
                    item.worry_level, target.0
                );
                throw_queue.push((target, item));
            }
            self.monkeys[current_monkey_index].count_of_item_inspections +=
                throw_queue.len() as u128;
            for (monkey_index, item) in throw_queue {
                self.monkeys[monkey_index.0 as usize].items.push(item);
            }
        }

        // for each monkey
        // - inspect item
        // -- worry level operation
        // -- monkey gets bored, divide by 3
        // -- predicament check
        // -- test execution
    }

    fn run_for_rounds(&mut self, count_of_rounds: u128) {
        for _ in 0..count_of_rounds {
            self.run();
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
                                .map(|worry_level| Item { worry_level })
                        })
                        .collect::<Result<Vec<_>, _>>()?;

                    let operation_str = monkey_lines[2].strip_prefix("  Operation: ").unwrap();
                    let operation = Operation::from_str(operation_str).unwrap();

                    let test_predicament_str = monkey_lines[3].strip_prefix("  Test: ").unwrap();
                    let test_predicament = Predicament::from_str(test_predicament_str).unwrap();

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
                            predicament: test_predicament,
                            on_true_target: MonkeyIndex(test_if_true),
                            on_false_target: MonkeyIndex(test_if_false),
                        },
                        count_of_item_inspections: 0,
                    })
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Monkey {
    index: MonkeyIndex,
    items: Vec<Item>,
    operation: Operation,
    test: Test,
    count_of_item_inspections: u128,
}

impl Monkey {
    fn take_items(&mut self) -> Vec<Item> {
        let mut monkey_items = Vec::new();
        std::mem::swap(&mut monkey_items, &mut self.items);
        monkey_items
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct MonkeyIndex(u8);

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Item {
    worry_level: u64,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Operation {
    Product { factor: u64 },
    ProductByFactorOld,
    Sum { summand: u64 },
}

impl Operation {
    fn apply(&self, value: u64) -> u64 {
        match self {
            Operation::Product { factor } => value * *factor,
            Operation::ProductByFactorOld => value * value,
            Operation::Sum { summand } => value + *summand,
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
    predicament: Predicament,
    on_true_target: MonkeyIndex,
    on_false_target: MonkeyIndex,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Predicament {
    Divisible { by: u64 },
}

impl Predicament {
    fn check(&self, value: u64) -> bool {
        match self {
            Predicament::Divisible { by } => value % *by == 0,
        }
    }
}

impl FromStr for Predicament {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let by = s.strip_prefix("divisible by ").unwrap().parse().unwrap();
        Ok(Self::Divisible { by })
    }
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
        monkey_keep_away.run_for_rounds(20);
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
                            Item { worry_level: 79 },
                            Item { worry_level: 98 },
                        ],
                        operation: Operation::Product { factor: 19 },
                        test: Test {
                            predicament: Predicament::Divisible { by: 23 },
                            on_true_target: MonkeyIndex(2),
                            on_false_target: MonkeyIndex(3)
                        },
                        count_of_item_inspections: 0,
                    },
                    Monkey {
                        index: MonkeyIndex(1),
                        items: vec![
                            Item { worry_level: 54 },
                            Item { worry_level: 65 },
                            Item { worry_level: 75 },
                            Item { worry_level: 74 },
                        ],
                        operation: Operation::Sum { summand: 6 },
                        test: Test {
                            predicament: Predicament::Divisible { by: 19 },
                            on_true_target: MonkeyIndex(2),
                            on_false_target: MonkeyIndex(0)
                        },
                        count_of_item_inspections: 0,
                    },
                    Monkey {
                        index: MonkeyIndex(2),
                        items: vec![
                            Item { worry_level: 79 },
                            Item { worry_level: 60 },
                            Item { worry_level: 97 },
                        ],
                        operation: Operation::ProductByFactorOld,
                        test: Test {
                            predicament: Predicament::Divisible { by: 13 },
                            on_true_target: MonkeyIndex(1),
                            on_false_target: MonkeyIndex(3)
                        },
                        count_of_item_inspections: 0,
                    },
                    Monkey {
                        index: MonkeyIndex(3),
                        items: vec![
                            Item { worry_level: 74 },
                        ],
                        operation: Operation::Sum { summand: 3 },
                        test: Test {
                            predicament: Predicament::Divisible { by: 17 },
                            on_true_target: MonkeyIndex(0),
                            on_false_target: MonkeyIndex(1)
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
        monkey_keep_away.run();

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
                    Item { worry_level: 20 },
                    Item { worry_level: 23 },
                    Item { worry_level: 27 },
                    Item { worry_level: 26 }
                ],
                vec![
                    Item { worry_level: 2080 },
                    Item { worry_level: 25 },
                    Item { worry_level: 167 },
                    Item { worry_level: 207 },
                    Item { worry_level: 401 },
                    Item { worry_level: 1046 }
                ],
                vec![],
                vec![],
            ]
        );

        Ok(())
    }
}
