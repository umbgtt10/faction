// Copyright (c) 2025-2026 Umberto Gotti
// SPDX-License-Identifier: MIT

use std::collections::{HashMap, VecDeque};
use std::net::{SocketAddr, TcpListener};
use std::sync::{Arc, Mutex, OnceLock};
use std::thread::available_parallelism;

use faction::types::PeerId;
use faction_protocol::transport_message::TransportMessage;
use faction_protocol::transport_trait::Transport;
use tokio::net::TcpListener as TokioTcpListener;
use tokio::runtime::{Builder as RuntimeBuilder, Runtime};
use tokio::sync::Mutex as AsyncMutex;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender, unbounded_channel};
use tokio::sync::oneshot::{Sender as OneshotSender, channel as oneshot_channel};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::Request;
use tonic::transport::{Channel, Endpoint, Server};

use crate::faction::Envelope;
use crate::faction::transport_client::TransportClient;
use crate::faction::transport_server::TransportServer;
use crate::transport::grpc::grpc_service::GrpcSvc;

type Inbox = Arc<Mutex<VecDeque<TransportMessage>>>;
type Clients = Arc<AsyncMutex<HashMap<PeerId, TransportClient<Channel>>>>;
type Tx = UnboundedSender<(PeerId, TransportMessage)>;

pub struct GrpcTransport {
    inbox: Inbox,
    _shutdown_tx: Option<OneshotSender<()>>,
    _tx: Tx,
}

impl Drop for GrpcTransport {
    fn drop(&mut self) {
        if let Some(tx) = self._shutdown_tx.take() {
            let _ = tx.send(());
        }
    }
}

impl GrpcTransport {
    fn encode(m: &TransportMessage) -> Vec<u8> {
        let (f, t): (PeerId, u8) = match m {
            TransportMessage::Ping { from } => (*from, 0),
            TransportMessage::Ready { from } => (*from, 1),
            TransportMessage::Bootstrapped { from } => (*from, 2),
        };
        let mut d = Vec::with_capacity(9);
        d.extend(&f.to_le_bytes());
        d.push(t);
        d
    }

    fn spawn_sender(mut rx: UnboundedReceiver<(PeerId, TransportMessage)>, clients: Clients) {
        Self::runtime().spawn(async move {
            while let Some((to, msg)) = rx.recv().await {
                let mut guard = clients.lock().await;
                if let Some(c) = guard.get_mut(&to) {
                    let d = Self::encode(&msg);
                    let _ = c.deliver(Request::new(Envelope { data: d })).await;
                }
            }
        });
    }

    fn runtime() -> &'static Runtime {
        static RT: OnceLock<Runtime> = OnceLock::new();
        RT.get_or_init(|| {
            RuntimeBuilder::new_multi_thread()
                .worker_threads(available_parallelism().map(|p| p.get()).unwrap_or(2))
                .enable_all()
                .build()
                .unwrap()
        })
    }

    fn spawn_server(inbox: Inbox, stream: TcpListenerStream) -> OneshotSender<()> {
        let (shutdown_tx, shutdown_rx) = oneshot_channel();
        Self::runtime().spawn(async move {
            tokio::select! {
                _ = Server::builder()
                    .add_service(TransportServer::new(GrpcSvc(inbox)))
                    .serve_with_incoming(stream) => {},
                _ = shutdown_rx => {},
            }
        });
        shutdown_tx
    }

    fn build_server(inbox: Inbox, listen_addr: SocketAddr) -> OneshotSender<()> {
        let l = TcpListener::bind(listen_addr).unwrap();
        l.set_nonblocking(true).unwrap();
        let tl = TokioTcpListener::from_std(l).unwrap();
        Self::spawn_server(inbox, TcpListenerStream::new(tl))
    }

    fn build_clients(peer_id: PeerId, peer_addrs: &[(PeerId, SocketAddr)]) -> Clients {
        let rt = Self::runtime();
        let mut map = HashMap::new();
        for &(pid, addr) in peer_addrs {
            if pid != peer_id {
                let ch = rt.block_on(async {
                    Endpoint::from_shared(format!("http://{addr}"))
                        .unwrap()
                        .connect()
                        .await
                        .unwrap()
                });
                map.insert(pid, TransportClient::new(ch));
            }
        }
        Arc::new(AsyncMutex::new(map))
    }

    pub fn new(
        listen_addr: SocketAddr,
        peer_id: PeerId,
        peer_addrs: &[(PeerId, SocketAddr)],
    ) -> Self {
        let _guard = Self::runtime().enter();
        let inbox: Inbox = Arc::new(Mutex::new(VecDeque::new()));

        let shutdown_tx = Self::build_server(inbox.clone(), listen_addr);
        let clients = Self::build_clients(peer_id, peer_addrs);

        let (tx, rx) = unbounded_channel();
        Self::spawn_sender(rx, clients);

        Self {
            inbox,
            _shutdown_tx: Some(shutdown_tx),
            _tx: tx,
        }
    }

    pub fn new_mesh(peer_ids: &[PeerId]) -> Vec<GrpcTransport> {
        let n = peer_ids.len();
        let rt = Self::runtime();
        let mut peer_addrs = Vec::new();
        let mut inboxes = Vec::new();
        let mut shutdown_txs = Vec::new();

        {
            let _guard = rt.enter();
            for _ in 0..n {
                let ib: Inbox = Arc::new(Mutex::new(VecDeque::new()));
                let ib_for_server = ib.clone();
                let l = TcpListener::bind("127.0.0.1:0").unwrap();
                l.set_nonblocking(true).unwrap();
                let a = l.local_addr().unwrap();
                peer_addrs.push((PeerId::default(), a));
                let tl = TokioTcpListener::from_std(l).unwrap();
                let stx = Self::spawn_server(ib_for_server, TcpListenerStream::new(tl));
                shutdown_txs.push(stx);
                inboxes.push(ib);
            }
        }

        let peer_addrs: Vec<(PeerId, SocketAddr)> = peer_ids
            .iter()
            .zip(peer_addrs.iter().map(|(_, a)| a))
            .map(|(&id, &a)| (id, a))
            .collect();

        let client_lists: Vec<Clients> = peer_ids
            .iter()
            .map(|&pid| Self::build_clients(pid, &peer_addrs))
            .collect();

        inboxes
            .into_iter()
            .zip(client_lists)
            .zip(shutdown_txs)
            .map(|((ib, clients), stx)| {
                let (tx, rx) = unbounded_channel();
                Self::spawn_sender(rx, clients);
                GrpcTransport {
                    inbox: ib,
                    _shutdown_tx: Some(stx),
                    _tx: tx,
                }
            })
            .collect()
    }
}

impl Transport for GrpcTransport {
    fn send(&mut self, to: PeerId, m: TransportMessage) {
        let _ = self._tx.send((to, m));
    }

    fn recv(&mut self) -> Option<TransportMessage> {
        self.inbox.lock().unwrap().pop_front()
    }
}
