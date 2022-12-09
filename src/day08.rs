use std::fmt::Display;
use std::str::FromStr;

use anyhow::Context;

use itertools::FoldWhile::{Continue, Done};
use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day08.input");

fn main() -> anyhow::Result<()> {
    let tree_map = TreeMap::from_str(INPUT)?;

    // PART 1 - 46 minutes 27 seconds
    let part_1_solution = tree_map.calculate_visibility_map()?.count_visible_fields();
    println!("part_1_solution: {part_1_solution}");

    // PART 2 - 29 minutes 48 seconds
    let part_2_solution = tree_map
        .calculate_scenic_score_map()
        .context("while calculating the scenic score map for part 2")?
        .find_highest_scenic_score()
        .copied()
        .ok_or_else(|| anyhow::anyhow!("No trees were in given area."))?;
    println!("part_2_solution: {part_2_solution}");

    Ok(())
}

#[derive(Debug, Eq, PartialEq, Clone)]
struct TreeMap(Vec<Vec<Tree>>);

impl TreeMap {
    fn calculate_visibility_map(&self) -> anyhow::Result<VisibilityMap> {
        Ok(VisibilityMap(
            self.0
                .iter()
                .enumerate()
                .map(|(row_index, tree_row)| {
                    tree_row
                        .iter()
                        .enumerate()
                        .map(|(column_index, _)| self.calculate_visibility(column_index, row_index))
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn calculate_visibility(
        &self,
        column_index: usize,
        row_index: usize,
    ) -> anyhow::Result<Visibility> {
        let current_tree = self
            .0
            .get(row_index)
            .and_then(|tree_row| tree_row.get(column_index))
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Did not find current tree at column #{column_index} and row #{row_index}."
                )
            })?;

        let check_height_ge = |look_column_index: usize,
                               look_row_index: usize|
         -> anyhow::Result<bool> {
            let look_tree = self.0
                .get(look_row_index)
                .and_then(|tree_row| tree_row.get(look_column_index))
                .ok_or_else(|| anyhow::anyhow!("Did not find look tree at column #{look_column_index} and row #{look_row_index}."))?;
            Ok(look_tree.height >= current_tree.height)
        };

        let any_bigger = |start_index: usize,
                          end_index: usize,
                          look_index_is_left: bool|
         -> anyhow::Result<Option<bool>> {
            (start_index..end_index)
                .map(|look_index| {
                    if look_index_is_left {
                        check_height_ge(look_index, row_index)
                    } else {
                        check_height_ge(column_index, look_index)
                    }
                })
                .find(|bigger| *bigger.as_ref().unwrap_or(&true))
                .transpose()
        };

        let left_start_index = 0;
        let left_end_index = column_index;
        let any_left_bigger = any_bigger(left_start_index, left_end_index, true)?.unwrap_or(false);

        let right_start_index = column_index.saturating_add(1);
        let right_end_index = self
            .0
            .get(row_index)
            .ok_or_else(|| anyhow::anyhow!("Could not get tree row #{row_index}"))?
            .len();
        let any_right_bigger =
            any_bigger(right_start_index, right_end_index, true)?.unwrap_or(false);

        let top_start_index = 0;
        let top_end_index = row_index;
        let any_top_bigger = any_bigger(top_start_index, top_end_index, false)?.unwrap_or(false);

        let bottom_start_index = row_index.saturating_add(1);
        let bottom_end_index = self.0.len();
        let any_bottom_bigger =
            any_bigger(bottom_start_index, bottom_end_index, false)?.unwrap_or(false);

        if any_left_bigger && any_right_bigger && any_top_bigger && any_bottom_bigger {
            Ok(Visibility::Invisible)
        } else {
            Ok(Visibility::Visible)
        }
    }

    fn calculate_scenic_score_map(&self) -> anyhow::Result<ScenicScoreMap> {
        Ok(ScenicScoreMap(
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
                        .collect::<Result<Vec<_>, _>>()
                })
                .collect::<Result<Vec<_>, _>>()?,
        ))
    }

