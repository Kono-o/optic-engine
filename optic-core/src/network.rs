//! Networking types for peer-to-peer and client-server communication.
//!
//! This module defines the data types used to configure and interact with
//! Optic's networking subsystem. The actual network I/O lives in
//! `optic-loop`; this crate provides the shared types:
//!
//! - [`PeerId`] — unique identifier for each connected peer.
//! - [`NetworkMode`] — whether to act as host or client.
//! - [`NetworkConfig`] — full configuration (mode + peer limits).
//! - [`NetworkEvents`] — per-frame events (joins, leaves, packets).

/// Unique identifier for a connected peer.
///
/// Peer IDs are assigned by the host during connection. The host itself
/// always has [`PeerId::SERVER`] (0).
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerId(pub u64);

impl PeerId {
    /// The host/server always has this ID.
    pub const SERVER: PeerId = PeerId(0);
}

/// Whether a [`NetworkConfig`] acts as a host or client.
#[derive(Clone, Debug)]
pub enum NetworkMode {
    /// Bind and listen on a port, accepting incoming connections.
    Host { port: u16 },
    /// Connect to a remote host at the given address.
    Client { addr: String },
}

/// Configuration for the networking subsystem.
///
/// Used with `GameBuilder::with_network` (see [`optic_loop`](https://docs.rs/optic-loop)).
#[derive(Clone, Debug)]
pub struct NetworkConfig {
    /// Whether to act as host or client.
    pub mode: NetworkMode,
    /// Maximum number of simultaneous peer connections.
    pub max_peers: u32,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            mode: NetworkMode::Host { port: 7777 },
            max_peers: 8,
        }
    }
}

impl NetworkConfig {
    /// Convenience: host on a given port with default max peers.
    pub fn host(port: u16) -> Self {
        Self { mode: NetworkMode::Host { port }, max_peers: 8 }
    }

    /// Convenience: connect to a remote address with default max peers.
    pub fn client(addr: impl Into<String>) -> Self {
        Self { mode: NetworkMode::Client { addr: addr.into() }, max_peers: 8 }
    }
}

/// Per-frame network events, drained from the background network thread.
///
/// These vectors are populated once per frame by `NetworkHandle::poll()`
/// and auto-cleared at the start of the next poll cycle. This mirrors the
/// one-frame lifecycle of button press/release events.
#[derive(Clone, Debug, Default)]
pub struct NetworkEvents {
    /// Peers that connected this frame.
    pub peers_connected: Vec<PeerId>,
    /// Peers that disconnected this frame.
    pub peers_disconnected: Vec<PeerId>,
    /// Raw packets received this frame, tagged with the sender's ID.
    pub packets: Vec<(PeerId, Vec<u8>)>,
}
