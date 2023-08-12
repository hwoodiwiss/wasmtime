;;! target = "aarch64"
;;!
;;! settings = ['enable_heap_access_spectre_mitigation=true']
;;!
;;! compile = true
;;!
;;! [globals.vmctx]
;;! type = "i64"
;;! vmctx = true
;;!
;;! [globals.heap_base]
;;! type = "i64"
;;! load = { base = "vmctx", offset = 0, readonly = true }
;;!
;;! # (no heap_bound global for static heaps)
;;!
;;! [[heaps]]
;;! base = "heap_base"
;;! min_size = 0x10000
;;! offset_guard_size = 0xffffffff
;;! index_type = "i64"
;;! style = { kind = "static", bound = 0x10000000 }

;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!
;; !!! GENERATED BY 'make-load-store-tests.sh' DO NOT EDIT !!!
;; !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!

(module
  (memory i64 1)

  (func (export "do_store") (param i64 i32)
    local.get 0
    local.get 1
    i32.store offset=0x1000)

  (func (export "do_load") (param i64) (result i32)
    local.get 0
    i32.load offset=0x1000))

;; function u0:0:
;; block0:
;;   ldr x10, [x2]
;;   add x10, x10, x0
;;   add x10, x10, #4096
;;   movz x11, #0
;;   movz w9, #61436
;;   movk w9, w9, #4095, LSL #16
;;   subs xzr, x0, x9
;;   csel x13, x11, x10, hi
;;   csdb
;;   str w1, [x13]
;;   b label1
;; block1:
;;   ret
;;
;; function u0:1:
;; block0:
;;   ldr x10, [x1]
;;   add x10, x10, x0
;;   add x10, x10, #4096
;;   movz x11, #0
;;   movz w9, #61436
;;   movk w9, w9, #4095, LSL #16
;;   subs xzr, x0, x9
;;   csel x13, x11, x10, hi
;;   csdb
;;   ldr w0, [x13]
;;   b label1
;; block1:
;;   ret
