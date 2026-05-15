;; minimal.wat — Minimal valid infusion plugin for integration tests.
;; Exports: name, version, enrich-single (satisfies INFUSION_REQUIRED_EXPORTS).
;; Tests: AC-4, AC-5, AC-6, AC-7, hot_reload tests.
(module
  (memory (export "memory") 1)
  ;; Plugin name stored at offset 0 (10 bytes: "minimal-ok")
  (data (i32.const 0) "minimal-ok")
  ;; Plugin version stored at offset 16 (5 bytes: "1.0.0")
  (data (i32.const 16) "1.0.0")

  ;; name() -> (ptr: i32, len: i32)
  (func (export "name") (result i32 i32)
    i32.const 0
    i32.const 10)

  ;; version() -> (ptr: i32, len: i32)
  (func (export "version") (result i32 i32)
    i32.const 16
    i32.const 5)

  ;; enrich-single(input_ptr, input_len, type_ptr, type_len) -> i32
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32)
    i32.const 0)

  ;; enrich-batch(inputs_ptr, inputs_len, type_ptr, type_len) -> (ptr: i32, len: i32)
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
    i32.const 0
    i32.const 0)
)
