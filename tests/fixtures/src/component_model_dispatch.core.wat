;; component_model_dispatch.core.wat — Core wasm module for the component_model_dispatch.prx fixture.
;;
;; Purpose: Exercises the Component Model dispatch path through the production host
;; interface registered by PluginRuntime::build_linker (host_functions.rs:340-470).
;;
;; The core module imports host::http-request using the canonical ABI (10 i32 params):
;;   params: method_ptr, method_len, url_ptr, url_len,
;;           headers_ptr, headers_len, body_is_some, body_ptr, body_len, retptr
;; The canonical ABI lowers the http-response record (status:u16, headers:list, body:list)
;; to memory at retptr. The export call-blocked reads status u16 from retptr+0.
;;
;; Data layout (memory offsets):
;;   0..3    : "GET" (method string, 3 bytes)
;;   16..40  : "https://blocked.test/path" (url string, 25 bytes)
;;   512     : retptr for http-response record output
;;               offset 0: status: u16 (2 bytes)
;;               offset 4: headers ptr: i32
;;               offset 8: headers len: i32
;;               offset 12: body ptr: i32
;;               offset 16: body len: i32
;;
;; This module is compiled into component_model_dispatch.prx using:
;;   wasm-tools component embed --world dispatch-test \
;;     tests/fixtures/src/component_model_dispatch.wit \
;;     tests/fixtures/src/component_model_dispatch.core.wat \
;;     -o /tmp/component_model_dispatch.embedded.wasm
;;   wasm-tools component new /tmp/component_model_dispatch.embedded.wasm \
;;     -o tests/fixtures/component_model_dispatch.prx
;;
;; See tests/fixtures/src/component_model_dispatch.README.md for full recipe.
;;
;; Fixture: tests/fixtures/component_model_dispatch.prx
;; Tests:   test_F_PASS5_HIGH_001_production_linker_dispatch_via_build_linker_route_a
;; BC:      BC-2.17.001 (http-request host function)
;; wasm-tools version used: 1.248.0

(module
  ;; Canonical ABI for http-request: 10 i32 parameters (no return — result via retptr).
  ;; Lowered form of: http-request(method:string, url:string, headers:list<tuple<string,string>>,
  ;;                                body:option<list<u8>>) -> http-response
  ;; Parameters: method_ptr, method_len,   (string lowering: ptr+len)
  ;;             url_ptr, url_len,          (string lowering: ptr+len)
  ;;             headers_ptr, headers_len,  (list lowering: ptr+len)
  ;;             body_is_some,              (option discriminant: 0=None)
  ;;             body_ptr, body_len,        (option payload: list ptr+len, unused when is_some=0)
  ;;             retptr                     (out-param pointer for http-response record)
  ;; Total: 10 params, 0 results (http-response record written to retptr)
  (type $http_request_t (func
    (param i32 i32   ;; method: ptr, len
           i32 i32   ;; url:    ptr, len
           i32 i32   ;; headers: ptr, len (empty list → ptr=0, len=0)
           i32       ;; body: is_some (0 = None)
           i32 i32   ;; body: ptr, len (unused when is_some=0)
           i32       ;; retptr: pointer to http-response output record in memory
    )
  ))

  ;; cabi_realloc signature: old_ptr, old_size, align, new_size -> new_ptr
  (type $realloc_t (func (param i32 i32 i32 i32) (result i32)))

  ;; call-blocked export type: () -> u16 (lifted from core i32)
  (type $call_blocked_t (func (result i32)))

  ;; Import host::http-request with canonical ABI (lowered 10-param form)
  (import "host" "http-request" (func $http_request (type $http_request_t)))

  ;; Memory: 2 pages (128 KiB). Page 0 holds data; offset 512 is retptr scratch.
  (memory (export "memory") 2)

  ;; Export cabi_realloc: returns a fixed scratch pointer (offset 2048).
  ;; This is sufficient for a component that never reallocates strings at runtime
  ;; (all strings are statically embedded in data sections).
  (func (export "cabi_realloc") (type $realloc_t)
    (param $old_ptr i32) (param $old_size i32) (param $align i32) (param $new_size i32)
    (result i32)
    i32.const 2048
  )

  ;; call-blocked: calls host::http-request("GET", "https://blocked.test/path", [], None)
  ;; and returns the status u16 read from the response record at retptr+0.
  ;;
  ;; The dispatch chain:
  ;;   1. Push method ("GET", ptr=0, len=3)
  ;;   2. Push url ("https://blocked.test/path", ptr=16, len=25)
  ;;   3. Push headers (empty list: ptr=0, len=0)
  ;;   4. Push body (None: is_some=0, ptr=0, len=0)
  ;;   5. Push retptr (scratch at offset 512)
  ;;   6. Call lowered http-request — production host writes http-response to retptr
  ;;   7. Load u16 from retptr+0 (the status field)
  ;;   8. Return status i32 (canonical lift produces Val::U16)
  (func (export "call-blocked") (type $call_blocked_t)
    (result i32)
    ;; method: "GET" at offset 0, length 3
    i32.const 0
    i32.const 3
    ;; url: "https://blocked.test/path" at offset 16, length 25
    i32.const 16
    i32.const 25
    ;; headers: empty list (ptr=0, len=0)
    i32.const 0
    i32.const 0
    ;; body: None (is_some=0, ptr=0, len=0)
    i32.const 0
    i32.const 0
    i32.const 0
    ;; retptr: offset 512 (scratch area for http-response record output)
    i32.const 512
    call $http_request
    ;; Load status u16 from retptr+0 (16-bit unsigned load, zero-extended to i32)
    i32.const 512
    i32.load16_u
  )

  ;; Data section: statically embed the string literals.
  ;; offset 0: "GET" (3 bytes, matches method ptr=0 len=3)
  (data (i32.const 0) "GET")
  ;; offset 16: "https://blocked.test/path" (25 bytes, matches url ptr=16 len=25)
  (data (i32.const 16) "https://blocked.test/path")
)
