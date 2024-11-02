use std::{error::Error, f64::consts::PI, fmt};

use crate::{
    internal_rating::InternalRatingDifference,
    rating::{Rating, RatingDifference, RatingScalar, Volatility},
    Instant, Periods, Score,
};

/// Used to configure a rating system.
///
/// # Example
///
/// ```
/// use liglicko2::{RatingScalar, RatingSystem};
///
/// let rating_system = RatingSystem::builder()
///     .min_rating(RatingScalar(-4000.0))
///     .default_rating(RatingScalar(0.0))
///     .max_rating(RatingScalar(4000.0))
///     .build();
/// ```
#[derive(Debug, Clone)]
pub struct RatingSystemBuilder {
    min_rating: RatingScalar,
    max_rating: RatingScalar,
    default_rating: RatingScalar,

    min_volatility: Volatility,
    max_volatility: Volatility,
    default_volatility: Volatility,

    min_deviation: RatingDifference,
    max_deviation: RatingDifference,

    first_advantage: RatingDifference,

    tau: f64,

    convergence_tolerance: f64,
    max_convergence_iterations: u32,

    max_rating_delta: RatingDifference,
    rating_regulator_factor: f64,
}

impl RatingSystemBuilder {
    /// Set the minimum rating allowed by the system. The default is `400.0`.
    pub fn min_rating(&mut self, min_rating: RatingScalar) -> &mut Self {
        assert!(!min_rating.0.is_nan());
        self.min_rating = min_rating;
        self
    }

    /// Set the maximum rating allowed by the system. The default is `4000.0`.
    pub fn max_rating(&mut self, max_rating: RatingScalar) -> &mut Self {
        assert!(!max_rating.0.is_nan());
        self.max_rating = max_rating;
        self
    }

    /// Set the default rating for new players. The default is `1500.0`.
    pub fn default_rating(&mut self, default_rating: RatingScalar) -> &mut Self {
        self.default_rating = default_rating;
        self
    }

    /// Set the minimum volatility allowed by the system. The default is `0.01`.
    pub fn min_volatility(&mut self, min_volatility: Volatility) -> &mut Self {
        assert!(min_volatility >= Volatility(0.0));
        self.min_volatility = min_volatility;
        self
    }

    /// Set the maximum volatility allowed by the system. The default is `0.1`.
    pub fn max_volatility(&mut self, max_volatility: Volatility) -> &mut Self {
        assert!(max_volatility >= Volatility(0.0));
        self.max_volatility = max_volatility;
        self
    }

    /// Set the default volatility for new players. The default is `0.09`.
    pub fn default_volatility(&mut self, default_volatility: Volatility) -> &mut Self {
        assert!(default_volatility >= Volatility(0.0));
        self.default_volatility = default_volatility;
        self
    }

    /// Set the minimum deviation allowed by the system. The default is `45.0`.
    pub fn min_deviation(&mut self, min_deviation: RatingDifference) -> &mut Self {
        assert!(min_deviation >= RatingDifference(0.0));
        self.min_deviation = min_deviation;
        self
    }

    /// Set the maximum deviation allowed by the system. The default is `500.0`.
    pub fn max_deviation(&mut self, max_deviation: RatingDifference) -> &mut Self {
        assert!(max_deviation >= RatingDifference(0.0));
        self.max_deviation = max_deviation;
        self
    }

    /// Set the inherent advantage for the first player. The default is `0.0`.
    pub fn first_advantage(&mut self, first_advantage: RatingDifference) -> &mut Self {
        self.first_advantage = first_advantage;
        self
    }

    /// Set the tau parameter for the rating system. Smaller tau leads to
    /// smaller changes to volatilities. The default is `0.75`.
    pub fn tau(&mut self, tau: f64) -> &mut Self {
        assert!(tau >= 0.0);
        self.tau = tau;
        self
    }

