use itertools::Itertools;
use std::fmt::Display;

const INPUT: &str = include_str!("../inputs/day14.input");

fn main() {
    // Part 1
    let start = std::time::Instant::now();
    let part_1_solution = part_1(INPUT);
    let end = std::time::Instant::now();
    println!("part_1_solution: {part_1_solution} ({:?})", end - start);

    // Part 2
    let start = std::time::Instant::now();
    let part_2_solution = part_2(INPUT);
    let end = std::time::Instant::now();
    println!("part_2_solution: {part_2_solution} ({:?})", end - start);
}

fn part_1(input: &str) -> u64 {
    part_n(input, false)
}

fn part_2(input: &str) -> u64 {
    part_n(input, true)
}

fn part_n(input: &str, with_rock_bottom: bool) -> u64 {
    let mut slice = VerticalCaveSlice::parse_rock_structure(
        input,
        Position2D { x: 500, y: 0 },
        with_rock_bottom,
    );
    loop {
        let sand_result = slice.tick();
        if matches!(
            sand_result,
            SandResult::Vanished | SandResult::CreationBlocked
        ) {
            break;
        }
    }
    slice.count_sand()
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct VerticalCaveSlice {
    sand_start: Position2D,
    slice_boundaries: Boundaries,
    slice: Vec<Vec<Element>>,
    active_sand: Option<Position2D>,
}

impl VerticalCaveSlice {
    fn parse_rock_structure(
        rock_structure: &str,
        sand_start: Position2D,
        with_rock_bottom: bool,
    ) -> Self {
        let rock_structure: Vec<Vec<Position2D>> = rock_structure
            .lines()
            .map(|line| {
                line.split(" -> ")
                    .map(|element| {
                        let [x, y]: [i64; 2] = element
                            .split(',')
                            .map(|element| element.parse().unwrap())
                            .collect::<Vec<_>>()
                            .try_into()
                            .unwrap();
                        Position2D { x, y }
                    })
                    .collect::<Vec<_>>()
            })
            .collect();

        let mut boundaries = Boundaries::from(sand_start);
        rock_structure
            .iter()
            .flatten()
            .for_each(|point| boundaries = boundaries.expand_to(*point));

        if with_rock_bottom {
            boundaries.bottom += 2;
        }

        let slice_height = usize::try_from(boundaries.bottom - boundaries.top + 1).unwrap();
        let slice_width = usize::try_from(1 + 2 * slice_height).unwrap();
        let slice = vec![vec![Element::Air; slice_width]; slice_height];
        let slice_boundaries = Boundaries {
            top: boundaries.top,
            bottom: boundaries.bottom,
            left: sand_start.x - slice_height as i64,
            right: sand_start.x + slice_height as i64,
        };

        let mut output = Self {
            sand_start,
            slice_boundaries,
            slice,
            active_sand: None,
        };

        rock_structure.iter().for_each(|rock_line| {
            rock_line
                .iter()
                .tuple_windows::<(_, _)>()
                .into_iter()
                .for_each(|(start_rock, end_rock)| {
                    start_rock
                        .to(end_rock)
                        .iter()
                        .for_each(|position| output.set_element(position, Element::Rock))
                })
        });

        if with_rock_bottom {
            for x in output.slice_boundaries.left..=output.slice_boundaries.right {
                output.set_element(
                    &Position2D {
                        x,
                        y: output.slice_boundaries.bottom,
                    },
                    Element::Rock,
                );
            }
        }

        return output;
    }

    fn tick(&mut self) -> SandResult {
        let is_blocked = |position: &Position2D| {
            matches!(self.get_element(position), Element::Sand | Element::Rock)
        };
        if let Some(active_sand) = self.active_sand {
            if active_sand.y + 1 > self.slice_boundaries.bottom {
                self.active_sand = None;
                SandResult::Vanished
            } else if !is_blocked(&active_sand.down()) {
                self.active_sand = Some(active_sand.down());
                SandResult::Moved
            } else if !is_blocked(&active_sand.down_left()) {
                self.active_sand = Some(active_sand.down_left());
                SandResult::Moved
            } else if !is_blocked(&active_sand.down_right()) {
                self.active_sand = Some(active_sand.down_right());
                SandResult::Moved
            } else {
                self.active_sand = None;
                self.set_element(&active_sand, Element::Sand);
                SandResult::Hardened
            }
        } else {
            if is_blocked(&self.sand_start) {
                SandResult::CreationBlocked
            } else {
                self.active_sand = Some(self.sand_start);
                SandResult::Created
            }
        }
    }

    fn set_element(&mut self, position: &Position2D, element: Element) {
        let SliceIndices { y, x } = self.slice_boundaries.calculate_slice_indices(position);
        self.slice[y][x] = element;
    }

    fn get_element(&self, position: &Position2D) -> &Element {
        let SliceIndices { y, x } = self.slice_boundaries.calculate_slice_indices(position);
        return &self.slice[y][x];
    }

    fn count_sand(&self) -> u64 {
        self.slice
            .iter()
            .flatten()
            .filter(|element| matches!(element, Element::Sand))
            .count() as u64
    }
}

impl Display for VerticalCaveSlice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for row in &self.slice {
            for column in row {
                match column {
                    Element::Air => write!(f, ".")?,
                    Element::Rock => write!(f, "#")?,
                    Element::Sand => write!(f, "o")?,
                }
            }
            writeln!(f)?;
        }
        Ok(())
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum SandResult {
    CreationBlocked,
    Created,
    Hardened,
    Moved,
    Vanished,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum Element {
    Air,
    Rock,
    Sand,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Position2D {
    x: i64,
    y: i64,
}

impl Position2D {
    fn to(&self, other: &Self) -> Vec<Self> {
        (self.x.min(other.x)..=self.x.max(other.x))
            .flat_map(|x| (self.y.min(other.y)..=self.y.max(other.y)).map(move |y| Self { x, y }))
            .collect()
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
struct Boundaries {
    top: i64,
    bottom: i64,
    left: i64,
    right: i64,
}

impl Boundaries {
    fn expand_to(&self, position: Position2D) -> Self {
        Self {
            top: self.top.min(position.y),
            bottom: self.bottom.max(position.y),
            left: self.left.min(position.x),
            right: self.right.max(position.x),
        }
    }

    fn calculate_slice_indices(&self, position: &Position2D) -> SliceIndices {
        let x = usize::try_from(position.x - self.left).unwrap();
        let y = usize::try_from(position.y - self.top).unwrap();
        SliceIndices { y, x }
    }
}

impl From<Position2D> for Boundaries {
    fn from(value: Position2D) -> Self {
        Self {
            top: value.y,
            bottom: value.y,
            left: value.x,
            right: value.x,
        }
    }
}

struct SliceIndices {
    y: usize,
    x: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "498,4 -> 498,6 -> 496,6
503,4 -> 502,4 -> 502,9 -> 494,9";

    #[test]
    fn test_part_1_default() {
        // Act
        let part_1_solution = part_1(TEST_INPUT);

        // Assert
        assert_eq!(part_1_solution, 24);
    }

    #[test]
    fn test_part_2_default() {
        // Act
        let part_2_solution = part_2(TEST_INPUT);

        // Assert
        assert_eq!(part_2_solution, 93);
    }

    #[test]
    fn test_vertical_cave_slice_from_str() {
        // Act
        let vertical_cave_slice =
            VerticalCaveSlice::parse_rock_structure(TEST_INPUT, Position2D { x: 500, y: 0 }, false);

        // Assert
        use Element::{Air, Rock};
        #[rustfmt::skip]
        let target = VerticalCaveSlice {
            sand_start: Position2D { x: 500, y: 0 },
            slice_boundaries: Boundaries {
                top: 0,
                bottom: 9,
                left: 490,
                right: 510,
            },
            slice: vec![
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air, Rock,  Air,  Air,  Air, Rock, Rock,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air, Rock,  Air,  Air,  Air, Rock,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air, Rock, Rock, Rock,  Air,  Air,  Air, Rock,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air, Rock,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air, Rock,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
                vec![ Air,  Air,  Air,  Air, Rock, Rock, Rock, Rock, Rock, Rock, Rock, Rock, Rock,  Air,  Air,  Air,  Air,  Air,  Air,  Air,  Air],
            ],
            active_sand: None,
        };
        assert_eq!(vertical_cave_slice, target);
    }
}
