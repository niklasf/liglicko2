use std::f64::consts::PI;

mod instant;
mod score;
mod public;

pub use instant::Instant;
pub use score::Score;

#[derive(Debug, Clone)]
pub struct Rating {
    pub rating: f64,
    pub deviation: f64,
    pub volatility: f64,
    pub at: Instant,
}

impl Rating {
    pub fn at(&self, rating_system: &RatingSystem, at: Instant) -> Rating {
        Rating::from(InternalRating::from(self).at(rating_system, at))
    }

    pub fn deviation(&self, rating_system: &RatingSystem, at: Instant) -> f64 {
        self.at(rating_system, at).last_deviation
    }
}

#[derive(Debug, Clone)]
pub struct RatingSystem {
    min_rating: f64,
    max_rating: f64,

    default_rating: f64,
    default_volatility: f64,

    min_deviation: f64,
    max_deviation: f64,

    first_advantage: f64,

    tau: f64,
}

impl Default for RatingSystem {
    fn default() -> RatingSystem {
        RatingSystem {
            min_rating: 400.0,
            max_rating: 4000.0,

            default_rating: 1500.0,
            default_volatility: 0.09,

            min_deviation: 0.0,
            max_deviation: 500.0,

            first_advantage: 0.0,

            tau: 0.75,
        }
    }
}

impl RatingSystem {
    pub fn new_rating(&self, at: Instant) -> Rating {
        Rating {
            rating: self.default_rating,
            deviation: self.max_deviation,
            volatility: self.default_volatility,
            at,
        }
    }

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

    pub fn tau(&self) -> f64 {
        self.tau
    }

    pub fn min_deviation(&self) -> f64 {
        self.min_deviation
    }

    fn min_internal_deviation(&self) -> f64 {
        self.min_deviation / INTERNAL_RATING_SCALE
    }

    pub fn max_deviation(&self) -> f64 {
        self.max_deviation
    }

    fn max_internal_deviation(&self) -> f64 {
        self.max_deviation / INTERNAL_RATING_SCALE
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