    /// Set the tolerance for the convergence in step 5.4 of the Glicko-2
    /// algorithm. The default is `1e-6`.
    pub fn convergence_tolerance(&mut self, convergence_tolerance: f64) -> &mut Self {
        assert!(convergence_tolerance > 0.0);
        self.convergence_tolerance = convergence_tolerance;
        self
    }

    /// Set the maximum number of iterations for the convergence in step 5.4 of
    /// the Glicko-2 algorithm. The default is `1000`.
    pub fn max_convergence_iterations(&mut self, max_convergence_iterations: u32) -> &mut Self {
        assert!(max_convergence_iterations > 0);
        self.max_convergence_iterations = max_convergence_iterations;
        self
    }

    /// Set the maximum rating step that a single game can cause. The default
    /// is `700.0`.
    pub fn max_rating_delta(&mut self, max_rating_delta: RatingDifference) -> &mut Self {
        assert!(max_rating_delta >= RatingDifference(0.0));
        self.max_rating_delta = max_rating_delta;
        self
    }

    /// Set the factor by which rating gains (but not losses) are multiplied.
    /// The default is `1.015`.
    pub fn rating_regulator_factor(&mut self, rating_regulator_factor: f64) -> &mut Self {
        assert!(rating_regulator_factor >= 0.0);
        self.rating_regulator_factor = rating_regulator_factor;
        self
    }

    pub fn build(&self) -> RatingSystem {
        assert!(self.min_rating <= self.max_rating);
        assert!(self.min_deviation <= self.max_deviation);
        assert!(self.min_volatility <= self.max_volatility);

        RatingSystem {
            min_rating: self.min_rating,
            max_rating: self.max_rating,
            default_rating: self.default_rating,

            min_volatility: self.min_volatility,
            max_volatility: self.max_volatility,
            default_volatility: self.default_volatility,

            min_deviation: self.min_deviation,
            max_deviation: self.max_deviation,

            first_advantage: self.first_advantage,

            tau: self.tau,

            convergence_tolerance: self.convergence_tolerance,
            max_convergence_iterations: self.max_convergence_iterations,

            max_rating_delta: self.max_rating_delta,
            rating_regulator_factor: self.rating_regulator_factor,
        }
    }
}

/// Rating system parameters. Used to perform the main operations provided
/// by the rating system.
///
/// - Construct a new rating.
/// - Calculate the expected score for a game between two rated players.
/// - Calculate the new ratings for two players after a game.
#[derive(Debug, Clone)]
pub struct RatingSystem {
    min_rating: RatingScalar,
    max_rating: RatingScalar,
    default_rating: RatingScalar,

    min_volatility: Volatility,
    max_volatility: Volatility,
    default_volatility: Volatility,

    min_deviation: RatingDifference,
    max_deviation: RatingDifference,

    first_advantage: RatingDifference,

    tau: f64,

    convergence_tolerance: f64,
    max_convergence_iterations: u32,

    max_rating_delta: RatingDifference,
    rating_regulator_factor: f64,
}

impl Default for RatingSystem {
    fn default() -> RatingSystem {
        RatingSystem::new()
    }
}

impl RatingSystem {
    /// Build a rating system with non-default parameters.
    ///
    /// Note that using non-default parameters waives the promises with regard
    /// to numeric stability and convergence of the rating update algorithm.
    pub fn builder() -> RatingSystemBuilder {
        // Remember to update docs if defaults are changed.
        RatingSystemBuilder {
            min_rating: RatingScalar(400.0),
            max_rating: RatingScalar(4000.0),
            default_rating: RatingScalar(1500.0),

            min_volatility: Volatility(0.01),
            max_volatility: Volatility(0.1),
            default_volatility: Volatility(0.09),

            min_deviation: RatingDifference(45.0),
            max_deviation: RatingDifference(500.0),

            first_advantage: RatingDifference(0.0),

            tau: 0.75,

            convergence_tolerance: 1e-6,
            max_convergence_iterations: 1000,

            max_rating_delta: RatingDifference(700.0),
            rating_regulator_factor: 1.015,
        }
    }

    pub fn new() -> RatingSystem {
        RatingSystem::builder().build()
    }

