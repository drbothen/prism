// ruleid: prism-no-log-secret
tracing::info!("leaked password: {}", pwd);

// ruleid: prism-no-log-secret
log::debug!("api_key={}", key);

// ok: prism-no-log-secret
tracing::info!("benign log without sensitive field");

// ruleid: prism-no-log-secret
info!("api_key leaked: {}", api_key);

// ruleid: prism-no-log-secret
warn!("password disclosure: {}", password);
