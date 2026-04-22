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
    pub async fn start(addr: SocketAddr) -> anyhow::Result<Self> {
        todo!("implement SyslogReceiver::start per AC-7")
    }

    /// Return a snapshot of all messages received since the last [`reset`](Self::reset).
    pub fn received_messages(&self) -> Vec<String> {
        todo!("implement SyslogReceiver::received_messages")
    }

    /// Clear all captured messages and reset internal state.
    pub fn reset(&self) {
        todo!("implement SyslogReceiver::reset")
    }
}
