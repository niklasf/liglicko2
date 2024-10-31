use std::ops;

/// A score or expectation value in the range `0.0..=1.0`, where `0.0` is a
/// loss and `1.0` is a win.
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Score(pub f64);

impl From<Score> for f64 {
    fn from(Score(score): Score) -> f64 {
        score
    }
}

impl Score {
    pub fn opposite(self) -> Score {
        Score(1.0 - self.0)
    }

    pub fn value(self) -> f64 {
        self.0
    }

    pub fn clamp(self, Score(min): Score, Score(max): Score) -> Score {
        Score(f64::from(self).clamp(min, max))
    }
}

impl Score {
    pub const LOSS: Score = Score(0.0);
    pub const DRAW: Score = Score(0.5);
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