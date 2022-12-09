use std::fmt::Display;
use std::str::FromStr;

use anyhow::Context;

use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day08.input");

fn main() -> anyhow::Result<()> {
    let tree_map = TreeMap::from_str(INPUT)?;

    // PART 1 - 46 minutes 27 seconds
    let part_1_solution = tree_map.calculate_visibility_map().count_visible_fields();
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 29 minutes 48 seconds
    let part_2_solution = tree_map
        .calculate_scenic_score_map()
        .find_highest_scenic_score()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No trees were in given area."))?;
    println!("part_2_solution: {part_2_solution}");

    Ok(())
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct TreeMap(Vec<Vec<Tree>>);

impl TreeMap {
    fn calculate_visibility_map(&self) -> VisibilityMap {
        VisibilityMap(
            self.0
                .iter()
                .enumerate()
                .map(|(row_index, tree_row)| {
                    tree_row
                        .iter()
                        .enumerate()
                        .map(|(column_index, _)| self.calculate_visibility(column_index, row_index))
                        .collect()
                })
                .collect(),
        )
    }

    fn calculate_visibility(&self, column_index: usize, row_index: usize) -> Visibility {
        let any_left_bigger = (0..column_index).any(|look_column_index| {
            self.0[row_index][look_column_index].height >= self.0[row_index][column_index].height
        });
        let any_right_bigger =
            ((column_index + 1)..self.0[row_index].len()).any(|look_column_index| {
                self.0[row_index][look_column_index].height
                    >= self.0[row_index][column_index].height
            });
        let any_top_bigger = (0..row_index).any(|look_row_index| {
            self.0[look_row_index][column_index].height >= self.0[row_index][column_index].height
        });
        let any_bottom_bigger = ((row_index + 1)..self.0.len()).any(|look_row_index| {
            self.0[look_row_index][column_index].height >= self.0[row_index][column_index].height
        });
        if any_left_bigger && any_right_bigger && any_top_bigger && any_bottom_bigger {
            Visibility::Invisible
        } else {
            Visibility::Visible
        }
    }

    fn calculate_scenic_score_map(&self) -> ScenicScoreMap {
        ScenicScoreMap(
            self.0
                .iter()
                .enumerate()
                .map(|(row_index, tree_row)| {
                    tree_row
                        .iter()
                        .enumerate()
                        .map(|(column_index, _)| {
                            self.calculate_scenic_score(column_index, row_index)
                        })
                        .collect()
                })
                .collect(),
        )
    }

    fn calculate_scenic_score(&self, column_index: usize, row_index: usize) -> ScenicScore {
        fn calculate<'trees, I>(mut iterator: I, tree: &'trees Tree) -> u64
        where
            I: Iterator<Item = &'trees Tree>,
        {
            iterator
                .fold_while(0, |view_distance, other_tree| {
                    if other_tree.height >= tree.height {
                        Done(view_distance + 1)
                    } else {
                        Continue(view_distance + 1)
                    }
                })
                .into_inner()
        }
        fn by_column<I>(
            iterator: I,
            row_index: usize,
            tree_map: &[Vec<Tree>],
        ) -> impl Iterator<Item = &Tree>
        where
            I: Iterator<Item = usize>,
        {
            iterator.map(move |look_column_index| &tree_map[row_index][look_column_index])
        }
        fn by_row<I>(
            iterator: I,
            column_index: usize,
            tree_map: &[Vec<Tree>],
        ) -> impl Iterator<Item = &Tree>
        where
            I: Iterator<Item = usize>,
        {
            iterator.map(move |look_row_index| &tree_map[look_row_index][column_index])
        }

        let tree = &self.0[row_index][column_index];
        let view_distance_left =
            calculate(by_column((0..column_index).rev(), row_index, &self.0), tree);
        let view_distance_right = calculate(
            by_column(
                (column_index + 1)..self.0[row_index].len(),
                row_index,
                &self.0,
            ),
            tree,
        );
        let view_distance_top =
            calculate(by_row((0..row_index).rev(), column_index, &self.0), tree);
        let view_distance_bottom = calculate(
            by_row((row_index + 1)..self.0.len(), column_index, &self.0),
            tree,
        );
        ScenicScore(
            view_distance_left * view_distance_right * view_distance_top * view_distance_bottom,
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

        let tree_map_line_lengths = tree_map.iter().map(Vec::len);
        if tree_map_line_lengths.clone().unique().count() == 1 {
            Ok(Self(tree_map))
        } else {
            Err(anyhow::anyhow!(
                "The input line lengths are not all the same length, found different line lengths ({:?}).",
                tree_map_line_lengths.collect::<Vec<_>>()
            ))
        }
    }
}

impl Display for TreeMap {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (row_index, tree_row) in self.0.iter().enumerate() {
            if row_index > 0 {
                writeln!(f)?;
            }
            for tree in tree_row {
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

impl FromStr for Tree {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self { height: s.parse()? })
    }
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct VisibilityMap(Vec<Vec<Visibility>>);

impl VisibilityMap {
    fn count_visible_fields(&self) -> usize {
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

#[derive(Debug, Eq, PartialEq, Clone)]
struct ScenicScoreMap(Vec<Vec<ScenicScore>>);

impl ScenicScoreMap {
    fn find_highest_scenic_score(&self) -> Option<&ScenicScore> {
        self.0.iter().filter_map(|row| row.iter().max()).max()
    }
}

#[derive(Debug, Eq, PartialEq, Copy, Clone, Ord, PartialOrd)]
struct ScenicScore(u64);

impl Display for ScenicScore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[allow(clippy::panic_in_result_fn)]
#[allow(clippy::indexing_slicing)]
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
        let visible_trees = tree_map.calculate_visibility_map().count_visible_fields();

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
                vec![Tree { height: 3 }, Tree { height: 0 }, Tree { height: 3 }, Tree { height: 7 }, Tree { height: 3 }],
                vec![Tree { height: 2 }, Tree { height: 5 }, Tree { height: 5 }, Tree { height: 1 }, Tree { height: 2 }],
                vec![Tree { height: 6 }, Tree { height: 5 }, Tree { height: 3 }, Tree { height: 3 }, Tree { height: 2 }],
                vec![Tree { height: 3 }, Tree { height: 3 }, Tree { height: 5 }, Tree { height: 4 }, Tree { height: 9 }],
                vec![Tree { height: 3 }, Tree { height: 5 }, Tree { height: 3 }, Tree { height: 9 }, Tree { height: 0 }],
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
        use Visibility::{Invisible, Visible};

        // Arrange
        let tree_map = TreeMap::from_str(TEST_INPUT)?;

        // Act
        let visibility_map = tree_map.calculate_visibility_map();

        // Assert
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

    #[test]
    fn test_part_2_default() -> anyhow::Result<()> {
        // Arrange
        let tree_map = TreeMap::from_str(TEST_INPUT)?;

        // Act
        let scenic_score_map = tree_map.calculate_scenic_score_map();
        let highest_scenic_score = scenic_score_map.find_highest_scenic_score();

        // Assert
        assert_eq!(scenic_score_map.0[1][2], ScenicScore(4));
        assert_eq!(scenic_score_map.0[3][2], ScenicScore(8));
        assert_eq!(highest_scenic_score, Some(&ScenicScore(8)));

        Ok(())
    }
}
