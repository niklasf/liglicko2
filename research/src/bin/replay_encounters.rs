use std::{error::Error as StdError, fmt, fs::File, io, io::Write, str::FromStr};

use chrono::{DateTime, NaiveDateTime};
use clap::Parser as _;
use compensated_summation::KahanBabuskaNeumaier;
use liglicko2::{
    deviance, Instant, Rating, RatingDifference, RatingScalar, RatingSystem, Score, Volatility,
};
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use serde_with::{serde_as, DisplayFromStr};
use thiserror::Error;
use uuid::Uuid;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

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
    fn get(&self, speed: Speed) -> &T {
        match speed {
            Speed::UltraBullet => &self.ultra_bullet,
            Speed::Bullet => &self.bullet,
            Speed::Blitz => &self.blitz,
            Speed::Rapid => &self.rapid,
            Speed::Classical => &self.classical,
            Speed::Correspondence => &self.correspondence,
        }
    }

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

#[derive(Debug, Copy, Clone, Default)]
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

impl fmt::Display for UtcDateTime {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            DateTime::from_timestamp(self.0, 0)
                .unwrap_or_default()
                .naive_utc()
        )
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
    utc_date_time: UtcDateTime,
    #[serde_as(as = "DisplayFromStr")]
    time_control: TimeControl,
}

struct Encounter {
    white: PlayerId,
    black: PlayerId,
    white_score: Score,
    utc_date_time: UtcDateTime,
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

    fn get(&self, name: &str) -> Option<PlayerId> {
        self.inner.get(name).copied()
    }

    fn len(&self) -> usize {
        self.inner.len()
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

    fn table(&self) -> &[Option<T>] {
        &self.inner
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
    fn sort_key(&self) -> impl Ord {
        OrderedFloat(-self.total_deviance.total())
    }

    fn to_instant(&self, UtcDateTime(timestamp): UtcDateTime) -> Instant {
        Instant(timestamp as f64 / (60.0 * 60.0 * 24.0) * self.rating_periods_per_day)
    }

    fn batch_encounters(&mut self, encounters: &[Encounter]) {
        for encounter in encounters {
            self.encounter(encounter);
        }
    }

    fn encounter(&mut self, encounter: &Encounter) {
        let now = self.to_instant(encounter.utc_date_time);
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

    fn estimate_avg_rating(&self, speed: Speed) -> f64 {
        let mut total_rating = KahanBabuskaNeumaier::default();
        let mut num_ratings: u64 = 0;

        let table = self.leaderboard.get(speed).table();
        let mut i = 0;
        while i < table.len() {
            if let Some(rating) = &table[i] {
                total_rating += f64::from(rating.rating);
                num_ratings += 1;
            }
            i += 2000;
        }

        total_rating.total() / num_ratings as f64
    }

    fn estimate_percentiles(&self, speed: Speed) -> (f64, f64, f64, f64, f64) {
        let mut samples = Vec::new();

        let table = self.leaderboard.get(speed).table();
        let mut i = 0;
        while i < table.len() {
            if let Some(rating) = &table[i] {
                samples.push(OrderedFloat(f64::from(rating.rating)));
            }
            i += 2000;
        }

        samples.sort_unstable();

        let p = |x: usize| {
            samples
                .get(samples.len() * x / 100)
                .copied()
                .map(f64::from)
                .unwrap_or(f64::NAN)
        };

        (p(1), p(10), p(50), p(90), p(99))
    }
}

fn write_report<W: Write>(
    mut writer: W,
    players: &PlayerIds,
    experiments: &mut [Experiment],
    last_date_time: UtcDateTime,
) -> io::Result<()> {
    let mut num_encounters = 0;
    let mut total_errors = 0;

    writeln!(
        writer,
        "min_deviation,max_deviation,default_volatility,tau,first_advantage,preview_opponent_deviation,rating_periods_per_day,avg_deviance"
    )?;

    for experiment in experiments.iter() {
        writeln!(
            writer,
            "{},{},{},{},{},{},{},{}",
            f64::from(experiment.rating_system.min_deviation()),
            f64::from(experiment.rating_system.max_deviation()),
            f64::from(experiment.rating_system.default_volatility()),
            experiment.rating_system.tau(),
            f64::from(experiment.rating_system.first_advantage()),
            experiment.rating_system.preview_opponent_deviation(),
            experiment.rating_periods_per_day,
            experiment.avg_deviance()
        )?;

        num_encounters = experiment.total_games; // Not summing
        total_errors += experiment.errors;
    }

    writeln!(writer, "# ---")?;

    let best_experiment = experiments.last().expect("at least one experiment");

    for (speed, name) in [
        (Speed::Blitz, "thibault"),
        (Speed::Blitz, "german11"),
        (Speed::Bullet, "revoof"),
        (Speed::Bullet, "drnykterstein"),
        (Speed::Bullet, "penguingim1"),
        (Speed::Blitz, "lance5500"),
        (Speed::Blitz, "somethingpretentious"),
        (Speed::Classical, "igormezentsev"),
    ] {
        if let Some(rating) = players
            .get(name)
            .and_then(|player_id| best_experiment.leaderboard.get(speed).get(player_id))
        {
            writeln!(
                writer,
                "# Sample {:?} rating of {}: {} (rd: {}, vola: {})",
                speed,
                name,
                f64::from(rating.rating),
                f64::from(rating.deviation),
                f64::from(rating.volatility)
            )?;
        }
    }
    writeln!(writer, "# ---")?;
    for speed in [
        Speed::UltraBullet,
        Speed::Bullet,
        Speed::Blitz,
        Speed::Bullet,
        Speed::Classical,
        Speed::Correspondence,
    ] {
        let (p1, p10, median, p90, p99) = best_experiment.estimate_percentiles(speed);
        let avg = best_experiment.estimate_avg_rating(speed);
        writeln!(
            writer,
            "# Estimated {speed:?} distribution: p1 {p1:.1}, p10 {p10:.1}, median {median:.1}, p90 {p90:.1}, p99 {p99:.1}, avg {avg:.1}",
        )?;
    }
    writeln!(writer, "# ---")?;
    writeln!(writer, "# Distinct players: {}", players.len())?;
    writeln!(
        writer,
        "# Processed encounters: {} (last at: {})",
        num_encounters, last_date_time
    )?;
    writeln!(writer, "# Total errors: {}", total_errors)?;
    writeln!(writer, "# ---")?;

    Ok(())
}

#[derive(clap::Parser)]
struct Opt {
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "45")]
    min_deviation: Vec<f64>,
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "500")]
    max_deviation: Vec<f64>,
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "0.09")]
    default_volatility: Vec<f64>,
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "0.75")]
    tau: Vec<f64>,
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "0")]
    first_advantage: Vec<f64>,
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "0,1")]
    preview_opponent_deviation: Vec<u8>,
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "0.21436")]
    rating_periods_per_day: Vec<f64>,
}

