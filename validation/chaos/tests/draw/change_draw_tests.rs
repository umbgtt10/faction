// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::draw::change_draw::ChangeDraw;
use faction_chaos::model::change::Change;
use faction_chaos::model::planned_change::PlannedChange;

// Every cluster the generator draws, by size and required count, with the
// changes a seed draws for it.
fn drawn() -> Vec<(usize, usize, Vec<PlannedChange>)> {
    (3..=6)
        .flat_map(|nodes| (1..=nodes).map(move |required| (nodes, required)))
        .flat_map(|(nodes, required)| {
            (0..50).map(move |seed| {
                (
                    nodes,
                    required,
                    ChangeDraw::new(seed).changes(nodes, required, 200),
                )
            })
        })
        .collect()
}

fn removals(changes: &[PlannedChange]) -> usize {
    changes
        .iter()
        .filter(|planned| matches!(planned.change, Change::Removal { .. }))
        .count()
}

#[test]
fn changes_are_due_within_the_chaos() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    assert!(
        drawn
            .iter()
            .all(|(_, _, changes)| changes.iter().all(|planned| planned.due < 200))
    );
}

#[test]
fn changes_come_due_together_in_some_scenarios_and_apart_in_others() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    let several: Vec<&Vec<PlannedChange>> = drawn
        .iter()
        .map(|(_, _, changes)| changes)
        .filter(|changes| changes.len() >= 2)
        .collect();
    let together =
        |changes: &&Vec<PlannedChange>| changes.iter().all(|planned| planned.due == changes[0].due);
    assert!(several.iter().any(together));
    assert!(several.iter().any(|changes| !together(changes)));
}

#[test]
fn changes_never_remove_below_any_required_count() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    for (nodes, required, changes) in drawn {
        let floor = nodes - removals(&changes);
        assert!(floor >= required, "{nodes} {required} {changes:?}");
        for planned in &changes {
            if let Change::RequiredCount { required_count } = planned.change {
                assert!((1..=floor).contains(&required_count), "{changes:?}");
            }
        }
    }
}

#[test]
fn changes_number_the_newcomers_on_from_the_genesis_peers() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    for (nodes, _, changes) in drawn {
        let newcomers: Vec<u64> = changes
            .iter()
            .filter_map(|planned| match planned.change {
                Change::Addition { peer_id } if peer_id >= nodes as u64 => Some(peer_id),
                _ => None,
            })
            .collect();
        let expected: Vec<u64> = (nodes as u64..nodes as u64 + newcomers.len() as u64).collect();
        assert_eq!(newcomers, expected);
    }
}

#[test]
fn changes_of_one_seed_are_drawn_alike() {
    // Arrange
    let first = ChangeDraw::new(9).changes(5, 2, 200);

    // Act
    let second = ChangeDraw::new(9).changes(5, 2, 200);

    // Assert
    assert_eq!(first, second);
}

#[test]
fn changes_take_a_member_back_only_after_its_removal() {
    // Arrange & Act
    let drawn = drawn();

    // Assert
    for (nodes, _, changes) in drawn {
        for planned in &changes {
            if let Change::Addition { peer_id } = planned.change
                && peer_id < nodes as u64
            {
                let after = planned.after.map(|after| changes[after].change);
                assert_eq!(after, Some(Change::Removal { peer_id }), "{changes:?}");
            }
        }
    }
}
