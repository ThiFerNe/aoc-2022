use std::ops::Div;
use std::str::FromStr;

use anyhow::Context;

use regex::{Captures, Regex};

const INPUT: &str = include_str!("../inputs/day05.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 37 minutes 4 seconds
    let crane_message_1 = calculate_crane_message_with_crane_mover_9000(INPUT)?;
    println!("crane_message_1: {crane_message_1}");

    // PART 2 - 4 minutes 27 seconds
    let crane_message_2 = calculate_crane_message_with_crane_mover_9001(INPUT)?;
    println!("crane_message_2: {crane_message_2}");

    Ok(())
}

fn calculate_crane_message_with_crane_mover_9000(input: &str) -> anyhow::Result<String> {
    let mut procedure = RearrangementProcedure::from_str(input)?;
    while procedure.work_left() {
        procedure.rearrange_as_crane_mover_9000()?;
    }
    Ok(procedure.stacks.get_top_crates_as_string())
}

fn calculate_crane_message_with_crane_mover_9001(input: &str) -> anyhow::Result<String> {
    let mut procedure = RearrangementProcedure::from_str(input)?;
    while procedure.work_left() {
        procedure.rearrange_as_crate_mover_9001()?;
    }
    Ok(procedure.stacks.get_top_crates_as_string())
}

#[derive(Debug, Eq, PartialEq)]
struct RearrangementProcedure {
    stacks: Stacks,
    procedure_steps: Vec<ProcedureStep>,
}

impl RearrangementProcedure {
    fn work_left(&self) -> bool {
        !self.procedure_steps.is_empty()
    }

    fn rearrange_as_crane_mover_9000(&mut self) -> anyhow::Result<()> {
        self.rearrange(|target_stack, crates| {
            crates
                .into_iter()
                .for_each(|single_crate| target_stack.push(single_crate));
        })
    }

    fn rearrange_as_crate_mover_9001(&mut self) -> anyhow::Result<()> {
        self.rearrange(|target_stack, crates| {
            crates
                .into_iter()
                .rev()
                .for_each(|single_crate| target_stack.push(single_crate));
        })
    }

    fn rearrange<F>(&mut self, reinsert_method: F) -> anyhow::Result<()>
    where
        F: Fn(&mut Vec<char>, Vec<char>),
    {
        if self.procedure_steps.is_empty() {
            Err(anyhow::anyhow!("No procedure steps left."))
        } else {
            let next_procedure = self.procedure_steps.remove(0);

            let source_stack_index =
                usize::from(next_procedure.from)
                    .checked_sub(1)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not subtract 1 from procedure step from `{}`.",
                            next_procedure.from
                        )
                    })?;
            let source_stack = self
                .stacks
                .get_stack_with_raw_index_mut(source_stack_index)
                .context("getting source stack")?;

            let crates_to_move = (0..next_procedure.count)
                .map(|_| {
                    source_stack.pop().ok_or_else(|| {
                        anyhow::anyhow!("Not enough crates on stack #{source_stack_index}.")
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            let target_stack_index =
                usize::from(next_procedure.to)
                    .checked_sub(1)
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not subtract 1 from procedure step to `{}`.",
                            next_procedure.to
                        )
                    })?;
            let target_stack = self
                .stacks
                .get_stack_with_raw_index_mut(target_stack_index)
                .context("getting target stack")?;

            reinsert_method(target_stack, crates_to_move);

            Ok(())
        }
    }
}

