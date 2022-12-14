use std::cell::RefCell;
use std::collections::VecDeque;
use std::fmt::Display;
use std::rc::Rc;
use std::str::FromStr;

use anyhow::Context;

const INPUT: &str = include_str!("../inputs/day10.input");

fn main() -> anyhow::Result<()> {
    let program = Program::from_str(INPUT)?;
    let mut communication_device = CommunicationDevice::default();
    communication_device.set_default_program(program);

    // PART 1 - 1 hour 39 minutes 43 seconds
    communication_device.reset();
    let part_1_solution = communication_device
        .calculate_sum_of_interesting_signal_strengths(1, vec![20, 60, 100, 140, 180, 220])
        .context("while calculating sum of interesting signal strengths")?;
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 4 minutes 4 seconds + 50 minutes 37 seconds = 54 minutes 41 seconds
    communication_device.reset();
    let part_2_solution = communication_device
        .calculate_crt_image(1)
        .context("while calculating crt image")?;
    println!("part_2_solution:\n{part_2_solution}");

    Ok(())
}

struct CommunicationDevice<const CRT_COLUMNS: usize = 40, const CRT_ROWS: usize = 6> {
    default_program: Option<Program>,
    clock_circuit: ClockCircuit,
    crt: Rc<RefCell<Crt<CRT_COLUMNS, CRT_ROWS>>>,
}

impl<const CRT_COLUMNS: usize, const CRT_ROWS: usize> CommunicationDevice<CRT_COLUMNS, CRT_ROWS> {
    fn new() -> Self {
        Self {
            default_program: None,
            clock_circuit: ClockCircuit::new(Cpu::new_with_register_x_value(0)),
            crt: Rc::new(RefCell::new(Crt::default())),
        }
    }

    fn set_default_program(&mut self, program: Program) {
        self.default_program = Some(program);
    }

    fn reset(&mut self) {
        self.clock_circuit.reset();
        RefCell::borrow_mut(&self.crt).reset();
    }

    fn calculate_sum_of_interesting_signal_strengths(
        &mut self,
        starting_cpu_x_register_value: i64,
        look_during_cycles: Vec<u128>,
    ) -> anyhow::Result<i64> {
        self.reset();
        if let Some(ref program) = self.default_program {
            self.clock_circuit.cpu.load(program.clone());
        }
        self.clock_circuit.cpu.x_register.value = starting_cpu_x_register_value;

        let signal_strength_sums: Rc<RefCell<i64>> = Rc::new(RefCell::new(0));
        let signal_strength_sums_clone = Rc::clone(&signal_strength_sums);
        self.clock_circuit.set_during_cycle_callback(move |cpu, completed_cycles| {
            if look_during_cycles.contains(&completed_cycles) {
                let mut signal_strength_sums_clone_borrow = RefCell::borrow_mut(&signal_strength_sums_clone);
                let signal_strength = i64::try_from(completed_cycles)
                    .with_context(|| format!("while converting {completed_cycles} into i64"))?
                    .checked_mul(cpu.x_register.value)
                    .ok_or_else(|| anyhow::anyhow!("Cannot multiply completed_cycles={completed_cycles} with cpu.x_register.value={} .", cpu.x_register.value))?;
                *signal_strength_sums_clone_borrow = signal_strength_sums_clone_borrow
                    .checked_add(signal_strength)
                    .ok_or_else(|| anyhow::anyhow!("Cannot add signal_strength={signal_strength} to signal_strength_sums."))?;
            }
            Ok(())
        });
        self.clock_circuit.run()?;

        let result: i64 = *RefCell::borrow(&signal_strength_sums);
        Ok(result)
    }

    fn calculate_crt_image(
        &mut self,
        starting_cpu_x_register_value: i64,
    ) -> anyhow::Result<String> {
        self.reset();
        if let Some(ref program) = self.default_program {
            self.clock_circuit.cpu.load(program.clone());
        }
        self.clock_circuit.cpu.x_register.value = starting_cpu_x_register_value;

        let crt_clone = Rc::clone(&self.crt);

        self.clock_circuit
            .set_during_cycle_callback(move |cpu, during_cycle| {
                RefCell::borrow_mut(&crt_clone)
                    .process_signal(cpu.x_register.value, during_cycle)
                    .with_context(|| {
                        format!("while processing signal during cycle #{during_cycle}")
                    })
            });
        self.clock_circuit.run()?;

        Ok(RefCell::borrow(&self.crt).to_string())
    }
}

impl Default for CommunicationDevice {
    fn default() -> Self {
        Self::new()
    }
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
                value.parse().context("while parsing AddX value.")?,
            ))
        } else {
            Err(anyhow::anyhow!("Do not recognize instruction."))
        }
    }
}

#[derive(Debug)]
struct Cpu {
    x_register: Register,
    instruction_register: Option<(Instruction, u128)>,
    loaded_instructions: Option<VecDeque<Instruction>>,
}

