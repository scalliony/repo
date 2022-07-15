declare namespace raw {
    @external("io", "log")
    function console_log(s: i32, l: i32): void

    @external("motor", "rotate")
    function motor_rotate(left: i32): void
    @external("motor", "move")
    function motor_move(dist: i32): void

    @external("sensors", "contact_s")
    function sensors_contact(ret: Entity): void
}

enum EntityType { Rock, Bot, Building }
enum Rotation { Up, Right, Down, Left }

@unmanaged
class Entity {
    id: i64;
    typ: u16;

    isValid(): bool {
        return this.id > 0 && this.typ >= 0
    }
    getType(): EntityType {
        return changetype<EntityType>(this.typ)
    }
}

export namespace io {
    /** Write string to logs without encoding (ascii only) */
    export function log(s: string): void {
        raw.io_log(changetype<i32>(s), changetype<i32>(s.length * 2))
    }
    /** Write string to logs with encoding */
    export function log_utf8(s: string): void {
        const s8 = String.UTF8.encode(s)
        raw.io_log(changetype<i32>(s8), changetype<i32>(s8.byteLength))
    }
}

export namespace motor {
    export function rotate(left: boolean): void {
        raw.motor_rotate(left ? -1 : 1)
    }
    export function rotate_left(): void {
        rotate(true)
    }
    export function rotate_right(): void {
        rotate(false)
    }

    /** Move forward of dist cells
      * Direction depends of current rotation
      * Actual movement is delayed */
    export function move(dist: u16): void {
        raw.motor_move(dist as i32)
    }
}

export namespace sensors {
    /** Check for entity just in front (depending of rotation)
      * Returns entity if something is in contact */
    export function contact(): Entity {
        let ret = new Entity()
        raw.sensors_contact(ret)
        return ret
    }
}