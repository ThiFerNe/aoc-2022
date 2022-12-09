use std::str::FromStr;

use anyhow::Context;

use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day09.input");

fn main() -> anyhow::Result<()> {
    let motion_series = MotionSeries::from_str(INPUT)?;

    // PART 1 - 1 hour 21 minutes 33 seconds
    let rope_states_1 = RopeState::<0>::default()
        .apply_motion_series_return_with_you(&motion_series)
        .context("while applying motion series to 2 knots rope.")?;
    let part_1_solution = count_unique_visited_tail_positions(&rope_states_1);
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 17 minutes 54 seconds
    let rope_states_2 = RopeState::<8>::default()
        .apply_motion_series_return_with_you(&motion_series)
        .context("while applying motion series to 10 knots rope.")?;
    let part_2_solution = count_unique_visited_tail_positions(&rope_states_2);
    println!("part_2_solution: {part_2_solution}");

    Ok(())
}

fn count_unique_visited_tail_positions<const ADDITIONAL: usize>(
    rope_states: &[RopeState<ADDITIONAL>],
) -> usize {
    rope_states
        .iter()
        .map(|rope_state| rope_state.tail_position)
        .unique()
        .count()
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct RopeState<const ADDITIONAL_KNOTS: usize = 0> {
    head_position: Position2D,
    between_positions: [Position2D; ADDITIONAL_KNOTS],
    tail_position: Position2D,
}

impl<const ADDITIONAL_KNOTS: usize> RopeState<ADDITIONAL_KNOTS> {
    fn apply_motion(&self, motion: &Motion) -> anyhow::Result<Vec<Self>> {
        let capacity = usize::try_from(motion.steps()).with_context(|| {
            format!(
                "while creating output vector, motion has too many steps ({})",
                motion.steps()
            )
        })?;
        let mut output = Vec::with_capacity(capacity);

        let mut current = *self;
        for _ in 0..motion.steps() {
            let new_head_position =
                motion.apply_one(&current.head_position).with_context(|| {
                    format!(
                        "while applying one {motion:?} to {:?}",
                        current.head_position
                    )
                })?;

            let mut new_between_positions: [Position2D; ADDITIONAL_KNOTS] =
                current.between_positions;
            for (index, current_position) in current.between_positions.iter().enumerate() {
                let prior_position: Position2D = *index
                    .checked_sub(1)
                    .and_then(|prior_index| new_between_positions.get(prior_index))
                    .unwrap_or(&new_head_position);
                let new_between_position: &mut Position2D = new_between_positions
                    .get_mut(index)
                    .ok_or_else(|| anyhow::anyhow!("Could not index into array with length {ADDITIONAL_KNOTS} with index {index}."))?;
                *new_between_position = move_b_one_closer_to_a(&prior_position, current_position)
                    .with_context(|| {
                    format!("while moving {prior_position:?} closer to {current_position:?}")
                })?;
            }

            let prior_position = new_between_positions.last().unwrap_or(&new_head_position);
            let new_tail_position = move_b_one_closer_to_a(prior_position, &current.tail_position)
                .with_context(|| {
                    format!(
                        "while moving prior {prior_position:?} closer to tail {:?}",
                        current.tail_position
                    )
                })?;

            let new_rope_state = Self {
                head_position: new_head_position,
                between_positions: new_between_positions,
                tail_position: new_tail_position,
            };

            output.push(new_rope_state);
            current = new_rope_state;
        }
        Ok(output)
    }

    fn apply_motion_series_return_with_you(
        &self,
        motion_series: &MotionSeries,
    ) -> anyhow::Result<Vec<Self>> {
        let capacity = motion_series.0.len().checked_add(1).ok_or_else(|| {
            anyhow::anyhow!(
                "Could not create output vector, there are too many motions (1 + {})",
                motion_series.0.len()
            )
        })?;
        let mut output = Vec::with_capacity(capacity);

        output.push(*self);

        let mut current = *self;
        for motion in &motion_series.0 {
            let rope_states = current
                .apply_motion(motion)
                .with_context(|| format!("while applying {motion:?} to {current:?}"))?;
            output.extend_from_slice(&rope_states);
            current = *rope_states
                .last()
                .ok_or_else(|| anyhow::anyhow!("Returned rope states after {motion:?} has been applied to {current:?} are empty."))?;
        }
        Ok(output)
    }
}

fn move_b_one_closer_to_a(
    position_a: &Position2D,
    position_b: &Position2D,
) -> anyhow::Result<Position2D> {
    let vector = position_b.vector_to(position_a).with_context(|| {
        anyhow::anyhow!("while calculating vector from {position_b:?} to {position_a:?}")
    })?;
    if vector.x == 0 && vector.y.abs() > 1 {
        Ok(Position2D {
            x: position_b.x,
            y: position_b.y.checked_add(vector.y.signum()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not add {} to {} to Y.",
                    vector.y.signum(),
                    position_b.y
                )
            })?,
        })
    } else if vector.x.abs() > 1 && vector.y == 0 {
        Ok(Position2D {
            x: position_b.x.checked_add(vector.x.signum()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not add {} to {} to X.",
                    vector.x.signum(),
                    position_b.x
                )
            })?,
            y: position_b.y,
        })
    } else if vector.x.abs() > 1 || vector.y.abs() > 1 {
        Ok(Position2D {
            x: position_b.x.checked_add(vector.x.signum()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not add {} to {} to X.",
                    vector.x.signum(),
                    position_b.x
                )
            })?,
            y: position_b.y.checked_add(vector.y.signum()).ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not add {} to {} to Y.",
                    vector.y.signum(),
                    position_b.y
                )
            })?,
        })
    } else {
        Ok(*position_b)
    }
}

