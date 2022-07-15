use super::bot::{self, Action};
use super::Error;
use sys::wasm::{self, spec::StoreRef, CallerMemoryExt};
use sys::Result;

pub const MIN_BOOT_FUEL: u64 = 64;
const LOG_FUEL_BASE: u64 = 16;
const LOG_FUEL_RATIO: u64 = 2;
pub const TURN_FUEL: u64 = 32;
pub const MOVE_FUEL: u64 = 256;

pub type VM = wasm::Linker<bot::Store>;
#[inline]
pub fn new_vm() -> Result<VM> {
    let mut vm = VM::new(&wasm::Engine::new());
    vm.add_wasi().add_export(wasm::spec::MAY_EXPORT_START.clone()).add_export(
        wasm::spec::LinkExport {
            name: "tick",
            required: true,
            value: wasm::spec::ExportType::UnitFunc,
        },
    );

    vm.add_func("io", "log", |mut bot: Caller, ptr: u32, len: u32| {
        bot.consume_fuel(len as u64 * LOG_FUEL_RATIO + LOG_FUEL_BASE)?;
        let (buf, ctx) = with_mem(&mut bot, ptr, len)?;
        ctx.write_log(buf);
        Ok(())
    })?;

    vm.add_func("motor", "forward", |mut bot: Caller| {
        bot.state_mut().action = Action::MotorForward
    })?
    .add_func("motor", "left", |mut bot: Caller| bot.state_mut().action = Action::MotorLeft)?
    .add_func("motor", "right", |mut bot: Caller| bot.state_mut().action = Action::MotorRight)?;

    vm.add_func("sensors", "contact", |bot: Caller| !bot.state().front.is_empty() as i32)?;

    Ok(vm)
}

type Caller<'a> = wasm::Caller<'a, bot::Store>;

fn with_mem<'a>(
    caller: &'a mut Caller,
    ptr: u32,
    len: u32,
) -> Result<(&'a mut [u8], &'a mut bot::Store), wasm::Trap> {
    let (data, ctx) = caller.memory()?.data_and_store_mut(caller);
    let ptr = ptr as usize;
    let len = len as usize;
    let mem = data.get_mut(ptr..ptr + len).ok_or_else(|| wasm::Trap::new("out of bound memory"))?;
    Ok((mem, ctx))
}

#[cold]
#[inline]
pub fn err_trap(ctx: &'static str, trap: wasm::Trap) -> Error {
    Error::new(ctx, trap.to_string())
}
