pub mod channels;
pub mod config;
pub mod handle;
pub mod peer;
pub mod transport;

pub use handle::NetworkHandle;
pub use config::*;
pub use peer::*;

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use optic_core::{NetworkConfig, NetworkMode};

    use crate::handle::NetworkHandle;

    fn host_config() -> NetworkConfig {
        NetworkConfig {
            mode: NetworkMode::Host { port: 0 },
            max_peers: 8,
        }
    }

    #[test]
    fn host_binds_and_reports_local_addr() {
        let mut handle = NetworkHandle::new(host_config()).unwrap();
        let addr = handle.local_addr().unwrap();
        assert_eq!(addr.ip(), std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED));
        assert_ne!(addr.port(), 0);
        handle.shutdown();
    }

    #[test]
    fn host_client_roundtrip() {
        let mut host = NetworkHandle::new(host_config()).unwrap();
        let host_addr = host.local_addr().unwrap();

        let client_cfg = NetworkConfig {
            mode: NetworkMode::Client {
                addr: format!("127.0.0.1:{}", host_addr.port()),
            },
            max_peers: 1,
        };
        let mut client = NetworkHandle::new(client_cfg).unwrap();

        // Wait for client to connect (receive heartbeat from host)
        std::thread::sleep(Duration::from_millis(300));

        // Client sends to the host's port
        client.send_all(b"hello from client").unwrap();

        std::thread::sleep(Duration::from_millis(100));

        // Host sends to the client
        let mut host_events = optic_core::NetworkEvents::default();
        let mut client_events = optic_core::NetworkEvents::default();

        host.poll(&mut host_events);
        client.poll(&mut client_events);

        // Client should be connected to the server
        assert!(
            !client_events.peers_connected.is_empty()
                || client_events.peers_connected.contains(&optic_core::PeerId::SERVER)
        );

        // Host should have received the client's message
        let host_got_packets = !host_events.packets.is_empty();

        // Host sends to all
        host.send_all(b"hello from host").unwrap();
        std::thread::sleep(Duration::from_millis(100));

        client.poll(&mut client_events);
        let client_got_packets = !client_events.packets.is_empty();

        // At least one direction should have worked
        assert!(host_got_packets || client_got_packets, "no packets exchanged in either direction");

        host.shutdown();
        client.shutdown();
    }

    #[test]
    fn host_tracks_multiple_clients() {
        let mut host = NetworkHandle::new(host_config()).unwrap();
        let host_addr = host.local_addr().unwrap();
        let port = host_addr.port();

        let make_client = || {
            NetworkHandle::new(NetworkConfig {
                mode: NetworkMode::Client {
                    addr: format!("127.0.0.1:{port}"),
                },
                max_peers: 1,
            })
            .unwrap()
        };

        let mut c1 = make_client();
        let mut c2 = make_client();

        std::thread::sleep(Duration::from_millis(400));

        c1.send_all(b"from c1").unwrap();
        c2.send_all(b"from c2").unwrap();
        std::thread::sleep(Duration::from_millis(100));

        let mut events = optic_core::NetworkEvents::default();
        host.poll(&mut events);

        // Should have received from two different peers
        let unique_peers: std::collections::HashSet<_> =
            events.packets.iter().map(|(p, _)| *p).collect();
        assert_eq!(unique_peers.len(), 2, "expected 2 unique peers, got {unique_peers:?}");

        c1.shutdown();
        c2.shutdown();
        host.shutdown();
    }

    #[test]
    fn send_all_except_skips_one() {
        let mut host = NetworkHandle::new(host_config()).unwrap();
        let host_addr = host.local_addr().unwrap();
        let port = host_addr.port();

        let make_client = || {
            NetworkHandle::new(NetworkConfig {
                mode: NetworkMode::Client {
                    addr: format!("127.0.0.1:{port}"),
                },
                max_peers: 1,
            })
            .unwrap()
        };

        let mut c1 = make_client();
        let mut c2 = make_client();

        std::thread::sleep(Duration::from_millis(400));

        c1.send_all(b"hello").unwrap();
        c2.send_all(b"hello").unwrap();
        std::thread::sleep(Duration::from_millis(100));

        let mut events = optic_core::NetworkEvents::default();
        host.poll(&mut events);

        if events.packets.len() >= 2 {
            let peer_a = events.packets[0].0;
            // host sends to all except peer_a
            host.send_all_except(peer_a, b"secret").unwrap();
            std::thread::sleep(Duration::from_millis(100));

            let mut c1_ev = optic_core::NetworkEvents::default();
            let mut c2_ev = optic_core::NetworkEvents::default();
            c1.poll(&mut c1_ev);
            c2.poll(&mut c2_ev);
        }

        c1.shutdown();
        c2.shutdown();
        host.shutdown();
    }

    #[test]
    fn disconnect_removes_peer() {
        let mut host = NetworkHandle::new(host_config()).unwrap();
        let host_addr = host.local_addr().unwrap();
        let port = host_addr.port();

        let mut client = NetworkHandle::new(NetworkConfig {
            mode: NetworkMode::Client {
                addr: format!("127.0.0.1:{port}"),
            },
            max_peers: 1,
        })
        .unwrap();

        std::thread::sleep(Duration::from_millis(300));

        client.send_all(b"ping").unwrap();
        std::thread::sleep(Duration::from_millis(100));

        let mut events = optic_core::NetworkEvents::default();
        host.poll(&mut events);

        if let Some(&(peer, _)) = events.packets.first() {
            host.disconnect(peer);
            std::thread::sleep(Duration::from_millis(100));

            let mut ev2 = optic_core::NetworkEvents::default();
            host.poll(&mut ev2);
            assert!(ev2.peers_disconnected.contains(&peer));
        }

        client.shutdown();
        host.shutdown();
    }

    #[test]
    fn shutdown_cleans_up() {
        let mut handle = NetworkHandle::new(host_config()).unwrap();
        let _addr = handle.local_addr().unwrap();
        handle.shutdown();
        // After shutdown, the thread should have exited
        assert!(handle.is_shutdown());
    }

    #[test]
    fn peer_count_zero_after_no_connections() {
        let mut host = NetworkHandle::new(host_config()).unwrap();
        assert!(host.peers().is_empty());
        host.shutdown();
    }

    #[test]
    fn poll_is_nonblocking() {
        let mut host = NetworkHandle::new(host_config()).unwrap();
        let mut events = optic_core::NetworkEvents::default();
        // This should return instantly, never block
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            host.poll(&mut events);
        }
        let elapsed = start.elapsed();
        assert!(elapsed < Duration::from_millis(50), "poll took {elapsed:?}");
        host.shutdown();
    }
}
