// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::fs::File;
use std::fs::OpenOptions;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone)]
pub struct SharedWriter {
    file: Arc<Mutex<BufWriter<File>>>,
}

impl SharedWriter {
    #[must_use]
    pub fn open(path: &Path) -> Self {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .unwrap();
        Self {
            file: Arc::new(Mutex::new(BufWriter::new(file))),
        }
    }

    pub fn write_line(&self, line: &str) {
        let mut file = self.file.lock().unwrap();
        let _ = writeln!(file, "{line}");
        let _ = file.flush();
    }
}
