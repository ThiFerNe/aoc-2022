use std::hash::Hash;
use std::ops::{Div, Rem};
use std::str::FromStr;

use anyhow::Context;

use itertools::Itertools;

const INPUT: &str = include_str!("../inputs/day03.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 47 minutes 17 seconds
    let sum_of_priorities = INPUT
        .lines()
        .map(Backpack::from_str)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .enumerate()
        .map(|(index, backpack)| {
            backpack
                .find_item_type_common_in_both_compartments()
                .context(format!("in backpack #{index}"))
        })
        .collect::<Result<Vec<BackpackItem>, _>>()?
        .into_iter()
        .enumerate()
        .map(|(index, backpack_item)| {
            backpack_item
                .convert_item_into_priority()
                .context(format!("in backpack #{index}"))
        })
        .collect::<Result<Vec<u32>, _>>()?
        .into_iter()
        .sum::<u32>();
    println!("sum_of_priorities: {sum_of_priorities}");

    // PART 2 - 26 minutes 25 seconds
    let sum_of_badge_priorities = INPUT
        .lines()
        .map(Backpack::from_str)
        .collect::<Result<Vec<_>, _>>()?
        .chunks(3)
        .enumerate()
        .map(|(index, elf_group)| {
            find_elf_group_badge(elf_group).context(format!("in elf group #{index}"))
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(BackpackItem::convert_item_into_priority)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<u32>();
    println!("sum_of_badge_priorities: {sum_of_badge_priorities}");
    Ok(())
}

struct Backpack {
    items: Vec<BackpackItem>,
}

impl Backpack {
    fn split_compartments(&self) -> anyhow::Result<(&[BackpackItem], &[BackpackItem])> {
        let half_size = self.items.len().div(2);
        Ok((
            self.items
                .get(..half_size)
                .ok_or_else(|| anyhow::anyhow!("Could not get first half of backpack items."))?,
            self.items
                .get(half_size..)
                .ok_or_else(|| anyhow::anyhow!("Could not get second half of backpack items."))?,
        ))
    }

    fn find_item_type_common_in_both_compartments(&self) -> anyhow::Result<BackpackItem> {
        let (first_compartment, second_compartment) = self.split_compartments()?;
        let common_in_both = find_common_item_types(first_compartment, second_compartment);
        if common_in_both.len() > 1 {
            Err(anyhow::anyhow!(
                "Found {} items ({:?}) common in both compartments ",
                common_in_both.len(),
                common_in_both
            ))
        } else {
            common_in_both
                .first()
                .copied()
                .copied()
                .ok_or_else(|| anyhow::anyhow!("Found not items common in both compartments."))
        }
    }
}

impl FromStr for Backpack {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let items = s
            .chars()
            .map(BackpackItem::try_from)
            .collect::<Result<Vec<_>, _>>()?;
        if items.len().rem(2) == 0 {
            Ok(Self { items })
        } else {
            Err(anyhow::anyhow!(
                "Items in the backpack are not evenly distributed."
            ))
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
struct BackpackItem(char);

impl TryFrom<char> for BackpackItem {
    type Error = anyhow::Error;

    fn try_from(value: char) -> Result<Self, Self::Error> {
        match value {
            'a'..='z' | 'A'..='Z' => Ok(Self(value)),
            _ => Err(anyhow::anyhow!(
                "Item '{}' is not within specified range of a..z or A..Z.",
                value
            )),
        }
    }
}

impl BackpackItem {
    fn convert_item_into_priority(self) -> Result<u32, anyhow::Error> {
        let offset = match self.0 {
            'a'..='z' => u32::from('a')
                .checked_sub(1)
                .ok_or_else(|| anyhow::anyhow!("Could not subtract 1 from u32::from('a').")),
            'A'..='Z' => u32::from('A')
                .checked_sub(27)
                .ok_or_else(|| anyhow::anyhow!("Could not subtract 27 from u32::from('A').")),
            value => Err(anyhow::anyhow!(
                "Item '{}' is not within specified range of a..z or A..Z.",
                value
            )),
        }?;
        u32::from(self.0).checked_sub(offset).ok_or_else(|| {
            anyhow::anyhow!("Could not convert backpack item '{}' into priority", self.0)
        })
    }
}

fn find_common_item_types<'item, T>(first: &'item [T], second: &'item [T]) -> Vec<&'item T>
where
    T: Eq + PartialEq + Hash,
{
    first
        .iter()
        .filter_map(|first_item| {
            second
                .iter()
                .any(|second_item| second_item == first_item)
                .then_some(first_item)
        })
        .unique()
        .collect::<Vec<_>>()
}

fn find_elf_group_badge(elf_group: &[Backpack]) -> anyhow::Result<BackpackItem> {
    let containing_elements = elf_group.iter().fold(None, |store, backpack| match store {
        None => Some(backpack.items.clone()),
        Some(mut s) => {
            s.retain(|backpack_item| backpack.items.contains(backpack_item));
            Some(s)
        }
    });
    match containing_elements {
        None => Err(anyhow::anyhow!("Elf group is empty.")),
        Some(items) if items.len() > 1 => Err(anyhow::anyhow!(
            "Elf group has {} shared items ({:?}).",
            items.len(),
            items
        )),
        Some(items) => items
            .first()
            .copied()
            .ok_or_else(|| anyhow::anyhow!("Elf group has no shared item.")),
    }
}
