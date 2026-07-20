// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use std::collections::HashMap;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{JoinHandle, sleep, spawn};
use std::time::Duration;

pub struct TcpTransport {
    outbound: HashMap<PeerId, TcpStream>,
    inbound: Arc<Mutex<Vec<TcpStream>>>,
    buf: Vec<u8>,
    _listener: TcpListener,
    _shutdown: Arc<AtomicBool>,
    _accept_thread: Option<JoinHandle<()>>,
}

impl Drop for TcpTransport {
    fn drop(&mut self) {
        self._shutdown.store(true, Ordering::Relaxed);
        if let Some(handle) = self._accept_thread.take() {
            // Wake up accept() by connecting to our own listener
            if let Ok(addr) = self._listener.local_addr() {
                let _ = TcpStream::connect(addr);
            }
            let _ = handle.join();
        }
    }
}

impl TcpTransport {
    pub fn new(
        listen_addr: SocketAddr,
        peer_id: PeerId,
        peer_addrs: &[(PeerId, SocketAddr)],
    ) -> Self {
        let inbound = Arc::new(Mutex::new(Vec::new()));
        let listener = TcpListener::bind(listen_addr).unwrap();

        {
            let probe = TcpStream::connect(listen_addr).unwrap();
            let (conn, _) = listener.accept().unwrap();
            drop(probe);
            drop(conn);
        }

        let shutdown = Arc::new(AtomicBool::new(false));
        let thread_listener = listener.try_clone().unwrap();
        let ib = inbound.clone();
        let sd = shutdown.clone();
        let accept_thread = spawn(move || {
            for incoming in thread_listener.incoming() {
                if sd.load(Ordering::Relaxed) {
                    break;
                }
                match incoming {
                    Ok(stream) => {
                        stream.set_nonblocking(true).unwrap();
                        ib.lock().unwrap().push(stream);
                    }
                    Err(_) => break,
                }
            }
        });

        let mut outbound = HashMap::new();
        for &(pid, addr) in peer_addrs {
            if pid != peer_id {
                let mut attempts = 0;
                let stream = loop {
                    match TcpStream::connect(addr) {
                        Ok(s) => break s,
                        Err(_) if attempts < 150 => {
                            attempts += 1;
                            sleep(Duration::from_millis(100));
                        }
                        Err(e) => panic!("failed to connect to {addr}: {e}"),
                    }
                };
                stream.set_nonblocking(true).unwrap();
                outbound.insert(pid, stream);
            }
        }

        Self {
            outbound,
            inbound,
            buf: Vec::new(),
            _listener: listener,
            _shutdown: shutdown,
            _accept_thread: Some(accept_thread),
        }
    }

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
        all.into_iter()
            .map(|streams| TcpTransport {
                outbound: streams,
                inbound: Arc::new(Mutex::new(Vec::new())),
                buf: Vec::new(),
                _listener: TcpListener::bind("127.0.0.1:0").unwrap(),
                _shutdown: Arc::new(AtomicBool::new(false)),
                _accept_thread: None,
            })
            .collect()
    }

    fn encode(from: PeerId, tag: u8) -> Vec<u8> {
        let mut data = Vec::with_capacity(9);
        data.extend(&from.to_le_bytes());
        data.push(tag);
        data
    }

    fn decode(data: &[u8]) -> Option<TransportMessage> {
        if data.len() < 9 {
            return None;
        }
        let from = PeerId::from_le_bytes(data[0..8].try_into().ok()?);
        match data[8] {
            0 => Some(TransportMessage::Ping { from }),
            1 => Some(TransportMessage::Ready { from }),
            2 => Some(TransportMessage::Bootstrapped { from }),
            _ => None,
        }
    }

    fn read_streams(streams: &mut dyn Iterator<Item = &mut TcpStream>, buf: &mut Vec<u8>) {
        for stream in streams {
            let mut tmp = [0u8; 64];
            match stream.read(&mut tmp) {
                Ok(0) => continue,
                Ok(n) => buf.extend_from_slice(&tmp[..n]),
                Err(ref e) if e.kind() == ErrorKind::WouldBlock => continue,
                Err(_) => continue,
            }
        }
    }
}

impl Transport for TcpTransport {
    fn send(&mut self, to: PeerId, message: TransportMessage) {
        if let Some(stream) = self.outbound.get_mut(&to) {
            let data = match &message {
                TransportMessage::Ping { from } => Self::encode(*from, 0),
                TransportMessage::Ready { from } => Self::encode(*from, 1),
                TransportMessage::Bootstrapped { from } => Self::encode(*from, 2),
            };
            let mut written = 0;
            while written < data.len() {
                match stream.write(&data[written..]) {
                    Ok(0) => break,
                    Ok(n) => written += n,
                    Err(ref e) if e.kind() == ErrorKind::WouldBlock => {
                        sleep(Duration::from_millis(1));
                    }
                    Err(_) => break,
                }
            }
        }
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        Self::read_streams(&mut self.outbound.values_mut(), &mut self.buf);
        Self::read_streams(&mut self.inbound.lock().unwrap().iter_mut(), &mut self.buf);

        if self.buf.len() >= 9 {
            let msg = Self::decode(&self.buf[..9]);
            self.buf.drain(..9);
            msg
        } else {
            None
        }
    }
}
