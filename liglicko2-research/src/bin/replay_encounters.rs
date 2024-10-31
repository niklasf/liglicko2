use std::{collections::BTreeMap, error::Error as StdError, io, str::FromStr};

use chrono::{DateTime, NaiveDateTime, Utc};
use liglicko2::{Instant, Rating, RatingSystem, Score};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

#[derive(Debug)]
enum TimeControl {
    Clock { limit: u32, increment: u32 },
    Correspondence,
}

#[derive(Debug, Error)]
#[error("invalid time control")]
struct InvalidTimeControl;

impl FromStr for TimeControl {
    type Err = InvalidTimeControl;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(if s == "-" {
            TimeControl::Correspondence
        } else {
            let mut parts = s.splitn(2, '+');
            let limit = parts
                .next()
                .ok_or(InvalidTimeControl)?
                .parse()
                .map_err(|_| InvalidTimeControl)?;
            let increment = parts
                .next()
                .ok_or(InvalidTimeControl)?
                .parse()
                .map_err(|_| InvalidTimeControl)?;
            TimeControl::Clock { limit, increment }
        })
    }
}

impl TimeControl {
    fn estimate_total_seconds(&self) -> Option<u32> {
        match *self {
            TimeControl::Clock { limit, increment } => Some(limit + 40 * increment),
            TimeControl::Correspondence => None,
        }
    }

    fn speed(&self) -> Speed {
        match self.estimate_total_seconds() {
            Some(0..30) => Speed::UltraBullet,
            Some(30..180) => Speed::Bullet,
            Some(180..480) => Speed::Blitz,
            Some(480..1500) => Speed::Rapid,
            Some(1500..) => Speed::Classical,
            None => Speed::Correspondence,
        }
    }
}

#[derive(Debug, Hash, Eq, PartialEq, Ord, PartialOrd, Copy, Clone)]
enum Speed {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

enum GameResult {
    WhiteWins,
    BlackWins,
    Draw,
}

#[derive(Debug, Error)]
#[error("invalid game result")]
struct InvalidGameResult;

impl FromStr for GameResult {
    type Err = InvalidGameResult;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "1-0" => GameResult::WhiteWins,
            "0-1" => GameResult::BlackWins,
            "1/2-1/2" => GameResult::Draw,
            _ => return Err(InvalidGameResult),
        })
    }
}

impl GameResult {
    fn white_score(self) -> Score {
        match self {
            GameResult::WhiteWins => Score::WIN,
            GameResult::BlackWins => Score::LOSS,
            GameResult::Draw => Score::DRAW,
        }
    }
}

struct UtcDateTime(DateTime<Utc>);

impl FromStr for UtcDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtcDateTime(
            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?.and_utc(),
        ))
    }
}

#[serde_as]
#[derive(Deserialize)]
struct Encounter {
    white: String,
    black: String,
    #[serde_as(as = "DisplayFromStr")]
    result: GameResult,
    #[serde_as(as = "DisplayFromStr")]
    date_time: UtcDateTime,
    #[serde_as(as = "DisplayFromStr")]
    time_control: TimeControl,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let rating_system = RatingSystem::new();
    let mut leaderboard: BTreeMap<(String, Speed), Rating> = BTreeMap::new();

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(io::stdin().lock());

    let to_instant = |date_time: UtcDateTime| Instant(date_time.0.timestamp() as f64);

    for encounter in reader.deserialize() {
        let encounter: Encounter = encounter?;
        let speed = encounter.time_control.speed();

        let white = leaderboard
            .get(&(encounter.white.clone(), speed))
            .cloned()
            .unwrap_or_else(|| rating_system.new_rating());

        let black = leaderboard
            .get(&(encounter.black.clone(), speed))
            .cloned()
            .unwrap_or_else(|| rating_system.new_rating());

        let (white, black) = rating_system
            .update_ratings(
                &white,
                &black,
                encounter.result.white_score(),
                to_instant(encounter.date_time),
            )
            .unwrap();

        leaderboard.insert((encounter.white, speed), white);
        leaderboard.insert((encounter.black, speed), black);
    }

    for ((player, speed), rating) in leaderboard {
        println!(
            "{},{:?},{},{},{}",
            player, speed, rating.rating.0, rating.deviation.0, rating.volatility.0
        );
    }

    Ok(())
}