impl FromStr for RearrangementProcedure {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let both_parts: [&str; 2] =
            s.split("\n\n")
                .collect::<Vec<&str>>()
                .try_into()
                .map_err(|vec: Vec<_>| {
                    anyhow::anyhow!("Split string is not 2 parts but {} ({:?})", vec.len(), vec)
                })?;
        Ok(Self {
            stacks: Stacks::from_str(both_parts[0])?,
            procedure_steps: both_parts[1]
                .lines()
                .enumerate()
                .map(|(index, line)| {
                    ProcedureStep::from_str(line).with_context(|| format!("on line #{index}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
struct Stacks(Vec<Vec<char>>);

impl Stacks {
    fn get_stack_with_raw_index_mut(&mut self, index: usize) -> anyhow::Result<&mut Vec<char>> {
        self.0
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("Did not find stack #{}", index))
    }

    fn get_top_crates_as_string(&self) -> String {
        self.0
            .iter()
            .map(|stack| stack.last().copied().unwrap_or(' '))
            .collect::<String>()
    }
}

impl FromStr for Stacks {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self (
            s
                .lines()
                .enumerate()
                .map(|(index, line)| {
                    let crates_in_line = line.chars().collect::<Vec<_>>();

                    let element_count = crates_in_line
                        .len()
                        .checked_sub(3)
                        .ok_or_else(|| anyhow::anyhow!("Stack line #{index} is empty."))?
                        .div(4)
                        .checked_add(1)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Stack line #{index} has too many elements.")
                        })?;

                    (0..element_count)
                        .map(|element_index| {
                            let stack_begin_index = element_index
                                .checked_mul(4)
                                .ok_or_else(|| anyhow::anyhow!("Stack line #{index} is too long."))?;
                            let stack_first_char = *crates_in_line
                                .get(stack_begin_index)
                                .ok_or_else(|| anyhow::anyhow!("Could not find #{element_index} stack in stack line #{index}"))?;

                            if stack_first_char == '[' {
                                let crate_name_index = stack_begin_index
                                    .checked_add(1)
                                    .ok_or_else(|| anyhow::anyhow!("Could not find crate name in #{element_index} stack in stack line #{index}"))?;
                                let crate_name = *crates_in_line
                                    .get(crate_name_index)
                                    .ok_or_else(|| anyhow::anyhow!("Could not get crate name in #{element_index} stack in stack line #{index}"))?;
                                Ok(Some(crate_name))
                            } else {
                                Ok(None)
                            }
                        })
                        .collect::<anyhow::Result<Vec<_>>>()
                })
                .collect::<anyhow::Result<Vec<_>>>()?
                .into_iter()
                .rev()
                .fold(Ok(Vec::new()), |mut stacks: anyhow::Result<Vec<Vec<char>>>, crate_line| {
                    if let Ok(ref mut ok_stacks) = stacks {
                        while ok_stacks.len() < crate_line.len() {
                            ok_stacks.push(Vec::new());
                        }
                        for (index, optional_crate) in crate_line.into_iter().enumerate() {
                            if let Some(single_crate) = optional_crate {
                                ok_stacks
                                    .get_mut(index)
                                    .ok_or_else(|| anyhow::anyhow!("Did not find stack index #{index}."))?
                                    .push(single_crate);
                            }
                        }
                    }
                    stacks
                })?,
        ))
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct ProcedureStep {
    count: u8,
    from: u8,
    to: u8,
}

impl FromStr for ProcedureStep {
    type Err = anyhow::Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        fn extract_byte_from_capture_group(
            captures: &Captures,
            index: usize,
        ) -> anyhow::Result<u8> {
            captures
                .get(index)
                .ok_or_else(|| anyhow::anyhow!("Missing capture group."))?
                .as_str()
                .parse::<u8>()
                .context("Capture group did not capture a number.")
        }

        let line_captures = Regex::new("move (\\d+) from (\\d+) to (\\d+)")
            .context("creating the regex in ProcedureStep::from_str")?
            .captures(input)
            .ok_or_else(|| anyhow::anyhow!("Input did not match expected pattern."))?;
        Ok(Self {
            count: extract_byte_from_capture_group(&line_captures, 1).context("with `count`")?,
            from: extract_byte_from_capture_group(&line_captures, 2).context("with `from`")?,
            to: extract_byte_from_capture_group(&line_captures, 3).context("with `to`")?,
        })
    }
}

#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "    [D]
[N] [C]
[Z] [M] [P]
 1   2   3

move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

    const TEST_PROCEDURE_STEPS: [ProcedureStep; 4] = [
        ProcedureStep {
            count: 1,
            from: 2,
            to: 1,
        },
        ProcedureStep {
            count: 3,
            from: 1,
            to: 3,
        },
        ProcedureStep {
            count: 2,
            from: 2,
            to: 1,
        },
        ProcedureStep {
            count: 1,
            from: 1,
            to: 2,
        },
    ];

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Act
        let result = calculate_crane_message_with_crane_mover_9000(TEST_INPUT)?;

        // Assert
        assert_eq!(result, "CMZ");

        Ok(())
    }

    #[test]
    fn test_procedure_step_from_str() -> anyhow::Result<()> {
        // Arrange
        let input: &str = "move 1 from 2 to 1
move 3 from 1 to 3
move 2 from 2 to 1
move 1 from 1 to 2";

        // Act
        let procedure_steps = input
            .lines()
            .map(ProcedureStep::from_str)
            .collect::<Result<Vec<_>, _>>()?;

        // Assert
        assert_eq!(&procedure_steps, &TEST_PROCEDURE_STEPS);

        Ok(())
    }

    #[test]
    fn test_crate_stacks_from_str() -> anyhow::Result<()> {
        // Arrange
        let input: &str = "    [D]
[N] [C]
[Z] [M] [P]
 1   2   3";

        // Act
        let crate_stacks = Stacks::from_str(input)?;

        // Assert
        assert_eq!(
            crate_stacks,
            Stacks(vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P'],])
        );

        Ok(())
    }

    #[test]
    fn test_rearrangement_procedure() -> anyhow::Result<()> {
        // Arrange
        let mut procedure = RearrangementProcedure {
            stacks: Stacks(vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P']]),
            procedure_steps: vec![
                ProcedureStep {
                    count: 1,
                    from: 2,
                    to: 1,
                },
                ProcedureStep {
                    count: 3,
                    from: 1,
                    to: 3,
                },
            ],
        };

        // Act 1
        procedure.rearrange_as_crane_mover_9000()?;

        // Assert 1
        assert_eq!(
            procedure.stacks,
            Stacks(vec![vec!['Z', 'N', 'D'], vec!['M', 'C'], vec!['P'],])
        );

        // Act 2
        procedure.rearrange_as_crane_mover_9000()?;

        // Assert 2
        assert_eq!(
            procedure.stacks,
            Stacks(vec![vec![], vec!['M', 'C'], vec!['P', 'D', 'N', 'Z'],])
        );

        Ok(())
    }

    #[test]
    fn test_rearrangement_procedure_rearrange_as_crate_mover_9001() -> anyhow::Result<()> {
        // Arrange
        let mut procedure = RearrangementProcedure {
            stacks: Stacks(vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P']]),
            procedure_steps: TEST_PROCEDURE_STEPS.to_vec(),
        };

        // Act 1
        procedure.rearrange_as_crate_mover_9001()?;

        // Assert 1
        assert_eq!(
            procedure.stacks,
            Stacks(vec![vec!['Z', 'N', 'D'], vec!['M', 'C'], vec!['P']])
        );

        // Act 2
        procedure.rearrange_as_crate_mover_9001()?;

        // Assert 2
        assert_eq!(
            procedure.stacks,
            Stacks(vec![vec![], vec!['M', 'C'], vec!['P', 'Z', 'N', 'D']])
        );

        // Act 3
        procedure.rearrange_as_crate_mover_9001()?;

        // Assert 3
        assert_eq!(
            procedure.stacks,
            Stacks(vec![vec!['M', 'C'], vec![], vec!['P', 'Z', 'N', 'D']])
        );

        // Act 4
        procedure.rearrange_as_crate_mover_9001()?;

        // Assert 4
        assert_eq!(
            procedure.stacks,
            Stacks(vec![vec!['M'], vec!['C'], vec!['P', 'Z', 'N', 'D']])
        );

        Ok(())
    }
}
