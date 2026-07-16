use thiserror::Error;
use time::{Duration, OffsetDateTime};

const SLOT_MINUTES: i64 = 30;
const MAX_SLOTS: usize = 2_880;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SlotError {
    #[error("reservation boundaries must be aligned to UTC half-hours")]
    Unaligned,
    #[error("reservation end must be after its start")]
    InvalidWindow,
    #[error("reservation window is too large")]
    TooLarge,
}

pub fn slot_boundaries(
    start: OffsetDateTime,
    end: OffsetDateTime,
) -> Result<Vec<OffsetDateTime>, SlotError> {
    if start.offset() != time::UtcOffset::UTC || end.offset() != time::UtcOffset::UTC {
        return Err(SlotError::Unaligned);
    }
    if start.second() != 0
        || start.nanosecond() != 0
        || !matches!(start.minute(), 0 | 30)
        || end.second() != 0
        || end.nanosecond() != 0
        || !matches!(end.minute(), 0 | 30)
    {
        return Err(SlotError::Unaligned);
    }
    if end <= start {
        return Err(SlotError::InvalidWindow);
    }

    let duration = end - start;
    let count = duration.whole_minutes() / SLOT_MINUTES;
    if count <= 0 || count as usize > MAX_SLOTS {
        return Err(SlotError::TooLarge);
    }

    Ok((0..count)
        .map(|index| start + Duration::minutes(index * SLOT_MINUTES))
        .collect())
}
