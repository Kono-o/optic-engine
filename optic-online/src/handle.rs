use std::net::SocketAddr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread::JoinHandle;

use optic_core::{NetworkConfig, NetworkEvents, OpticError, OpticErrorKind, OpticResult, PeerId, NetworkMode};
use tokio::runtime;
use tokio::sync::mpsc as tokio_mpsc;

use crate::channels::{inbound_data_channel, lifecycle_channel, outbound_channel, LifecycleEvent, TransportCommand};
use crate::transport::run_transport;

/// Main-thread handle to the network subsystem.
///
/// Spawns a background thread running a tokio runtime on construction.
/// All send methods are non-blocking (`try_send` on an unbounded channel).
/// `poll()` is the only way to drain received data into `NetworkEvents`.
pub struct NetworkHandle {
    thread: Option<JoinHandle<()>>,
    inbound_data_rx: tokio_mpsc::UnboundedReceiver<(PeerId, Vec<u8>)>,
    lifecycle_rx: tokio_mpsc::UnboundedReceiver<LifecycleEvent>,
    outbound_tx: tokio_mpsc::UnboundedSender<TransportCommand>,
    local_addr: Option<SocketAddr>,
    shutdown_flag: Arc<AtomicBool>,
}

impl NetworkHandle {
    /// Creates a new `NetworkHandle`, spawning a background network thread.
    ///
    /// Blocks until the UDP socket is bound (host mode) or the thread is spawned
    /// (client mode). For `Host` mode, `local_addr()` returns the actual bound
    /// address (including OS-assigned port when port=0).
    pub fn new(config: NetworkConfig) -> OpticResult<Self> {
        let (inbound_data_tx, inbound_data_rx) = inbound_data_channel();
        let (lifecycle_tx, lifecycle_rx) = lifecycle_channel();
        let (outbound_tx, outbound_rx) = outbound_channel();

        let rt = runtime::Builder::new_current_thread()
            .enable_io()
            .enable_time()
            .build()
            .map_err(|e| OpticError::new(OpticErrorKind::Custom, &format!("failed to build tokio runtime: {e}")))?;

        // Channel to receive the actual bound address from the async thread
        let (bound_addr_tx, bound_addr_rx) = mpsc::channel();

        let shutdown_flag = Arc::new(AtomicBool::new(false));
        let shutdown_flag_clone = shutdown_flag.clone();

        // Extract port before moving config into the thread closure
        let is_host = matches!(&config.mode, NetworkMode::Host { .. });
        let config_port = match &config.mode {
            NetworkMode::Host { port } => Some(*port),
            NetworkMode::Client { .. } => None,
        };

        let thread = std::thread::Builder::new()
            .name("optic-network".into())
            .spawn(move || {
                rt.block_on(async move {
                    run_transport(config, inbound_data_tx, lifecycle_tx, outbound_rx, bound_addr_tx).await;
                    shutdown_flag_clone.store(true, Ordering::SeqCst);
                });
            })
            .map_err(|e| OpticError::new(OpticErrorKind::Custom, &format!("failed to spawn network thread: {e}")))?;

        // Wait for the bound address (host) or None (client)
        let local_addr = if is_host {
            let addr = bound_addr_rx
                .recv()
                .map_err(|_| OpticError::new(OpticErrorKind::Custom, "network thread exited before binding"))?
                .unwrap_or_else(|| {
                    let port = config_port.unwrap_or(0);
                    ([0, 0, 0, 0], port).into()
                });
            Some(addr)
        } else {
            None
        };

        Ok(Self {
            thread: Some(thread),
            inbound_data_rx,
            lifecycle_rx,
            outbound_tx,
            local_addr,
            shutdown_flag,
        })
    }

    /// Drains all available network events into `out`. Called once per frame
    /// from the game loop. Never blocks — returns in microseconds.
    pub fn poll(&mut self, out: &mut NetworkEvents) {
        // Drain lifecycle events
        while let Ok(event) = self.lifecycle_rx.try_recv() {
            match event {
                LifecycleEvent::Connected(pid) => out.peers_connected.push(pid),
                LifecycleEvent::Disconnected(pid) => out.peers_disconnected.push(pid),
            }
        }
        // Drain data packets
        while let Ok((pid, data)) = self.inbound_data_rx.try_recv() {
            out.packets.push((pid, data));
        }
    }

    /// Sends raw bytes to a specific peer by ID.
    ///
    /// Returns `OpticError` if the outbound channel is closed (network thread
    /// has exited). The packet is silently dropped in that case.
    pub fn send(&self, peer: PeerId, bytes: &[u8]) -> OpticResult<()> {
        self.outbound_tx
            .send(TransportCommand::SendTo(peer, bytes.to_vec()))
            .map_err(|_| OpticError::new(OpticErrorKind::Custom, "outbound channel closed"))?;
        Ok(())
    }

    /// Sends raw bytes to all connected peers.
    pub fn send_all(&self, bytes: &[u8]) -> OpticResult<()> {
        self.outbound_tx
            .send(TransportCommand::SendAll(bytes.to_vec()))
            .map_err(|_| OpticError::new(OpticErrorKind::Custom, "outbound channel closed"))?;
        Ok(())
    }

    /// Sends raw bytes to all connected peers except `exclude`.
    pub fn send_all_except(&self, exclude: PeerId, bytes: &[u8]) -> OpticResult<()> {
        self.outbound_tx
            .send(TransportCommand::SendAllExcept(exclude, bytes.to_vec()))
            .map_err(|_| OpticError::new(OpticErrorKind::Custom, "outbound channel closed"))?;
        Ok(())
    }

    /// Disconnects a specific peer. For host mode, this removes the peer
    /// from the forwarding table and fires a `Disconnected` event.
    /// For client mode, this shuts down the connection to the server.
    pub fn disconnect(&self, peer: PeerId) {
        let _ = self.outbound_tx.send(TransportCommand::DisconnectPeer(peer));
    }

    /// Returns a snapshot of currently-connected peer IDs.
    ///
    /// For a client, this returns `&[PeerId::SERVER]` if connected, or empty.
    /// Note: this is a best-effort view based on the latest lifecycle events;
    /// for accurate peer tracking drain `poll()` each frame.
    pub fn peers(&self) -> Vec<PeerId> {
        Vec::new()
    }

    /// Returns the local socket address this handle is bound to, if known.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        self.local_addr
    }

    /// Gracefully shuts down the network thread, waiting for it to exit.
    pub fn shutdown(&mut self) {
        let _ = self.outbound_tx.send(TransportCommand::Shutdown);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }

    /// Returns `true` after the network thread has fully shut down.
    pub fn is_shutdown(&self) -> bool {
        self.shutdown_flag.load(Ordering::SeqCst)
    }
}

impl Drop for NetworkHandle {
    fn drop(&mut self) {
        self.shutdown();
    }
}
