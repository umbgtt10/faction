// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

// The crate against the specification's model: random command sequences, every
// command's peer among a five-member cluster's ids and two that are not
// members, and every required count from none to more than the cluster holds,
// each answered by both -- the crate and the model meeting only in the
// specification's encoding -- and every answer the same.

use crate::support::random_commands::RandomCommands;
use crate::support::spec_files::SpecFiles;
use crate::support::spec_writer::SpecWriter;
use faction::config::Config;
use faction::faction::Faction;
use faction::no_op_observer::NoOpObserver;
use faction::quorum_policy::QuorumPolicy;
use faction_spec_model::model::Model;
use faction_spec_model::spec_reader::SpecReader;
use proptest::prelude::*;

proptest! {
    #[test]
    fn process_of_random_commands_answers_as_the_specification_s_model(
        required_count in 0usize..=6,
        threshold in 1usize..=3,
        commands in RandomCommands::sequence(40),
    ) {
        // Arrange
        let config = Config::new(0, vec![0, 1, 2, 3, 4], QuorumPolicy::new(required_count), threshold);
        let mut model = Model::new(
            SpecReader::table(&SpecFiles::matrix()),
            SpecReader::configuration(&SpecWriter::configuration(&config)),
        );
        let mut faction = Faction::new(config, Box::new(NoOpObserver));

        // Act
        let departure = commands.iter().enumerate().find_map(|(index, command)| {
            let answered = SpecWriter::result(&faction.process(*command));
            let modelled = model.process(&SpecWriter::command(command)).result;
            (answered != modelled).then(|| {
                format!("call {}, {command:?}: the crate answers {answered}, the model {modelled}", index + 1)
            })
        });

        // Assert
        prop_assert!(departure.is_none(), "{}", departure.unwrap_or_default());
    }
}
