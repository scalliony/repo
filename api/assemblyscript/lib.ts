import { io, motor, sensors } from "./api"

/// Called at boot-time (optional)
export function _start(): void {
  io.log('Starting')
}

/// Called at each tick (required)
export function tick(): void {
  const front = sensors.contact()
  if (front.isValid()) {
    motor.rotate_left()
  }
  motor.move(2)
}