#![allow(clippy::unwrap_used, clippy::expect_used)]
// TD-WV0-08: SyslogReceiver loopback source gate.
//
// Tests verify:
//   1. A datagram sent from 127.0.0.1 is received (loopback path).
//   2. The loopback guard predicate rejects non-loopback IPs (unit-level).
//      Injecting a datagram from a non-loopback source is not feasible in a
//      standard CI environment without raw-socket/root privileges; that
//      contract is verified by the unit tests in syslog.rs::tests instead.
//
// See also: ac_6_syslog_receiver.rs (AC-6 acceptance test).

use prism_dtu_common::SyslogReceiver;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

const LOOPBACK_SYSLOG_MSG: &str =
    "<34>1 2026-04-24T00:00:00Z prism-host td-wv0-08 - - - loopback gate test";

/// TD-WV0-08 AC-1: Datagram from 127.0.0.1 passes the loopback gate and is received.
#[tokio::test]
async fn test_td_wv0_08_loopback_sender_is_received() {
    let bind_addr: SocketAddr = "127.0.0.1:0"
        .parse()
        .expect("static loopback bind address is valid");
    let receiver = SyslogReceiver::start(bind_addr)
        .await
        .expect("TD-WV0-08: SyslogReceiver::start must succeed");

    let dest = receiver.bound_addr();

    // Send from 127.0.0.1 (loopback) — must be accepted.
    let sender = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("TD-WV0-08: bind sender socket");
    sender
        .send_to(LOOPBACK_SYSLOG_MSG.as_bytes(), dest)
        .await
        .expect("TD-WV0-08: send loopback datagram");

    // Allow the background task to process the datagram.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let messages = receiver.received_messages();
    assert!(
        !messages.is_empty(),
        "TD-WV0-08: loopback datagram must be received (not dropped)"
    );
    assert!(
        messages.iter().any(|m| m.contains("loopback gate test")),
        "TD-WV0-08: received message must contain the loopback gate test payload"
    );
}
