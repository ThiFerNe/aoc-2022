use anyhow::Context;
use itertools::Itertools;
use std::fmt::Display;
use std::ops::Range;
use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day08.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 46 minutes 27 seconds
    let part_1_solution = TreeMap::from_str(INPUT)?
        .calculate_visibility_map()
        .count_visibles();
    println!("part_1_solution: {part_1_solution}");

    Ok(())
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct TreeMap(Vec<Vec<Tree>>);

impl TreeMap {
    fn calculate_visibility_map(&self) -> VisibilityMap {
        let check_line = |mut column_range: Range<usize>,
                          column_index: usize,
                          line_index: usize|
         -> Visibility {
            let any_bigger = column_range.any(|look_column_index| {
                self.0[line_index][look_column_index].height
                    >= self.0[line_index][column_index].height
            });
            if any_bigger {
                Visibility::Invisible
            } else {
                Visibility::Visible
            }
        };
        let check_column =
            |mut line_range: Range<usize>, column_index: usize, line_index: usize| -> Visibility {
                let any_bigger = line_range.any(|look_line_index| {
                    self.0[look_line_index][column_index].height
                        >= self.0[line_index][column_index].height
                });
                if any_bigger {
                    Visibility::Invisible
                } else {
                    Visibility::Visible
                }
            };
        VisibilityMap(
            self.0
                .iter()
                .enumerate()
                .map(|(line_index, tree_line)| {
                    tree_line
                        .iter()
                        .enumerate()
                        .map(|(column_index, _)| {
                            check_line(0..column_index, column_index, line_index)
                                .and(|| {
                                    check_line(
                                        (column_index + 1)..(self.0[line_index].len()),
                                        column_index,
                                        line_index,
                                    )
                                })
                                .and(|| check_column(0..line_index, column_index, line_index))
                                .and(|| {
                                    check_column(
                                        (line_index + 1)..(self.0.len()),
                                        column_index,
                                        line_index,
                                    )
                                })
                        })
                        .collect()
                })
                .collect(),
        )
    }
}

impl FromStr for TreeMap {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let tree_map = s
            .lines()
            .enumerate()
            .map(|(line_index, line)| {
                line.chars()
                    .enumerate()
                    .map(|(char_index, character)| {
                        Tree::from_str(&character.to_string()).with_context(|| {
                            anyhow::anyhow!("column #{char_index} and line #{line_index}")
                        })
                    })
                    .collect::<Result<Vec<_>, _>>()
            })
            .collect::<Result<Vec<Vec<_>>, _>>()?;
        let count_of_different_tree_map_line_lengths =
            tree_map.iter().map(|vec| vec.len()).unique().count();
        if count_of_different_tree_map_line_lengths == 1 {
            Ok(Self(tree_map))
        } else {
            Err(anyhow::anyhow!("The input line lengths are not all the same length, found {count_of_different_tree_map_line_lengths} different line lengths."))
        }
    }
}

impl Display for TreeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (line_index, tree_line) in self.0.iter().enumerate() {
            if line_index > 0 {
                writeln!(f)?;
            }
            for tree in tree_line {
                write!(f, "{}", tree.height)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
struct Tree {
    height: u8,
}

impl Tree {
    #[allow(dead_code)]
    fn of_height(height: u8) -> Self {
        Self { height }
    }
}

impl FromStr for Tree {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { height: s.parse()? })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct VisibilityMap(Vec<Vec<Visibility>>);

impl VisibilityMap {
    fn count_visibles(&self) -> usize {
        self.0
            .iter()
            .map(|line| {
                line.iter()
                    .filter(|visibility| **visibility == Visibility::Visible)
                    .count()
            })
            .sum()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone)]
enum Visibility {
    Visible,
    Invisible,
}

impl Visibility {
    fn and<F>(&self, other: F) -> Self
    where
        F: Fn() -> Self,
    {
        match self {
            Visibility::Visible => Visibility::Visible,
            Visibility::Invisible => other(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "30373
25512
65332
33549
35390";

    #[test]
    fn test_part_1_default() -> anyhow::Result<()> {
        // Arrange
        let tree_map = TreeMap::from_str(TEST_INPUT)?;

        // Act
        let visible_trees = tree_map.calculate_visibility_map().count_visibles();

        // Assert
        assert_eq!(visible_trees, 21);

        Ok(())
    }

    #[test]
    fn test_tree_map_from_str() -> anyhow::Result<()> {
        // Act
        let tree_map = TreeMap::from_str(TEST_INPUT)?;

        // Assert
        #[rustfmt::skip]
        assert_eq!(
            tree_map,
            TreeMap(vec![
                vec![Tree::of_height(3), Tree::of_height(0), Tree::of_height(3), Tree::of_height(7), Tree::of_height(3)],
                vec![Tree::of_height(2), Tree::of_height(5), Tree::of_height(5), Tree::of_height(1), Tree::of_height(2)],
                vec![Tree::of_height(6), Tree::of_height(5), Tree::of_height(3), Tree::of_height(3), Tree::of_height(2)],
                vec![Tree::of_height(3), Tree::of_height(3), Tree::of_height(5), Tree::of_height(4), Tree::of_height(9)],
                vec![Tree::of_height(3), Tree::of_height(5), Tree::of_height(3), Tree::of_height(9), Tree::of_height(0)],
            ]),
        );

        Ok(())
    }

    #[test]
    fn test_tree_map_display() -> anyhow::Result<()> {
        // Arrange
        let tree_map = TreeMap::from_str(TEST_INPUT)?;

        // Act
        let tree_map_string = tree_map.to_string();

        // Assert
        assert_eq!(tree_map_string, TEST_INPUT);

        Ok(())
    }

    #[test]
    fn test_tree_map_calculate_visibility_map() -> anyhow::Result<()> {
        // Arrange
        let tree_map = TreeMap::from_str(TEST_INPUT)?;

        // Act
        let visibility_map = tree_map.calculate_visibility_map();

        // Assert
        use Visibility::{Invisible, Visible};
        #[rustfmt::skip]
        assert_eq!(
            visibility_map,
            VisibilityMap(vec![
                vec![   Visible,   Visible,   Visible,   Visible,   Visible],
                vec![   Visible,   Visible,   Visible, Invisible,   Visible],
                vec![   Visible,   Visible, Invisible,   Visible,   Visible],
                vec![   Visible, Invisible,   Visible, Invisible,   Visible],
                vec![   Visible,   Visible,   Visible,   Visible,   Visible],
            ])
        );

        Ok(())
    }
}
