use std::f64::consts::PI;

mod instant;
mod score;
mod rating_system;
mod rating;

pub use instant::Instant;
pub use score::Score;
pub use rating::Rating;
pub use rating_system::{RatingSystemBuilder, RatingSystem};

struct InternalRatingSystem {
    first_advantage: f64,
}


impl RatingSystem {
    pub fn expected_score(&self, first: &Rating, second: &Rating, at: Instant) -> Score {
        Score(expectation_value(
            &InternalRating::from(first).at(self, at),
            &InternalRating::from(second).at(self, at),
            self.first_advantage / INTERNAL_RATING_SCALE,
        ))
    }

    pub fn update_ratings(
        &self,
        first: &Rating,
        second: &Rating,
        score: Score,
        at: Instant,
    ) -> (Rating, Rating) {
        let first = InternalRating::from(first).at(self, at);
        let second = InternalRating::from(second).at(self, at);
        (Rating::from(first.update(self, score, &second, self.first_advantage / INTERNAL_RATING_SCALE)),
         Rating::from(second.update(self, score.opposite(), &first, -self.first_advantage / INTERNAL_RATING_SCALE)))
    }
}

/// Log likelihood deviance metric that can be used to evaluate the quality of
/// rating system predictions.
///
/// Lower is better.
///
/// See https://www.kaggle.com/c/ChessRatings2/overview/evaluation.
pub fn deviance(Score(expected): Score, Score(actual): Score) -> f64 {
    let expected = expected.clamp(0.01, 0.99);
    -(actual * expected.log10() + (1.0 - actual) * (1.0 - expected).log10())
}

const INTERNAL_RATING_SCALE: f64 = 173.7178;

struct InternalRating {
    rating: f64,
    last_deviation: f64,
    volatility: f64,
    updated_at: Option<Instant>,
}

impl From<&Rating> for InternalRating {
    fn from(rating: &Rating) -> InternalRating {
        InternalRating {
            rating: rating.rating / INTERNAL_RATING_SCALE,
            last_deviation: rating.last_deviation / INTERNAL_RATING_SCALE,
            volatility: rating.volatility,
            updated_at: rating.updated_at,
        }
    }
}

impl From<InternalRating> for Rating {
    fn from(rating: InternalRating) -> Rating {
        Rating {
            rating: rating.rating * INTERNAL_RATING_SCALE,
            last_deviation: rating.last_deviation * INTERNAL_RATING_SCALE,
            volatility: rating.volatility,
            updated_at: rating.updated_at,
        }
    }
}

impl InternalRating {
    fn at(&self, rating_system: &RatingSystem, at: Instant) -> InternalRating {
        InternalRating {
            last_deviation: self.deviation(rating_system, at),
            updated_at: Some(at),
            ..*self
        }
    }

    fn deviation(&self, rating_system: &RatingSystem, Instant(at): Instant) -> f64 {
        let elapsed_periods = match self.updated_at {
            Some(Instant(updated_at)) if at > updated_at => at - updated_at,
            _ => 0.0,
        };

        (self.last_deviation.powi(2) + elapsed_periods * rating_system.tau.powi(2))
            .sqrt()
            .clamp(rating_system.min_internal_deviation(), rating_system.max_internal_deviation())
    }

    fn update(&self, rating_system: &RatingSystem, score: Score, them: &InternalRating, our_advantage: f64) -> InternalRating {
        // Step 3
        let g_them = g(them.last_deviation);
        let e = expectation_value(self, them, our_advantage);
        let v = 1.0 / (g_them.powi(2) * e * (1.0 - e));

        // Step 4
        let delta = v * g_them * (f64::from(score) - e);
    }
}

fn g(deviation: f64) -> f64 {
    1.0 / (1.0 + 3.0 * deviation.powi(2) / PI.powi(2)).sqrt()
}

fn expectation_value(first: &InternalRating, second: &InternalRating, internal_first_advantage: f64) -> f64 {
    1.0 / (1.0 + (-g(second.last_deviation) * (first.rating + internal_first_advantage - second.rating)).exp())
}
