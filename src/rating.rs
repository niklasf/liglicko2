use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign, Neg};

use crate::{internal_rating::InternalRatingDifference, Instant};

/// Number representing playing strength, such that the difference between two
/// ratings can be used to predict an expected score. Higher is better.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct RatingScalar(pub f64);

impl RatingScalar {
    pub fn clamp(self, RatingScalar(min): RatingScalar, RatingScalar(max): RatingScalar) -> RatingScalar {
        RatingScalar(f64::from(self).clamp(min, max))
    }
}

impl From<RatingScalar> for f64 {
    fn from(RatingScalar(rating): RatingScalar) -> f64 {
        rating
    }
}

impl Sub<RatingScalar> for RatingScalar {
    type Output = RatingDifference;

    fn sub(self, rhs: RatingScalar) -> RatingDifference {
        RatingDifference(self.0 - rhs.0)
    }
}

impl Add<RatingDifference> for RatingScalar {
    type Output = RatingScalar;

    fn add(self, RatingDifference(difference): RatingDifference) -> RatingScalar {
        RatingScalar(self.0 + difference)
    }
}

impl AddAssign<RatingDifference> for RatingScalar {
    fn add_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 += difference;
    }
}

impl Sub<RatingDifference> for RatingScalar {
    type Output = RatingScalar;

    fn sub(self, RatingDifference(difference): RatingDifference) -> RatingScalar {
        RatingScalar(self.0 - difference)
    }
}

impl SubAssign<RatingDifference> for RatingScalar {
    fn sub_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 -= difference;
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

impl RatingDifference {
    pub fn clamp(self, RatingDifference(min): RatingDifference, RatingDifference(max): RatingDifference) -> RatingDifference {
        RatingDifference(f64::from(self).clamp(min, max))
    }

    pub fn abs(self) -> RatingDifference {
        RatingDifference(self.0.abs())
    }

    pub(crate) fn internal(self) -> InternalRatingDifference {
        InternalRatingDifference::from(self)
    }
}

impl Add<RatingDifference> for RatingDifference {
    type Output = RatingDifference;

    fn add(self, RatingDifference(difference): RatingDifference) -> RatingDifference {
        RatingDifference(self.0 + difference)
    }
}

impl AddAssign<RatingDifference> for RatingDifference {
    fn add_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 += difference;
    }
}

impl Sub<RatingDifference> for RatingDifference {
    type Output = RatingDifference;

    fn sub(self, RatingDifference(difference): RatingDifference) -> RatingDifference {
        RatingDifference(self.0 - difference)
    }
}

impl SubAssign<RatingDifference> for RatingDifference {
    fn sub_assign(&mut self, RatingDifference(difference): RatingDifference) {
        self.0 -= difference;
    }
}

impl Mul<f64> for RatingDifference {
    type Output = RatingDifference;

    fn mul(self, scalar: f64) -> RatingDifference {
        RatingDifference(self.0 * scalar)
    }
}

impl Mul<RatingDifference> for f64 {
    type Output = RatingDifference;

    fn mul(self, RatingDifference(difference): RatingDifference) -> RatingDifference {
        RatingDifference(self * difference)
    }
}

impl MulAssign<f64> for RatingDifference {
    fn mul_assign(&mut self, scalar: f64) {
        self.0 *= scalar;
    }
}

impl Div<f64> for RatingDifference {
    type Output = RatingDifference;

    fn div(self, scalar: f64) -> RatingDifference {
        RatingDifference(self.0 / scalar)
    }
}

impl DivAssign<f64> for RatingDifference {
    fn div_assign(&mut self, scalar: f64) {
        self.0 /= scalar;
    }
}

impl Neg for RatingDifference {
    type Output = RatingDifference;

    fn neg(self) -> RatingDifference {
        RatingDifference(-self.0)
    }
}

/// Number representing the degree of expected fluctuation in a rating.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Volatility(pub f64);

impl Volatility {
    pub fn clamp(self, Volatility(min): Volatility, Volatility(max): Volatility) -> Volatility {
        Volatility(f64::from(self).clamp(min, max))
    }

    pub(crate) fn sq(self) -> f64 {
        self.0 * self.0
    }
}

impl From<Volatility> for f64 {
    fn from(Volatility(volatility): Volatility) -> f64 {
        volatility
    }
}

/// A rating at a specific point in time.
#[derive(Debug, Clone)]
pub struct Rating {
    pub rating: RatingScalar,
    pub deviation: RatingDifference,
    pub volatility: Volatility,
    pub at: Instant,
}