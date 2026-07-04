# optic-online

UDP networking for the Optic engine — background tokio thread with
connection management.

Provides a [`NetworkHandle`] for sending/receiving unreliable messages
over UDP with optional peer tracking and event polling.

```rust
use optic_core::NetworkConfig;
use optic_online::NetworkHandle;

let config = NetworkConfig::host(7777);
let mut net = NetworkHandle::new(config)?;
net.send_all(b"hello peers");
```

[`NetworkHandle`]: https://docs.rs/optic-online/latest/optic_online/handle/struct.NetworkHandle.html
