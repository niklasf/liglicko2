/// An instant in time. A difference of 1.0 represents a *rating period* in
/// Glicko2 terminology.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Instant(pub f64);

impl From<Instant> for f64 {
    fn from(Instant(instant): Instant) -> f64 {
        instant
    }
}

/// A score or expectation value in the range `0.0..=1.0`, where `0.0` is a
/// loss, `1.0` is a win.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Score(pub f64);

impl From<Score> for f64 {
    fn from(Score(score): Score) -> f64 {
        score
    }
}

#[derive(Debug, Clone)]
pub struct Rating {
    pub rating: f64,
    pub last_deviation: f64,
    pub volatility: f64,
    pub updated_at: Option<Instant>,
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
    pub fn default_rating(&self) -> Rating {
        Rating {
            rating: self.default_rating,
            last_deviation: self.max_deviation,
            volatility: self.default_volatility,
            updated_at: None,
        }
    }

    pub fn expected_score(&self, first: &Rating, second: &Rating) -> Score {
        todo!()
    }

    pub fn update_ratings(
        &self,
        first: &Rating,
        second: &Rating,
        score: Score,
    ) -> (Rating, Rating) {
        todo!()
    }

    pub fn tau(&self) -> f64 {
        self.tau
    }

    pub fn min_deviation(&self) -> f64 {
        self.min_deviation
    }

    pub fn max_deviation(&self) -> f64 {
        self.max_deviation
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
            Some(Instant(updated_at)) => at - updated_at,
            None => 0.0,
        };

        (self.last_deviation.powi(2) + elapsed_periods * rating_system.tau.powi(2))
            .sqrt()
            .clamp(rating_system.min_deviation, rating_system.max_deviation)
    }
}
