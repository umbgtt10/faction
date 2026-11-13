// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_spec_model::rules::change::Change;
use faction_spec_model::rules::rule::Rule;
use faction_spec_model::rules::rule_book::RuleBook;
use faction_spec_model::rules::situation::Situation;
use std::collections::BTreeSet;

#[test]
fn row_ids_names_each_known_row_once_the_probe_s_first() {
    // Arrange & Act
    let row_ids = RuleBook::row_ids();

    // Assert
    let distinct: BTreeSet<&str> = row_ids.iter().copied().collect();
    assert_eq!(row_ids.len(), 56);
    assert_eq!(distinct.len(), 56);
    assert_eq!(row_ids[0], "probed");
}

#[test]
fn rule_of_a_row_that_concludes_on_a_new_count_sets_the_count_first() {
    // Arrange & Act
    let rule = RuleBook::rule("count-concluded");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::CountConcludes,
            change: Change::CountSet,
        }
    );
}

#[test]
fn rule_of_a_row_that_concludes_on_readiness_counts_the_peer_in() {
    // Arrange & Act
    let rule = RuleBook::rule("ready-concluded");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::PeerConcludes,
            change: Change::PeerCollects,
        }
    );
}

#[test]
#[should_panic(expected = "a row the model knows")]
fn rule_of_a_row_the_model_does_not_know_panics() {
    // Arrange & Act & Assert
    let _ = RuleBook::rule("ready-forgotten");
}

#[test]
fn rule_of_the_row_answering_what_a_stage_does_not_admit_changes_nothing() {
    // Arrange & Act
    let rule = RuleBook::rule("not-admitted");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::NotAdmitted,
            change: Change::Nothing,
        }
    );
}

#[test]
fn rule_of_the_row_completing_an_addition_admits_the_pending_peer() {
    // Arrange & Act
    let rule = RuleBook::rule("addition-completed");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::PeerIsPending,
            change: Change::AdditionCompletes,
        }
    );
}

#[test]
fn rule_of_the_row_refusing_a_removal_asks_whether_the_quorum_can_spare_the_peer() {
    // Arrange & Act
    let rule = RuleBook::rule("removal-refused");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::QuorumCannotSpare,
            change: Change::Nothing,
        }
    );
}

#[test]
fn rule_of_the_row_removing_the_node_itself_asks_whether_the_peer_is_the_node() {
    // Arrange & Act
    let rule = RuleBook::rule("node-removed");

    // Assert
    assert_eq!(
        rule,
        Rule {
            situation: Situation::PeerIsSelf,
            change: Change::NodeRemoved,
        }
    );
}
