use std::default;

use crate::Rating;

#[derive(Debug, Clone)]
pub struct RatingSystemBuilder {
    min_rating: f64,
    max_rating: f64,

    default_rating: f64,
    default_volatility: f64,

    min_deviation: f64,
    max_deviation: f64,

    first_advantage: f64,

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

    pub fn min_rating(&mut self, min_rating: f64) -> &mut Self {
        assert!(!min_rating.is_nan());
        self.min_rating = min_rating;
        self
    }

    pub fn max_rating(&mut self, max_rating: f64) -> &mut Self {
        assert!(!max_rating.is_nan());
        self.max_rating = max_rating;
        self
    }

    pub fn default_rating(&mut self, default_rating: f64) -> &mut Self {
        self.default_rating = default_rating;
        self
    }

    pub fn default_volatility(&mut self, default_volatility: f64) -> &mut Self {
        assert!(default_volatility >= 0.0);
        self.default_volatility = default_volatility;
        self
    }

    pub fn min_deviation(&mut self, min_deviation: f64) -> &mut Self {
        assert!(min_deviation >= 0.0);
        self.min_deviation = min_deviation;
        self
    }

    pub fn max_deviation(&mut self, max_deviation: f64) -> &mut Self {
        assert!(!max_deviation.is_nan());
        self.max_deviation = max_deviation;
        self
    }

    pub fn first_advantage(&mut self, first_advantage: f64) -> &mut Self {
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

    pub fn min_rating(&self) -> f64 {
        self.min_rating
    }

    pub fn max_rating(&self) -> f64 {
        self.max_rating
    }

    pub fn default_rating(&self) -> f64 {
        self.default_rating
    }

    pub fn default_volatility(&self) -> f64 {
        self.default_volatility
    }

    pub fn min_deviation(&self) -> f64 {
        self.min_deviation
    }

    pub fn max_deviation(&self) -> f64 {
        self.max_deviation
    }

    pub fn first_advantage(&self) -> f64 {
        self.first_advantage
    }

    pub fn tau(&self) -> f64 {
        self.tau
    }
}