impl Cpu {
    fn new_with_register_x_value(value: i64) -> Self {
        Self {
            x_register: Register::new(value),
            instruction_register: None,
            loaded_instructions: None,
        }
    }

    fn reset(&mut self) {
        self.x_register.value = 0;
        self.instruction_register = None;
        self.loaded_instructions = None;
    }

    fn load(&mut self, program: Program) {
        self.loaded_instructions = Some(program.instructions);
    }

    fn calculate_needed_cycles(instruction: Instruction) -> u128 {
        match instruction {
            Instruction::NoOp => 1,
            Instruction::AddX(_) => 2,
        }
    }

    fn has_instruction_left(&self) -> bool {
        self.instruction_register.is_some()
            || self
                .loaded_instructions
                .as_ref()
                .map_or(false, |loaded_instructions| !loaded_instructions.is_empty())
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
                let needed_cycles = Self::calculate_needed_cycles(next_instruction);
                (next_instruction, needed_cycles)
            }
            Some(stored_instruction) => stored_instruction,
        };
        if remaining_cycles > 1 {
            let new_remaining_cycles = remaining_cycles.checked_sub(1).ok_or_else(|| {
                CPUTickError::Internal(
                    "Could no reduce remaining cycles, which is probably an internal error."
                        .to_owned(),
                )
            })?;
            self.instruction_register = Some((current_instruction, new_remaining_cycles));
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
    #[error("An internal error occurred ({0}).")]
    Internal(String),
}

#[derive(Debug)]
struct Register {
    value: i64,
}

impl Register {
    fn new(value: i64) -> Self {
        Self { value }
    }
}

type CycleCallback = Box<dyn FnMut(&Cpu, u128) -> anyhow::Result<()>>;

struct ClockCircuit {
    cycles_completed: u128,
    cpu: Cpu,
    cycle_completed_callback: Option<CycleCallback>,
    during_cycle_callback: Option<CycleCallback>,
}

impl ClockCircuit {
    fn new(cpu: Cpu) -> Self {
        Self {
            cycles_completed: 0,
            cpu,
            cycle_completed_callback: None,
            during_cycle_callback: None,
        }
    }

    fn reset(&mut self) {
        self.cycles_completed = 0;
        self.cpu.reset();
    }

    #[allow(dead_code)]
    fn set_cycle_completed_callback<F>(&mut self, cycle_completed_callback: F)
    where
        F: FnMut(&Cpu, u128) -> anyhow::Result<()> + 'static,
    {
        self.cycle_completed_callback = Some(Box::new(cycle_completed_callback));
    }

    fn set_during_cycle_callback<F>(&mut self, during_cycle_callback: F)
    where
        F: FnMut(&Cpu, u128) -> anyhow::Result<()> + 'static,
    {
        self.during_cycle_callback = Some(Box::new(during_cycle_callback));
    }

    fn run(&mut self) -> anyhow::Result<()> {
        loop {
            let next_cycles_completed = self
                .cycles_completed
                .checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Reached maximum cycle count."))?;
            if self.cpu.has_instruction_left() {
                if let Some(ref mut during_cycle_callback) = self.during_cycle_callback {
                    (during_cycle_callback)(&self.cpu, next_cycles_completed)
                        .context("while calling during_cycle_callback")?;
                }
            }
            match self.cpu.tick() {
                Ok(_) => (),
                Err(CPUTickError::ProgramEnded) => break Ok(()),
                other => other.with_context(|| format!("after cycle {}", self.cycles_completed))?,
            }
            self.cycles_completed = next_cycles_completed;
            if let Some(ref mut cycle_completed_callback) = self.cycle_completed_callback {
                (cycle_completed_callback)(&self.cpu, self.cycles_completed)
                    .context("while calling cycle_completed_callback")?;
            }
        }
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct Crt<const COLUMNS: usize = 40, const ROWS: usize = 6> {
    buffer: [[Pixel; COLUMNS]; ROWS],
}

impl<const COLUMNS: usize, const ROWS: usize> Crt<COLUMNS, ROWS> {
    fn reset(&mut self) {
        self.buffer.iter_mut().for_each(|buffer_line| {
            buffer_line
                .iter_mut()
                .for_each(|pixel| *pixel = Pixel::Dark);
        });
    }

    fn process_signal(&mut self, signal: i64, during_cycle: u128) -> anyhow::Result<()> {
        let during_cycle_zero_based = usize::try_from(during_cycle)
            .with_context(|| format!("Provided during_cycle={during_cycle} is too big, because could not convert it from u128 to usize."))?
            .checked_sub(1)
            .ok_or_else(|| anyhow::anyhow!("Provided during_cycle={during_cycle} is probably 0, which is not allowed, because could not subtract 1 from it."))?;
        let row = during_cycle_zero_based
            .checked_div(COLUMNS)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Set COLUMNS={COLUMNS} might be 0, because could not divide with it."
                )
            })?;
        if row >= ROWS {
            Err(anyhow::anyhow!("Received signal with during_cycle={during_cycle} outside of row range (is: {row}, max: {ROWS})."))
        } else {
            let column = during_cycle_zero_based
                .checked_rem(COLUMNS)
                .ok_or_else(|| anyhow::anyhow!("Set COLUMNS={COLUMNS} might be 0, because could not calculate a remainder with it."))?;
            let column_signed = isize::try_from(column)
                .with_context(|| format!("Provided during_cycle={during_cycle} is too big, because could not convert column={column} from usize into isize."))?;

            let next_column = column_signed.checked_add(1)
                .ok_or_else(|| anyhow::anyhow!("Provided during_cycle={during_cycle} is too big, because could not add 1 to column_signed={column_signed} ."))?;
            let previous_column = column_signed.checked_sub(1)
                .ok_or_else(|| anyhow::anyhow!("Provided during_cycle={during_cycle} is too small, because could not subtract 1 from column_signed={column_signed} ."))?;

            let columns = isize::try_from(COLUMNS)
                .with_context(|| format!("Set COLUMNS={COLUMNS} is too big, because could not convert COLUMNS={COLUMNS} from usize to isize."))?;

            if next_column < 0 || previous_column >= columns {
                Err(anyhow::anyhow!("Received signal outside of column range (is: {column_signed}, max: {COLUMNS})."))
            } else {
                let signal_signed = isize::try_from(signal)
                    .with_context(|| format!("Provided signal={signal} is too big, because could not convert it from i64 to isize."))?;
                let bigger_signal_signed = signal_signed.checked_add(1)
                    .ok_or_else(|| anyhow::anyhow!("Provided signal={signal_signed} is too big, because could not add 1 to it."))?;
                let smaller_signal_signed = signal_signed.checked_sub(1)
                    .ok_or_else(|| anyhow::anyhow!("Provided signal={signal_signed} is too small, because could not subtract 1 from it."))?;

                if column_signed == bigger_signal_signed
                    || column_signed == signal_signed
                    || column_signed == smaller_signal_signed
                {
                    *self.buffer
                        .get_mut(row)
                        .and_then(|buffer_row| buffer_row.get_mut(column))
                        .ok_or_else(|| anyhow::anyhow!("Could not find pixel at column={column} and row={row}; might be an internal error."))? = Pixel::Lit;
                }
                Ok(())
            }
        }
    }
}

