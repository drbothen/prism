//! Unit and integration tests for `prism-sensors`.
//!
//! Test modules are named `bc_S_SS_NNN` per the factory test-writer convention.
//! Each module covers one behavioral contract (BC) anchor for this story.
//!
//! Story: S-2.06 | BCs: BC-2.01.002, BC-2.01.010, BC-2.01.013, BC-2.01.014
//! Story: S-2.08 | EventBufferStore ops + EventPoller loop tests (bodies in next dispatch)

pub mod bc_2_01_002;
pub mod bc_2_01_010;
pub mod bc_2_01_013;
pub mod bc_2_01_014;
pub mod bc_2_01_http_semaphore;

// S-2.08: test modules (test-writer dispatch)
pub mod event_buffer_tests;
pub mod poller_tests;
pub mod table_dispatch_tests;

// WGC-W2-002 — evict_expired must scan backend, not just write_cache
pub mod wgc_w2_002_evict_backend;
