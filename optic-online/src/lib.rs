//! UDP networking for the Optic engine.
//!
//! Runs a tokio runtime on a dedicated background thread, communicating
//! with the main thread via mpsc channels. All I/O is non-blocking from
//! the game loop's perspective.
//!
//! # Architecture
//!
//! ```text
//! Game loop (main thread)          Network thread
//! ┌────────────────────┐          ┌─────────────────────┐
//! │ NetworkHandle      │─tx cmd──▶│ tokio runtime        │
//! │   .send_to()      │          │   UdpSocket          │
//! │   .send_all()     │◀─data────│   heartbeat keepalive│
//! │   .poll()         │◀─events──│   peer lifecycle      │
//! └────────────────────┘          └─────────────────────┘
//! ```
//!
//! # Feature flag
//!
//! This crate is optional. Enable it with the `online` feature:
//!
//! ```toml
//! optic = { git = "..", features = ["online"] }
//! ```
//!
//! Then configure in your `GameBuilder`:
//!
//! ```ignore
//! use optic_engine::*;
//!
//! let game = GameBuilder::new()
//!     .with_network(NetworkConfig::host(7777))
//!     .build(App)?
//!     .enable_networking();
//! ```

pub mod channels;
pub mod config;
pub mod handle;
pub mod peer;
pub mod transport;

pub use handle::NetworkHandle;
pub use config::*;
pub use peer::*;
