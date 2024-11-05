use std::{error::Error as StdError, fs::File, io, io::Write};

use clap::Parser as _;
use compensated_summation::KahanBabuskaNeumaier;
use liglicko2::{
    deviance, Instant, Rating, RatingDifference, RatingScalar, RatingSystem, Score, Volatility,
};
use liglicko2_research::{
    encounter::{BySpeed, RawEncounter, Speed, UtcDateTime},
    player::{ByPlayerId, PlayerId, PlayerIds},
};
use ordered_float::OrderedFloat;
use rayon::prelude::*;
use uuid::Uuid;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

struct Encounter {
    white: PlayerId,
    black: PlayerId,
    white_score: Score,
    utc_date_time: UtcDateTime,
    speed: Speed,
}

#[derive(Default, Clone)]
struct Wdl {
    wins: u64,
    draws: u64,
    losses: u64,
}

#[derive(Default)]
struct DeviationHistogram {
    buckets: Vec<Wdl>,
}

impl DeviationHistogram {
    pub fn record(&mut self, deviation: RatingDifference, score: Score) {
        let bucket = f64::from(deviation).round() as usize;
        if self.buckets.len() <= bucket {
            self.buckets.resize_with(bucket + 1, Wdl::default);
        }
        match score {
            Score::WIN => self.buckets[bucket].wins += 1,
            Score::DRAW => self.buckets[bucket].draws += 1,
            Score::LOSS => self.buckets[bucket].losses += 1,
            _ => panic!("bad score {score:?}"),
        }
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
    deviation_histogram: DeviationHistogram,
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

        self.deviation_histogram
            .record(white.deviation, encounter.white_score);
        self.deviation_histogram
            .record(black.deviation, encounter.white_score.opposite());

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

    fn estimate_avg_rating(&self, speed: Speed, at: Instant) -> f64 {
        let mut total_rating = KahanBabuskaNeumaier::default();
        let mut num_ratings: u64 = 0;

        let table = self.leaderboard.get(speed).values();
        let mut i = 0;
        while i < table.len() {
            if let Some(rating) = &table[i] {
                if self.rating_system.preview_deviation(rating, at) < RatingDifference(60.0) {
                    total_rating += f64::from(rating.rating);
                    num_ratings += 1;
                }
            }
            i += 1 + table.len() / 100_000;
        }

        total_rating.total() / num_ratings as f64
    }

    fn estimate_percentiles(&self, speed: Speed, at: Instant) -> (f64, f64, f64, f64, f64) {
        let mut samples = Vec::new();

        let table = self.leaderboard.get(speed).values();
        let mut i = 0;
        while i < table.len() {
            if let Some(rating) = &table[i] {
                if self.rating_system.preview_deviation(rating, at) < RatingDifference(60.0) {
                    samples.push(OrderedFloat(f64::from(rating.rating)));
                }
            }
            i += 1 + table.len() / 100_000;
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
        "min_deviation,max_deviation,default_volatility,tau,first_advantage,rating_periods_per_day,avg_deviance"
    )?;

    for experiment in experiments.iter() {
        writeln!(
            writer,
            "{},{},{},{},{},{},{:.6}",
            f64::from(experiment.rating_system.min_deviation()),
            f64::from(experiment.rating_system.max_deviation()),
            f64::from(experiment.rating_system.default_volatility()),
            experiment.rating_system.tau(),
            f64::from(experiment.rating_system.first_advantage()),
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
        (Speed::Blitz, "tbest"),
        (Speed::Classical, "igormezentsev"),
    ] {
        if let Some(rating) = players
            .get(name)
            .and_then(|player_id| best_experiment.leaderboard.get(speed).get(player_id))
        {
            writeln!(
                writer,
                "# Sample {:?} rating of {}: {:.1} (rd: {:.3}, vola: {:.5})",
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
        Speed::Rapid,
        Speed::Classical,
        Speed::Correspondence,
    ] {
        let (p1, p10, median, p90, p99) =
            best_experiment.estimate_percentiles(speed, best_experiment.to_instant(last_date_time));
        let avg =
            best_experiment.estimate_avg_rating(speed, best_experiment.to_instant(last_date_time));
        writeln!(
            writer,
            "# Estimated {speed:?} distribution: p1={p1:.1} p10={p10:.1} p50={median:.1} p90={p90:.1} p99={p99:.1}, avg={avg:.1}",
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
    #[clap(long, value_delimiter = ',', num_args = 1.., default_value = "0.21436")]
    rating_periods_per_day: Vec<f64>,

    #[clap(long, default_value = "1.02")]
    regulator_factor: f64,
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
                        for &rating_periods_per_day in &opt.rating_periods_per_day {
                            experiments.push(Experiment {
                                rating_system: RatingSystem::builder()
                                    .min_rating(RatingScalar(-f64::INFINITY))
                                    .max_rating(RatingScalar(f64::INFINITY))
                                    .regulator_factor(opt.regulator_factor)
                                    .min_deviation(RatingDifference(min_deviation))
                                    .max_deviation(RatingDifference(max_deviation))
                                    .default_volatility(Volatility(default_volatility))
                                    .tau(tau)
                                    .first_advantage(RatingDifference(first_advantage))
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
        // Process batch
        experiments
            .par_iter_mut()
            .for_each(|experiment| experiment.batch_encounters(batch));

        batch.clear();

        // Dump report
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
        write_report(io::stdout(), players, &mut experiments, last_date_time)?;

        // Dump deviation histogram for best experiment
        let best_experiment = experiments.last().expect("at least one experiment");
        let mut deviation_histogram_file = File::create(format!(
            "{}deviation-histogram-{}.csv",
            if final_batch { "" } else { "progress-" },
            process_uuid
        ))?;
        writeln!(deviation_histogram_file, "deviation,wins,draws,losses")?;
        for (deviation, wdl) in best_experiment
            .deviation_histogram
            .buckets
            .iter()
            .enumerate()
        {
            writeln!(
                deviation_histogram_file,
                "{},{},{},{}",
                deviation, wdl.wins, wdl.draws, wdl.losses
            )?;
        }

        Ok(())
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

        if batch.len() >= 1_000_000 {
            process_batch(&mut batch, &players, last_date_time, false)?;
        }
    }

    process_batch(&mut batch, &players, last_date_time, true)?;

    Ok(())
}
