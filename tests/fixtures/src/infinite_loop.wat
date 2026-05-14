;; infinite_loop.wat — Plugin that enters an infinite loop in enrich-single.
;; Tests: AC-13 (BC-2.17.004 CPU timeout via epoch interruption).
(module
  (memory (export "memory") 1)
  ;; Plugin name stored at offset 0 (16 bytes: "infinite-loop-ok")
  (data (i32.const 0) "infinite-loop-ok")
  ;; Plugin version stored at offset 16 (5 bytes: "0.1.0")
  (data (i32.const 16) "0.1.0")

  ;; name() -> (ptr: i32, len: i32)
  (func (export "name") (result i32 i32)
    i32.const 0
    i32.const 16)

  ;; version() -> (ptr: i32, len: i32)
  (func (export "version") (result i32 i32)
    i32.const 16
    i32.const 5)

  ;; enrich-single: enters an infinite loop (will be interrupted by epoch mechanism)
  (func (export "enrich-single") (param i32 i32 i32 i32) (result i32)
    (block $break
      (loop $loop
        ;; Tight loop — epoch interrupt fires after EPOCH_TICKS deadline
        br $loop))
    i32.const 0)

  ;; enrich-batch: same infinite loop pattern
  (func (export "enrich-batch") (param i32 i32 i32 i32) (result i32 i32)
    (block $break
      (loop $loop
        br $loop))
    i32.const 0
    i32.const 0)
)
