use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::Context;

const INPUT: &str = include_str!("../inputs/day10.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 39 minutes 43 seconds
    let program = Program::from_str(INPUT)?;

    let mut cpu = CPU::new_with_register_x_value(1);
    cpu.load(program);

    let signal_strength_sums = Rc::new(RefCell::new(0));
    let signal_strength_sums_clone = Rc::clone(&signal_strength_sums);
    let mut clock_circuit = ClockCircuit::new(cpu);
    clock_circuit.set_during_cycle_callback(move |cpu, completed_cycles| match completed_cycles {
        20 | 60 | 100 | 140 | 180 | 220 => {
            *RefCell::borrow_mut(&signal_strength_sums_clone) +=
                i64::try_from(completed_cycles).unwrap() * cpu.x_register.value;
        }
        _ => (),
    });
    clock_circuit.run()?;

    let part_1_solution = *RefCell::borrow(&signal_strength_sums);
    println!("part_1_solution: {part_1_solution}");

    Ok(())
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct Program {
    instructions: VecDeque<Instruction>,
}

impl FromStr for Program {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Program {
            instructions: s
                .lines()
                .enumerate()
                .map(|(index, line)| {
                    Instruction::from_str(line).with_context(|| format!("in line #{index}"))
                })
                .collect::<Result<VecDeque<_>, _>>()?,
        })
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Instruction {
    NoOp,
    AddX(i8),
}

impl FromStr for Instruction {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lowercase = s.to_lowercase();
        if lowercase == "noop" {
            Ok(Self::NoOp)
        } else if let Some(value) = lowercase.strip_prefix("addx ") {
            Ok(Self::AddX(
                value
                    .parse()
                    .with_context(|| format!("while parsing AddX value."))?,
            ))
        } else {
            Err(anyhow::anyhow!("Do not recognize instruction."))
        }
    }
}

struct CPU {
    x_register: Register,
    instruction_register: Option<(Instruction, u128)>,
    loaded_instructions: Option<VecDeque<Instruction>>,
}

impl CPU {
    fn new_with_register_x_value(value: i64) -> Self {
        Self {
            x_register: Register::new(value),
            instruction_register: None,
            loaded_instructions: None,
        }
    }

    fn load(&mut self, program: Program) {
        self.loaded_instructions = Some(program.instructions);
    }

    fn calculate_needed_cycles(instruction: &Instruction) -> u128 {
        match instruction {
            Instruction::NoOp => 1,
            Instruction::AddX(_) => 2,
        }
    }

    fn tick(&mut self) -> Result<(), CPUTickError> {
        let (current_instruction, remaining_cycles) = match self.instruction_register.take() {
            None => {
                let next_instruction = self
                    .loaded_instructions
                    .as_mut()
                    .ok_or(CPUTickError::NoProgramLoaded)?
                    .pop_front()
                    .ok_or(CPUTickError::ProgramEnded)?;
                let needed_cycles = Self::calculate_needed_cycles(&next_instruction);
                (next_instruction, needed_cycles)
            }
            Some(stored_instruction) => stored_instruction,
        };
        if remaining_cycles > 1 {
            self.instruction_register = Some((current_instruction, remaining_cycles - 1));
            Ok(())
        } else {
            match current_instruction {
                Instruction::NoOp => Ok(()),
                Instruction::AddX(value) => {
                    self.x_register.value = self
                        .x_register
                        .value
                        .checked_add(i64::from(value))
                        .ok_or(CPUTickError::Overflow)?;
                    Ok(())
                }
            }
        }
    }
}

#[derive(Debug, thiserror::Error)]
enum CPUTickError {
    #[error("No program is loaded.")]
    NoProgramLoaded,
    #[error("Program ended.")]
    ProgramEnded,
    #[error("Overflow happened.")]
    Overflow,
}

struct Register {
    value: i64,
}

impl Register {
    fn new(value: i64) -> Self {
        Self { value }
    }
}

struct ClockCircuit {
    cycles_completed: u128,
    cpu: CPU,
    cycle_completed_callback: Option<Box<dyn FnMut(&CPU, u128)>>,
    during_cycle_callback: Option<Box<dyn FnMut(&CPU, u128)>>,
}

impl ClockCircuit {
    fn new(cpu: CPU) -> Self {
        Self {
            cycles_completed: 0,
            cpu,
            cycle_completed_callback: None,
            during_cycle_callback: None,
        }
    }

    fn set_cycle_completed_callback<F>(&mut self, cycle_completed_callback: F)
    where
        F: FnMut(&CPU, u128) + 'static,
    {
        self.cycle_completed_callback = Some(Box::new(cycle_completed_callback));
    }

    fn set_during_cycle_callback<F>(&mut self, during_cycle_callback: F)
    where
        F: FnMut(&CPU, u128) + 'static,
    {
        self.during_cycle_callback = Some(Box::new(during_cycle_callback));
    }