fn main() -> Result<(), Box<dyn StdError>> {
    let opt = Opt::parse();

    let process_uuid = Uuid::now_v7();

    let mut experiments = Vec::new();

    for &min_deviation in &opt.min_deviation {
        for &max_deviation in &opt.max_deviation {
            for &default_volatility in &opt.default_volatility {
                for &tau in &opt.tau {
                    for &first_advantage in &opt.first_advantage {
                        for &preview_opponent_deviation in &opt.preview_opponent_deviation {
                            for &rating_periods_per_day in &opt.rating_periods_per_day {
                                experiments.push(Experiment {
                                    rating_system: RatingSystem::builder()
                                        .rating_regulator_factor(1.0)
                                        .min_rating(RatingScalar(-f64::INFINITY))
                                        .max_rating(RatingScalar(f64::INFINITY))
                                        .min_deviation(RatingDifference(min_deviation))
                                        .max_deviation(RatingDifference(max_deviation))
                                        .default_volatility(Volatility(default_volatility))
                                        .tau(tau)
                                        .first_advantage(RatingDifference(first_advantage))
                                        .preview_opponent_deviation(preview_opponent_deviation != 0)
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

    println!("# Parallel experiments: {}", experiments.len());
    println!("# ---");

    let mut reader = csv::Reader::from_reader(io::stdin().lock());

    let mut players = PlayerIds::default();

    let mut batch = Vec::new();

    let mut process_batch = |batch: &mut Vec<Encounter>,
                             players: &PlayerIds,
                             last_date_time: UtcDateTime,
                             final_batch: bool|
     -> io::Result<()> {
        experiments
            .par_iter_mut()
            .for_each(|experiment| experiment.batch_encounters(batch));

        batch.clear();

        experiments.sort_by_key(Experiment::sort_key);
        write_report(
            File::create(format!(
                "{}report-{}.csv",
                if final_batch { "" } else { "progress-" },
                process_uuid
            ))?,
            players,
            &mut experiments,
            last_date_time,
        )?;
        write_report(io::stdout(), players, &mut experiments, last_date_time)
    };

    let mut last_date_time = UtcDateTime::default();

    for encounter in reader.deserialize() {
        let encounter: RawEncounter = encounter?;
        last_date_time = encounter.utc_date_time;

        batch.push(Encounter {
            white: players.get_or_insert(encounter.white),
            black: players.get_or_insert(encounter.black),
            white_score: match encounter.result.white_score() {
                Some(score) => score,
                None => continue,
            },
            speed: encounter.time_control.speed(),
            utc_date_time: encounter.utc_date_time,
        });

        if batch.len() >= 100_000 {
            process_batch(&mut batch, &players, last_date_time, false)?;
        }
    }

    process_batch(&mut batch, &players, last_date_time, true)?;

    Ok(())
}
