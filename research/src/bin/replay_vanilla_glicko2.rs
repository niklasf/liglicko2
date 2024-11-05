use std::{error::Error as StdError, f64::consts::PI, io};

use compensated_summation::KahanBabuskaNeumaier;
use glicko2::{GameResult, Glicko2Rating};
use liglicko2::{deviance, Score};
use liglicko2_research::{
    encounter::{BySpeed, PgnResult, RawEncounter, UtcDateTime},
    player::{ByPlayerId, PlayerIds},
};

#[derive(Debug, Default)]
struct PlayerState {
    rating: Glicko2Rating,
    pending: Vec<GameResult>,
}

impl PlayerState {
    fn live_rating(&self) -> Glicko2Rating {
        let unbounded = glicko2::new_rating(self.rating, &self.pending, 0.2);
        Glicko2Rating {
            value: unbounded.value.clamp((400.0 - 1500.0) / 173.7178, (4000.0 - 1500.0) / 173.7178),
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

fn main() -> Result<(), Box<dyn StdError>> {
    let mut reader = csv::Reader::from_reader(io::stdin().lock());

    let mut players = PlayerIds::default();
    let mut states: BySpeed<ByPlayerId<PlayerState>> = BySpeed::default();
    let mut last_rating_period = UtcDateTime::default();
    let mut total_encounters: u64 = 0;
    let mut total_deviance = KahanBabuskaNeumaier::default();

    for encounter in reader.deserialize() {
        let encounter: RawEncounter = encounter?;

        // Commit rating period
        if encounter.utc_date_time.as_seconds() > last_rating_period.as_seconds() + 24 * 60 * 60
        {
            for states in states.values_mut() {
                for state in states.values_mut() {
                    if let Some(state) = state {
                        state.commit();
                    }
                }
            }
            last_rating_period = encounter.utc_date_time; // Close enough, because encounters are dense
            println!(
                "Rating period ending at {}: avg deviance {} over {} encounters",
                last_rating_period,
                total_deviance.total() / total_encounters as f64,
                total_encounters
            );
        }

        // Update deviance using live ratings
        let white = players.get_or_insert(encounter.white);
        let black = players.get_or_insert(encounter.black);
        let states = states.get_mut(encounter.time_control.speed());

        total_deviance += deviance(
            expectation_value(
                states
                    .get(white)
                    .map_or_else(Glicko2Rating::unrated, |state| state.live_rating()),
                states
                    .get(black)
                    .map_or_else(Glicko2Rating::unrated, |state| state.live_rating()),
            ),
            if let Some(actual) = encounter.result.white_score() {
                actual
            } else {
                continue;
            },
        );
        total_encounters += 1;

        // Record game result as pending in rating period
        let white_rating = states
            .get(white)
            .map_or_else(Glicko2Rating::unrated, |state| state.rating);
        let black_rating = states
            .get(black)
            .map_or_else(Glicko2Rating::unrated, |state| state.rating);

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

    println!(
        "Final result: avg deviance {} over {} encounters",
        total_deviance.total() / total_encounters as f64,
        total_encounters
    );

    Ok(())
}
