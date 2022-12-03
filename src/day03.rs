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
                .find_item_type_which_is_in_both_compartments()
                .map(|option| option.ok_or_else(|| anyhow::anyhow!("Did not find a shared item")))
                .context(format!("in Backpack no. {index}"))
        })
        .collect::<Result<Result<Vec<BackpackItem>, _>, _>>()??
        .into_iter()
        .map(BackpackItem::convert_item_into_priority)
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
            let containing_elements = elf_group.iter().fold(None, |store, backpack| match store {
                None => Some(backpack.full_inventory.chars().unique().collect::<Vec<_>>()),
                Some(mut s) => {
                    s.retain(|c| backpack.full_inventory.contains(|c2| c2 == *c));
                    Some(s)
                }
            });
            match containing_elements {
                None => Err(anyhow::anyhow!("Elf group #{index} is empty.")),
                Some(items) if items.len() > 1 => Err(anyhow::anyhow!(
                    "Elf group #{index} has {} shared items ({:?}).",
                    items.len(),
                    items
                )),
                Some(items) => items
                    .first()
                    .copied()
                    .ok_or_else(|| anyhow::anyhow!("Elf group #{index} has no shared item.")),
            }
        })
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(BackpackItem)
        .map(BackpackItem::convert_item_into_priority)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .sum::<u32>();
    println!("sum_of_badge_priorities: {sum_of_badge_priorities}");
    Ok(())
}

struct Backpack {
    full_inventory: String,
    first_compartment: String,
    second_compartment: String,
}

impl Backpack {
    fn find_item_type_which_is_in_both_compartments(
        &self,
    ) -> Result<Option<BackpackItem>, anyhow::Error> {
        let wrongly_stored_items_found =
            find_chars_found_in_both(&self.first_compartment, &self.second_compartment);
        match wrongly_stored_items_found.len() {
            0 => Ok(None),
            1 => Ok(wrongly_stored_items_found
                .first()
                .copied()
                .map(BackpackItem)),
            n => Err(anyhow::anyhow!(
                "Found {n} items multiple times in the second compartment"
            )),
        }
    }
}

impl FromStr for Backpack {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let char_count = s.chars().count();
        if char_count.rem(2) == 0 {
            let half_char_count = char_count.div(2);
            Ok(Self {
                full_inventory: s.to_string(),
                first_compartment: s.chars().take(half_char_count).collect::<String>(),
                second_compartment: s.chars().skip(half_char_count).collect::<String>(),
            })
        } else {
            Err(anyhow::anyhow!(
                "Items in the backpack are not evenly distributed."
            ))
        }
    }
}

#[derive(Copy, Clone)]
struct BackpackItem(char);

impl BackpackItem {
    fn convert_item_into_priority(self) -> Result<u32, anyhow::Error> {
        let offset = match self.0 {
            'a'..='z' => u32::from('a')
                .checked_sub(1)
                .ok_or_else(|| anyhow::anyhow!("Could not subtract 1 from 'a'.")),
            'A'..='Z' => u32::from('A')
                .checked_sub(27)
                .ok_or_else(|| anyhow::anyhow!("Could not subtract 27 from 'A'.")),
            _ => {
                return Err(anyhow::anyhow!(
                    "Item '{}' is not within specified range of a..z or A..Z.",
                    self.0
                ))
            }
        }?;
        u32::from(self.0)
            .checked_sub(offset)
            .ok_or_else(|| anyhow::anyhow!("Could not convert item into priority"))
    }
}

fn find_chars_found_in_both(first: &str, second: &str) -> Vec<char> {
    first
        .chars()
        .filter_map(|first_item| {
            second
                .chars()
                .any(|second_item| second_item == first_item)
                .then_some(first_item)
        })
        .unique()
        .collect::<Vec<_>>()
}
