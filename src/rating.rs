use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use crate::{internal_rating::InternalRatingDifference, Instant};

/// Number representing playing strength, such that the difference between two
/// ratings can be used to predict an expected score. Higher is better.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct RatingScalar(pub f64);

impl From<RatingScalar> for f64 {
    #[inline]
    fn from(RatingScalar(rating): RatingScalar) -> f64 {
        rating
    }
}

impl From<f64> for RatingScalar {
    #[inline]
    fn from(rating: f64) -> RatingScalar {
        RatingScalar(rating)
    }
}

impl RatingScalar {
    #[must_use]
    #[inline]
    pub fn clamp(self, min: RatingScalar, max: RatingScalar) -> RatingScalar {
        RatingScalar(self.0.clamp(min.0, max.0))
    }
}

impl Sub<RatingScalar> for RatingScalar {
    type Output = RatingDifference;

    #[inline]
    fn sub(self, rhs: RatingScalar) -> RatingDifference {
        RatingDifference(self.0 - rhs.0)
    }
}

impl Add<RatingDifference> for RatingScalar {
    type Output = RatingScalar;

    #[inline]
    fn add(self, RatingDifference(difference): RatingDifference) -> RatingScalar {
        RatingScalar(self.0 + difference)
    }
}

impl AddAssign<RatingDifference> for RatingScalar {
    #[inline]
    fn add_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 += difference;
    }
}

impl Sub<RatingDifference> for RatingScalar {
    type Output = RatingScalar;

    #[inline]
    fn sub(self, RatingDifference(difference): RatingDifference) -> RatingScalar {
        RatingScalar(self.0 - difference)
    }
}

impl SubAssign<RatingDifference> for RatingScalar {
    #[inline]
    fn sub_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 -= difference;
    }
}

/// A difference between two ratings.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct RatingDifference(pub f64);

impl From<RatingDifference> for f64 {
    #[inline]
    fn from(RatingDifference(difference): RatingDifference) -> f64 {
        difference
    }
}

impl From<f64> for RatingDifference {
    #[inline]
    fn from(difference: f64) -> RatingDifference {
        RatingDifference(difference)
    }
}

impl RatingDifference {
    #[must_use]
    #[inline]
    pub fn clamp(self, min: RatingDifference, max: RatingDifference) -> RatingDifference {
        RatingDifference(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub fn abs(self) -> RatingDifference {
        RatingDifference(self.0.abs())
    }

    #[inline]
    pub(crate) fn to_internal(self) -> InternalRatingDifference {
        InternalRatingDifference::from_external(self)
    }
}

impl Add<RatingDifference> for RatingDifference {
    type Output = RatingDifference;

    #[inline]
    fn add(self, RatingDifference(difference): RatingDifference) -> RatingDifference {
        RatingDifference(self.0 + difference)
    }
}

impl AddAssign<RatingDifference> for RatingDifference {
    #[inline]
    fn add_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 += difference;
    }
}

impl Sub<RatingDifference> for RatingDifference {
    type Output = RatingDifference;

    #[inline]
    fn sub(self, RatingDifference(difference): RatingDifference) -> RatingDifference {
        RatingDifference(self.0 - difference)
    }
}

impl SubAssign<RatingDifference> for RatingDifference {
    #[inline]
    fn sub_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 -= difference;
    }
}

impl Mul<f64> for RatingDifference {
    type Output = RatingDifference;

    #[inline]
    fn mul(self, scalar: f64) -> RatingDifference {
        RatingDifference(self.0 * scalar)
    }
}

impl Mul<RatingDifference> for f64 {
    type Output = RatingDifference;

    #[inline]
    fn mul(self, RatingDifference(difference): RatingDifference) -> RatingDifference {
        RatingDifference(self * difference)
    }
}

impl MulAssign<f64> for RatingDifference {
    #[inline]
    fn mul_assign(&mut self, scalar: f64) {
        self.0 *= scalar;
    }
}

impl Div<f64> for RatingDifference {
    type Output = RatingDifference;

    #[inline]
    fn div(self, scalar: f64) -> RatingDifference {
        RatingDifference(self.0 / scalar)
    }
}

impl DivAssign<f64> for RatingDifference {
    #[inline]
    fn div_assign(&mut self, scalar: f64) {
        self.0 /= scalar;
    }
}

impl Neg for RatingDifference {
    type Output = RatingDifference;

    #[inline]
    fn neg(self) -> RatingDifference {
        RatingDifference(-self.0)
    }
}

/// Number representing the degree of expected fluctuation in a rating.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Volatility(pub f64);

impl Volatility {
    #[must_use]
    #[inline]
    pub fn clamp(self, min: Volatility, max: Volatility) -> Volatility {
        Volatility(self.0.clamp(min.0, max.0))
    }

    #[inline]
    pub(crate) fn sq(self) -> f64 {
        self.0 * self.0
    }
}

impl From<Volatility> for f64 {
    #[inline]
    fn from(Volatility(volatility): Volatility) -> f64 {
        volatility
    }
}

impl From<f64> for Volatility {
    #[inline]
    fn from(volatility: f64) -> Volatility {
        Volatility(volatility)
    }
}

/// A rating at a specific point in time.
#[derive(Debug, Clone, PartialEq)]
pub struct Rating {
    /// Number indicating playing strength. Higher is better. The difference
    /// between two ratings determines the expected score in a game between
    /// the two players.
    pub rating: RatingScalar,
    /// Uncertainty in the rating. A range from rating minus twice the deviation
    /// to rating plus twice the deviation approximately represents a 95%
    /// confidence interval.
    pub deviation: RatingDifference,
    /// Number indicating the degree of expected fluctuation in the rating.
    pub volatility: Volatility,
    /// Point in time at which the rating was last updated.
    pub at: Instant,
}
