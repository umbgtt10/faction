// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::timer::wall_time::WallTime;
use std::time::Duration;

fn at(seconds: u64) -> WallTime {
    WallTime::new(Duration::from_secs(seconds))
}

#[test]
fn after_moves_the_moment_on_by_the_time_elapsed() {
    // Arrange
    let origin = at(1_767_225_600);

    // Act
    let later = origin.after(Duration::from_millis(1_750));

    // Assert
    assert_eq!(later.to_stamp(), "2026-01-01T00:00:01.750Z");
}

#[test]
fn after_past_the_longest_duration_stops_there() {
    // Arrange
    let last = WallTime::new(Duration::MAX);

    // Act
    let later = last.after(Duration::from_secs(1));

    // Assert
    assert_eq!(later, last);
}

#[test]
fn nanoseconds_counts_them_since_the_epoch() {
    // Arrange & Act
    let nanoseconds = at(1_767_225_600).nanoseconds();

    // Assert
    assert_eq!(nanoseconds, 1_767_225_600_000_000_000);
}

#[test]
fn nanoseconds_past_what_a_u64_holds_returns_its_most() {
    // Arrange & Act
    let nanoseconds = WallTime::new(Duration::MAX).nanoseconds();

    // Assert
    assert_eq!(nanoseconds, u64::MAX);
}

// A run's folder is chosen as the newest by its name sorted as text, so every
// field is padded to its width.
#[test]
fn to_folder_name_pads_every_field_to_a_fixed_width() {
    // Arrange & Act
    let name = at(1_141_333_925).to_folder_name();

    // Assert
    assert_eq!(name, "2006-03-02_21-12-05");
}

#[test]
fn to_folder_name_sorts_as_text_in_the_order_the_moments_came() {
    // Arrange
    let earlier = at(1_767_225_599).to_folder_name();

    // Act
    let later = at(1_767_225_600).to_folder_name();

    // Assert
    assert_eq!(
        [earlier.as_str(), later.as_str()],
        ["2025-12-31_23-59-59", "2026-01-01_00-00-00"]
    );
    assert!(earlier < later);
}

#[test]
fn to_stamp_at_the_unix_epoch_returns_the_first_of_january_1970() {
    // Arrange & Act
    let stamp = at(0).to_stamp();

    // Assert
    assert_eq!(stamp, "1970-01-01T00:00:00.000Z");
}

#[test]
fn to_stamp_keeps_the_milliseconds() {
    // Arrange & Act
    let stamp = WallTime::new(Duration::from_millis(1_794_486_645_123)).to_stamp();

    // Assert
    assert_eq!(stamp, "2026-11-12T12:30:45.123Z");
}

#[test]
fn to_stamp_on_a_leap_day_returns_the_twenty_ninth_of_february() {
    // Arrange & Act
    let stamp = at(1_709_164_800).to_stamp();

    // Assert
    assert_eq!(stamp, "2024-02-29T00:00:00.000Z");
}

#[test]
fn to_stamp_on_the_last_second_of_a_year_stays_in_it() {
    // Arrange & Act
    let stamp = at(1_767_225_599).to_stamp();

    // Assert
    assert_eq!(stamp, "2025-12-31T23:59:59.000Z");
}

// 2000 is divisible by 400, and a leap year.
#[test]
fn to_stamp_treats_2000_as_a_leap_year() {
    // Arrange & Act
    let stamp = at(951_782_400).to_stamp();

    // Assert
    assert_eq!(stamp, "2000-02-29T00:00:00.000Z");
}

// 2100 is divisible by 4 and not by 400, and no leap year: the one case a
// rule of every fourth year gets wrong.
#[test]
fn to_stamp_treats_2100_as_a_common_year() {
    // Arrange & Act
    let stamp = at(4_107_542_400).to_stamp();

    // Assert
    assert_eq!(stamp, "2100-03-01T00:00:00.000Z");
}
