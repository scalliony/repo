use anyhow::{bail, Result};
use std::{
    io::{self, Write},
    sync::{Arc, RwLock},
};
use tracing::instrument;
use wasi_common::pipe::WritePipe;
use wasmtime::*;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

pub struct Computer<T>(Linker<StoreData<T>>);
impl<T: 'static> Computer<T> {
    pub fn new() -> Self {
        let mut config = Config::new();
        if let Ok(cache_config) = std::env::var("WASMTIME_CONFIG") {
            config.cache_config_load(cache_config).unwrap();
        } else {
            config.cache_config_load_default().unwrap();
        }
        config.consume_fuel(true);
        let engine = Engine::new(&config).unwrap();
        let mut linker = Linker::new(&engine);
        wasmtime_wasi::add_to_linker(&mut linker, StoreData::<T>::get_wasi).unwrap();
        Self(linker)
    }

    pub fn add_func<P, A>(
        &mut self,
        module: &str,
        name: &str,
        func: impl IntoFunc<StoreData<T>, P, A>,
    ) -> Result<&mut Self> {
        self.0.func_wrap(module, name, func)?;
        Ok(self)
    }
}

pub struct Program<T>(InstancePre<StoreData<T>>);
impl<T> Program<T> {
    fn new(c: &Computer<T>, bytes: impl AsRef<[u8]>, data: T) -> Result<Self> {
        let module = Module::new(c.0.engine(), bytes)?;
        let inner = c.0.instantiate_pre(&mut StoreData::of(c.0.engine(), data), &module)?;
        Self(inner).has_flat_func("_start", false)
    }

    pub fn has_flat_func(self, name: &str, required: bool) -> Result<Self> {
        if let Some(ex) = self.0.module().get_export(name) {
            match ex {
                wasmtime::ExternType::Func(typ) => {
                    if typ.params().len() != 0 || typ.results().len() != 0 {
                        bail!("'{}' function signature is () -> ()", name)
                    }
                }
                _ => {
                    bail!("'{}' export is not a function", name)
                }
            }
        } else if required {
            bail!("Missing '{}' function", name)
        }
        Ok(self)
    }
}
impl<T> Program<T>
where
    T: Default,
{
    #[instrument(skip_all)]
    pub fn compile(c: &Computer<T>, bytes: impl AsRef<[u8]>) -> Result<Self> {
        Self::new(c, bytes, T::default())
    }
}

pub struct StoreData<T> {
    //MAYBE: bounded Vec
    log: Arc<RwLock<io::Cursor<Vec<u8>>>>,
    wasi: WasiCtx,
    data: T,
}
impl<T> StoreData<T> {
    fn of(engine: &Engine, data: T) -> Store<Self> {
        let log = Arc::new(RwLock::new(io::Cursor::new(vec![])));
        let wasi = WasiCtxBuilder::new()
            .stdout(Box::new(WritePipe::from_shared(log.clone())))
            .stderr(Box::new(WritePipe::from_shared(log.clone())))
            .build();

        Store::new(engine, StoreData { log, wasi, data })
    }

    fn get_wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }

    fn read_log(&mut self) -> String {
        let mut cursor = self.log.write().unwrap();
        cursor.set_position(0);
        let bytes = cursor.get_mut();
        let s = String::from_utf8_lossy(bytes).into_owned();
        bytes.clear();
        s
    }
    pub fn write_log(&mut self, v: &str) {
        let mut cursor = self.log.write().unwrap();
        cursor.write_all(v.as_bytes()).unwrap()
    }
}

pub type Func<P, R> = wasmtime::TypedFunc<P, R>;
pub struct Process<T>(Store<StoreData<T>>);
impl<T> Process<T> {
    #[instrument(skip_all)]
    pub fn instantiate(
        pre: &Program<T>,
        data: T,
        initial_fuel: u64,
    ) -> (Self, Result<Instance, Trap>) {
        let mut process = Self(StoreData::of(pre.0.module().engine(), data));
        process.add_fuel(initial_fuel).unwrap();

        let instance = pre.0.instantiate(&mut process.0).unwrap();
        if let Ok(start_func) = instance.get_typed_func::<(), (), _>(&mut process.0, "_start") {
            tracing::trace!("calling _start");
            if let Err(trap) = start_func.call(&mut process.0, ()) {
                return (process, Err(trap));
            }
        }
        (process, Ok(instance))
    }

    pub fn get_func<P, R>(&mut self, instance: &Instance, name: &str) -> Result<Func<P, R>>
    where
        P: WasmParams,
        R: WasmResults,
    {
        instance.get_typed_func::<P, R, _>(&mut self.0, name)
    }
    pub fn call<P, R>(&mut self, func: Func<P, R>, params: P) -> Result<R, Trap>
    where
        P: WasmParams,
        R: WasmResults,
    {
        func.call(&mut self.0, params)
    }

    pub fn data(&mut self) -> &mut T {
        &mut self.0.data_mut().data
    }
    pub fn read_log(&mut self) -> String {
        self.0.data_mut().read_log()
    }

    pub fn fuel(&mut self) -> u64 {
        self.0.consume_fuel(0).unwrap_or(0)
    }
    pub fn add_fuel(&mut self, v: u64) -> Result<()> {
        i64::try_from(self.fuel().checked_add(v).unwrap_or(u64::MAX))?;
        self.0.add_fuel(v)
    }
}

pub type Trap = wasmtime::Trap;
