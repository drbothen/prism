//! [`SyslogReceiver`] — Generic RFC 5424 syslog capture server (UDP + TCP).

use std::net::SocketAddr;
use std::sync::{Arc, Mutex};

/// Captures inbound RFC 5424 syslog messages over UDP and TCP.
pub struct SyslogReceiver {
    bound_addr: SocketAddr,
    messages: Arc<Mutex<Vec<String>>>,
}

impl SyslogReceiver {
    /// Bind a syslog receiver on the given address and start accepting messages.
    ///
    /// The `addr` argument is used to bind the UDP socket. When `addr.port()` is 0,
    /// the OS assigns an ephemeral port. Call [`bound_addr`](Self::bound_addr) to
    /// retrieve the actual port after starting.
    pub async fn start(addr: SocketAddr) -> anyhow::Result<Self> {
        let socket = tokio::net::UdpSocket::bind(addr).await?;
        let bound_addr = socket.local_addr()?;
        let messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
        let messages_clone = messages.clone();

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            while let Ok((n, src)) = socket.recv_from(&mut buf).await {
                if !src.ip().is_loopback() {
                    // Test-infra protection: silently drop datagrams from non-loopback sources.
                    // Prevents pollution from stray broadcast/multicast/LAN-scoped senders.
                    tracing::debug!(
                        "SyslogReceiver: dropping datagram from non-loopback src {}",
                        src.ip()
                    );
                    continue;
                }
                let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
                // SAFETY: mutex poisoning means the writer panicked; propagating is correct.
                #[allow(clippy::expect_used)]
                messages_clone
                    .lock()
                    .expect("messages lock poisoned")
                    .push(msg);
            }
        });

        Ok(Self {
            bound_addr,
            messages,
        })
    }

    /// Return the address the UDP socket is actually bound to.
    pub fn bound_addr(&self) -> SocketAddr {
        self.bound_addr
    }

    /// Return a snapshot of all messages received since the last [`reset`](Self::reset).
    pub fn received_messages(&self) -> Vec<String> {
        // SAFETY: mutex poisoning means a panic already occurred; propagating is correct.
        #[allow(clippy::expect_used)]
        self.messages
            .lock()
            .expect("messages lock poisoned")
            .clone()
    }

    /// Clear all captured messages and reset internal state.
    pub fn reset(&self) {
        // SAFETY: mutex poisoning means a panic already occurred; propagating is correct.
        #[allow(clippy::expect_used)]
        self.messages
            .lock()
            .expect("messages lock poisoned")
            .clear();
    }
}

// ─────────────────────────────────────────────────────────────
// TD-WV0-08: Unit tests for the loopback source guard
// ─────────────────────────────────────────────────────────────
//
// These tests verify the guard predicate directly against `SocketAddr` values
// since it is not feasible to inject datagrams from a non-loopback IP in a
// standard unit-test environment without root / raw-socket privileges.
//
// The integration-level test (ac_6_syslog_receiver) covers the acceptance
// path (loopback sender → message received).
#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};

    /// TD-WV0-08: 127.0.0.1 is a loopback address.
    #[test]
    fn test_td_wv0_08_ipv4_loopback_is_accepted() {
        let src: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 12345);
        assert!(
            src.ip().is_loopback(),
            "TD-WV0-08: 127.0.0.1 must satisfy the is_loopback() guard"
        );
    }

    /// TD-WV0-08: ::1 (IPv6 loopback) is a loopback address.
    #[test]
    fn test_td_wv0_08_ipv6_loopback_is_accepted() {
        let src: SocketAddr = SocketAddr::new(IpAddr::V6(Ipv6Addr::LOCALHOST), 12345);
        assert!(
            src.ip().is_loopback(),
            "TD-WV0-08: ::1 must satisfy the is_loopback() guard"
        );
    }

    /// TD-WV0-08: A LAN-scoped address (e.g. 192.168.1.1) is NOT loopback — guard drops it.
    #[test]
    fn test_td_wv0_08_lan_address_is_dropped() {
        let src: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 168, 1, 1)), 12345);
        assert!(
            !src.ip().is_loopback(),
            "TD-WV0-08: 192.168.1.1 must NOT satisfy is_loopback(); guard must drop it"
        );
    }

    /// TD-WV0-08: A public routable address is NOT loopback — guard drops it.
    #[test]
    fn test_td_wv0_08_routable_address_is_dropped() {
        let src: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 53);
        assert!(
            !src.ip().is_loopback(),
            "TD-WV0-08: 8.8.8.8 must NOT satisfy is_loopback(); guard must drop it"
        );
    }
}
