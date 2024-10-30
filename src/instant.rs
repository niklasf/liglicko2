/// An instant in time. A difference of `1.0` represents a *rating period* in
/// Glicko2 terminology.
#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Instant(pub f64);

impl From<Instant> for f64 {
    fn from(Instant(instant): Instant) -> f64 {
        instant
    }
}

impl Instant {
    pub fn elapsed_periods(self, since: Instant) -> f64 {
        self.0 - since.0
    }
}