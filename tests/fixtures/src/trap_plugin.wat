;; trap_plugin.wat — Plugin that traps (unreachable) on enrich-single.
;; Tests: AC-10 (BC-2.17.001 panic isolation).
(module
  (memory (export "memory") 1)
  ;; Plugin name stored at offset 0 (11 bytes: "trap-plugin")
  (data (i32.const 0) "trap-plugin")
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

  ;; enrich-single: always traps (unreachable instruction)
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32)
    unreachable)

  ;; enrich-batch: always traps
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
    unreachable)
)
