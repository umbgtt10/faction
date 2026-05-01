// Copyright 2025 Umberto Gotti <umberto.gotti@umbertogotti.dev>
// Licensed under the Apache License, Version 2.0
// http://www.apache.org/licenses/LICENSE-2.0

use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{TcpListener, TcpStream};

use faction::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;

pub struct TcpTransport {
    streams: HashMap<PeerId, TcpStream>,
    buf: Vec<u8>,
}

impl TcpTransport {
    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<TcpTransport> {
        let n = peer_ids.len();
        let mut listeners: Vec<TcpListener> = Vec::new();
        let mut addrs: Vec<String> = Vec::new();
        for _ in 0..n {
            let listener = TcpListener::bind("127.0.0.1:0").unwrap();
            addrs.push(listener.local_addr().unwrap().to_string());
            listeners.push(listener);
        }
        let mut all: Vec<HashMap<PeerId, TcpStream>> = (0..n).map(|_| HashMap::new()).collect();
        for i in 0..n {
            for j in (i + 1)..n {
                let a_to_b = TcpStream::connect(&addrs[j]).unwrap();
                a_to_b.set_nonblocking(true).unwrap();
                all[i].insert(peer_ids[j], a_to_b);
                let (b_to_a, _) = listeners[j].accept().unwrap();
                b_to_a.set_nonblocking(true).unwrap();
                all[j].insert(peer_ids[i], b_to_a);
            }
        }
        all.into_iter().map(|streams| TcpTransport { streams, buf: Vec::new() }).collect()
    }

    fn encode(from: PeerId, tag: u8) -> Vec<u8> {
        let mut data = Vec::with_capacity(9);
        data.extend(&from.to_le_bytes());
        data.push(tag);
        data
    }

    fn decode(data: &[u8]) -> Option<TransportMessage> {
        if data.len() < 9 { return None; }
        let from = PeerId::from_le_bytes(data[0..8].try_into().ok()?);
        match data[8] {
            0 => Some(TransportMessage::Ping { from }),
            1 => Some(TransportMessage::Ready { from }),
            2 => Some(TransportMessage::Bootstrapped { from }),
            _ => None,
        }
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        if let Some(stream) = self.streams.get_mut(&to) {
            let data = match &message {
                TransportMessage::Ping { from } => Self::encode(*from, 0),
                TransportMessage::Ready { from } => Self::encode(*from, 1),
                TransportMessage::Bootstrapped { from } => Self::encode(*from, 2),
            };
            let _ = stream.write(&data);
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        for stream in self.streams.values_mut() {
            let mut tmp = [0u8; 64];
            match stream.read(&mut tmp) {
                Ok(0) => continue,
                Ok(n) => self.buf.extend_from_slice(&tmp[..n]),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(_) => continue,
            }
        }
        if self.buf.len() >= 9 {
            let msg = Self::decode(&self.buf[..9]);
            self.buf.drain(..9);
            msg
        } else {
            None
        }
    }
}
