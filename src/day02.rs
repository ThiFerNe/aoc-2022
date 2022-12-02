use std::str::FromStr;

const INPUT: &str = include_str!("../inputs/day02.input");

fn main() -> anyhow::Result<()> {
    // PART 1 - 20 minutes 45 seconds
    let own_total_score_1 = INPUT
        .lines()
        .map(RockPaperScissorsRound::from_str)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(|round| u64::from(round.score()))
        .sum::<u64>();
    println!("own_total_score_1: {own_total_score_1}");

    // PART 2 - 11 minutes 2 seconds
    let own_total_score_2 = INPUT
        .lines()
        .map(RockPaperScissorsElfProposition::from_str)
        .collect::<Result<Vec<_>, _>>()?
        .into_iter()
        .map(RockPaperScissorsRound::from)
        .map(|round| u64::from(round.score()))
        .sum::<u64>();
    println!("own_total_score_2: {own_total_score_2}");

    Ok(())
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
            RockPaperScissorsSign::Rock => 1,
            RockPaperScissorsSign::Paper => 2,
            RockPaperScissorsSign::Scissors => 3,
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
struct RockPaperScissorsRound {
    enemy_selection: RockPaperScissorsSign,
    own_selection: RockPaperScissorsSign,
}

impl RockPaperScissorsRound {
    fn winner(self) -> RockPaperScissorsWinner {
        use RockPaperScissorsSign::{Paper, Rock, Scissors};
        match (self.own_selection, self.enemy_selection) {
            (Rock, Rock) | (Paper, Paper) | (Scissors, Scissors) => RockPaperScissorsWinner::Draw,
            (Rock, Scissors) | (Scissors, Paper) | (Paper, Rock) => RockPaperScissorsWinner::Myself,
            (Rock, Paper) | (Paper, Scissors) | (Scissors, Rock) => RockPaperScissorsWinner::Enemy,
        }
    }

    fn score(self) -> u8 {
        self.own_selection
            .selection_score()
            .saturating_add(self.winner().score())
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
            .map_err(|a: Vec<RockPaperScissorsSign>| {
                anyhow::anyhow!(
                    "Failed to convert Vec with len {} to array with len 2.",
                    a.len()
                )
            })?;
        Ok(Self {
            enemy_selection: selections[0],
            own_selection: selections[1],
        })
    }
}

impl From<RockPaperScissorsElfProposition> for RockPaperScissorsRound {
    fn from(value: RockPaperScissorsElfProposition) -> Self {
        use RockPaperScissorsSign::{Paper, Rock, Scissors};
        use RockPaperScissorsWinner::{Draw, Enemy, Myself};
        let own_selection = match (value.enemy_selection, value.target_winner) {
            (Rock, Myself) | (Scissors, Enemy) | (Paper, Draw) => Paper,
            (Paper, Myself) | (Rock, Enemy) | (Scissors, Draw) => Scissors,
            (Scissors, Myself) | (Paper, Enemy) | (Rock, Draw) => Rock,
        };
        Self {
            enemy_selection: value.enemy_selection,
            own_selection,
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

#[derive(Debug, Copy, Clone)]
struct RockPaperScissorsElfProposition {
    enemy_selection: RockPaperScissorsSign,
    target_winner: RockPaperScissorsWinner,
}

impl FromStr for RockPaperScissorsElfProposition {
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
