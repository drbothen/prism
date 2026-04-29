;; loop_plugin.wat — Plugin with an infinite loop (for timeout/CPU limit testing).
;;
;; Used by: AC-3 (BC-2.17.004 CPU time limit test)
;;   "Simulate plugin timeout (WAT module with infinite loop) → verify Err(Timeout)
;;    within configured deadline, host process continues"
;;
;; TV-17-004-timeout: WAT module with infinite loop; 5s deadline →
;;   Err(PluginError::Timeout) within deadline + 1s tolerance

(module
  (memory (export "memory") 1)

  (data (i32.const 0) "loop-plugin")  ;; name
  (data (i32.const 16) "0.1.0")       ;; version

  (func (export "name") (result i32 i32)
    i32.const 0
    i32.const 11
  )

  (func (export "version") (result i32 i32)
    i32.const 16
    i32.const 5
  )

  ;; enrich-single spins in an infinite loop — must be interrupted by epoch deadline
  (func (export "enrich-single")
    (param i32 i32 i32 i32) (result i32)
    block $break
      loop $spin
        ;; Tight spin — wasmtime epoch interruption will fire at the deadline
        br $spin
      end
    end
    ;; Never reached
    i32.const 0
  )

  ;; enrich-batch also spins
  (func (export "enrich-batch")
    (param i32 i32 i32 i32) (result i32 i32)
    block $break
      loop $spin
        br $spin
      end
    end
    ;; Never reached
    i32.const 0
    i32.const 0
  )
)
