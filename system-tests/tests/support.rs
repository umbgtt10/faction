// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::env::var;
use std::path::PathBuf;

use chrono::Utc;

fn logs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("logs")
}

fn run_id() -> String {
    var("FACTION_LOG_RUN").unwrap_or_else(|_| Utc::now().format("%Y%m%d_%H%M%S").to_string())
}

pub fn log_dir(label: &str) -> PathBuf {
    logs_root().join(run_id()).join(label.to_lowercase())
}
