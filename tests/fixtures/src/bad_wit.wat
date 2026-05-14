;; bad_wit.wat — Plugin missing required WIT exports (no enrich-single).
;; Tests: AC-6 (BC-2.17.006 WIT validation rejects missing export).
;; Has name() and version() but NOT enrich-single/enrich-batch/fetch-page/fire-alert.
(module
  (memory (export "memory") 1)
  ;; Plugin name stored at offset 0 (11 bytes: "bad-wit-pkg")
  (data (i32.const 0) "bad-wit-pkg")
  ;; Plugin version stored at offset 16 (5 bytes: "0.1.0")
  (data (i32.const 16) "0.1.0")

  ;; name() -> (ptr: i32, len: i32)
  (func (export "name") (result i32 i32)
    i32.const 0
    i32.const 11)

  ;; version() -> (ptr: i32, len: i32)
  (func (export "version") (result i32 i32)
    i32.const 16
    i32.const 5)

  ;; Intentionally missing: enrich-single, enrich-batch, fetch-page, fire-alert, etc.
  ;; This will cause WIT validation to fail (E-PLUGIN-001 InvalidInterface).
)
