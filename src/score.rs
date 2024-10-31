use std::ops;

/// A score or expectation value in the range `0.0..=1.0`, where `0.0` is a
/// loss and `1.0` is a win.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Score(pub f64);

impl From<Score> for f64 {
    #[inline]
    fn from(Score(score): Score) -> f64 {
        score
    }
}

impl Score {
    #[inline]
    pub fn opposite(self) -> Score {
        Score(1.0 - self.0)
    }

    #[inline]
    pub fn value(self) -> f64 {
        self.0
    }

    #[inline]
    pub fn clamp(self, min: Score, max: Score) -> Score {
        Score(self.value().clamp(min.value(), max.value()))
    }
}

impl Score {
    /// `Score(0.0)`.
    pub const LOSS: Score = Score(0.0);
    /// `Score(0.5)`.
    pub const DRAW: Score = Score(0.5);
    /// `Score(1.0)`.
    pub const WIN: Score = Score(1.0);
}

impl ops::Add<Score> for Score {
    type Output = Score;

    fn add(self, rhs: Score) -> Score {
        Score(self.0 + rhs.0)
    }
}

impl ops::AddAssign<Score> for Score {
    fn add_assign(&mut self, rhs: Score) {
        *self = *self + rhs;
    }
}

impl ops::Sub<Score> for Score {
    type Output = Score;

    fn sub(self, rhs: Score) -> Score {
        Score(self.0 - rhs.0)
    }
}

impl ops::SubAssign<Score> for Score {
    fn sub_assign(&mut self, rhs: Score) {
        *self = *self - rhs;
    }
}
