// AC-6: SyslogReceiver captures RFC 5424 UDP message.
//
// Test starts a SyslogReceiver, sends a well-formed RFC 5424 message via UDP,
// then asserts received_messages() contains the message.
//
// Expected failure mode: SyslogReceiver::start is todo!() — panics at runtime.

use prism_dtu_common::SyslogReceiver;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// Minimal valid RFC 5424 syslog message.
const RFC5424_MSG: &str =
    "<34>1 2026-04-21T00:00:00Z prism-host prism-dtu-common - - - test message";

#[tokio::test]
async fn ac_6_syslog_receiver_captures_udp_rfc5424_message() {
    let bind_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    let receiver: SyslogReceiver = SyslogReceiver::start(bind_addr)
        .await
        .expect("AC-6: SyslogReceiver::start must succeed");

    // Send a UDP syslog datagram to the receiver's bound address.
    let receiver_udp_addr = receiver.bound_addr(); // real OS-assigned port after bind
    let sender = UdpSocket::bind("127.0.0.1:0")
        .await
        .expect("AC-6: bind sender socket");
    sender
        .send_to(RFC5424_MSG.as_bytes(), receiver_udp_addr)
        .await
        .expect("AC-6: send syslog datagram");

    // Wait briefly for the receiver task to process the datagram.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let messages = receiver.received_messages();
    assert!(
        !messages.is_empty(),
        "AC-6: received_messages() must be non-empty after sending a UDP syslog datagram"
    );
    assert!(
        messages.iter().any(|m: &String| m.contains("test message")),
        "AC-6: received message must contain the syslog payload text"
    );
}
