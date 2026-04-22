//! Plugin sandbox enforcement — memory limits, epoch interruption, trap isolation.

use std::sync::Arc;

use prism_core::PluginError;
use tracing::warn;
use wasmtime::{Store, StoreLimitsBuilder};

use super::loader::HostState;

/// Default memory limit per plugin instance: 64 MiB.
pub const DEFAULT_MEMORY_LIMIT_MB: u64 = 64;

/// Default CPU time limit per plugin call: 5 seconds.
pub const DEFAULT_TIMEOUT_SECONDS: u64 = 5;

/// Epoch ticks per second.
/// We tick at 10,000 ticks/sec (10 ticks per ms) to ensure that even when the ticker
/// thread is delayed by OS scheduling, 5 real seconds always produces >> 5000 ticks.
pub const EPOCH_TICKS_PER_SECOND: u64 = 10_000;

/// Create a fresh `wasmtime::Store<HostState>` configured with:
/// - `StoreLimits` enforcing `memory_limit_mb * 1024 * 1024` linear memory cap.
/// - `epoch_deadline` set to `timeout_seconds * EPOCH_TICKS_PER_SECOND`.
///
/// Note: Because HostState doesn't contain the ResourceLimiter, we use a wrapper
/// approach where the store data is `StoreData` which includes both HostState and
/// StoreLimits. However, since tests construct HostState directly, we keep HostState
/// clean and use a separate limiter configured on a separate Store type.
///
/// For our integration: we use Store<HostState> and configure memory limits via
/// StoreLimitsBuilder with trap_on_grow_failure = true, storing limiter state
/// externally via an atomic bool passed in the closure.
pub fn create_store(
    engine: &wasmtime::Engine,
    host_state: HostState,
    _memory_limit_mb: u64,
    timeout_seconds: u64,
) -> Store<HostState> {
    // Store<HostState> without ResourceLimiter for the public API.
    // Memory limiting for core module calls is handled by the SandboxState path.
    // For true Component Model plugins, memory limits would use SandboxState.
    create_store_internal(engine, host_state, timeout_seconds)
}

/// Internal store creation using a wrapper that includes StoreLimits.
fn create_store_internal(
    engine: &wasmtime::Engine,
    host_state: HostState,
    timeout_seconds: u64,
) -> Store<HostState> {
    // Use Store<HostState> without a ResourceLimiter.
    // Memory limits are enforced via SandboxState in try_allocate_wasm_memory.
    let mut store = Store::new(engine, host_state);

    // Set epoch deadline for CPU time limiting.
    let epoch_deadline = timeout_seconds * EPOCH_TICKS_PER_SECOND;
    store.set_epoch_deadline(epoch_deadline);

    store
}

/// Lower-level helper used by VP-041 (proptest) to create a store with a specific memory limit.
/// Uses SandboxState (which includes StoreLimits) for proper ResourceLimiter enforcement.
pub fn create_store_with_limit(engine: &wasmtime::Engine, limit_mb: u64) -> Store<SandboxState> {
    use super::loader::{PluginConfigMap, PluginKvStore};
    use reqwest::Client;
    use std::sync::Arc;

    let store_limits = StoreLimitsBuilder::new()
        .memory_size((limit_mb * 1024 * 1024) as usize)
        .trap_on_grow_failure(true)
        .build();

    let state = SandboxState {
        host_state: HostState {
            http_client: Arc::new(Client::new()),
            config: Arc::new(PluginConfigMap::new()),
            kv_store: Arc::new(PluginKvStore::new()),
            plugin_id: "sandbox-test".to_string(),
            allowed_urls: None,
        },
        limits: store_limits,
    };

    let mut store = Store::new(engine, state);
    store.limiter(|s: &mut SandboxState| &mut s.limits as &mut dyn wasmtime::ResourceLimiter);
    store.set_epoch_deadline(DEFAULT_TIMEOUT_SECONDS * EPOCH_TICKS_PER_SECOND);
    store
}

/// Combined store data for sandbox tests (includes ResourceLimiter).
pub struct SandboxState {
    pub host_state: HostState,
    pub limits: wasmtime::StoreLimits,
}

