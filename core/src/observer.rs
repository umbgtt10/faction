// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use alloc::vec::Vec;

use crate::cluster_view::ClusterView;
use crate::command::Command;
use crate::transition::Transition;

pub trait Observer: Send {
    fn observe(&mut self, command: Command, transition: Transition);

    fn observe_query(&mut self, command: Command, cluster_view: ClusterView);

    fn observe_rejection(
        &mut self,
        command: Command,
        cluster_view: ClusterView,
        admissible: Vec<Command>,
    );
}
