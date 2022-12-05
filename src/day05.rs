use std::ops::Div;
use std::str::FromStr;

use anyhow::Context;

use regex::Regex;

const INPUT: &str = include_str!("../inputs/day05.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 37 minutes 4 seconds
    let crane_message = calculate_resulting_crane_message(INPUT)?;
    println!("crane_message: {crane_message}");
    Ok(())
}

fn calculate_resulting_crane_message(input: &str) -> anyhow::Result<String> {
    let mut procedure = RearrangementProcedure::from_str(input)?;
    while !procedure.is_finished() {
        println!("{:?}", procedure.crate_stacks);
        procedure.rearrange();
    }
    println!("{:?}", procedure.crate_stacks);
    Ok(procedure
        .crate_stacks
        .stacks
        .iter()
        .map(|stack| stack.last().copied().unwrap_or(' '))
        .collect::<String>())
}

#[derive(Debug, Eq, PartialEq)]
struct RearrangementProcedure {
    crate_stacks: CrateStacks,
    procedure_steps: Vec<ProcedureStep>,
}

impl RearrangementProcedure {
    fn is_finished(&self) -> bool {
        self.procedure_steps.is_empty()
    }

    fn rearrange(&mut self) {
        if !self.procedure_steps.is_empty() {
            let next_procedure = self.procedure_steps.remove(0);
            println!("next_procedure: {next_procedure:?}");
            let mut temp = Vec::with_capacity(next_procedure.count.into());
            while temp.len() < next_procedure.count.into() {
                let m: &mut Vec<char> = self
                    .crate_stacks
                    .stacks
                    .get_mut(usize::from(next_procedure.from) - 1)
                    .unwrap();
                temp.push(m.pop().unwrap());
            }
            for m in temp {
                let mm: &mut Vec<char> = self
                    .crate_stacks
                    .stacks
                    .get_mut(usize::from(next_procedure.to) - 1)
                    .unwrap();
                mm.push(m);
            }
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
            crate_stacks: CrateStacks::from_str(both_parts[0])?,
            procedure_steps: both_parts[1]
                .lines()
                .map(ProcedureStep::from_str)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
struct CrateStacks {
    stacks: Vec<Vec<char>>,
}

impl FromStr for CrateStacks {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            stacks: s
                .lines()
                .enumerate()
                .map(|(index, line)| {
                    let line = line.chars().collect::<Vec<_>>();
                    let element_count = line
                        .len()
                        .checked_sub(3)
                        .ok_or_else(|| anyhow::anyhow!("Stack line #{index} is empty."))?
                        .div(4)
                        .checked_add(1)
                        .ok_or_else(|| {
                            anyhow::anyhow!("Stack line #{index} has too many elements.")
                        })?;
                    let mut output = Vec::with_capacity(element_count);
                    for element_index in 0..element_count {
                        let stack_begin_index = element_index
                            .checked_mul(4)
                            .ok_or_else(|| anyhow::anyhow!("Stack line #{index} is too long."))?;
                        if *line
                            .get(stack_begin_index)
                            .ok_or_else(|| anyhow::anyhow!("TODO"))?
                            == '['
                        {
                            let crate_name_index = stack_begin_index
                                .checked_add(1)
                                .ok_or_else(|| anyhow::anyhow!("TODO"))?;
                            output.push(Some(
                                *line
                                    .get(crate_name_index)
                                    .ok_or_else(|| anyhow::anyhow!("TODO"))?,
                            ));
                        } else {
                            output.push(None);
                        }
                    }
                    Ok::<_, anyhow::Error>(output)
                })
                .collect::<Result<Vec<_>, _>>()?
                .into_iter()
                .rev()
                .fold(Vec::new(), |mut stacks, crate_line| {
                    while stacks.len() < crate_line.len() {
                        stacks.push(Vec::new());
                    }
                    for (index, optional_crate) in crate_line.into_iter().enumerate() {
                        if let Some(_crate) = optional_crate {
                            stacks[index].push(_crate);
                        }
                    }
                    stacks
                }),
        })
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ProcedureStep {
    count: u8,
    from: u8,
    to: u8,
}

impl FromStr for ProcedureStep {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let re = Regex::new("move (\\d+) from (\\d+) to (\\d+)")
            .context(format!("creating the regex in ProcedureStep::from_str"))?;
        let m = re
            .captures(s)
            .ok_or_else(|| anyhow::anyhow!("Input did not match expected pattern"))?;
        let count = m
            .get(1)
            .ok_or_else(|| anyhow::anyhow!(""))?
            .as_str()
            .parse::<u8>()
            .context(format!(""))?;
        let from = m
            .get(2)
            .ok_or_else(|| anyhow::anyhow!(""))?
            .as_str()
            .parse::<u8>()
            .context(format!(""))?;
        let to = m
            .get(3)
            .ok_or_else(|| anyhow::anyhow!(""))?
            .as_str()
            .parse::<u8>()
            .context(format!(""))?;
        Ok(Self { count, from, to })
    }
}

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

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Act
        let result = calculate_resulting_crane_message(TEST_INPUT)?;

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
        assert_eq!(
            procedure_steps,
            vec![
                ProcedureStep {
                    count: 1,
                    from: 2,
                    to: 1
                },
                ProcedureStep {
                    count: 3,
                    from: 1,
                    to: 3
                },
                ProcedureStep {
                    count: 2,
                    from: 2,
                    to: 1
                },
                ProcedureStep {
                    count: 1,
                    from: 1,
                    to: 2
                },
            ]
        );

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
        let crate_stacks = CrateStacks::from_str(input)?;

        // Assert
        assert_eq!(
            crate_stacks,
            CrateStacks {
                stacks: vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P'],],
            }
        );

        Ok(())
    }

    #[test]
    fn test_rearrangement_procedure() {
        // Arrange
        let mut procedure = RearrangementProcedure {
            crate_stacks: CrateStacks {
                stacks: vec![vec!['Z', 'N'], vec!['M', 'C', 'D'], vec!['P']],
            },
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
        procedure.rearrange();

        // Assert 1
        assert_eq!(
            procedure.crate_stacks,
            CrateStacks {
                stacks: vec![vec!['Z', 'N', 'D'], vec!['M', 'C'], vec!['P'],]
            }
        );

        // Act 2
        procedure.rearrange();

        // Assert 2
        assert_eq!(
            procedure.crate_stacks,
            CrateStacks {
                stacks: vec![vec![], vec!['M', 'C'], vec!['P', 'D', 'N', 'Z'],]
            }
        );
    }
}
