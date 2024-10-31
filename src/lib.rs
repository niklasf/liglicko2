mod instant;
mod internal_rating;
mod rating;
mod rating_system;
mod score;

pub use instant::{Instant, Periods};
pub use rating::{Rating, RatingDifference, RatingScalar, Volatility};
pub use rating_system::{RatingSystem, RatingSystemBuilder};
pub use score::Score;

/// Log likelihood deviance metric that can be used to evaluate the quality of
/// rating system predictions.
///
/// Lower is better.
///
/// See <https://www.kaggle.com/c/ChessRatings2/overview/evaluation>.
pub fn deviance(Score(expected): Score, Score(actual): Score) -> f64 {
    let expected = expected.clamp(0.01, 0.99);
    -(actual * expected.log10() + (1.0 - actual) * (1.0 - expected).log10())
}
