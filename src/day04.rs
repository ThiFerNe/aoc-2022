use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day04.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 24 minutes 45 seconds
    let part_1_solution = calculate_count_of_fully_containing_pairs(INPUT)?;
    println!("part_1_solution: {part_1_solution}");

    Ok(())
}

fn calculate_count_of_fully_containing_pairs(input: &str) -> anyhow::Result<usize> {
    Ok(input
        .lines()
        .map(ElfPair::from_str)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .filter(|elf_pair| elf_pair.one_fully_contains_the_other())
        .count())
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct ElfPair(SectionAssignment, SectionAssignment);

impl ElfPair {
    fn one_fully_contains_the_other(self) -> bool {
        self.0.fully_contains(self.1) || self.1.fully_contains(self.0)
    }
}

impl FromStr for ElfPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pair: [SectionAssignment; 2] = s
            .split(',')
            .map(SectionAssignment::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|vec: Vec<_>| {
                anyhow::anyhow!(
                    "Did not get 2 section assignments but {} ({:?})",
                    vec.len(),
                    vec
                )
            })?;
        Ok(Self(pair[0], pair[1]))
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct SectionAssignment {
    from: SectionId,
    to: SectionId,
}

impl SectionAssignment {
    fn fully_contains(self, other: Self) -> bool {
        self.from <= other.from && self.to >= other.to
    }
}

impl FromStr for SectionAssignment {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let pair: [SectionId; 2] = s
            .split('-')
            .map(SectionId::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|vec: Vec<_>| {
                anyhow::anyhow!("Did not get 2 section ids but {} ({:?})", vec.len(), vec)
            })?;
        Ok(Self {
            from: pair[0].min(pair[1]),
            to: pair[0].max(pair[1]),
        })
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
struct SectionId(u8);

impl FromStr for SectionId {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(s.parse::<u8>()?))
    }
}

#[allow(clippy::panic_in_result_fn)]
#[cfg(test)]
mod tests {
    use super::*;

    const TEST_INPUT: &str = "2-4,6-8
2-3,4-5
5-7,7-9
2-8,3-7
6-6,4-6
2-6,4-8";

    #[test]
    fn default_test_part_1() -> anyhow::Result<()> {
        // Act
        let count = calculate_count_of_fully_containing_pairs(TEST_INPUT)?;

        // Assert
        assert_eq!(count, 2);

        Ok(())
    }
}