    fn calculate_scenic_score(
        &self,
        column_index: usize,
        row_index: usize,
    ) -> anyhow::Result<ScenicScore> {
        fn calculate<'trees, I>(mut iterator: I, tree: &'trees Tree) -> anyhow::Result<u64>
        where
            I: Iterator<Item = anyhow::Result<&'trees Tree>>,
        {
            iterator
                .fold_while(
                    Ok(0),
                    |view_distance_result: anyhow::Result<u64>, other_tree_result| {
                        match view_distance_result {
                            Ok(view_distance) => {
                                let increment = view_distance.checked_add(1).ok_or_else(|| {
                                    anyhow::anyhow!(
                                        "Could not increase view distance, because it got too big."
                                    )
                                });
                                match other_tree_result {
                                    Ok(other_tree) => {
                                        if other_tree.height >= tree.height {
                                            Done(increment)
                                        } else {
                                            Continue(increment)
                                        }
                                    }
                                    Err(other_tree_error) => Done(Err(other_tree_error)),
                                }
                            }
                            Err(_) => Done(view_distance_result),
                        }
                    },
                )
                .into_inner()
        }
        fn by_column<I>(
            iterator: I,
            row_index: usize,
            tree_map: &[Vec<Tree>],
        ) -> impl Iterator<Item = anyhow::Result<&Tree>>
        where
            I: Iterator<Item = usize>,
        {
            iterator.map(move |look_column_index| tree_map
                .get(row_index)
                .and_then(|tree_row| tree_row.get(look_column_index))
                .ok_or_else(|| anyhow::anyhow!("Could not get tree in column #{look_column_index} and row #{row_index}")))
        }
        fn by_row<I>(
            iterator: I,
            column_index: usize,
            tree_map: &[Vec<Tree>],
        ) -> impl Iterator<Item = anyhow::Result<&Tree>>
        where
            I: Iterator<Item = usize>,
        {
            iterator.map(move |look_row_index| tree_map
                .get(look_row_index)
                .and_then(|tree_row| tree_row.get(column_index))
                .ok_or_else(|| anyhow::anyhow!("Could not get tree in column #{column_index} and row #{look_row_index}")))
        }

        let tree = self
            .0
            .get(row_index)
            .and_then(|tree_row| tree_row.get(column_index))
            .ok_or_else(|| anyhow::anyhow!("Could not find tree to calculate scenic score."))?;
        let tree_row_length = self
            .0
            .get(row_index)
            .map(Vec::len)
            .ok_or_else(|| anyhow::anyhow!("Could not get tree row with row #{row_index}."))?;
        let view_distance_left =
            calculate(by_column((0..column_index).rev(), row_index, &self.0), tree)
                .context("while calculating left view distance")?;
        let view_distance_right = calculate(
            by_column(
                column_index.saturating_add(1)..tree_row_length,
                row_index,
                &self.0,
            ),
            tree,
        )
        .context("while calculating right view distance")?;
        let view_distance_top =
            calculate(by_row((0..row_index).rev(), column_index, &self.0), tree)
                .context("while calculating top view distance")?;
        let view_distance_bottom = calculate(
            by_row(
                row_index.saturating_add(1)..self.0.len(),
                column_index,
                &self.0,
            ),
            tree,
        )
        .context("while calculating bottom view distance")?;
        Ok(ScenicScore(
            view_distance_left
                .checked_mul(view_distance_right)
                .and_then(|intermediate_value| intermediate_value.checked_mul(view_distance_top))
                .and_then(|intermediate_value| intermediate_value.checked_mul(view_distance_bottom))
                .ok_or_else(|| anyhow::anyhow!("Failed making a product of the individual view distance scores, because it got too big."))?
        ))
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
        let visible_trees = tree_map.calculate_visibility_map()?.count_visible_fields();

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
        let visibility_map = tree_map.calculate_visibility_map()?;

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
        let scenic_score_map = tree_map.calculate_scenic_score_map()?;
        let highest_scenic_score = scenic_score_map.find_highest_scenic_score();

        // Assert
        assert_eq!(scenic_score_map.0[1][2], ScenicScore(4));
        assert_eq!(scenic_score_map.0[3][2], ScenicScore(8));
        assert_eq!(highest_scenic_score, Some(&ScenicScore(8)));

        Ok(())
    }
}
