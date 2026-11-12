// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use xtask::artifacts::run_stamp::RunStamp;

#[test]
fn to_directory_name_at_the_unix_epoch_returns_the_first_of_january_1970() {
    // Arrange & Act
    let name = RunStamp::new(0).to_directory_name();

    // Assert
    assert_eq!(name, "1970-01-01_00-00-00");
}

#[test]
fn to_directory_name_on_a_leap_day_returns_the_twenty_ninth_of_february() {
    // Arrange & Act
    let name = RunStamp::new(1_709_164_800).to_directory_name();

    // Assert
    assert_eq!(name, "2024-02-29_00-00-00");
}

// The directory name is sorted as text to decide which runs to keep, so every
// field has to be zero-padded to a fixed width. An unpadded month would sort
// "2026-9" after "2026-10" and delete the wrong run.
#[test]
fn to_directory_name_pads_every_field_to_a_fixed_width() {
    // Arrange & Act
    let name = RunStamp::new(1_141_333_925).to_directory_name();

    // Assert
    assert_eq!(name, "2006-03-02_21-12-05");
    assert_eq!(name.len(), 19);
}

// Two stamps a second apart must sort in the order they were made, which is
// what the retention sweep relies on to decide which run is oldest.
#[test]
fn to_directory_name_sorts_chronologically_as_text() {
    // Arrange
    let earlier = RunStamp::new(1_767_225_599).to_directory_name();
    let later = RunStamp::new(1_767_225_600).to_directory_name();

    // Act & Assert
    assert!(earlier < later, "{earlier} should sort before {later}");
    assert_eq!(earlier, "2025-12-31_23-59-59");
    assert_eq!(later, "2026-01-01_00-00-00");
}

// 2100 is divisible by 4 and is not a leap year. The one case a naive
// every-fourth-year rule gets wrong, and the reason this converts through the
// full civil algorithm rather than counting.
#[test]
fn to_directory_name_treats_2100_as_a_common_year() {
    // Arrange & Act
    let name = RunStamp::new(4_107_542_400).to_directory_name();

    // Assert
    assert_eq!(name, "2100-03-01_00-00-00");
}

#[test]
fn to_directory_name_uses_the_last_second_of_a_day_rather_than_rolling_over() {
    // Arrange & Act
    let name = RunStamp::new(1_767_225_599).to_directory_name();

    // Assert
    assert_eq!(name, "2025-12-31_23-59-59");
}

#[test]
fn to_directory_name_with_a_year_2000_timestamp_returns_the_millennium() {
    // Arrange & Act
    let name = RunStamp::new(946_684_800).to_directory_name();

    // Assert
    assert_eq!(name, "2000-01-01_00-00-00");
}
