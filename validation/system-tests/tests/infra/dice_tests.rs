// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_system_tests::infra::dice::Dice;

#[test]
fn below_draws_under_its_bound() {
    // Arrange
    let mut dice = Dice::new(42);

    // Act
    let draws: Vec<u64> = (0..5).map(|_| dice.below(10)).collect();

    // Assert
    assert_eq!(draws, vec![5, 0, 9, 9, 1]);
}

#[test]
fn draw_of_seed_one_follows_the_xorshift_sequence() {
    // Arrange
    let mut dice = Dice::new(1);

    // Act
    let draws: Vec<u64> = (0..3).map(|_| dice.draw()).collect();

    // Assert
    assert_eq!(
        draws,
        vec![
            1_082_269_761,
            1_152_992_998_833_853_505,
            11_177_516_664_432_764_457
        ]
    );
}

#[test]
fn new_of_an_even_seed_draws_as_its_odd_neighbour() {
    // Arrange
    let mut even = Dice::new(6);
    let mut odd = Dice::new(7);

    // Act
    let draws: Vec<(u64, u64)> = (0..3).map(|_| (even.draw(), odd.draw())).collect();

    // Assert
    assert!(draws.iter().all(|(even, odd)| even == odd));
}

#[test]
fn rolls_at_a_hundred_always_succeeds() {
    // Arrange
    let mut dice = Dice::new(7);

    // Act
    let successes = (0..100).filter(|_| dice.rolls(100)).count();

    // Assert
    assert_eq!(successes, 100);
}

#[test]
fn rolls_at_fifty_succeeds_as_often_as_its_seed_draws() {
    // Arrange
    let mut dice = Dice::new(7);

    // Act
    let successes = (0..100).filter(|_| dice.rolls(50)).count();

    // Assert
    assert_eq!(successes, 53);
}

#[test]
fn rolls_at_zero_never_succeeds() {
    // Arrange
    let mut dice = Dice::new(7);

    // Act
    let successes = (0..100).filter(|_| dice.rolls(0)).count();

    // Assert
    assert_eq!(successes, 0);
}
