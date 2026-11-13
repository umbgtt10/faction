// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

const LENGTH: usize = 4;

// The bytes one connection has delivered, handed out a whole frame at a time:
// a frame split across reads waits for its rest.
#[derive(Debug, Default)]
pub struct FrameBuffer {
    bytes: Vec<u8>,
}

impl FrameBuffer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn next_frame(&mut self) -> Option<Vec<u8>> {
        let length = u32::from_le_bytes(self.bytes.get(..LENGTH)?.try_into().ok()?);
        let end = LENGTH + usize::try_from(length).ok()?;
        (self.bytes.len() >= end).then(|| self.bytes.drain(..end).collect())
    }

    pub fn push(&mut self, bytes: &[u8]) {
        self.bytes.extend_from_slice(bytes);
    }
}
