use std::str::FromStr;

use anyhow::Context;

use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day09.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 1 hour 21 minutes 33 seconds
    let motion_series = MotionSeries::from_str(INPUT)?;
    let rope_states = RopeState::default().apply_motion_series_return_with_you(&motion_series);
    let part_1_solution = count_unique_visited_tail_positions(&rope_states);
    println!("part_1_solution: {part_1_solution}");

    Ok(())
}

fn count_unique_visited_tail_positions(rope_states: &[RopeState]) -> usize {
    rope_states
        .iter()
        .map(|rope_state| rope_state.tail_position)
        .unique()
        .count()
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct RopeState {
    head_position: Position2D,
    tail_position: Position2D,
}

impl RopeState {
    fn apply_motion(&self, motion: &Motion) -> Vec<Self> {
        let mut output = Vec::with_capacity(usize::try_from(motion.steps()).unwrap());
        let mut current = *self;
        for _ in 0..motion.steps() {
            let new_head_position = motion.apply_one(&current.head_position);
            let new_tail_position =
                Self::move_tail_one_closer_to_head(&new_head_position, &current.tail_position);
            let new = Self {
                head_position: new_head_position,
                tail_position: new_tail_position,
            };
            output.push(new);
            current = new;
        }
        output
    }

    fn move_tail_one_closer_to_head(
        head_position: &Position2D,
        tail_position: &Position2D,
    ) -> Position2D {
        let vector = tail_position.vector_to(head_position);
        if vector.x == 0 && vector.y.abs() > 1 {
            Position2D {
                x: tail_position.x,
                y: tail_position.y + vector.y.signum(),
            }
        } else if vector.x.abs() > 1 && vector.y == 0 {
            Position2D {
                x: tail_position.x + vector.x.signum(),
                y: tail_position.y,
            }
        } else if vector.x.abs() > 1 || vector.y.abs() > 1 {
            Position2D {
                x: tail_position.x + vector.x.signum(),
                y: tail_position.y + vector.y.signum(),
            }
        } else {
            *tail_position
        }
    }

    fn apply_motion_series_return_with_you(&self, motion_series: &MotionSeries) -> Vec<Self> {
        let mut output = Vec::with_capacity(motion_series.0.len() + 1);
        output.push(*self);
        let mut current = *self;
        for motion in &motion_series.0 {
            let rope_states = current.apply_motion(motion);
            output.extend_from_slice(&rope_states);
            current = *rope_states.last().unwrap();
        }
        output
    }
}

impl Default for RopeState {
    fn default() -> Self {
        Self {
            head_position: Position2D::zero(),
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

    fn vector_to(&self, other: &Self) -> Vector2D {
        Vector2D {
            x: other.x - self.x,
            y: other.y - self.y,
        }
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
        match self {
            Motion::Right(steps) => *steps,
            Motion::Left(steps) => *steps,
            Motion::Up(steps) => *steps,
            Motion::Down(steps) => *steps,
        }
    }

    fn apply_one(&self, position: &Position2D) -> Position2D {
        match self {
            Motion::Right(_) => Position2D {
                x: position.x + 1,
                y: position.y,
            },
            Motion::Left(_) => Position2D {
                x: position.x - 1,
                y: position.y,
            },
            Motion::Up(_) => Position2D {
                x: position.x,
                y: position.y + 1,
            },
            Motion::Down(_) => Position2D {
                x: position.x,
                y: position.y - 1,
            },
        }
    }
}

impl FromStr for Motion {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: [&str; 2] =
            s.split(" ")
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
        let rope_states = RopeState::default().apply_motion_series_return_with_you(&motion_series);
        let count_of_unique_visited_tail_positions =
            count_unique_visited_tail_positions(&rope_states);

        // Assert
        assert_eq!(count_of_unique_visited_tail_positions, 13);

        Ok(())
    }

    #[test]
    fn test_motion_series_from_str() -> anyhow::Result<()> {
        use Motion::*;

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
    fn test_rope_state_apply_motion() {
        // Arrange
        let initial_state = RopeState {
            head_position: Position2D { x: 0, y: 0 },
            tail_position: Position2D { x: 0, y: 0 },
        };

        // Act
        let rope_states_1 = initial_state.apply_motion(&Motion::Right(4));

        // Assert
        assert_eq!(
            rope_states_1,
            vec![
                RopeState {
                    head_position: Position2D { x: 1, y: 0 },
                    tail_position: Position2D { x: 0, y: 0 }
                },
                RopeState {
                    head_position: Position2D { x: 2, y: 0 },
                    tail_position: Position2D { x: 1, y: 0 }
                },
                RopeState {
                    head_position: Position2D { x: 3, y: 0 },
                    tail_position: Position2D { x: 2, y: 0 }
                },
                RopeState {
                    head_position: Position2D { x: 4, y: 0 },
                    tail_position: Position2D { x: 3, y: 0 }
                }
            ]
        );
    }

    #[test]
    fn test_rope_state_move_tail_one_closer_to_head() {
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

        for ((head_pos, tail_pos), target_tail_pos) in tests {
            // Act
            let new_tail_pos = RopeState::move_tail_one_closer_to_head(&head_pos, &tail_pos);

            // Assert
            assert_eq!(
                new_tail_pos, target_tail_pos,
                "during head_pos={:?} and tail_pos={:?}",
                head_pos, tail_pos
            );
        }
    }
}
