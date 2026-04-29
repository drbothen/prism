;; trap_plugin.wat — Plugin that unconditionally traps on any call.
;;
;; Used by: AC-2 (BC-2.17.001 panic isolation test)
;;   "Simulate plugin trap → verify Err(Trapped) returned, host process continues"
;;
;; TV-17-001-happy: Call plugin method; WASM executes `unreachable` →
;;   Err(PluginError::Trapped { plugin_id, message })

(module
  (memory (export "memory") 1)

  (data (i32.const 0) "trap-plugin")  ;; name
  (data (i32.const 16) "0.1.0")       ;; version

  (func (export "name") (result i32 i32)
    i32.const 0
    i32.const 11
  )

  (func (export "version") (result i32 i32)
    i32.const 16
    i32.const 5
  )

  ;; enrich-single always traps — simulates a plugin crash (BC-2.17.001)
  (func (export "enrich-single")
    (param i32 i32 i32 i32) (result i32)
    unreachable
  )

  ;; enrich-batch always traps
  (func (export "enrich-batch")
    (param i32 i32 i32 i32) (result i32 i32)
    unreachable
  )
)
