// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::cluster_view::ClusterView;
use faction::instance::peer_id::PeerId;
use faction::process_result::ProcessResult;

// The view a result carries: every result carries one, whatever it answered.
pub struct ResultView;

impl ResultView {
    #[must_use]
    pub fn of<P: PeerId>(result: ProcessResult<P>) -> ClusterView<P> {
        match result {
            ProcessResult::Accepted { cluster_view, .. }
            | ProcessResult::Rejected { cluster_view, .. }
            | ProcessResult::Probed { cluster_view, .. } => cluster_view,
        }
    }
}
