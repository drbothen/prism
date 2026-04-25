// S-2.01 — Dirty bit crash-recovery protocol (stub).
//
// set_dirty / clear_dirty / check_dirty_on_startup over the `dirty_bits` CF.
// All writes use `WriteOptions { sync: true }` for crash durability (BC-2.15.005).
// Implementer fills this in during step (c) TDD.