    fn run(&mut self) -> anyhow::Result<()> {
        loop {
            if let Some(during_cycle_callback) = &mut self.during_cycle_callback {
                (during_cycle_callback)(&self.cpu, self.cycles_completed + 1);
            }
            match self.cpu.tick() {
                Ok(_) => (),
                Err(CPUTickError::ProgramEnded) => break Ok(()),
                other => other.with_context(|| format!("after cycle {}", self.cycles_completed))?,
            }
            self.cycles_completed += 1;
            if let Some(cycle_completed_callback) = &mut self.cycle_completed_callback {
                (cycle_completed_callback)(&self.cpu, self.cycles_completed);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    const TEST_INPUT: &str = "addx 15
addx -11
addx 6
addx -3
addx 5
addx -1
addx -8
addx 13
addx 4
noop
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx 5
addx -1
addx -35
addx 1
addx 24
addx -19
addx 1
addx 16
addx -11
noop
noop
addx 21
addx -15
noop
noop
addx -3
addx 9
addx 1
addx -3
addx 8
addx 1
addx 5
noop
noop
noop
noop
noop
addx -36
noop
addx 1
addx 7
noop
noop
noop
addx 2
addx 6
noop
noop
noop
noop
noop
addx 1
noop
noop
addx 7
addx 1
noop
addx -13
addx 13
addx 7
noop
addx 1
addx -33
noop
noop
noop
addx 2
noop
noop
noop
addx 8
noop
addx -1
addx 2
addx 1
noop
addx 17
addx -9
addx 1
addx 1
addx -3
addx 11
noop
noop
addx 1
noop
addx 1
noop
noop
addx -13
addx -19
addx 1
addx 3
addx 26
addx -30
addx 12
addx -1
addx 3
addx 1
noop
noop
noop
addx -9
addx 18
addx 1
addx 2
noop
noop
addx 9
noop
noop
noop
addx -1
addx 2
addx -37
addx 1
addx 3
noop
addx 15
addx -21
addx 22
addx -6
addx 1
noop
addx 2
addx 1
noop
addx -10
noop
noop
addx 20
addx 1
addx 2
addx 2
addx -6
addx -11
noop
noop
noop";

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        let program = Program::from_str(TEST_INPUT)?;
        let mut cpu = CPU::new_with_register_x_value(1);
        cpu.load(program);
        let signal_strength_sums = Rc::new(RefCell::new(0));
        let signal_strength_sums_clone = Rc::clone(&signal_strength_sums);
        let mut clock_circuit = ClockCircuit::new(cpu);
        clock_circuit.set_during_cycle_callback(move |cpu, completed_cycles| {
            let error_message = format!(
                "during cycle {completed_cycles}, remaining {} loaded instructions and currently {:?} in cpu",
                cpu.loaded_instructions.as_ref().unwrap().len(),
                cpu.instruction_register
            );
            match completed_cycles {
                20 => assert_eq!(cpu.x_register.value, 21, "{error_message}"),
                60 => assert_eq!(cpu.x_register.value, 19, "{error_message}"),
                100 => assert_eq!(cpu.x_register.value, 18, "{error_message}"),
                140 => assert_eq!(cpu.x_register.value, 21, "{error_message}"),
                180 => assert_eq!(cpu.x_register.value, 16, "{error_message}"),
                220 => assert_eq!(cpu.x_register.value, 18, "{error_message}"),
                _ => (),
            }
            match completed_cycles {
                20 | 60 | 100 | 140 | 180 | 220 => {
                    *RefCell::borrow_mut(&signal_strength_sums_clone) += i64::try_from(completed_cycles).unwrap() * cpu.x_register.value;
                },
                _ => (),
            }
        });
        clock_circuit.run()?;
        assert_eq!(*RefCell::borrow(&signal_strength_sums), 13_140);

        Ok(())
    }

    #[test]
    fn test_program_from_str() -> anyhow::Result<()> {
        // Arrange
        let small_program_str = "noop
addx 3
addx -5";

        // Act
        let program = Program::from_str(small_program_str)?;

        // Assert
        assert_eq!(
            program.instructions,
            vec![
                Instruction::NoOp,
                Instruction::AddX(3),
                Instruction::AddX(-5),
            ],
        );

        Ok(())
    }

    #[test]
    fn test_cpu_without_clock_circuit() -> anyhow::Result<()> {
        let mut cpu = CPU::new_with_register_x_value(1);
        assert_eq!(cpu.x_register.value, 1);

        cpu.load(Program {
            instructions: VecDeque::from(vec![
                Instruction::NoOp,
                Instruction::AddX(3),
                Instruction::AddX(-5),
            ]),
        });

        cpu.tick()?;
        assert_eq!(cpu.x_register.value, 1);

        cpu.tick()?;
        assert_eq!(cpu.x_register.value, 1);

        cpu.tick()?;
        assert_eq!(cpu.x_register.value, 4);

        cpu.tick()?;
        assert_eq!(cpu.x_register.value, 4);

        cpu.tick()?;
        assert_eq!(cpu.x_register.value, -1);

        Ok(())
    }

    #[test]
    fn test_cpu_with_clock_circuit() -> anyhow::Result<()> {
        let mut cpu = CPU::new_with_register_x_value(1);
        assert_eq!(cpu.x_register.value, 1);

        cpu.load(Program {
            instructions: VecDeque::from(vec![Instruction::NoOp, Instruction::AddX(3)]),
        });

        let cycle_history = Rc::new(RefCell::new(Vec::new()));
        let cycle_history_cloned = Rc::clone(&cycle_history);
        let mut clock_circuit = ClockCircuit::new(cpu);
        clock_circuit.set_cycle_completed_callback(move |cpu: &CPU, cycles_completed: u128| {
            RefCell::borrow_mut(&cycle_history_cloned).push(cpu.x_register.value);
            assert_eq!(
                cycles_completed,
                RefCell::borrow(&cycle_history_cloned).len() as u128
            );
        });
        assert_eq!(clock_circuit.cycles_completed, 0);
        assert_eq!(*RefCell::borrow(&cycle_history), vec![]);

        clock_circuit.run()?;
        assert_eq!(clock_circuit.cycles_completed, 3);
        assert_eq!(*RefCell::borrow(&cycle_history), vec![1, 1, 4]);

        clock_circuit.cpu.load(Program {
            instructions: VecDeque::from(vec![Instruction::AddX(-5)]),
        });

        clock_circuit.run()?;
        assert_eq!(clock_circuit.cycles_completed, 5);
        assert_eq!(*RefCell::borrow(&cycle_history), vec![1, 1, 4, 4, -1]);

        Ok(())
    }
}
