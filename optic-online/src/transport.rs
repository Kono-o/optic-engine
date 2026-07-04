use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use optic_core::{NetworkConfig, NetworkMode, PeerId};
use tokio::net::UdpSocket;
use tokio::sync::mpsc as tokio_mpsc;

use crate::channels::{LifecycleEvent, TransportCommand};

const PEER_TIMEOUT: Duration = Duration::from_secs(10);
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(2);

/// Runs the network I/O loop on a tokio runtime. Spawned in its own OS thread.
/// Sends the bound local address through `bound_addr_tx` as soon as the socket
/// is bound (host mode) or immediately (client mode sends `None`).
pub(crate) async fn run_transport(
    config: NetworkConfig,
    inbound_data_tx: tokio_mpsc::UnboundedSender<(PeerId, Vec<u8>)>,
    lifecycle_tx: tokio_mpsc::UnboundedSender<LifecycleEvent>,
    mut outbound_rx: tokio_mpsc::UnboundedReceiver<TransportCommand>,
    bound_addr_tx: mpsc::Sender<Option<SocketAddr>>,
) {
    let result = run_transport_inner(config, inbound_data_tx, lifecycle_tx, &mut outbound_rx, bound_addr_tx).await;
    if let Err(e) = result {
        optic_core::log_warn!("[optic-online] transport error: {e}");
    }
}

async fn run_transport_inner(
    config: NetworkConfig,
    inbound_data_tx: tokio_mpsc::UnboundedSender<(PeerId, Vec<u8>)>,
    lifecycle_tx: tokio_mpsc::UnboundedSender<LifecycleEvent>,
    outbound_rx: &mut tokio_mpsc::UnboundedReceiver<TransportCommand>,
    bound_addr_tx: mpsc::Sender<Option<SocketAddr>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match &config.mode {
        NetworkMode::Host { port } => run_host(*port, config.max_peers, inbound_data_tx, lifecycle_tx, outbound_rx, bound_addr_tx).await,
        NetworkMode::Client { addr } => run_client(addr, inbound_data_tx, lifecycle_tx, outbound_rx, bound_addr_tx).await,
    }
}

// ── Host mode ──────────────────────────────────────────────────────────────

struct PeerEntry {
    addr: SocketAddr,
    last_recv: Instant,
}

async fn run_host(
    port: u16,
    max_peers: u32,
    inbound_data_tx: tokio_mpsc::UnboundedSender<(PeerId, Vec<u8>)>,
    lifecycle_tx: tokio_mpsc::UnboundedSender<LifecycleEvent>,
    outbound_rx: &mut tokio_mpsc::UnboundedReceiver<TransportCommand>,
    bound_addr_tx: mpsc::Sender<Option<SocketAddr>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let bind_addr: SocketAddr = ([0, 0, 0, 0], port).into();
    let socket = UdpSocket::bind(bind_addr).await?;
    let local_addr = socket.local_addr().ok();
    let _ = bound_addr_tx.send(local_addr);

    let mut buf = vec![0u8; 2048];

    let mut peers: HashMap<PeerId, PeerEntry> = HashMap::new();
    let mut addr_to_peer: HashMap<SocketAddr, PeerId> = HashMap::new();
    let mut next_peer_id: u64 = 1;

    let mut heartbeat_interval = tokio::time::interval(HEARTBEAT_INTERVAL);
    let mut stale_check = tokio::time::interval(PEER_TIMEOUT);

    loop {
        tokio::select! {
            // ── Receive from socket ────────────────────────────────────────
            result = socket.recv_from(&mut buf) => {
                let (n, src) = result?;
                let data = buf[..n].to_vec();
                let peer_id = match addr_to_peer.get(&src) {
                    Some(&pid) => pid,
                    None => {
                        if peers.len() as u32 >= max_peers {
                            continue;
                        }
                        let pid = PeerId(next_peer_id);
                        next_peer_id += 1;
                        addr_to_peer.insert(src, pid);
                        peers.insert(pid, PeerEntry { addr: src, last_recv: Instant::now() });
                        let _ = lifecycle_tx.send(LifecycleEvent::Connected(pid));
                        pid
                    }
                };
                if let Some(entry) = peers.get_mut(&peer_id) {
                    entry.last_recv = Instant::now();
                }
                let _ = inbound_data_tx.send((peer_id, data));
            }

            // ── Outbound commands ──────────────────────────────────────────
            cmd = outbound_rx.recv() => {
                match cmd {
                    Some(TransportCommand::SendTo(peer, data)) => {
                        if let Some(entry) = peers.get(&peer) {
                            let _ = socket.send_to(&data, entry.addr).await;
                        }
                    }
                    Some(TransportCommand::SendAll(data)) => {
                        for entry in peers.values() {
                            let _ = socket.send_to(&data, entry.addr).await;
                        }
                    }
                    Some(TransportCommand::SendAllExcept(exclude, data)) => {
                        for (pid, entry) in peers.iter() {
                            if *pid != exclude {
                                let _ = socket.send_to(&data, entry.addr).await;
                            }
                        }
                    }
                    Some(TransportCommand::DisconnectPeer(peer)) => {
                        if let Some(entry) = peers.remove(&peer) {
                            addr_to_peer.remove(&entry.addr);
                            let _ = lifecycle_tx.send(LifecycleEvent::Disconnected(peer));
                        }
                    }
                    Some(TransportCommand::Shutdown) | None => break,
                }
            }

            // ── Heartbeat ──────────────────────────────────────────────────
            _ = heartbeat_interval.tick() => {
                for entry in peers.values() {
                    let _ = socket.send_to(&[], entry.addr).await;
                }
            }

            // ── Stale peer detection ───────────────────────────────────────
            _ = stale_check.tick() => {
                let now = Instant::now();
                let stale: Vec<PeerId> = peers.iter()
                    .filter(|(_, e)| now.duration_since(e.last_recv) > PEER_TIMEOUT)
                    .map(|(pid, _)| *pid)
                    .collect();
                for pid in stale {
                    if let Some(entry) = peers.remove(&pid) {
                        addr_to_peer.remove(&entry.addr);
                        let _ = lifecycle_tx.send(LifecycleEvent::Disconnected(pid));
                    }
                }
            }
        }
    }

    Ok(())
}

