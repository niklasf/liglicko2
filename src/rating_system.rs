use crate::Instant;
use crate::Score;
use crate::{
    internal_rating::InternalRatingDifference,
    rating::{Rating, RatingDifference, RatingScalar, Volatility},
};
use std::f64::consts::PI;

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
}

impl Default for RatingSystemBuilder {
    fn default() -> RatingSystemBuilder {
        RatingSystemBuilder::new()
    }
}

impl RatingSystemBuilder {
    pub fn new() -> RatingSystemBuilder {
        RatingSystemBuilder {
            min_rating: RatingScalar(400.0),
            max_rating: RatingScalar(4000.0),
            default_rating: RatingScalar(1500.0),

            min_volatility: Volatility(0.0),
            max_volatility: Volatility(0.1),
            default_volatility: Volatility(0.09),

            min_deviation: RatingDifference(45.0),
            max_deviation: RatingDifference(500.0),

            first_advantage: RatingDifference(0.0),

            tau: 0.75,
        }
    }

    pub fn min_rating(&mut self, min_rating: RatingScalar) -> &mut Self {
        assert!(!f64::from(min_rating).is_nan());
        self.min_rating = min_rating;
        self
    }

    pub fn max_rating(&mut self, max_rating: RatingScalar) -> &mut Self {
        assert!(!f64::from(max_rating).is_nan());
        self.max_rating = max_rating;
        self
    }

    pub fn default_rating(&mut self, default_rating: RatingScalar) -> &mut Self {
        self.default_rating = default_rating;
        self
    }

    pub fn min_volatility(&mut self, min_volatility: Volatility) -> &mut Self {
        assert!(min_volatility >= Volatility(0.0));
        self.min_volatility = min_volatility;
        self
    }

    pub fn max_volatility(&mut self, max_volatility: Volatility) -> &mut Self {
        assert!(max_volatility >= Volatility(0.0));
        self.max_volatility = max_volatility;
        self
    }

    pub fn default_volatility(&mut self, default_volatility: Volatility) -> &mut Self {
        assert!(default_volatility >= Volatility(0.0));
        self.default_volatility = default_volatility;
        self
    }

    pub fn min_deviation(&mut self, min_deviation: RatingDifference) -> &mut Self {
        assert!(min_deviation >= RatingDifference(0.0));
        self.min_deviation = min_deviation;
        self
    }

    pub fn max_deviation(&mut self, max_deviation: RatingDifference) -> &mut Self {
        assert!(max_deviation >= RatingDifference(0.0));
        self.max_deviation = max_deviation;
        self
    }

    pub fn first_advantage(&mut self, first_advantage: RatingDifference) -> &mut Self {
        self.first_advantage = first_advantage;
        self
    }

    pub fn tau(&mut self, tau: f64) -> &mut Self {
        assert!(tau >= 0.0);
        self.tau = tau;
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
        }
    }
}

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
}

impl Default for RatingSystem {
    fn default() -> RatingSystem {
        RatingSystem::new()
    }
}

impl RatingSystem {
    pub fn builder() -> RatingSystemBuilder {
        RatingSystemBuilder::default()
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

    pub fn initial_rating(&self) -> Rating {
        Rating {
            rating: self.default_rating.clamp(self.min_rating, self.max_rating),
            deviation: self.max_deviation,
            volatility: self.default_volatility,
            at: Instant::default(),
        }
    }

    pub fn preview_deviation(&self, rating: &Rating, now: Instant) -> RatingDifference {
        RatingDifference::from(new_deviation(
            rating.deviation.internal(),
            rating.volatility,
            now.elapsed_periods_since(rating.at),
        ))
        .clamp(self.min_deviation, self.max_deviation)
    }

    pub fn expected_score(&self, first: &Rating, second: &Rating, now: Instant) -> Score {
        expectation_value(
            (first.rating + self.first_advantage - second.rating).internal(),
            g(self.preview_deviation(second, now).internal()),
        )
    }

    pub fn update_ratings(
        &self,
        first: &Rating,
        second: &Rating,
        score: Score,
        now: Instant,
    ) -> (Rating, Rating) {
        (
            self.update_rating(first, second, score, now, self.first_advantage),
            self.update_rating(second, first, score.opposite(), now, -self.first_advantage),
        )
    }

    fn update_rating(
        &self,
        us: &Rating,
        them: &Rating,
        score: Score,
        now: Instant,
        advantage: RatingDifference,
    ) -> Rating {
        let phi = us.deviation.internal();

        // Step 3
        let their_g = g(self.preview_deviation(them, now).internal()); // Novel
        let expected = expectation_value((us.rating + advantage - them.rating).internal(), their_g);
        let v = 1.0 / (their_g.powi(2) * f64::from(expected) * f64::from(expected.opposite()));

        // Step 4
        let delta = v * their_g * f64::from(score - expected);

        // Step 5.1
        let a = f64::ln(f64::from(us.volatility).powi(2));
        let f = |x: f64| {
            f64::exp(x) * (delta.powi(2) - f64::from(phi).powi(2) - v - f64::exp(x))
                / (2.0 * (f64::from(phi).powi(2) + v + f64::exp(x)).powi(2))
                - (x - a) / self.tau.powi(2)
        };

        // Step 5.2
        let mut big_a = a;
        let mut big_b = if delta.powi(2) > f64::from(phi).powi(2) + v {
            f64::ln(delta.powi(2) - f64::from(phi).powi(2) - v)
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
        while f64::abs(big_b - big_a) > CONVERGENCE_TOLERANCE {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                panic!("Failed to converge");
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
            us.deviation.internal(),
            sigma_prime,
            now.elapsed_periods_since(us.at),
        );

        // Step 7
        let phi_prime =
            InternalRatingDifference(1.0 / f64::sqrt(1.0 / f64::from(phi_star).powi(2) + 1.0 / v))
                .clamp(self.min_deviation.internal(), self.max_deviation.internal());
        let mu_prime_diff = InternalRatingDifference(
            f64::from(phi_prime).powi(2) * their_g * f64::from(score - expected),
        );

        // Step 8
        Rating {
            rating: (us.rating + mu_prime_diff.into()).clamp(self.min_rating, self.max_rating),
            deviation: RatingDifference::from(phi_prime),
            volatility: sigma_prime.clamp(self.min_volatility, self.max_volatility),
            at: now,
        }
    }
}

fn g(InternalRatingDifference(deviation): InternalRatingDifference) -> f64 {
    1.0 / (1.0 + 3.0 * deviation.powi(2) / PI.powi(2)).sqrt()
}

fn expectation_value(
    InternalRatingDifference(diff): InternalRatingDifference,
    their_g: f64,
) -> Score {
    Score(1.0 / (1.0 + f64::exp(-their_g * diff)))
}

fn new_deviation(
    InternalRatingDifference(deviation): InternalRatingDifference,
    Volatility(volatility): Volatility,
    elapsed_periods: f64,
) -> InternalRatingDifference {
    InternalRatingDifference(f64::sqrt(
        deviation.powi(2) + f64::max(elapsed_periods, 0.0) * volatility.powi(2),
    ))
}

const CONVERGENCE_TOLERANCE: f64 = 0.000001;

const MAX_ITERATIONS: u32 = 10000;