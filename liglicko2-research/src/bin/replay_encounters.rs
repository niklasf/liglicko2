use rustc_hash::FxHashMap;
use std::{error::Error as StdError, io, str::FromStr};

use chrono::{DateTime, NaiveDateTime, Utc};
use compensated_summation::KahanBabuskaNeumaier;
use liglicko2::deviance;
use liglicko2::{Instant, Rating, RatingDifference, RatingSystem, Score};
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

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum GameResult {
    Unknown,
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
            "*" => GameResult::Unknown,
            _ => return Err(InvalidGameResult),
        })
    }
}

impl GameResult {
    fn white_score(self) -> Option<Score> {
        Some(match self {
            GameResult::WhiteWins => Score::WIN,
            GameResult::BlackWins => Score::LOSS,
            GameResult::Draw => Score::DRAW,
            GameResult::Unknown => return None,
        })
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

#[derive(Default)]
struct Experiment {
    rating_system: RatingSystem,
    rating_periods_per_day: f64,
    leaderboard: FxHashMap<(String, Speed), Rating>,
    total_deviance: KahanBabuskaNeumaier<f64>,
    total_games: u64,
    errors: u64,
}

impl Experiment {
    fn to_instant(&self, date_time: &UtcDateTime) -> Instant {
        Instant(date_time.0.timestamp() as f64 / (60.0 * 60.0 * 24.0) * self.rating_periods_per_day)
    }

    fn encounter(&mut self, encounter: &Encounter) {
        let Some(actual_score) = encounter.result.white_score() else {
            return;
        };
        let speed = encounter.time_control.speed();
        let now = self.to_instant(&encounter.date_time);

        let white = self
            .leaderboard
            .get(&(encounter.white.clone(), speed))
            .cloned()
            .unwrap_or_else(|| self.rating_system.new_rating());

        let black = self
            .leaderboard
            .get(&(encounter.black.clone(), speed))
            .cloned()
            .unwrap_or_else(|| self.rating_system.new_rating());

        self.total_deviance += deviance(
            self.rating_system.expected_score(&white, &black, now),
            actual_score,
        );
        self.total_games += 1;

        let (white, black) = self
            .rating_system
            .update_ratings(&white, &black, actual_score, now)
            .unwrap_or_else(|_| {
                self.errors += 1;
                (
                    self.rating_system.new_rating(),
                    self.rating_system.new_rating(),
                )
            });

        self.leaderboard
            .insert((encounter.white.clone(), speed), white);
        self.leaderboard
            .insert((encounter.black.clone(), speed), black);
    }

    fn avg_deviance(&self) -> f64 {
        self.total_deviance.total() / self.total_games as f64
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let mut experiments = [Experiment {
        rating_system: RatingSystem::builder()
            .preview_opponent_deviation(true)
            .first_advantage(RatingDifference(8.0))
            .build(),
        rating_periods_per_day: 0.21436,
        ..Default::default()
    }];

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(io::stdin().lock());

    for encounter in reader.deserialize() {
        let encounter: Encounter = encounter?;
        for experiment in &mut experiments {
            experiment.encounter(&encounter);
        }
    }

    println!("min_deviation,max_deviation,default_volatility,tau,first_advantage,rating_periods_per_day,preview_opponent_deviation,total_games,errors,avg_deviance");
    for experiment in experiments {
        println!(
            "{},{},{},{},{},{},{},{},{},{}",
            f64::from(experiment.rating_system.min_deviation()),
            f64::from(experiment.rating_system.max_deviation()),
            f64::from(experiment.rating_system.default_volatility()),
            experiment.rating_system.tau(),
            f64::from(experiment.rating_system.first_advantage()),
            experiment.rating_periods_per_day,
            experiment.rating_system.preview_opponent_deviation(),
            experiment.total_games,
            experiment.errors,
            experiment.avg_deviance()
        );
    }

    Ok(())
}
