use std::{error::Error as StdError, f64::consts::PI, io};

use compensated_summation::KahanBabuskaNeumaier;
use glicko2::{GameResult, Glicko2Rating};
use liglicko2::{deviance, Score};
use liglicko2_research::{
    encounter::{BySpeed, PgnResult, RawEncounter, UtcDateTime},
    player::{ByPlayerId, PlayerIds},
};
use ordered_float::OrderedFloat;

#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const WHITE_ADVANTAGE: f64 = 0.0; // 12.0 / 173.7178

#[derive(Debug, Default)]
struct PlayerState {
    rating: Glicko2Rating,
    pending: Vec<GameResult>,
}

impl PlayerState {
    fn live_rating(&self) -> Glicko2Rating {
        let unbounded =
            glicko2::new_rating(self.rating, &self.pending, 0.2).unwrap_or_else(|err| {
                eprintln!("{}: {:?}", err, self);
                Glicko2Rating::unrated()
            });

        Glicko2Rating {
            value: unbounded.value,
            deviation: unbounded.deviation.clamp(30.0 / 173.7178, 350.0 / 173.7178),
            volatility: unbounded.volatility.clamp(0.01, 0.1),
        }
    }

    fn commit(&mut self) {
        self.rating = self.live_rating();
        self.pending.clear();
    }
}

fn expectation_value(white: Glicko2Rating, black: Glicko2Rating) -> Score {
    Score(
        1.0 / (1.0
            + f64::exp(
                -g(f64::hypot(white.deviation, black.deviation)) * (white.value - black.value),
            )),
    )
}

fn g(deviation: f64) -> f64 {
    1.0 / f64::sqrt(1.0 + 3.0 * deviation.powi(2) / PI.powi(2))
}

fn with_offset(rating: Glicko2Rating, offset: f64) -> Glicko2Rating {
    Glicko2Rating {
        value: rating.value + offset,
        ..rating
    }
}

#[derive(Default)]
struct Stats {
    values: Vec<f64>,
}

impl Stats {
    pub fn add(&mut self, value: f64) {
        self.values.push(value);
    }

    pub fn prepare(&mut self) {
        self.values.sort_by_key(|&value| OrderedFloat(value));
    }

    pub fn mean(&self) -> f64 {
        let mut sum = KahanBabuskaNeumaier::default();
        for &value in &self.values {
            sum += value;
        }
        sum.total() / self.values.len() as f64
    }

    pub fn percentile(&self, percentile: usize) -> f64 {
        let index = self.values.len() * percentile / 100;
        self.values
            .get(index)
            .copied()
            .unwrap_or_else(|| self.values.last().copied().unwrap_or(f64::NAN))
    }

    pub fn csv_header(prefix: &str) -> String {
        format!("{prefix}_mean,{prefix}_p0,{prefix}_p10,{prefix}_p20,{prefix}_p30,{prefix}_p40,{prefix}_p50,{prefix}_p60,{prefix}_p70,{prefix}_p80,{prefix}_p90,{prefix}_p100")
    }

    pub fn csv(&self) -> String {
        format!(
            "{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6},{:.6}",
            self.mean(),
            self.percentile(0),
            self.percentile(10),
            self.percentile(20),
            self.percentile(30),
            self.percentile(40),
            self.percentile(50),
            self.percentile(60),
            self.percentile(70),
            self.percentile(80),
            self.percentile(90),
            self.percentile(100),
        )
    }
}

fn main() -> Result<(), Box<dyn StdError>> {
    let mut reader = csv::Reader::from_reader(io::stdin().lock());

    let mut players = PlayerIds::default();
    let mut states: BySpeed<ByPlayerId<PlayerState>> = BySpeed::default();
    let mut last_rating_period = UtcDateTime::default();
    let mut total_encounters: u64 = 0;
    let mut total_deviance = KahanBabuskaNeumaier::default();

    println!(
        "rating_period,avg_deviance,encounters,players,{},{},{}",
        Stats::csv_header("rating"),
        Stats::csv_header("deviation"),
        Stats::csv_header("volatility")
    );

    for encounter in reader.deserialize() {
        let encounter: RawEncounter = encounter?;
        let speed = encounter.time_control.speed();

        // Commit rating period
        if encounter.utc_date_time.as_seconds() > last_rating_period.as_seconds() + 7 * 24 * 60 * 60
        {
            let mut rating_stats = Stats::default();
            let mut deviation_stats = Stats::default();
            let mut volatility_stats = Stats::default();

            for states in states.values_mut() {
                for state in states.values_mut() {
                    if let Some(state) = state {
                        state.commit();

                        rating_stats.add(state.rating.value);
                        deviation_stats.add(state.rating.deviation);
                        volatility_stats.add(state.rating.volatility);
                    }
                }
            }

            last_rating_period = encounter.utc_date_time; // Close enough, because encounters are dense

            rating_stats.prepare();
            deviation_stats.prepare();
            volatility_stats.prepare();

            println!(
                "{},{:.6},{},{},{},{},{}",
                last_rating_period,
                total_deviance.total() / total_encounters as f64,
                total_encounters,
                players.len(),
                rating_stats.csv(),
                deviation_stats.csv(),
                volatility_stats.csv(),
            );
        }

        // Update deviance using live ratings
        let white = players.get_or_insert(encounter.white);
        let black = players.get_or_insert(encounter.black);
        let states = states.get_mut(speed);

        total_deviance += deviance(
            expectation_value(
                states
                    .get(white)
                    .map_or_else(Glicko2Rating::unrated, |state| state.live_rating()),
                with_offset(
                    states
                        .get(black)
                        .map_or_else(Glicko2Rating::unrated, |state| state.live_rating()),
                    -WHITE_ADVANTAGE,
                ),
            ),
            if let Some(actual) = encounter.result.white_score() {
                actual
            } else {
                continue;
            },
        );
        total_encounters += 1;

        // Record game result as pending in rating period
        let white_rating = with_offset(
            states
                .get(white)
                .map_or_else(Glicko2Rating::unrated, |state| state.rating),
            WHITE_ADVANTAGE,
        );
        let black_rating = with_offset(
            states
                .get(black)
                .map_or_else(Glicko2Rating::unrated, |state| state.rating),
            -WHITE_ADVANTAGE,
        );

        states
            .get_mut_or_insert_with(white, PlayerState::default)
            .pending
            .push(match encounter.result {
                PgnResult::WhiteWins => GameResult::win(black_rating),
                PgnResult::BlackWins => GameResult::loss(black_rating),
                PgnResult::Draw => GameResult::draw(black_rating),
                PgnResult::Unknown => continue,
            });

        states
            .get_mut_or_insert_with(black, PlayerState::default)
            .pending
            .push(match encounter.result {
                PgnResult::WhiteWins => GameResult::loss(white_rating),
                PgnResult::BlackWins => GameResult::win(white_rating),
                PgnResult::Draw => GameResult::draw(white_rating),
                PgnResult::Unknown => continue,
            });
    }

    eprintln!(
        "Final result: avg deviance {:.6} over {} encounters",
        total_deviance.total() / total_encounters as f64,
        total_encounters
    );

    Ok(())
}