impl<const ADDITIONAL: usize> Default for RopeState<ADDITIONAL> {
    fn default() -> Self {
        Self {
            head_position: Position2D::zero(),
            between_positions: [Position2D::zero(); ADDITIONAL],
            tail_position: Position2D::zero(),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Hash)]
struct Position2D {
    x: i64,
    y: i64,
}

impl Position2D {
    fn zero() -> Self {
        Self { x: 0, y: 0 }
    }

    fn vector_to(&self, other: &Self) -> anyhow::Result<Vector2D> {
        Ok(Vector2D {
            x: other
                .x
                .checked_sub(self.x)
                .ok_or_else(|| anyhow::anyhow!("X distance of this to other is too big."))?,
            y: other
                .y
                .checked_sub(self.y)
                .ok_or_else(|| anyhow::anyhow!("Y distance of this to other is too big."))?,
        })
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Vector2D {
    x: i64,
    y: i64,
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct MotionSeries(Vec<Motion>);

impl FromStr for MotionSeries {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(
            s.lines()
                .enumerate()
                .map(|(index, line)| {
                    Motion::from_str(line)
                        .with_context(|| anyhow::anyhow!("while parsing motion line #{index}"))
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Motion {
    Right(u64),
    Left(u64),
    Up(u64),
    Down(u64),
}

impl Motion {
    fn steps(&self) -> u64 {
        match *self {
            Motion::Right(steps)
            | Motion::Left(steps)
            | Motion::Up(steps)
            | Motion::Down(steps) => steps,
        }
    }

    fn apply_one(&self, position: &Position2D) -> anyhow::Result<Position2D> {
        let error = || {
            anyhow::anyhow!("Cannot apply motion to this position, because afterwards it would be out of bounds.")
        };
        match *self {
            Motion::Right(_) => Ok(Position2D {
                x: position.x.checked_add(1).ok_or_else(error)?,
                y: position.y,
            }),
            Motion::Left(_) => Ok(Position2D {
                x: position.x.checked_sub(1).ok_or_else(error)?,
                y: position.y,
            }),
            Motion::Up(_) => Ok(Position2D {
                x: position.x,
                y: position.y.checked_add(1).ok_or_else(error)?,
            }),
            Motion::Down(_) => Ok(Position2D {
                x: position.x,
                y: position.y.checked_sub(1).ok_or_else(error)?,
            }),
        }
    }
}

impl FromStr for Motion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: [&str; 2] =
            s.split(' ')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|vec: Vec<_>| {
                    anyhow::anyhow!("Input has not 2 but {} parts ({:?}).", vec.len(), vec)
                })?;
        let amount = parts[1]
            .parse()
            .with_context(|| format!("while parsing motion amount \"{}\"", parts[1]))?;
        match parts[0].to_lowercase().as_str() {
            "r" => Ok(Self::Right(amount)),
            "l" => Ok(Self::Left(amount)),
            "u" => Ok(Self::Up(amount)),
            "d" => Ok(Self::Down(amount)),
            _ => Err(anyhow::anyhow!("Invalid input.")),
        }
    }
}

#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "R 4
U 4
L 3
D 1
R 4
D 1
L 5
R 2";

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Act
        let motion_series = MotionSeries::from_str(TEST_INPUT)?;
        let rope_states = RopeState::<0>::default()
            .apply_motion_series_return_with_you(&motion_series)
            .context("while applying motion series to two knot rope")?;
        let count_of_unique_visited_tail_positions =
            count_unique_visited_tail_positions(&rope_states);

        // Assert
        assert_eq!(count_of_unique_visited_tail_positions, 13);

        Ok(())
    }

    #[test]
    fn test_motion_series_from_str() -> anyhow::Result<()> {
        use Motion::{Down, Left, Right, Up};

        // Act
        let motion_series = MotionSeries::from_str(TEST_INPUT)?;

        // Assert
        assert_eq!(
            motion_series,
            MotionSeries(vec![
                Right(4),
                Up(4),
                Left(3),
                Down(1),
                Right(4),
                Down(1),
                Left(5),
                Right(2),
            ])
        );

        Ok(())
    }

    #[test]
    fn test_rope_state_apply_motion() -> anyhow::Result<()> {
        // Arrange
        let initial_state = RopeState {
            head_position: Position2D { x: 0, y: 0 },
            between_positions: [],
            tail_position: Position2D { x: 0, y: 0 },
        };

        // Act
        let rope_states_1 = initial_state
            .apply_motion(&Motion::Right(4))
            .context("while applying 4 Right motions")?;

        // Assert
        assert_eq!(
            rope_states_1,
            vec![
                RopeState {
                    head_position: Position2D { x: 1, y: 0 },
                    between_positions: [],
                    tail_position: Position2D { x: 0, y: 0 }
                },
                RopeState {
                    head_position: Position2D { x: 2, y: 0 },
                    between_positions: [],
                    tail_position: Position2D { x: 1, y: 0 }
                },
                RopeState {
                    head_position: Position2D { x: 3, y: 0 },
                    between_positions: [],
                    tail_position: Position2D { x: 2, y: 0 }
                },
                RopeState {
                    head_position: Position2D { x: 4, y: 0 },
                    between_positions: [],
                    tail_position: Position2D { x: 3, y: 0 }
                }
            ]
        );

        Ok(())
    }

    #[test]
    fn test_rope_state_move_b_one_closer_to_a() -> anyhow::Result<()> {
        // Arrange
        #[rustfmt::skip]
        let tests = vec![
            ((Position2D { x: 2, y: 1 }, Position2D { x: 1, y: 1 }), Position2D { x: 1, y: 1 }), // don't move
            ((Position2D { x: 3, y: 1 }, Position2D { x: 1, y: 1 }), Position2D { x: 2, y: 1 }), // on same
            ((Position2D { x: 3, y: 1 }, Position2D { x: 1, y: 1 }), Position2D { x: 2, y: 1 }), // one horizontal
            ((Position2D { x: 1, y: 1 }, Position2D { x: 1, y: 3 }), Position2D { x: 1, y: 2 }), // one vertical
            ((Position2D { x: 2, y: 3 }, Position2D { x: 1, y: 1 }), Position2D { x: 2, y: 2 }), // one diagonal
            ((Position2D { x: 3, y: 2 }, Position2D { x: 1, y: 1 }), Position2D { x: 2, y: 2 }), // one diagonal
        ];

        for ((position_a, position_b), target_position_b) in tests {
            // Act
            let new_b_position =
                move_b_one_closer_to_a(&position_a, &position_b).with_context(|| {
                    format!("while moving {position_b:?} one closer to {position_a:?}")
                })?;

            // Assert
            assert_eq!(
                new_b_position, target_position_b,
                "during position_a={:?} and position_b={:?}",
                position_a, position_b
            );
        }

        Ok(())
    }
}
