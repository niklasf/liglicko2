//! Implementation of the Lichess-flavored Glicko-2 rating system.
//!
//! See <http://glicko.net/glicko/glicko2.pdf> for a description of the
//! original Glicko-2 rating system.
//!
//! Lichess has made some modifications:
//!
//! - Optimized default parameters based on Lichess data. Optimal parameters
//!   depend on the application, so this will not be ideal for all use cases.
//! - All rating components are clamped to specific ranges, so that even
//!   pathological scenarios cannot cause degenerate results.
//! - Glicko-2 updates ratings in bulk in discrete *rating periods*. Lichess
//!   instead updates pairs of ratings, so that ratings can be immediately
//!   updated after each game.
//! - Lichess keeps the time decay of rating deviations, but generalizes it
//!   to work with fractional rating periods.
//!
//! When using the provided default parameters, this implementations promises:
//!
//! - If all inputs are non-NaN, then all outputs will be non-NaN.
//! - All methods return in `O(1)`, in particular there are no panics and the
//!   internal iterative algorithm always converges.

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
///
/// # Example
///
/// ```
/// use liglicko2::{deviance, Score};
///
/// let actual = Score(0.0);
///
/// let close_guess = deviance(Score(0.1), actual);
/// // 0.0457...
/// let indifferent_guess = deviance(Score(0.5), actual);
/// // 0.3010 ...
/// let far_guess = deviance(Score(0.95), actual);
/// // 1.3010 ...
///
/// assert!(close_guess < indifferent_guess);
/// assert!(indifferent_guess < far_guess);
/// ```
pub fn deviance(expected: Score, actual: Score) -> f64 {
    let expected = expected.value().clamp(0.01, 0.99);
    let actual = actual.value();

    -(actual * expected.log10() + (1.0 - actual) * (1.0 - expected).log10())
}
