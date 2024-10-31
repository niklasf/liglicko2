#![no_main]

use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use liglicko2::{Instant, Rating, RatingDifference, RatingScalar, RatingSystem, Score, Volatility};

#[derive(Arbitrary)]
struct ArbitraryRating {
    rating: f64,
    deviation: f64,
    volatility: f64,
    at: f64,
}

impl ArbitraryRating {
    fn into_clamped(self) -> Option<Rating> {
        if self.rating.is_nan()
            || self.deviation.is_nan()
            || self.volatility.is_nan()
            || self.at.is_nan()
        {
            None
        } else {
            Some(Rating {
                rating: RatingScalar(self.rating.clamp(-10000.0, 10000.0)),
                deviation: RatingDifference(self.deviation.clamp(0.0, 1000.0)),
                volatility: Volatility(self.volatility.clamp(0.0, 1.0)),
                at: Instant(self.at),
            })
        }
    }
}

#[derive(Arbitrary)]
struct Encounter {
    first: ArbitraryRating,
    second: ArbitraryRating,
    score: f64,
    at: f64,
}

fn assert_rating(glicko: Rating) {
    assert!(!f64::from(glicko.rating).is_nan());
    assert!(!f64::from(glicko.deviation).is_nan());
    assert!(!f64::from(glicko.volatility).is_nan());
    assert!(!f64::from(glicko.at).is_nan());
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    let Ok(encounter) = Encounter::arbitrary(&mut u) else {
        return;
    };

    let (Some(first), Some(second)) = (
        encounter.first.into_clamped(),
        encounter.second.into_clamped(),
    ) else {
        return;
    };

    if encounter.score.is_nan() || encounter.at.is_nan() {
        return;
    }

    let rating_system = RatingSystem::new();

    let (first, second) = rating_system.update_ratings(
        &first,
        &second,
        Score(encounter.score.clamp(0.0, 1.0)),
        Instant(encounter.at),
    );
    assert_rating(first);
    assert_rating(second);
});
