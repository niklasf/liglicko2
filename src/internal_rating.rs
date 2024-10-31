use crate::rating::RatingDifference;

pub const INTERNAL_RATING_SCALE: f64 = 173.7178;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InternalRatingDifference(pub f64);

impl From<InternalRatingDifference> for f64 {
    fn from(InternalRatingDifference(difference): InternalRatingDifference) -> f64 {
        difference
    }
}

impl From<RatingDifference> for InternalRatingDifference {
    fn from(RatingDifference(difference): RatingDifference) -> InternalRatingDifference {
        InternalRatingDifference(difference / INTERNAL_RATING_SCALE)
    }
}

impl From<InternalRatingDifference> for RatingDifference {
    fn from(InternalRatingDifference(difference): InternalRatingDifference) -> RatingDifference {
        RatingDifference(difference * INTERNAL_RATING_SCALE)
    }
}

impl InternalRatingDifference {
    pub fn clamp(self, InternalRatingDifference(min): InternalRatingDifference, InternalRatingDifference(max): InternalRatingDifference) -> InternalRatingDifference {
        InternalRatingDifference(f64::from(self).clamp(min, max))
    }

    pub fn sq(self) -> f64 {
        self.0 * self.0
    }
}