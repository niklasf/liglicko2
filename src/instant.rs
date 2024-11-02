use std::ops::{Add, AddAssign, Sub, SubAssign};

/// An instant in time. A difference of `1.0` represents a *rating period* in
/// Glicko2 terminology.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Instant(pub f64);

impl From<Instant> for f64 {
    #[inline]
    fn from(Instant(instant): Instant) -> f64 {
        instant
    }
}

impl From<f64> for Instant {
    #[inline]
    fn from(value: f64) -> Instant {
        Instant(value)
    }
}

impl Instant {
    #[inline]
    pub fn elapsed_since(self, since: Instant) -> Periods {
        Periods(self.0 - since.0)
    }
}

impl Sub for Instant {
    type Output = Periods;

    #[inline]
    fn sub(self, rhs: Instant) -> Periods {
        Periods(self.0 - rhs.0)
    }
}

impl Add<Periods> for Instant {
    type Output = Instant;

    #[inline]
    fn add(self, rhs: Periods) -> Instant {
        Instant(self.0 + rhs.0)
    }
}

impl AddAssign<Periods> for Instant {
    #[inline]
    fn add_assign(&mut self, rhs: Periods) {
        self.0 += rhs.0;
    }
}

impl Sub<Periods> for Instant {
    type Output = Instant;

    #[inline]
    fn sub(self, rhs: Periods) -> Instant {
        Instant(self.0 - rhs.0)
    }
}

impl SubAssign<Periods> for Instant {
    #[inline]
    fn sub_assign(&mut self, rhs: Periods) {
        self.0 -= rhs.0;
    }
}

/// Number of rating periods between two instants in time.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Periods(pub f64);

impl From<Periods> for f64 {
    #[inline]
    fn from(Periods(period): Periods) -> f64 {
        period
    }
}

impl From<f64> for Periods {
    #[inline]
    fn from(value: f64) -> Periods {
        Periods(value)
    }
}

impl Periods {
    #[must_use]
    #[inline]
    pub fn max(self, other: Periods) -> Periods {
        Periods(f64::max(self.0, other.0))
    }

    #[must_use]
    #[inline]
    pub fn min(self, other: Periods) -> Periods {
        Periods(f64::min(self.0, other.0))
    }
}

impl Add for Periods {
    type Output = Periods;

    #[inline]
    fn add(self, rhs: Periods) -> Periods {
        Periods(self.0 + rhs.0)
    }
}

impl AddAssign for Periods {
    #[inline]
    fn add_assign(&mut self, rhs: Periods) {
        self.0 += rhs.0;
    }
}

impl Sub for Periods {
    type Output = Periods;

    #[inline]
    fn sub(self, rhs: Periods) -> Periods {
        Periods(self.0 - rhs.0)
    }
}

impl SubAssign for Periods {
    #[inline]
    fn sub_assign(&mut self, rhs: Periods) {
        self.0 -= rhs.0;
    }
}
