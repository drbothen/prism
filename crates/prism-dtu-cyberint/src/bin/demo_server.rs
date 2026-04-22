//! Standalone demo server for S-6.09 VHS recordings.
//!
//! Starts the CyberintClone on a fixed port (18090) and prints "READY" to stdout
//! once the server is listening. Runs until killed (Ctrl-C or SIGTERM).
//!
//! Usage (from the worktree root):
//!   cargo run -p prism-dtu-cyberint --features dtu --bin demo_server
//!
//! Only compiled when the `dtu` feature is active — never ships in production.
#![cfg(feature = "dtu")]

use prism_dtu_cyberint::CyberintClone;
use prism_dtu_common::BehavioralClone;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut clone = CyberintClone::new()?;
    clone.start().await?;
    let addr = clone.bound_addr();
    println!("READY http://{addr}");

    // Park until SIGTERM/Ctrl-C.
    tokio::signal::ctrl_c().await?;
    Ok(())
}
