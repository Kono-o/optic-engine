use optic_core::PeerId;
use tokio::sync::mpsc;

/// Commands the main thread sends to the network thread.
#[derive(Debug)]
pub(crate) enum TransportCommand {
    SendTo(PeerId, Vec<u8>),
    SendAll(Vec<u8>),
    SendAllExcept(PeerId, Vec<u8>),
    DisconnectPeer(PeerId),
    Shutdown,
}

/// Lifecycle events the network thread sends back to the main thread.
#[derive(Debug)]
pub(crate) enum LifecycleEvent {
    Connected(PeerId),
    Disconnected(PeerId),
}

/// Creates the channel pair for inbound data (network → main thread).
pub(crate) fn inbound_data_channel() -> (mpsc::UnboundedSender<(PeerId, Vec<u8>)>, mpsc::UnboundedReceiver<(PeerId, Vec<u8>)>) {
    mpsc::unbounded_channel()
}

/// Creates the channel pair for lifecycle events (network → main thread).
pub(crate) fn lifecycle_channel() -> (mpsc::UnboundedSender<LifecycleEvent>, mpsc::UnboundedReceiver<LifecycleEvent>) {
    mpsc::unbounded_channel()
}

/// Creates the channel pair for outbound commands (main thread → network).
pub(crate) fn outbound_channel() -> (mpsc::UnboundedSender<TransportCommand>, mpsc::UnboundedReceiver<TransportCommand>) {
    mpsc::unbounded_channel()
}
