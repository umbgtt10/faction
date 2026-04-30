// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use crate::cluster_view::ClusterView;

pub trait StateClusterView {
    fn cluster_view(&self, previous: &ClusterView) -> ClusterView;
}
