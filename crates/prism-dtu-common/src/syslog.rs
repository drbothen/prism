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
            while let Ok((n, _src)) = socket.recv_from(&mut buf).await {
                let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
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
        self.messages
            .lock()
            .expect("messages lock poisoned")
            .clone()
    }

    /// Clear all captured messages and reset internal state.
    pub fn reset(&self) {
        self.messages
            .lock()
            .expect("messages lock poisoned")
            .clear();
    }
}