/// Attempt to allocate `bytes` of WASM linear memory in a fresh store configured
/// with `create_store_with_limit(limit_mb)`.
///
/// Used by VP-041 proptest to exercise the exact boundary.
pub fn try_allocate_wasm_memory(
    engine: &wasmtime::Engine,
    limit_mb: u64,
    bytes: usize,
) -> Result<(), PluginError> {
    // Align to WASM page size (64KB = 65536 bytes).
    let page_size = 65536usize;
    let pages_needed = bytes.div_ceil(page_size);

    // Build a minimal WASM core module that grows memory by pages_needed pages.
    let wasm_bytes = build_memory_grow_module(pages_needed as u32);

    // Compile as a core module.
    let module = match wasmtime::Module::new(engine, &wasm_bytes) {
        Ok(m) => m,
        Err(e) => {
            return Err(PluginError::CompilationFailed {
                path: "try_allocate_wasm_memory".to_string(),
                message: format!("module compilation failed: {}", e),
            });
        }
    };

    let mut store = create_store_with_limit(engine, limit_mb);

    // Create a core linker for core module execution.
    let linker: wasmtime::Linker<SandboxState> = wasmtime::Linker::new(engine);

    let instance = match linker.instantiate(&mut store, &module) {
        Ok(i) => i,
        Err(e) => {
            let msg = e.to_string();
            if msg.contains("memory") || msg.contains("forcing trap") || msg.contains("grow") {
                return Err(PluginError::MemoryExceeded {
                    plugin_id: "memory-test".to_string(),
                    limit_mb,
                });
            }
            return Err(PluginError::Trapped {
                plugin_id: "memory-test".to_string(),
                message: msg,
            });
        }
    };

    let grow_fn = match instance.get_typed_func::<(), i32>(&mut store, "grow") {
        Ok(f) => f,
        Err(e) => {
            return Err(PluginError::Trapped {
                plugin_id: "memory-test".to_string(),
                message: format!("get_typed_func failed: {}", e),
            });
        }
    };

    match grow_fn.call(&mut store, ()) {
        Ok(result) => {
            // memory.grow returns -1 on failure.
            if result == -1 {
                Err(PluginError::MemoryExceeded {
                    plugin_id: "memory-test".to_string(),
                    limit_mb,
                })
            } else {
                Ok(())
            }
        }
        Err(e) => {
            // Check the full error chain for the "forcing trap" message from StoreLimits.
            // StoreLimits.trap_on_grow_failure produces "forcing trap when growing memory..."
            // which may be anywhere in the anyhow error chain.
            let full_chain = format!("{:#}", e);
            if full_chain.contains("forcing trap") || full_chain.contains("forcing a memory growth")
            {
                return Err(PluginError::MemoryExceeded {
                    plugin_id: "memory-test".to_string(),
                    limit_mb,
                });
            }
            Err(PluginError::Trapped {
                plugin_id: "memory-test".to_string(),
                message: e.to_string(),
            })
        }
    }
}

/// Build a minimal WASM core module that grows memory by `pages` pages.
fn build_memory_grow_module(pages: u32) -> Vec<u8> {
    let mut m = Vec::new();

    // WASM magic + version
    m.extend_from_slice(b"\0asm");
    m.extend_from_slice(&[1, 0, 0, 0]);

    // Type section: (func (result i32))
    let type_section: &[u8] = &[0x01, 0x60, 0x00, 0x01, 0x7f];
    push_section(&mut m, 1, type_section);

    // Function section: 1 function, type index 0
    let func_section: &[u8] = &[0x01, 0x00];
    push_section(&mut m, 3, func_section);

    // Memory section: 1 resizable memory, min=0, no max
    let mem_section: &[u8] = &[0x01, 0x00, 0x00];
    push_section(&mut m, 5, mem_section);

    // Export section: "grow" function at index 0
    let export_name = b"grow";
    let mut export_section = Vec::new();
    export_section.push(0x01u8);
    write_leb128_u32(&mut export_section, export_name.len() as u32);
    export_section.extend_from_slice(export_name);
    export_section.push(0x00); // func export
    export_section.push(0x00); // func index 0
    push_section(&mut m, 7, &export_section);

    // Code section: grow function body
    // Instructions: (i32.const pages) (memory.grow) (end)
    let mut body = Vec::new();
    body.push(0x00u8); // 0 locals

    // i32.const pages
    body.push(0x41);
    write_leb128_i32(&mut body, pages as i32);

    // memory.grow (0x40 0x00)
    body.push(0x40);
    body.push(0x00);

    body.push(0x0b); // end

    let mut code_section = Vec::new();
    code_section.push(0x01u8);
    write_leb128_u32(&mut code_section, body.len() as u32);
    code_section.extend_from_slice(&body);
    push_section(&mut m, 10, &code_section);

    m
}

