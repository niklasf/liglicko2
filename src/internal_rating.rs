use crate::rating::RatingDifference;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InternalRatingDifference(pub f64);

impl InternalRatingDifference {
    pub fn from_external(
        RatingDifference(difference): RatingDifference,
    ) -> InternalRatingDifference {
        InternalRatingDifference(difference / INTERNAL_RATING_SCALE)
    }

    pub fn to_external(self) -> RatingDifference {
        RatingDifference(self.0 * INTERNAL_RATING_SCALE)
    }
}

impl InternalRatingDifference {
    #[must_use]
    #[inline]
    pub fn sq(self) -> f64 {
        self.0 * self.0
    }

    #[must_use]
    pub fn hypot(self, other: InternalRatingDifference) -> InternalRatingDifference {
        InternalRatingDifference(f64::hypot(self.0, other.0))
    }
}

const INTERNAL_RATING_SCALE: f64 = 173.7178;
