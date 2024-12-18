//! Lichess-flavored Glicko-2 rating system system with fractional rating
//! periods and instant rating updates.
//!
//! This does not (yet) exactly match the Lichess implementation.
//! Instead, it's a proof of concept for potential improvements and parameter
//! tweaks.
//!
//! See <http://glicko.net/glicko/glicko2.pdf> for a description of the
//! original Glicko-2 rating system. The following changes have been made:
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
//! - Allows considering an inherent advantage for the first player in a game.
//!
//! # Errors
//!
//! When using the provided default parameters, this implementations promises:
//!
//! - If all inputs are non-NaN, then all outputs will be non-NaN.
//! - The will never be a [`ConvergenceError`].
//!
//! # Examples
//!
//! ```
//! use liglicko2::{RatingSystem, Score, Instant, Periods};
//!
//! let system = RatingSystem::new();
//!
//! let alice = system.new_rating();
//! let bob = system.new_rating();
//!
//! let now = Instant::default() + Periods(2.3);
//!
//! // Initial prediction is indifferent.
//! let expected_score = system.expected_score(&alice, &bob, now);
//! assert!(Score(0.49) < expected_score && expected_score < Score(0.51));
//!
//! // Alice wins. Update ratings.
//! let (alice, bob) = system.update_ratings(&alice, &bob, Score::WIN, now).unwrap();
//! assert!(alice.rating > bob.rating);
//!
//! let now = now + Periods(1.0);
//!
//! // Alice is expected to win the next game.
//! let expected_score = system.expected_score(&alice, &bob, now);
//! assert!(Score(0.79) < expected_score, "{expected_score:?}");
//! ```

mod instant;
mod internal_rating;
mod rating;
mod rating_system;
mod score;

pub use instant::{Instant, Periods};
pub use rating::{Rating, RatingDifference, RatingScalar, Volatility};
pub use rating_system::{ConvergenceError, RatingSystem, RatingSystemBuilder};
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