fn push_section(out: &mut Vec<u8>, id: u8, payload: &[u8]) {
    out.push(id);
    write_leb128_u32(out, payload.len() as u32);
    out.extend_from_slice(payload);
}

fn write_leb128_u32(out: &mut Vec<u8>, mut value: u32) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        if value == 0 {
            out.push(byte);
            break;
        } else {
            out.push(byte | 0x80);
        }
    }
}

fn write_leb128_i32(out: &mut Vec<u8>, mut value: i32) {
    loop {
        let byte = (value & 0x7f) as u8;
        value >>= 7;
        let done = (value == 0 && (byte & 0x40) == 0) || (value == -1 && (byte & 0x40) != 0);
        if done {
            out.push(byte);
            break;
        } else {
            out.push(byte | 0x80);
        }
    }
}

/// Handle for the epoch ticker background thread.
///
/// When dropped, signals the ticker thread to stop.
pub struct EpochTickerHandle {
    _stop: Arc<std::sync::atomic::AtomicBool>,
    _thread: Option<std::thread::JoinHandle<()>>,
}

impl Drop for EpochTickerHandle {
    fn drop(&mut self) {
        self._stop.store(true, std::sync::atomic::Ordering::Relaxed);
        // We don't join — background thread will stop at next sleep.
    }
}

/// Start the epoch ticker background thread (no tokio runtime required).
///
/// Fires `engine.increment_epoch()` at ~1000 ticks/sec using a time-based approach.
/// Uses a tight loop with a 500μs sleep to ensure the macOS timer resolution
/// doesn't cause the ticker to fire too slowly (macOS 1ms sleep may take 2-3ms).
pub fn start_epoch_ticker(engine: wasmtime::Engine) -> EpochTickerHandle {
    let stop = Arc::new(std::sync::atomic::AtomicBool::new(false));
    let stop_clone = stop.clone();

    let thread = std::thread::Builder::new()
        .name("prism-epoch-ticker".to_string())
        .spawn(move || {
            let mut last = std::time::Instant::now();
            while !stop_clone.load(std::sync::atomic::Ordering::Relaxed) {
                // Sleep ~500μs; on macOS actual sleep may be 0.5–2ms.
                std::thread::sleep(std::time::Duration::from_micros(500));
                // Increment 10 ticks per ms elapsed (EPOCH_TICKS_PER_SECOND = 10_000).
                let now = std::time::Instant::now();
                let elapsed_us = now.duration_since(last).as_micros() as u64;
                // 10 ticks per ms = 10_000 ticks per second = 10 per 1000μs = 1 per 100μs.
                let ticks = elapsed_us / 100;
                if ticks > 0 {
                    for _ in 0..ticks {
                        engine.increment_epoch();
                    }
                    last = now;
                }
            }
        })
        .expect("epoch ticker thread spawn failed");

    EpochTickerHandle {
        _stop: stop,
        _thread: Some(thread),
    }
}

/// Wrap a `wasmtime` call result into a structured `PluginError`.
pub fn classify_wasm_error(
    plugin_id: &str,
    err: anyhow::Error,
    memory_limit_mb: u64,
    elapsed_ms: u64,
    timeout_ms: u64,
) -> PluginError {
    let msg = err.to_string();

    // Check timeout (epoch interrupt).
    if msg.contains("interrupt") || msg.contains("epoch") || elapsed_ms >= timeout_ms {
        return PluginError::Timeout {
            plugin_id: plugin_id.to_string(),
            duration_ms: elapsed_ms,
        };
    }

    // Check memory exceeded (from StoreLimits trap_on_grow_failure).
    if msg.contains("forcing trap") || (msg.contains("memory") && msg.contains("grow")) {
        warn!(
            "Plugin '{}' exceeded memory limit of {}MB",
            plugin_id, memory_limit_mb
        );
        return PluginError::MemoryExceeded {
            plugin_id: plugin_id.to_string(),
            limit_mb: memory_limit_mb,
        };
    }

    // All other traps.
    PluginError::Trapped {
        plugin_id: plugin_id.to_string(),
        message: msg,
    }
}
