# AC-6: SyslogReceiver captures RFC 5424 UDP messages

## Acceptance Criterion

Given `SyslogReceiver` is started, When a well-formed RFC 5424 syslog
message is sent to its UDP port, Then `received_messages()` returns a list containing
that message.

## Test

- File: `crates/prism-dtu-common/tests/ac_6_syslog_receiver.rs`
- Function: `ac_6_syslog_receiver_captures_udp_rfc5424_message`
- Test command: `cargo test --features prism-dtu-common/dtu --test ac_6_syslog_receiver`

## Implementation (excerpt)

File: `crates/prism-dtu-common/src/syslog.rs`

```rust
pub async fn start(addr: SocketAddr) -> anyhow::Result<Self> {
    let socket = tokio::net::UdpSocket::bind(addr).await?;
    let bound_addr = socket.local_addr()?;
    let messages: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let messages_clone = messages.clone();

    tokio::spawn(async move {
        let mut buf = vec![0u8; 65536];
        while let Ok((n, _src)) = socket.recv_from(&mut buf).await {
            let msg = String::from_utf8_lossy(&buf[..n]).into_owned();
            messages_clone.lock().expect("messages lock poisoned").push(msg);
        }
    });

    Ok(Self { bound_addr, messages })
}
```

## Test output

```
running 1 test
test ac_6_syslog_receiver_captures_udp_rfc5424_message ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s
```

## Mapping

`SyslogReceiver::start` binds a UDP socket with port 0 (OS-assigned), spawns a background task that reads datagrams into a shared `Arc<Mutex<Vec<String>>>`, and exposes `bound_addr()` for the sender to target; the test sends the RFC 5424 message `<34>1 2026-04-21T00:00:00Z ...` and asserts `received_messages()` is non-empty and contains "test message".
