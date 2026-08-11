use super::{TimestampError, UtcTimestamp};

#[test]
fn timestamp_unit_boundary() {
    assert_eq!(
        UtcTimestamp::new(i64::MAX, 999_999_999)
            .map(|timestamp| (timestamp.seconds(), timestamp.nanoseconds())),
        Ok((i64::MAX, 999_999_999))
    );
    assert_eq!(
        UtcTimestamp::new(i64::MIN, 1_000_000_000),
        Err(TimestampError::NanosecondsOutOfRange)
    );
}
