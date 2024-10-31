use crate::rating::RatingDifference;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InternalRatingDifference(pub f64);

impl From<InternalRatingDifference> for f64 {
    #[inline]
    fn from(InternalRatingDifference(difference): InternalRatingDifference) -> f64 {
        difference
    }
}

impl From<RatingDifference> for InternalRatingDifference {
    #[inline]
    fn from(RatingDifference(difference): RatingDifference) -> InternalRatingDifference {
        InternalRatingDifference(difference / INTERNAL_RATING_SCALE)
    }
}

impl From<InternalRatingDifference> for RatingDifference {
    #[inline]
    fn from(InternalRatingDifference(difference): InternalRatingDifference) -> RatingDifference {
        RatingDifference(difference * INTERNAL_RATING_SCALE)
    }
}

impl InternalRatingDifference {
    pub fn clamp(
        self,
        InternalRatingDifference(min): InternalRatingDifference,
        InternalRatingDifference(max): InternalRatingDifference,
    ) -> InternalRatingDifference {
        InternalRatingDifference(f64::from(self).clamp(min, max))
    }

    #[inline]
    pub fn sq(self) -> f64 {
        self.0 * self.0
    }
}

const INTERNAL_RATING_SCALE: f64 = 173.7178;
