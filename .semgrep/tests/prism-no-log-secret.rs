// ruleid: prism-no-log-secret
tracing::info!("leaked password: {}", pwd);

// ruleid: prism-no-log-secret
log::debug!("api_key={}", key);

// ok
tracing::info!("benign log without sensitive field");
