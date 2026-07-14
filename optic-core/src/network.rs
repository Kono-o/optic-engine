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
/// Assigned by the host during connection, a `PeerId` tags every packet and
/// connection event so the game logic can identify which remote player or
/// client an event belongs to. The host itself always has [`PeerId::SERVER`]
/// (0). Use this to look up player state, route messages, or enforce
/// per-player game rules.
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
///
/// Selects the role of the local machine in a multiplayer session. `Host`
/// binds to a port and accepts incoming connections, while `Client` connects
/// to a remote host. This is set once during engine initialization and
/// cannot be changed at runtime.
#[derive(Clone, Debug)]
pub enum NetworkMode {
    /// Bind and listen on a port, accepting incoming connections.
    Host { port: u16 },
    /// Connect to a remote host at the given address.
    Client { addr: String },
}

/// Configuration for the networking subsystem.
///
/// Bundles the network role (host or client) and connection limits into a
/// single struct that is passed to `GameBuilder::with_network`. Use the
/// convenience constructors [`host`](Self::host) and [`client`](Self::client)
/// for the common cases, or build the struct manually for full control.
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
/// Collected once per frame by `NetworkHandle::poll()` and auto-cleared at
/// the start of the next poll cycle. Game logic should iterate these vectors
/// each frame to detect new connections, disconnections, and incoming
/// packets — they are only valid for the single frame in which they appear.
#[derive(Clone, Debug, Default)]
pub struct NetworkEvents {
    /// Peers that connected this frame.
    pub peers_connected: Vec<PeerId>,
    /// Peers that disconnected this frame.
    pub peers_disconnected: Vec<PeerId>,
    /// Raw packets received this frame, tagged with the sender's ID.
    pub packets: Vec<(PeerId, Vec<u8>)>,
}
