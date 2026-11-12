// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use crate::artifacts::run_log_failure::RunLogFailure;
use std::fs::create_dir;
use std::fs::create_dir_all;
use std::fs::read_dir;
use std::fs::remove_dir_all;
use std::io::Error;
use std::path::Path;
use std::path::PathBuf;

// YYYY-MM-DD_HH-MM-SS. Matched by shape rather than parsed, because the only
// question being asked is whether this directory belongs to the rotation --
// anything else under logs/ was put there by hand or by another tool and is
// not this code's to delete.
const STAMP_LENGTH: usize = 19;

// The root every stage 5 run keeps its logs under, one directory per run.
pub struct RunLogDirectory {
    root: PathBuf,
    runs_to_keep: usize,
}

impl RunLogDirectory {
    pub fn new(root: PathBuf, runs_to_keep: usize) -> Self {
        Self { root, runs_to_keep }
    }

    // Makes this run's directory and drops the oldest beyond the limit. The new
    // one is created first so it counts towards the total, which is what makes
    // "keep 30" mean thirty runs of history rather than thirty plus this one.
    //
    // Created rather than reused: slotgate writes one log per job, so a
    // directory that was already there could hold a log from a job that no
    // longer exists, and it would read afterwards as a result of this run. A
    // directory this call made holds nothing it did not write.
    pub fn prepare(&self, name: &str) -> Result<PathBuf, RunLogFailure> {
        create_dir_all(&self.root).map_err(|error| RunLogFailure::Create {
            path: self.root.clone(),
            reason: error.to_string(),
        })?;

        let run_directory = self.root.join(name);
        create_dir(&run_directory).map_err(|error| RunLogFailure::Create {
            path: run_directory.clone(),
            reason: error.to_string(),
        })?;

        for stale in self.expired()? {
            remove_dir_all(&stale).map_err(|error| RunLogFailure::Remove {
                path: stale.clone(),
                reason: error.to_string(),
            })?;
        }

        Ok(run_directory)
    }

    // Newest first by name, which is chronological because the stamp is fixed
    // width and most significant first.
    fn expired(&self) -> Result<Vec<PathBuf>, RunLogFailure> {
        let mut stamped = self.stamped_directories()?;
        stamped.sort();
        stamped.reverse();

        Ok(stamped.into_iter().skip(self.runs_to_keep).collect())
    }

    fn is_run_stamp(path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return false;
        };

        name.len() == STAMP_LENGTH
            && name
                .chars()
                .enumerate()
                .all(|(index, character)| match index {
                    4 | 7 | 13 | 16 => character == '-',
                    10 => character == '_',
                    _ => character.is_ascii_digit(),
                })
    }

    fn stamped_directories(&self) -> Result<Vec<PathBuf>, RunLogFailure> {
        let unreadable = |error: Error| RunLogFailure::Read {
            path: self.root.clone(),
            reason: error.to_string(),
        };

        let mut stamped = Vec::new();
        for entry in read_dir(&self.root).map_err(unreadable)? {
            let path = entry.map_err(unreadable)?.path();
            if path.is_dir() && Self::is_run_stamp(&path) {
                stamped.push(path);
            }
        }

        Ok(stamped)
    }
}
