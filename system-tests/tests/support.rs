// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::path::PathBuf;

use chrono::Utc;

pub fn log_path(label: &str) -> PathBuf {
    let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
    PathBuf::from("logs").join(format!("{timestamp}_{label}.jsonl").to_lowercase())
}
