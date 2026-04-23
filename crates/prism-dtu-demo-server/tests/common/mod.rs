//! Shared test helpers for `prism-dtu-demo-server` integration tests.
//!
//! Compiled only with `#[cfg(test)]` semantics (test binaries only).

use prism_dtu_demo_server::config::{CloneConfig, ClonesConfig, DemoConfig};

/// Build a `DemoConfig` with all 6 clones enabled on ephemeral ports (port = 0).
pub fn all_clones_ephemeral_config() -> DemoConfig {
    DemoConfig {
        harness: Default::default(),
        clones: ClonesConfig {
            crowdstrike: CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            },
            claroty: CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            },
            cyberint: CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            },
            armis: CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            },
            threatintel: CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            },
            nvd: CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            },
        },
    }
}

/// Build a `DemoConfig` with only the named clone enabled on an ephemeral port.
pub fn single_clone_config(name: &str) -> DemoConfig {
    let mut cfg = DemoConfig::default();
    // disable all
    cfg.clones.crowdstrike.enabled = false;
    cfg.clones.claroty.enabled = false;
    cfg.clones.cyberint.enabled = false;
    cfg.clones.armis.enabled = false;
    cfg.clones.threatintel.enabled = false;
    cfg.clones.nvd.enabled = false;

    match name {
        "crowdstrike" => {
            cfg.clones.crowdstrike = CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            }
        }
        "claroty" => {
            cfg.clones.claroty = CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            }
        }
        "cyberint" => {
            cfg.clones.cyberint = CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            }
        }
        "armis" => {
            cfg.clones.armis = CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            }
        }
        "threatintel" => {
            cfg.clones.threatintel = CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            }
        }
        "nvd" => {
            cfg.clones.nvd = CloneConfig {
                enabled: true,
                port: 0,
                ..Default::default()
            }
        }
        other => panic!("unknown clone name in single_clone_config: {other}"),
    }
    cfg
}

/// Build a minimal HTTP client with a short timeout.
pub fn http_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .build()
        .expect("failed to build HTTP client")
}

/// Poll `url` with GET until it returns HTTP 200, or panic after `timeout`.
pub async fn wait_for_200(client: &reqwest::Client, url: &str, timeout: std::time::Duration) {
    let deadline = tokio::time::Instant::now() + timeout;
    loop {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status() == 200 {
                return;
            }
        }
        if tokio::time::Instant::now() >= deadline {
            panic!("endpoint {url} did not return 200 within {timeout:?}");
        }
        tokio::time::sleep(std::time::Duration::from_millis(25)).await;
    }
}
