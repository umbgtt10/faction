// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_chaos::tee::Tee;
use std::io::Result as IoResult;
use std::io::Write;
use std::sync::Arc;
use std::sync::Mutex;

#[derive(Clone, Default)]
struct SharedOutput {
    bytes: Arc<Mutex<Vec<u8>>>,
}

impl SharedOutput {
    fn text(&self) -> String {
        String::from_utf8(self.bytes.lock().unwrap().clone()).unwrap()
    }
}

impl Write for SharedOutput {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        self.bytes.lock().unwrap().extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

#[test]
fn write_writes_every_byte_to_both_writers() {
    // Arrange
    let (terminal, log) = (SharedOutput::default(), SharedOutput::default());
    let mut tee = Tee::new(Box::new(terminal.clone()), Box::new(log.clone()));

    // Act
    writeln!(tee, "soak from seed 5: 0 scenarios, 0 failed").unwrap();
    tee.flush().unwrap();

    // Assert
    assert_eq!(terminal.text(), "soak from seed 5: 0 scenarios, 0 failed\n");
    assert_eq!(log.text(), terminal.text());
}
