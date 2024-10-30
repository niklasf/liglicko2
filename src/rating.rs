use crate::Instant;

/// Number representing playing strength, such that the difference between two
/// ratings can be used to predict an expected score. Higher is better.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct RatingScalar(pub f64);

impl From<RatingScalar> for f64 {
    fn from(RatingScalar(rating): RatingScalar) -> f64 {
        rating
    }
}

/// A difference between two ratings.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct RatingDifference(pub f64);

impl From<RatingDifference> for f64 {
    fn from(RatingDifference(difference): RatingDifference) -> f64 {
        difference
    }
}

/// Number representing the degree of expected fluctuation in a rating.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Volatility(pub f64);

/// A rating at a specific point in time.
#[derive(Debug, Clone)]
pub struct Rating {
    pub rating: RatingScalar,
    pub deviation: RatingDifference,
    pub volatility: Volatility,
    pub at: Instant,
}