impl<const COLUMNS: usize, const ROWS: usize> Display for Crt<COLUMNS, ROWS> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (line, row) in self.buffer.iter().enumerate() {
            if line > 0 {
                writeln!(f)?;
            }
            for column in row {
                write!(f, "{column}")?;
            }
        }
        Ok(())
    }
}

impl<const COLUMNS: usize, const ROWS: usize> Default for Crt<COLUMNS, ROWS> {
    fn default() -> Self {
        Self {
            buffer: [[Pixel::Dark; COLUMNS]; ROWS],
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Pixel {
    Lit,
    Dark,
}

impl Display for Pixel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Pixel::Lit => write!(f, "#"),
            Pixel::Dark => write!(f, "."),
        }
    }
}

#[allow(clippy::panic_in_result_fn)]
#[allow(clippy::as_conversions)]
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
        // Arrange
        let program = Program::from_str(TEST_INPUT)?;
        let mut communication_device = CommunicationDevice::default();
        communication_device.set_default_program(program);

        // Act
        let signal_strength_sums = communication_device
            .calculate_sum_of_interesting_signal_strengths(1, vec![20, 60, 100, 140, 180, 220])?;

        // Assert
        assert_eq!(signal_strength_sums, 13_140);

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
        let mut cpu = Cpu::new_with_register_x_value(1);
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
        let mut new_cpu = Cpu::new_with_register_x_value(1);
        assert_eq!(new_cpu.x_register.value, 1);

        new_cpu.load(Program {
            instructions: VecDeque::from(vec![Instruction::NoOp, Instruction::AddX(3)]),
        });

        let cycle_history = Rc::new(RefCell::new(Vec::new()));
        let cycle_history_cloned = Rc::clone(&cycle_history);
        let mut clock_circuit = ClockCircuit::new(new_cpu);
        clock_circuit.set_cycle_completed_callback(move |cpu: &Cpu, cycles_completed: u128| {
            RefCell::borrow_mut(&cycle_history_cloned).push(cpu.x_register.value);
            assert_eq!(
                cycles_completed,
                RefCell::borrow(&cycle_history_cloned).len() as u128
            );
            Ok(())
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

    #[test]
    fn part_2_default() -> anyhow::Result<()> {
        // Arrange
        let program = Program::from_str(TEST_INPUT)?;
        let mut communication_device = CommunicationDevice::default();
        communication_device.set_default_program(program);

        // Act
        let produced_image = communication_device.calculate_crt_image(1)?;

        // Assert
        assert_eq!(
            produced_image,
            "##..##..##..##..##..##..##..##..##..##..
###...###...###...###...###...###...###.
####....####....####....####....####....
#####.....#####.....#####.....#####.....
######......######......######......####
#######.......#######.......#######....."
        );

        Ok(())
    }
}
