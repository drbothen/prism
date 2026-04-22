;; noop_infusion.wat — Minimal valid infusion plugin fixture for S-1.15 tests.
;;
;; This WAT module is compiled to noop_infusion.wasm during tests/build.rs.
;; It is wrapped as a WASM Component Model component using `wasm-tools component new`
;; to produce noop_infusion.prx for integration tests.
;;
;; Implements the prism:infusion-plugin@0.1.0 world:
;;   - name()          → "noop-infusion"
;;   - version()       → "0.1.0"
;;   - enrich-single() → null (no enrichment data)
;;   - enrich-batch()  → empty list
;;
;; Used by: AC-1 (load valid plugin), AC-8 (KV scoping test)

(module
  ;; Memory for string data
  (memory (export "memory") 1)

  ;; String constants
  (data (i32.const 0) "noop-infusion")  ;; len=13, offset=0
  (data (i32.const 16) "0.1.0")         ;; len=5,  offset=16

  ;; name() → pointer to "noop-infusion" string
  (func (export "name") (result i32 i32)
    i32.const 0   ;; ptr
    i32.const 13  ;; len
  )

  ;; version() → pointer to "0.1.0" string
  (func (export "version") (result i32 i32)
    i32.const 16  ;; ptr
    i32.const 5   ;; len
  )

  ;; enrich-single(input_value_ptr, input_value_len, input_type_ptr, input_type_len)
  ;; → returns None (option tag 0)
  (func (export "enrich-single")
    (param i32 i32 i32 i32) (result i32)
    ;; Return tag 0 = None (option)
    i32.const 0
  )

  ;; enrich-batch(inputs_ptr, inputs_len, input_type_ptr, input_type_len)
  ;; → returns empty list (ptr=0, len=0)
  (func (export "enrich-batch")
    (param i32 i32 i32 i32) (result i32 i32)
    i32.const 0
    i32.const 0
  )
)
