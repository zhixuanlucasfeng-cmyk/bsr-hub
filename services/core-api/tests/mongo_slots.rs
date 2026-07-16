use core_api::adapters::mongo::slots::slot_boundaries;
use time::macros::datetime;

#[test]
fn half_hour_window_creates_one_slot() {
    let slots = slot_boundaries(
        datetime!(2026-07-16 10:00 UTC),
        datetime!(2026-07-16 10:30 UTC),
    )
    .unwrap();
    assert_eq!(slots, vec![datetime!(2026-07-16 10:00 UTC)]);
}

#[test]
fn multi_hour_window_contains_every_half_hour_start() {
    let slots = slot_boundaries(
        datetime!(2026-07-16 10:00 UTC),
        datetime!(2026-07-16 12:00 UTC),
    )
    .unwrap();
    assert_eq!(
        slots,
        vec![
            datetime!(2026-07-16 10:00 UTC),
            datetime!(2026-07-16 10:30 UTC),
            datetime!(2026-07-16 11:00 UTC),
            datetime!(2026-07-16 11:30 UTC),
        ]
    );
}

#[test]
fn unaligned_or_backwards_windows_are_rejected() {
    assert!(
        slot_boundaries(
            datetime!(2026-07-16 10:10 UTC),
            datetime!(2026-07-16 10:40 UTC),
        )
        .is_err()
    );
    assert!(
        slot_boundaries(
            datetime!(2026-07-16 11:00 UTC),
            datetime!(2026-07-16 10:30 UTC),
        )
        .is_err()
    );
}

#[test]
fn reservations_are_bounded_to_sixty_days() {
    assert!(
        slot_boundaries(
            datetime!(2026-01-01 00:00 UTC),
            datetime!(2026-03-02 00:30 UTC),
        )
        .is_err()
    );
}
