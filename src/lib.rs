mod instant;
mod score;
mod rating_system;
mod rating;
mod internal_rating;

pub use instant::Instant;
pub use score::Score;
pub use rating::Rating;
pub use rating_system::{RatingSystemBuilder, RatingSystem};

/// Log likelihood deviance metric that can be used to evaluate the quality of
/// rating system predictions.
///
/// Lower is better.
///
/// See https://www.kaggle.com/c/ChessRatings2/overview/evaluation.
pub fn deviance(Score(expected): Score, Score(actual): Score) -> f64 {
    let expected = expected.clamp(0.01, 0.99);
    -(actual * expected.log10() + (1.0 - actual) * (1.0 - expected).log10())
}