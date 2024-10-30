use crate::Instant;
use crate::Score;
use crate::{
    internal_rating::InternalRatingDifference,
    rating::{Rating, RatingDifference, RatingScalar, Volatility},
};
use std::f64::consts::PI;

#[derive(Debug, Clone)]
pub struct RatingSystemBuilder {
    min_rating: RatingScalar,
    max_rating: RatingScalar,

    default_rating: RatingScalar,
    default_volatility: Volatility,

    min_deviation: RatingDifference,
    max_deviation: RatingDifference,

    first_advantage: RatingDifference,

    tau: f64,
}

impl Default for RatingSystemBuilder {
    fn default() -> RatingSystemBuilder {
        RatingSystemBuilder::new()
    }
}

impl RatingSystemBuilder {
    pub fn new() -> RatingSystemBuilder {
        RatingSystemBuilder {
            min_rating: RatingScalar(400.0),
            max_rating: RatingScalar(4000.0),

            default_rating: RatingScalar(1500.0),
            default_volatility: Volatility(0.09),

            min_deviation: RatingDifference(45.0),
            max_deviation: RatingDifference(500.0),

            first_advantage: RatingDifference(0.0),

            tau: 0.75,
        }
    }

    pub fn min_rating(&mut self, min_rating: RatingScalar) -> &mut Self {
        assert!(!f64::from(min_rating).is_nan());
        self.min_rating = min_rating;
        self
    }

    pub fn max_rating(&mut self, max_rating: RatingScalar) -> &mut Self {
        assert!(!f64::from(max_rating).is_nan());
        self.max_rating = max_rating;
        self
    }

    pub fn default_rating(&mut self, default_rating: RatingScalar) -> &mut Self {
        self.default_rating = default_rating;
        self
    }

    pub fn default_volatility(&mut self, default_volatility: Volatility) -> &mut Self {
        assert!(default_volatility >= Volatility(0.0));
        self.default_volatility = default_volatility;
        self
    }

    pub fn min_deviation(&mut self, min_deviation: RatingDifference) -> &mut Self {
        assert!(min_deviation >= RatingDifference(0.0));
        self.min_deviation = min_deviation;
        self
    }

    pub fn max_deviation(&mut self, max_deviation: RatingDifference) -> &mut Self {
        assert!(!f64::from(max_deviation).is_nan());
        self.max_deviation = max_deviation;
        self
    }

    pub fn first_advantage(&mut self, first_advantage: RatingDifference) -> &mut Self {
        self.first_advantage = first_advantage;
        self
    }

    pub fn tau(&mut self, tau: f64) -> &mut Self {
        assert!(tau >= 0.0);
        self.tau = tau;
        self
    }

    pub fn build(&self) -> RatingSystem {
        assert!(self.min_rating <= self.max_rating);
        assert!(self.min_deviation <= self.max_deviation);

        RatingSystem {
            min_rating: self.min_rating,
            max_rating: self.max_rating,

            default_rating: self.default_rating,
            default_volatility: self.default_volatility,

            min_deviation: self.min_deviation,
            max_deviation: self.max_deviation,

            first_advantage: self.first_advantage,

            tau: self.tau,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RatingSystem {
    min_rating: RatingScalar,
    max_rating: RatingScalar,

    default_rating: RatingScalar,
    default_volatility: Volatility,

    min_deviation: RatingDifference,
    max_deviation: RatingDifference,

    first_advantage: RatingDifference,

    tau: f64,
}

impl Default for RatingSystem {
    fn default() -> RatingSystem {
        RatingSystem::new()
    }
}

impl RatingSystem {
    pub fn builder() -> RatingSystemBuilder {
        RatingSystemBuilder::default()
    }

    pub fn new() -> RatingSystem {
        RatingSystem::builder().build()
    }

    pub fn min_rating(&self) -> RatingScalar {
        self.min_rating
    }

    pub fn max_rating(&self) -> RatingScalar {
        self.max_rating
    }

    pub fn default_rating(&self) -> RatingScalar {
        self.default_rating
    }

    pub fn default_volatility(&self) -> Volatility {
        self.default_volatility
    }

    pub fn min_deviation(&self) -> RatingDifference {
        self.min_deviation
    }

    pub fn max_deviation(&self) -> RatingDifference {
        self.max_deviation
    }

    pub fn first_advantage(&self) -> RatingDifference {
        self.first_advantage
    }

    pub fn tau(&self) -> f64 {
        self.tau
    }

    pub fn initial_rating(&self) -> Rating {
        Rating {
            rating: self.default_rating.clamp(self.min_rating, self.max_rating),
            deviation: self.max_deviation,
            volatility: self.default_volatility,
            at: Instant::default(),
        }
    }

    pub fn preview_deviation(&self, rating: &Rating, now: Instant) -> RatingDifference {
        RatingDifference::from(new_deviation(
            rating.deviation.into(),
            rating.volatility,
            f64::max(now.elapsed_periods_since(rating.at), 0.0),
        ))
        .clamp(self.min_deviation, self.max_deviation)
    }

    pub fn expected_score(&self, first: &Rating, second: &Rating, now: Instant) -> Score {
        expectation_value(
            (first.rating + self.first_advantage - second.rating).into(),
            self.preview_deviation(second, now).into()
        )
    }
}

fn g(InternalRatingDifference(deviation): InternalRatingDifference) -> f64 {
    1.0 / (1.0 + 3.0 * deviation.powi(2) / PI.powi(2)).sqrt()
}

fn expectation_value(
    InternalRatingDifference(our_advantage): InternalRatingDifference,
    their_deviation: InternalRatingDifference,
) -> Score {
    Score(1.0 / (1.0 + f64::exp(-g(their_deviation) * our_advantage)))
}

fn new_deviation(
    InternalRatingDifference(deviation): InternalRatingDifference,
    Volatility(volatility): Volatility,
    elapsed_periods: f64,
) -> InternalRatingDifference {
    InternalRatingDifference(f64::sqrt(
        deviation.powi(2) + elapsed_periods * volatility.powi(2),
    ))
}
