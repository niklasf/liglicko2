#![no_main]

use arbitrary::Arbitrary;
use arbitrary::Unstructured;
use libfuzzer_sys::fuzz_target;
use liglicko2::{Instant, Rating, RatingDifference, RatingScalar, RatingSystem, Score, Volatility};

#[derive(Arbitrary, Debug)]
struct ArbitraryRating {
    rating: f64,
    deviation: f64,
    volatility: f64,
    at: f64,
}

impl ArbitraryRating {
    fn non_nan(&self) -> Option<Rating> {
        if self.rating.is_nan()
            || self.deviation.is_nan()
            || self.volatility.is_nan()
            || self.at.is_nan()
        {
            None
        } else {
            Some(Rating {
                rating: RatingScalar(self.rating),
                deviation: RatingDifference(self.deviation),
                volatility: Volatility(self.volatility),
                at: Instant(self.at),
            })
        }
    }
}

#[derive(Arbitrary, Debug)]
struct Encounter {
    first: ArbitraryRating,
    second: ArbitraryRating,
    score: f64,
    at: f64,
}

fn assert_rating(glicko: Rating, encounter: &Encounter) {
    assert!(
        !f64::from(glicko.rating).is_nan(),
        "invalid rating produced by {encounter:?}"
    );
    assert!(
        !f64::from(glicko.deviation).is_nan(),
        "invalid deviation produced by {encounter:?}"
    );
    assert!(
        !f64::from(glicko.volatility).is_nan(),
        "invalid volatility produced by {encounter:?}"
    );
    assert!(
        !f64::from(glicko.at).is_nan(),
        "invalid instant produced by {encounter:?}"
    );
}

fuzz_target!(|data: &[u8]| {
    let mut u = Unstructured::new(data);

    let Ok(encounter) = Encounter::arbitrary(&mut u) else {
        return;
    };

    let (Some(first), Some(second)) = (encounter.first.non_nan(), encounter.second.non_nan())
    else {
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
    assert_rating(first, &encounter);
    assert_rating(second, &encounter);
});
