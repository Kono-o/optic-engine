/// Unique identifier for a connected peer.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct PeerId(pub u64);

impl PeerId {
    pub const SERVER: PeerId = PeerId(0);
}

#[derive(Clone, Debug)]
pub enum NetworkMode {
    Host { port: u16 },
    Client { addr: String },
}

#[derive(Clone, Debug)]
pub struct NetworkConfig {
    pub mode: NetworkMode,
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

/// Per-frame network events, drained from the network thread's channels.
/// Mirrors the one-frame lifecycle of `Is::Pressed` — vectors auto-clear each frame.
#[derive(Clone, Debug, Default)]
pub struct NetworkEvents {
    pub peers_connected: Vec<PeerId>,
    pub peers_disconnected: Vec<PeerId>,
    pub packets: Vec<(PeerId, Vec<u8>)>,
}
