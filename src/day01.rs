use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day01.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 9 minutes 10 seconds
    let mut calories_per_elf = INPUT
        .split("\n\n")
        .map(|elves_calories| {
            elves_calories
                .lines()
                .map(u64::from_str)
                .collect::<Result<Vec<_>, _>>()
        })
        .collect::<Result<Vec<Vec<_>>, _>>()?
        .into_iter()
        .map(|elves_calories| elves_calories.iter().sum::<u64>())
        .collect::<Vec<_>>();

    let calories_of_elf_with_maximum = calories_per_elf
        .iter()
        .max()
        .ok_or_else(|| anyhow::anyhow!("No elf found"))?;
    println!("calories_of_elf_with_maximum: {calories_of_elf_with_maximum}");

    // PART 2 - 3 minutes 56 seconds
    calories_per_elf.sort_unstable();
    if calories_per_elf.len() < 3 {
        return Err(anyhow::anyhow!(
            "Need at minimum 3 elves for the second puzzle part."
        ));
    }
    let calories_of_top_three_elves = calories_per_elf.iter().rev().take(3).sum::<u64>();
    println!("calories_of_top_three_elves: {calories_of_top_three_elves}");

    Ok(())
}
