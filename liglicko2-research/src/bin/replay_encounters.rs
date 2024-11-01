use ordered_float::OrderedFloat;
use rustc_hash::FxHashMap;
use std::{error::Error as StdError, io, str::FromStr};

use chrono::NaiveDateTime;
use compensated_summation::KahanBabuskaNeumaier;
use liglicko2::{deviance, Volatility};
use liglicko2::{Instant, Rating, RatingDifference, RatingSystem, Score};
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;

use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

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

#[derive(Debug, Clone, Default)]
struct BySpeed<T> {
    ultra_bullet: T,
    bullet: T,
    blitz: T,
    rapid: T,
    classical: T,
    correspondence: T,
}

impl<T> BySpeed<T> {
    fn get_mut(&mut self, speed: Speed) -> &mut T {
        match speed {
            Speed::UltraBullet => &mut self.ultra_bullet,
            Speed::Bullet => &mut self.bullet,
            Speed::Blitz => &mut self.blitz,
            Speed::Rapid => &mut self.rapid,
            Speed::Classical => &mut self.classical,
            Speed::Correspondence => &mut self.correspondence,
        }
    }
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

#[derive(Debug, Copy, Clone)]
struct UtcDateTime(i64);

impl FromStr for UtcDateTime {
    type Err = chrono::ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(UtcDateTime(
            NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S")?
                .and_utc()
                .timestamp(),
        ))
    }
}

#[serde_as]
#[derive(Deserialize)]
struct RawEncounter {
    white: String,
    black: String,
    #[serde_as(as = "DisplayFromStr")]
    result: GameResult,
    #[serde_as(as = "DisplayFromStr")]
    date_time: UtcDateTime,
    #[serde_as(as = "DisplayFromStr")]
    time_control: TimeControl,
}

struct Encounter {
    white: PlayerId,
    black: PlayerId,
    white_score: Score,
    date_time: UtcDateTime,
    speed: Speed,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
struct PlayerId(usize);

#[derive(Default)]
struct PlayerIds {
    inner: FxHashMap<Box<str>, PlayerId>,
}

impl PlayerIds {
    fn get_or_insert(&mut self, name: String) -> PlayerId {
        let next_id = PlayerId(self.inner.len());
        *self.inner.entry(name.into_boxed_str()).or_insert(next_id)
    }
}

struct ByPlayerId<T> {
    inner: Vec<Option<T>>,
}

impl<T> Default for ByPlayerId<T> {
    fn default() -> Self {
        ByPlayerId { inner: Vec::new() }
    }
}

impl<T> ByPlayerId<T> {
    fn get(&self, PlayerId(id): PlayerId) -> Option<&T> {
        match self.inner.get(id) {
            Some(Some(t)) => Some(t),
            _ => None,
        }
    }

    fn set(&mut self, PlayerId(id): PlayerId, value: T) {
        if self.inner.len() <= id {
            self.inner.resize_with(id + 1, || None);
        }
        self.inner[id] = Some(value);
    }
}

#[derive(Default)]
struct Experiment {
    rating_system: RatingSystem,
    rating_periods_per_day: f64,
    leaderboard: BySpeed<ByPlayerId<Rating>>,
    total_deviance: KahanBabuskaNeumaier<f64>,
    total_games: u64,
    errors: u64,
}

impl Experiment {
    fn to_instant(&self, UtcDateTime(timestamp): UtcDateTime) -> Instant {
        Instant(timestamp as f64 / (60.0 * 60.0 * 24.0) * self.rating_periods_per_day)
    }

    fn encounter(&mut self, encounter: &Encounter) {
        let now = self.to_instant(encounter.date_time);
        let leaderboard = self.leaderboard.get_mut(encounter.speed);

        let white = leaderboard
            .get(encounter.white)
            .cloned()
            .unwrap_or_else(|| self.rating_system.new_rating());

        let black = leaderboard
            .get(encounter.black)
            .cloned()
            .unwrap_or_else(|| self.rating_system.new_rating());

        self.total_deviance += deviance(
            self.rating_system.expected_score(&white, &black, now),
            encounter.white_score,
        );
        self.total_games += 1;

        let (white, black) = self
            .rating_system
            .update_ratings(&white, &black, encounter.white_score, now)
            .unwrap_or_else(|_| {
                self.errors += 1;
                (
                    self.rating_system.new_rating(),
                    self.rating_system.new_rating(),
                )
            });

        leaderboard.set(encounter.white, white);
        leaderboard.set(encounter.black, black);
    }

    fn avg_deviance(&self) -> f64 {
        self.total_deviance.total() / self.total_games as f64
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let mut experiments = Vec::new();

    for min_deviation in [40.0, 45.0, 50.0] {
        for max_deviation in [450.0, 500.0, 550.0] {
            for default_volatility in [0.08, 0.09, 0.1] {
                for tau in [0.6, 0.75, 0.9] {
                    for first_advantage in [0.0, 8.0, 11.0] {
                        for preview_opponent_deviation in [true, false] {
                            for rating_periods_per_day in [0.2, 0.21436, 0.23] {
                                experiments.push(Experiment {
                                    rating_system: RatingSystem::builder()
                                        .min_deviation(RatingDifference(min_deviation))
                                        .max_deviation(RatingDifference(max_deviation))
                                        .default_volatility(Volatility(default_volatility))
                                        .tau(tau)
                                        .first_advantage(RatingDifference(first_advantage))
                                        .preview_opponent_deviation(preview_opponent_deviation)
                                        .build(),
                                    rating_periods_per_day,
                                    ..Default::default()
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    println!("# Experiments: {}", experiments.len());

    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .from_reader(io::stdin().lock());

    let mut players = PlayerIds::default();

    let mut total_encounters: u64 = 0;
    for encounter in reader.deserialize() {
        total_encounters += 1;
        if total_encounters % 10_000 == 0 {
            eprintln!("# Processing encounter {} ...", total_encounters);
        }

        let encounter: RawEncounter = encounter?;

        let encounter = Encounter {
            white: players.get_or_insert(encounter.white),
            black: players.get_or_insert(encounter.black),
            white_score: match encounter.result.white_score() {
                Some(score) => score,
                None => continue,
            },
            speed: encounter.time_control.speed(),
            date_time: encounter.date_time,
        };

        for experiment in &mut experiments {
            experiment.encounter(&encounter);
        }
    }

    experiments.sort_by_key(|experiment| OrderedFloat(-experiment.total_deviance.total()));

    println!("# Total encounters: {}", total_encounters);
    println!("min_deviation,max_deviation,default_volatility,tau,first_advantage,rating_periods_per_day,preview_opponent_deviation,errors,avg_deviance");
    for experiment in experiments {
        println!(
            "{},{},{},{},{},{},{},{},{}",
            f64::from(experiment.rating_system.min_deviation()),
            f64::from(experiment.rating_system.max_deviation()),
            f64::from(experiment.rating_system.default_volatility()),
            experiment.rating_system.tau(),
            f64::from(experiment.rating_system.first_advantage()),
            experiment.rating_periods_per_day,
            experiment.rating_system.preview_opponent_deviation(),
            experiment.errors,
            experiment.avg_deviance()
        );
    }

    Ok(())
}
