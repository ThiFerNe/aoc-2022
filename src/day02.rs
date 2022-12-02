use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day02.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 20 minutes 45 seconds
    let score_by_guesswork = RockPaperScissorsMatch::from_guess(INPUT)?.score()?;
    println!("score_by_guesswork: {score_by_guesswork}");

    // PART 2 - 11 minutes 2 seconds
    let score_by_elf_explanation = RockPaperScissorsMatch::from_elf_information(INPUT)?.score()?;
    println!("score_by_elf_explanation: {score_by_elf_explanation}");

    Ok(())
}

struct RockPaperScissorsMatch {
    rounds: Vec<RockPaperScissorsRound>,
}

impl RockPaperScissorsMatch {
    fn from_guess(input: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            rounds: input
                .lines()
                .map(RockPaperScissorsRound::from_str)
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn from_elf_information(input: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            rounds: input
                .lines()
                .map(|line| {
                    RockPaperScissorsElfExplanation::from_str(line)
                        .map(RockPaperScissorsRound::from)
                })
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    fn score(&self) -> Result<u64, anyhow::Error> {
        Ok(self
            .rounds
            .iter()
            .map(|round| {
                round
                    .score()
                    .map(u64::from)
                    .ok_or_else(|| anyhow::anyhow!("Could not calculate round score."))
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .sum::<u64>())
    }
}

#[derive(Debug, Copy, Clone)]
struct RockPaperScissorsElfExplanation {
    enemy_selection: RockPaperScissorsSign,
    target_winner: RockPaperScissorsWinner,
}

impl FromStr for RockPaperScissorsElfExplanation {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let selections: [&str; 2] =
            s.split(' ')
                .collect::<Vec<_>>()
                .try_into()
                .map_err(|a: Vec<&str>| {
                    anyhow::anyhow!(
                        "Failed to convert Vec with len {} to array with len 2.",
                        a.len()
                    )
                })?;
        Ok(Self {
            enemy_selection: RockPaperScissorsSign::from_str(selections[0])?,
            target_winner: match selections[1] {
                "X" => RockPaperScissorsWinner::Enemy,
                "Y" => RockPaperScissorsWinner::Draw,
                "Z" => RockPaperScissorsWinner::Myself,
                other => {
                    return Err(anyhow::anyhow!(
                        "Unexpected second proposition string found \"{other}\""
                    ))
                }
            },
        })
    }
}

#[derive(Debug, Copy, Clone)]
struct RockPaperScissorsRound {
    enemy_selection: RockPaperScissorsSign,
    own_selection: RockPaperScissorsSign,
}

impl RockPaperScissorsRound {
    fn winner(self) -> RockPaperScissorsWinner {
        use RockPaperScissorsSign::{Paper, Rock, Scissors};
        use RockPaperScissorsWinner::{Draw, Enemy, Myself};
        match self.own_selection {
            Rock => match self.enemy_selection {
                Rock => Draw,
                Paper => Enemy,
                Scissors => Myself,
            },
            Paper => match self.enemy_selection {
                Rock => Myself,
                Paper => Draw,
                Scissors => Enemy,
            },
            Scissors => match self.enemy_selection {
                Rock => Enemy,
                Paper => Myself,
                Scissors => Draw,
            },
        }
    }

    fn score(self) -> Option<u8> {
        self.own_selection
            .selection_score()
            .checked_add(self.winner().score())
    }
}

impl FromStr for RockPaperScissorsRound {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let selections: [RockPaperScissorsSign; 2] = s
            .split(' ')
            .map(RockPaperScissorsSign::from_str)
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|vec: Vec<_>| {
                anyhow::anyhow!(
                    "Failed to convert Vec with len {} to array with len 2.",
                    vec.len()
                )
            })?;
        Ok(Self {
            enemy_selection: selections[0],
            own_selection: selections[1],
        })
    }
}

impl From<RockPaperScissorsElfExplanation> for RockPaperScissorsRound {
    fn from(value: RockPaperScissorsElfExplanation) -> Self {
        use RockPaperScissorsSign::{Paper, Rock, Scissors};
        use RockPaperScissorsWinner::{Draw, Enemy, Myself};
        Self {
            enemy_selection: value.enemy_selection,
            own_selection: match value.target_winner {
                Draw => value.enemy_selection,
                Myself => match value.enemy_selection {
                    Rock => Paper,
                    Paper => Scissors,
                    Scissors => Rock,
                },
                Enemy => match value.enemy_selection {
                    Rock => Scissors,
                    Paper => Rock,
                    Scissors => Paper,
                },
            },
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum RockPaperScissorsSign {
    Rock,
    Paper,
    Scissors,
}

impl RockPaperScissorsSign {
    fn selection_score(self) -> u8 {
        match self {
            Self::Rock => 1,
            Self::Paper => 2,
            Self::Scissors => 3,
        }
    }
}

impl FromStr for RockPaperScissorsSign {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "A" | "X" => Ok(Self::Rock),
            "B" | "Y" => Ok(Self::Paper),
            "C" | "Z" => Ok(Self::Scissors),
            _ => Err(anyhow::anyhow!("Unexpected string \"{s}\" found")),
        }
    }
}

#[derive(Debug, Copy, Clone)]
enum RockPaperScissorsWinner {
    Enemy,
    Draw,
    Myself,
}

impl RockPaperScissorsWinner {
    fn score(self) -> u8 {
        match self {
            Self::Enemy => 0,
            Self::Draw => 3,
            Self::Myself => 6,
        }
    }
}
