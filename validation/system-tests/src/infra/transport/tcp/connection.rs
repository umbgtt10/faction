// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction_protocol::transport_message::TransportMessage;
use std::io::{ErrorKind, Read, Write};
use std::iter::from_fn;
use std::net::TcpStream;
use std::time::Duration;

use crate::infra::transport::frame_buffer::FrameBuffer;
use crate::infra::transport::wire_codec::WireCodec;
use crate::traits::pause::Pause;

const CHUNK: usize = 64;

// One TCP connection with the bytes it has delivered so far, kept apart from
// every other connection's: a frame split across reads is joined with its own
// rest, never with another connection's bytes.
pub struct Connection {
    stream: TcpStream,
    frames: FrameBuffer,
}

impl Connection {
    #[must_use]
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            frames: FrameBuffer::new(),
        }
    }

    pub fn fill(&mut self) {
        let mut chunk = [0u8; CHUNK];
        if let Ok(read) = self.stream.read(&mut chunk) {
            self.frames.push(&chunk[..read]);
        }
    }

    // Every whole message this connection has delivered, in the order it sent
    // them; a frame not yet whole waits for its rest.
    pub fn messages(&mut self) -> Vec<TransportMessage> {
        from_fn(|| self.frames.next_frame())
            .filter_map(|frame| WireCodec::decode(&frame))
            .collect()
    }

    pub fn write(&mut self, data: &[u8], pause: &dyn Pause) -> bool {
        let mut written = 0;
        while written < data.len() {
            match self.stream.write(&data[written..]) {
                Ok(0) => return false,
                Ok(n) => written += n,
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                    pause.pause(Duration::from_millis(1));
                }
                Err(_) => return false,
            }
        }
        true
    }
}
