(module
    ;; Functions from game api
    (import "io" "log" (func $io_log (param i32 i32)))
    (import "sensors" "contact" (func $sensors_contact (result i32)))
    (import "motor" "forward" (func $motor_forward))
    (import "motor" "left" (func $motor_left))

    ;; Memory must be exported
    (memory 1)
    (export "memory" (memory 0))
    ;; It contains constants like this string
    (data (i32.const 0) "Explorer")

    ;; Called at boot-time (optional)
    (func $main (export "_start")
        ;; Log "Explorer"
        (call $io_log
            (i32.const 0) ;; Pointer to the start of the string
            (i32.const 8))) ;; String length

    ;; Called each tick (required)
    (func $tick (export "tick")
        (if
            (i32.gt_s ;; Sensor is in contact
                (call $sensors_contact)
                (i32.const 0))
            (then (call $motor_left)) ;; Turn left
            (else (call $motor_forward)))) ;; Go forward
)