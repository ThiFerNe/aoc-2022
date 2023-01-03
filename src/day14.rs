use std::collections::HashSet;
use std::fmt::Display;
use std::io::Write;
use std::str::FromStr;

use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day14.input");

fn main() {
    // Part 1
    let part_1_solution = calculate_units_of_sand_staying_on_rocks(INPUT);
    println!("part_1_solution: {part_1_solution}");

    // Part 2
    let part_2_solution = calculate_units_of_sand_staying_on_rocks_with_infinite_rock_bottom(INPUT);
    println!("part_2_solution: {part_2_solution}");
}

fn calculate_units_of_sand_staying_on_rocks(input: &str) -> u64 {
    VerticalCaveSlice::parse_rock_scan(Position2D { x: 500, y: 0 }, input)
        .unwrap()
        .steps_till_full_of_sand()
        .still_sand
        .len() as u64
}

fn calculate_units_of_sand_staying_on_rocks_with_infinite_rock_bottom(input: &str) -> u64 {
    let m = VerticalCaveSlice::parse_rock_scan(Position2D { x: 500, y: 0 }, input)
        .unwrap()
        .with_infinite_rock_bottom()
        .steps_till_full_of_sand();
    std::fs::File::create("out")
        .unwrap()
        .write_all(m.to_string().as_bytes())
        .unwrap();
    m.still_sand.len() as u64
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct VerticalCaveSlice {
    sand_start: Position2D,
    rocks: HashSet<Position2D>,
    moving_sand: Option<Position2D>,
    still_sand: HashSet<Position2D>,
    infinite_rock_bottom_y: Option<i64>,
}

impl VerticalCaveSlice {
    fn parse_rock_scan(sand_start: Position2D, rock_scan: &str) -> anyhow::Result<Self> {
        Ok(Self {
            sand_start,
            rocks: rock_scan
                .lines()
                .flat_map(|line| {
                    line.split(" -> ")
                        .map(|corner| Position2D::from_str(corner).unwrap())
                        .tuple_windows::<(_, _)>()
                        .flat_map(|(from, to)| from.to(&to))
                })
                .unique()
                .collect::<HashSet<_>>(),
            moving_sand: None,
            still_sand: HashSet::new(),
            infinite_rock_bottom_y: None,
        })
    }

    fn rock_boundaries(&self) -> Option<Boundaries> {
        self.rocks.iter().fold(None, |i, a| match i {
            None => Some(Boundaries {
                top: a.y,
                bottom: a.y,
                left: a.x,
                right: a.x,
            }),
            Some(previous) => Some(previous.expand_by(a)),
        })
    }

    fn step(&self) -> Self {
        fn flow(from: &VerticalCaveSlice) -> VerticalCaveSlice {
            let boundaries = from.rock_boundaries().unwrap().expand_by(&from.sand_start);

            let mut to = from.clone();
            if let Some(previous_position) = to.moving_sand {
                let down = previous_position.down();
                let down_left = previous_position.down_left();
                let down_right = previous_position.down_right();

                let is_blocked =
                    |new: &Position2D| from.rocks.contains(new) || from.still_sand.contains(new);

                let (new_position, found_end) = if !is_blocked(&down) {
                    (down, false)
                } else if !is_blocked(&down_left) {
                    (down_left, false)
                } else if !is_blocked(&down_right) {
                    (down_right, false)
                } else {
                    (previous_position, true)
                };

                if found_end {
                    to.still_sand.insert(new_position);
                    to.moving_sand = None;
                } else {
                    match &from.infinite_rock_bottom_y {
                        Some(infinite_rock_bottom_y) => {
                            if new_position.y >= *infinite_rock_bottom_y {
                                to.still_sand.insert(previous_position);
                                to.moving_sand = None;
                            } else {
                                to.moving_sand = Some(new_position);
                            }
                        }
                        None => {
                            if boundaries.contains(&new_position) {
                                to.moving_sand = Some(new_position);
                            } else {
                                to.moving_sand = None;
                            }
                        }
                    }
                }
            } else {
                to.moving_sand = Some(to.sand_start);
            }

            return to;
        }

        return flow(self);
    }

    fn steps_till_sand_resting(&self) -> Self {
        let mut steps = 0;
        let started_with_filled_sand_start = self.still_sand.contains(&self.sand_start)
            || match self.moving_sand {
                Some(moving_sand) => moving_sand == self.sand_start,
                None => false,
            };

        let mut current = self.clone();
        loop {
            let next = current.step();
            steps += 1;

            if next.moving_sand.is_none() {
                return next;
            }

            let created_new_sand = current.moving_sand.is_none() && next.moving_sand.is_some();
            if created_new_sand {
                if started_with_filled_sand_start {
                    return current;
                } else if steps > 1 {
                    return current;
                }
            }
            current = next;
        }
    }

    fn steps_till_full_of_sand(&self) -> Self {
        let mut current = self.clone();
        let mut last = std::time::Instant::now();
        loop {
            let next = current.steps_till_sand_resting();
            if last.elapsed().as_secs_f64() > 1f64 || next.still_sand.len() % 2000 == 0 {
                println!("{} Sand", next.still_sand.len());
            }
            last = std::time::Instant::now();
            if current == next {
                return next;
            }
            current = next;
        }
    }

    fn with_infinite_rock_bottom(mut self) -> Self {
        self.infinite_rock_bottom_y = Some(self.rock_boundaries().unwrap().bottom + 2);
        self
    }
}

impl Display for VerticalCaveSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut boundaries = self.rock_boundaries().unwrap().expand_by(&self.sand_start);
        for sands in &self.still_sand {
            boundaries = boundaries.expand_by(sands);
        }
        let Boundaries {
            top,
            bottom,
            left,
            right,
        } = boundaries;

        // --- HORIZONTAL AXIS NUMBERS ---
        let mut horizontal_numbers_i64 = vec![left, 500, right];
        horizontal_numbers_i64.sort();
        let mut horizontal_numbers = horizontal_numbers_i64
            .iter()
            .map(|l| l.to_string())
            .collect::<Vec<_>>();

        let mut horizontal_start_lines = Vec::new();
        while horizontal_numbers.iter().map(String::len).max().unwrap() > 0 {
            let line = horizontal_numbers
                .iter_mut()
                .map(String::pop)
                .map(|next_char| next_char.unwrap_or(' '))
                .collect::<String>();
            horizontal_start_lines.insert(0, line);
        }

        for horizontal_start_line in horizontal_start_lines {
            writeln!(f)?;
            write!(f, "  ")?;
            for column in left..=right {
                if let Some((horizontal_start_line_index, _)) = horizontal_numbers_i64
                    .iter()
                    .find_position(|number| **number == column)
                {
                    write!(
                        f,
                        "{}",
                        horizontal_start_line
                            .chars()
                            .skip(horizontal_start_line_index)
                            .take(1)
                            .collect::<String>()
                    )?;
                } else {
                    write!(f, " ")?;
                }
            }
        }

        // --- LINES ---
        let bottom = match &self.infinite_rock_bottom_y {
            Some(infinite_rock_bottom_y) => *infinite_rock_bottom_y,
            None => bottom,
        };

        let top_digits = top.to_string().len();
        let bottom_digits = bottom.to_string().len();
        let max_length_y_axis_digits = bottom_digits.max(top_digits);

        for row in top..=bottom {
            writeln!(f)?;
            write!(f, "{row:>width$} ", width = max_length_y_axis_digits)?;
            for column in left..=right {
                if let Some(_) = self.infinite_rock_bottom_y.filter(|y| *y == row) {
                    write!(f, "#")?;
                } else {
                    let position = Position2D { x: column, y: row };
                    if self.rocks.contains(&position) {
                        write!(f, "#")?;
                    } else if self.still_sand.contains(&position) {
                        write!(f, "o")?;
                    } else if self.sand_start == position {
                        write!(f, "+")?;
                    } else {
                        write!(f, ".")?;
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash, Ord, PartialOrd)]
struct Position2D {
    x: i64,
    y: i64,
}

impl Position2D {
    fn to(&self, other: &Self) -> Vec<Self> {
        (self.x.min(other.x)..=self.x.max(other.x))
            .flat_map(|x| (self.y.min(other.y)..=self.y.max(other.y)).map(move |y| Self { x, y }))
            .collect::<Vec<_>>()
    }

    fn down(&self) -> Self {
        Self {
            x: self.x,
            y: self.y + 1,
        }
    }

    fn down_left(&self) -> Self {
        Self {
            x: self.x - 1,
            y: self.y + 1,
        }
    }

    fn down_right(&self) -> Self {
        Self {
            x: self.x + 1,
            y: self.y + 1,
        }
    }
}

impl FromStr for Position2D {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let [x, y]: [i64; 2] = s
            .split(',')
            .map(|element| element.parse().unwrap())
            .collect::<Vec<_>>()
            .try_into()
            .unwrap();
        Ok(Self { x, y })
    }
}

struct Boundaries {
    top: i64,
    bottom: i64,
    left: i64,
    right: i64,
}

impl Boundaries {
    fn expand_by(&self, position: &Position2D) -> Self {
        Self {
            top: self.top.min(position.y),
            bottom: self.bottom.max(position.y),
            left: self.left.min(position.x),
            right: self.right.max(position.x),
        }
    }

    fn contains(&self, position: &Position2D) -> bool {
        position.x >= self.left
            && position.x <= self.right
            && position.y >= self.top
            && position.y <= self.bottom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";

    #[test]
    fn test_part_1_default() {
        // Act
        let units_of_sand = calculate_units_of_sand_staying_on_rocks(TEST_INPUT);

        // Assert
        assert_eq!(units_of_sand, 24);
    }

    #[test]
    fn test_part_2_default() {
        // Act
        let units_of_sand =
            calculate_units_of_sand_staying_on_rocks_with_infinite_rock_bottom(TEST_INPUT);

        // Assert
        assert_eq!(units_of_sand, 93);
    }

    #[test]
    fn test_vertical_cave_slice_parse_rock_scan() {
        // Act
        let vertical_cave_slice =
            VerticalCaveSlice::parse_rock_scan(Position2D { x: 500, y: 0 }, TEST_INPUT).unwrap();

        // Assert
        assert_eq!(
            vertical_cave_slice,
            VerticalCaveSlice {
                sand_start: Position2D { x: 500, y: 0 },
                rocks: HashSet::from([
                    // 498,4 -> 498,6
                    Position2D { x: 498, y: 4 },
                    Position2D { x: 498, y: 5 },
                    Position2D { x: 498, y: 6 },
                    // (498,6) -> 496,6
                    Position2D { x: 496, y: 6 },
                    Position2D { x: 497, y: 6 },
                    // 503,4 -> 502,4
                    Position2D { x: 502, y: 4 },
                    Position2D { x: 503, y: 4 },
                    // (502,4) -> 502,9
                    Position2D { x: 502, y: 5 },
                    Position2D { x: 502, y: 6 },
                    Position2D { x: 502, y: 7 },
                    Position2D { x: 502, y: 8 },
                    Position2D { x: 502, y: 9 },
                    // (502,9) -> 494,9
                    Position2D { x: 494, y: 9 },
                    Position2D { x: 495, y: 9 },
                    Position2D { x: 496, y: 9 },
                    Position2D { x: 497, y: 9 },
                    Position2D { x: 498, y: 9 },
                    Position2D { x: 499, y: 9 },
                    Position2D { x: 500, y: 9 },
                    Position2D { x: 501, y: 9 },
                ]),
                moving_sand: None,
                still_sand: HashSet::new(),
                infinite_rock_bottom_y: None,
            }
        );
    }

    #[test]
    fn test_vertical_cave_slice_to_string() {
        // Arrange
        let vertical_cave_slice =
            VerticalCaveSlice::parse_rock_scan(Position2D { x: 500, y: 0 }, TEST_INPUT).unwrap();

        // Act
        let vertical_cave_slice_string = vertical_cave_slice.to_string();

        // Assert
        assert_eq!(
            vertical_cave_slice_string,
            "
  4     5  5
  9     0  0
  4     0  3
0 ......+...
1 ..........
2 ..........
3 ..........
4 ....#...##
5 ....#...#.
6 ..###...#.
7 ........#.
8 ........#.
9 #########."
        );
    }

    #[test]
    fn test_position2d_to() {
        // Arrange
        let from_a = Position2D { x: 498, y: 4 };
        let to_a = Position2D { x: 498, y: 6 };

        let from_b = Position2D { x: 498, y: 6 };
        let to_b = Position2D { x: 496, y: 6 };

        // Act
        let range_a = from_a.to(&to_a);
        let range_b = from_b.to(&to_b);

        // Assert
        assert_eq!(
            range_a,
            vec![
                Position2D { x: 498, y: 4 },
                Position2D { x: 498, y: 5 },
                Position2D { x: 498, y: 6 },
            ]
        );
        assert_eq!(
            range_b,
            vec![
                Position2D { x: 496, y: 6 },
                Position2D { x: 497, y: 6 },
                Position2D { x: 498, y: 6 },
            ]
        );
    }

    #[test]
    fn test_vertical_cave_slice_step() {
        // Arrange
        let vertical_cave_slice_0 =
            VerticalCaveSlice::parse_rock_scan(Position2D { x: 500, y: 0 }, TEST_INPUT).unwrap();
        assert_eq!(vertical_cave_slice_0.still_sand, HashSet::new());

        // Act 1
        let vertical_cave_slice_1 = vertical_cave_slice_0.step();

        // Assert 1
        assert_eq!(vertical_cave_slice_0.rocks, vertical_cave_slice_1.rocks);
        assert_eq!(
            vertical_cave_slice_0.sand_start,
            vertical_cave_slice_1.sand_start
        );

        assert_eq!(
            vertical_cave_slice_1.moving_sand,
            Some(Position2D { x: 500, y: 0 })
        );
        assert_eq!(vertical_cave_slice_1.still_sand, HashSet::new(),);

        // Act 2
        let vertical_cave_slice_2 = vertical_cave_slice_1.step();

        // Assert 2
        assert_eq!(vertical_cave_slice_0.rocks, vertical_cave_slice_2.rocks);
        assert_eq!(
            vertical_cave_slice_0.sand_start,
            vertical_cave_slice_2.sand_start
        );

        assert_eq!(
            vertical_cave_slice_2.moving_sand,
            Some(Position2D { x: 500, y: 1 })
        );
        assert_eq!(vertical_cave_slice_2.still_sand, HashSet::new(),);
    }

    #[test]
    fn test_vertical_cave_slice_steps_till_sand_resting() {
        // Arrange
        let vertical_cave_slice_0 =
            VerticalCaveSlice::parse_rock_scan(Position2D { x: 500, y: 0 }, TEST_INPUT).unwrap();

        // Act
        let vertical_cave_slice_1 = vertical_cave_slice_0.steps_till_sand_resting();

        // Assert
        assert_eq!(vertical_cave_slice_0.rocks, vertical_cave_slice_1.rocks);
        assert_eq!(
            vertical_cave_slice_0.sand_start,
            vertical_cave_slice_1.sand_start
        );

        assert_eq!(
            vertical_cave_slice_1.still_sand,
            HashSet::from([Position2D { x: 500, y: 8 }])
        );
    }

    #[test]
    fn test_vertical_cave_slice_steps_till_sand_resting_when_falling_into_bottom() {
        // Arrange
        let vertical_cave_slice =
            VerticalCaveSlice::parse_rock_scan(Position2D { x: 200, y: 0 }, TEST_INPUT).unwrap();

        // Act
        let new_vertical_cave_slice = vertical_cave_slice.steps_till_sand_resting();

        // Assert
        assert_eq!(new_vertical_cave_slice, vertical_cave_slice);
    }
}