// ── Client mode ────────────────────────────────────────────────────────────

async fn run_client(
    addr: &str,
    inbound_data_tx: tokio_mpsc::UnboundedSender<(PeerId, Vec<u8>)>,
    lifecycle_tx: tokio_mpsc::UnboundedSender<LifecycleEvent>,
    outbound_rx: &mut tokio_mpsc::UnboundedReceiver<TransportCommand>,
    bound_addr_tx: mpsc::Sender<Option<SocketAddr>>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Client doesn't have a meaningful bound address — signal immediately
    let _ = bound_addr_tx.send(None);

    let server_addr: SocketAddr = addr.parse()
        .map_err(|_| format!("invalid server address: {addr}"))?;
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect(server_addr).await?;

    // UDP "connect" is just setting the default destination — the socket is
    // immediately usable. We consider the client connected right away.
    let mut connected = true;
    let _ = lifecycle_tx.send(LifecycleEvent::Connected(PeerId::SERVER));

    let mut buf = vec![0u8; 2048];

    let connect_timeout = Duration::from_secs(5);
    let mut connect_timeout_interval = tokio::time::interval(connect_timeout);
    connect_timeout_interval.tick().await;

    loop {
        tokio::select! {
            // ── Receive from server ────────────────────────────────────────
            result = socket.recv(&mut buf) => {
                let n = result?;
                let data = buf[..n].to_vec();
                if !connected {
                    connected = true;
                    let _ = lifecycle_tx.send(LifecycleEvent::Connected(PeerId::SERVER));
                }
                let _ = inbound_data_tx.send((PeerId::SERVER, data));
            }

            // ── Outbound commands ──────────────────────────────────────────
            cmd = outbound_rx.recv() => {
                match cmd {
                    Some(TransportCommand::SendTo(_, data))
                    | Some(TransportCommand::SendAll(data))
                    | Some(TransportCommand::SendAllExcept(_, data)) => {
                        let _ = socket.send(&data).await;
                    }
                    Some(TransportCommand::DisconnectPeer(_)) => {
                        let _ = lifecycle_tx.send(LifecycleEvent::Disconnected(PeerId::SERVER));
                        break;
                    }
                    Some(TransportCommand::Shutdown) | None => break,
                }
            }

            // ── Connection timeout ────────────────────────────────────────
            _ = connect_timeout_interval.tick() => {
                if !connected {
                    let _ = lifecycle_tx.send(LifecycleEvent::Disconnected(PeerId::SERVER));
                    break;
                }
            }
        }
    }

    Ok(())
}