    pub fn min_rating(&self) -> RatingScalar {
        self.min_rating
    }

    pub fn max_rating(&self) -> RatingScalar {
        self.max_rating
    }

    pub fn default_rating(&self) -> RatingScalar {
        self.default_rating
    }

    pub fn min_volatility(&self) -> Volatility {
        self.min_volatility
    }

    pub fn max_volatility(&self) -> Volatility {
        self.max_volatility
    }

    pub fn default_volatility(&self) -> Volatility {
        self.default_volatility
    }

    pub fn min_deviation(&self) -> RatingDifference {
        self.min_deviation
    }

    pub fn max_deviation(&self) -> RatingDifference {
        self.max_deviation
    }

    pub fn first_advantage(&self) -> RatingDifference {
        self.first_advantage
    }

    pub fn tau(&self) -> f64 {
        self.tau
    }

    pub fn convergence_tolerance(&self) -> f64 {
        self.convergence_tolerance
    }

    pub fn max_convergence_iterations(&self) -> u32 {
        self.max_convergence_iterations
    }

    pub fn max_rating_delta(&self) -> RatingDifference {
        self.max_rating_delta
    }

    pub fn rating_regulator_factor(&self) -> f64 {
        self.rating_regulator_factor
    }

    /// Construct an initial rating for a new player.
    pub fn new_rating(&self) -> Rating {
        Rating {
            rating: self.default_rating.clamp(self.min_rating, self.max_rating),
            deviation: self.max_deviation,
            volatility: self
                .default_volatility
                .clamp(self.min_volatility, self.max_volatility),
            at: Instant::default(),
        }
    }

    /// Preview the rating deviation that a player will have at a future
    /// point in time if no games are played until then.
    pub fn preview_deviation(&self, rating: &Rating, at: Instant) -> RatingDifference {
        let rating = self.clamp_rating(rating);

        RatingDifference::from(new_deviation(
            rating.deviation.to_internal(),
            rating.volatility,
            at.elapsed_since(rating.at),
        ))
        .clamp(self.min_deviation, self.max_deviation)
    }

    /// Calculate the expected score for the first player in a game against the
    /// second player.
    pub fn expected_score(&self, first: &Rating, second: &Rating, now: Instant) -> Score {
        let first = self.clamp_rating(first);
        let second = self.clamp_rating(second);

        expectation_value(
            (first.rating - second.rating + self.first_advantage).to_internal(),
            g(InternalRatingDifference::hypot(
                self.preview_deviation(&first, now).to_internal(),
                self.preview_deviation(&second, now).to_internal(),
            )),
        )
    }

    /// Update the ratings of both players, given the score of a game between
    /// between them.
    ///
    /// # Errors
    ///
    /// Errors if the internal iterative algorithm does not converge within
    /// the maximum number of iterations. Will not happen when using default
    /// parameters for the rating system.
    pub fn update_ratings(
        &self,
        first: &Rating,
        second: &Rating,
        score: Score,
        now: Instant,
    ) -> Result<(Rating, Rating), ConvergenceError> {
        let first = self.clamp_rating(first);
        let second = self.clamp_rating(second);
        let score = score.clamp(Score::LOSS, Score::WIN);

        Ok((
            self.update_rating(&first, &second, score, now, self.first_advantage)?,
            self.update_rating(
                &second,
                &first,
                score.opposite(),
                now,
                -self.first_advantage,
            )?,
        ))
    }

