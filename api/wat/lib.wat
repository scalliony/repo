(module
    (import "io" "log" (func $log (param i32 i32)))

    (memory 1)
    (export "memory" (memory 0))

    ;; Write 'Hello, scalliony' to memory at an offset of 8 bytes
    (data (i32.const 8) "Hello, scalliony")

    ;; Called at boot-time (optional)
    (func $main (export "_start")
        (call $log
            (i32.const 8) ;; A pointer to the start of the 'Hello, scalliony' string
            (i32.const 16) ;; String length
        )
    )

    ;; Called at each tick (required)
    (func (export "tick"))
)