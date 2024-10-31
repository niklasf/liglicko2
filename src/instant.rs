use std::ops::{Add, AddAssign, Sub, SubAssign};

/// An instant in time. A difference of `1.0` represents a *rating period* in
/// Glicko2 terminology.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Instant(pub f64);

impl From<Instant> for f64 {
    fn from(Instant(instant): Instant) -> f64 {
        instant
    }
}

impl From<f64> for Instant {
    fn from(value: f64) -> Instant {
        Instant(value)
    }
}

impl Instant {
    pub fn elapsed_since(self, since: Instant) -> Periods {
        Periods(self.0 - since.0)
    }
}

impl Sub for Instant {
    type Output = Periods;

    fn sub(self, rhs: Instant) -> Periods {
        Periods(self.0 - rhs.0)
    }
}

impl Add<Periods> for Instant {
    type Output = Instant;

    fn add(self, rhs: Periods) -> Instant {
        Instant(self.0 + rhs.0)
    }
}

impl AddAssign<Periods> for Instant {
    fn add_assign(&mut self, rhs: Periods) {
        self.0 += rhs.0;
    }
}

impl Sub<Periods> for Instant {
    type Output = Instant;

    fn sub(self, rhs: Periods) -> Instant {
        Instant(self.0 - rhs.0)
    }
}

impl SubAssign<Periods> for Instant {
    fn sub_assign(&mut self, rhs: Periods) {
        self.0 -= rhs.0;
    }
}

/// Number of rating periods between two instants in time.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Periods(pub f64);

impl From<Periods> for f64 {
    fn from(Periods(period): Periods) -> f64 {
        period
    }
}

impl From<f64> for Periods {
    fn from(value: f64) -> Periods {
        Periods(value)
    }
}

impl Add for Periods {
    type Output = Periods;

    fn add(self, rhs: Periods) -> Periods {
        Periods(self.0 + rhs.0)
    }
}

impl AddAssign for Periods {
    fn add_assign(&mut self, rhs: Periods) {
        self.0 += rhs.0;
    }
}

impl Sub for Periods {
    type Output = Periods;

    fn sub(self, rhs: Periods) -> Periods {
        Periods(self.0 - rhs.0)
    }
}

impl SubAssign for Periods {
    fn sub_assign(&mut self, rhs: Periods) {
        self.0 -= rhs.0;
    }
}
