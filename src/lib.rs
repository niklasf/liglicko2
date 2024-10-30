#[derive(Debug, Copy, Clone, PartialOrd, PartialEq, Default)]
pub struct Instant(pub f64);

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd, Default)]
pub struct Score(pub f64);

#[derive(Debug, Clone)]
pub struct Rating {
    pub rating: f64,
    pub rd: f64,
    pub volatility: f64,
    pub updated_at: Option<Instant>,
}

#[derive(Debug, Clone)]
pub struct RatingSystem {
    min_rating: Option<f64>,
    max_rating: Option<f64>,

    default_rating: f64,
    max_rd: f64,
    default_volatility: f64,

    first_advantage: f64,

    tau: f64,
}

impl Default for RatingSystem {
    fn default() -> RatingSystem {
        RatingSystem {
            min_rating: Some(400.0),
            max_rating: Some(4000.0),

            default_rating: 1500.0,
            max_rd: 500.0,
            default_volatility: 0.09,

            first_advantage: 0.0,

            tau: 0.75,
        }
    }
}

impl RatingSystem {
    pub fn default_rating(&self) -> Rating {
        Rating {
            rating: self.default_rating,
            rd: self.max_rd,
            volatility: self.default_volatility,
            updated_at: None,
        }
    }

    pub fn expected_score(&self, first: &Rating, second: &Rating) -> Score {
        todo!()
    }

    pub fn update_ratings(&self, first: &Rating, second: &Rating, score: Score) -> (Rating, Rating) {
        todo!()
    }
}

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