    fn update_rating(
        &self,
        us: &Rating,
        them: &Rating,
        score: Score,
        now: Instant,
        advantage: RatingDifference,
    ) -> Result<Rating, ConvergenceError> {
        // Step 2
        let phi = self.preview_deviation(us, now - Periods(1.0)).to_internal(); // Notable change!

        // Step 3
        let their_g = g(self
            .preview_deviation(them, now - Periods(1.0)) // Notable change!
            .to_internal());

        let expected =
            expectation_value((us.rating - them.rating + advantage).to_internal(), their_g);
        let v = 1.0 / (their_g.powi(2) * expected.value() * expected.opposite().value());

        // Step 4
        let delta = v * their_g * Score::value(score - expected);

        // Step 5.1
        let a = f64::ln(us.volatility.sq());
        let f = |x: f64| {
            f64::exp(x) * (delta.powi(2) - phi.sq() - v - f64::exp(x))
                / (2.0 * (phi.sq() + v + f64::exp(x)).powi(2))
                - (x - a) / self.tau.powi(2)
        };

        // Step 5.2
        let mut big_a = a;
        let mut big_b = if delta.powi(2) > phi.sq() + v {
            f64::ln(delta.powi(2) - phi.sq() - v)
        } else {
            let mut k = 1.0;
            while f(a - k * self.tau) < 0.0 {
                k += 1.0;
            }
            a - k * self.tau
        };

        // Step 5.3
        let mut f_a = f(big_a);
        let mut f_b = f(big_b);

        // Step 5.4
        let mut iterations = 0;
        while f64::abs(big_b - big_a) > self.convergence_tolerance {
            iterations += 1;
            if iterations > self.max_convergence_iterations {
                return Err(ConvergenceError { _priv: () });
            }

            let big_c = big_a + (big_a - big_b) * f_a / (f_b - f_a);
            let f_c = f(big_c);

            if f_c * f_b <= 0.0 {
                big_a = big_b;
                f_a = f_b;
            } else {
                f_a /= 2.0;
            }

            big_b = big_c;
            f_b = f_c;
        }

        // Step 5.5
        let sigma_prime = Volatility(f64::exp(big_a / 2.0));

        // Step 6
        let phi_star = new_deviation(
            phi,
            sigma_prime,
            Periods::min(now.elapsed_since(us.at), Periods(1.0)), // Notable change!
        );

        // Step 7
        let phi_prime = InternalRatingDifference(1.0 / f64::sqrt(1.0 / phi_star.sq() + 1.0 / v));
        let mu_prime_diff =
            InternalRatingDifference(phi_prime.sq() * their_g * Score::value(score - expected));

        // Step 8
        Ok(self.clamp_rating(&Rating {
            rating: us.rating
                + self
                    .regulate(RatingDifference::from(mu_prime_diff))
                    .clamp(-self.max_rating_delta, self.max_rating_delta),
            deviation: RatingDifference::from(phi_prime),
            volatility: sigma_prime,
            at: now,
        }))
    }

    fn regulate(&self, diff: RatingDifference) -> RatingDifference {
        if diff > RatingDifference(0.0) {
            self.rating_regulator_factor * diff
        } else {
            diff
        }
    }

    fn clamp_rating(&self, rating: &Rating) -> Rating {
        Rating {
            rating: rating.rating.clamp(self.min_rating, self.max_rating),
            deviation: rating
                .deviation
                .clamp(self.min_deviation, self.max_deviation),
            volatility: rating
                .volatility
                .clamp(self.min_volatility, self.max_volatility),
            at: rating.at,
        }
    }
}

fn g(deviation: InternalRatingDifference) -> f64 {
    1.0 / f64::sqrt(1.0 + 3.0 * deviation.sq() / PI.powi(2))
}

fn expectation_value(InternalRatingDifference(diff): InternalRatingDifference, g: f64) -> Score {
    Score(1.0 / (1.0 + f64::exp(-g * diff)))
}

fn new_deviation(
    deviation: InternalRatingDifference,
    volatility: Volatility,
    elapsed: Periods,
) -> InternalRatingDifference {
    InternalRatingDifference(f64::sqrt(
        deviation.sq() + Periods::max(elapsed, Periods(0.0)).0 * volatility.sq(),
    ))
}

/// Glicko-2 rating update algorithm failed to convergence.
#[derive(Clone)]
pub struct ConvergenceError {
    _priv: (),
}

impl fmt::Debug for ConvergenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ConvergenceError").finish_non_exhaustive()
    }
}

impl fmt::Display for ConvergenceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "failed to converge")
    }
}

impl Error for ConvergenceError {}
