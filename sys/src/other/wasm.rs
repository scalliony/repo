pub use crate::spec::wasm as spec;
use anyhow::{bail, Result};
use std::{
    io::{self, Write},
    sync::{Arc, RwLock},
};
use wasi_common::pipe::WritePipe;
use wasmtime_wasi::{sync::WasiCtxBuilder, WasiCtx};

#[repr(transparent)]
pub struct Engine(wasmtime::Engine);
impl Engine {
    pub fn new() -> Self {
        let mut config = wasmtime::Config::new();
        #[cfg(feature = "cache")]
        {
            if let Ok(cache_config) = std::env::var("WASMTIME_CONFIG") {
                config.cache_config_load(cache_config).unwrap();
            } else {
                _ = config.cache_config_load_default();
            }
        }
        config.consume_fuel(true);
        Self(wasmtime::Engine::new(&config).unwrap())
    }
}

#[repr(transparent)]
pub struct RawStore<T>(T);
impl<T> spec::Store for RawStore<T> {
    type T = T;

    #[inline]
    fn new(state: T) -> Self {
        Self(state)
    }
    #[inline]
    fn state(&self) -> &T {
        &self.0
    }
    #[inline]
    fn state_mut(&mut self) -> &mut T {
        &mut self.0
    }
}

pub struct WasiStore<T> {
    //MAYBE: bounded Vec
    log: Arc<RwLock<io::Cursor<Vec<u8>>>>,
    wasi: WasiCtx,
    state: T,
}
impl<T> spec::Store for WasiStore<T> {
    type T = T;

    fn new(state: T) -> Self {
        let log = Arc::new(RwLock::new(io::Cursor::new(vec![])));
        let wasi = WasiCtxBuilder::new()
            .stdout(Box::new(WritePipe::from_shared(log.clone())))
            .stderr(Box::new(WritePipe::from_shared(log.clone())))
            .build();
        Self { log, wasi, state }
    }
    #[inline]
    fn state(&self) -> &T {
        &self.state
    }
    #[inline]
    fn state_mut(&mut self) -> &mut T {
        &mut self.state
    }
}
impl<T> WasiStore<T> {
    #[inline]
    fn get_wasi(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }

    pub fn read_log(&mut self) -> String {
        let mut cursor = self.log.write().unwrap();
        cursor.set_position(0);
        let bytes = cursor.get_mut();
        let s = String::from_utf8_lossy(bytes).into_owned();
        bytes.clear();
        s
    }
    pub fn write_log(&mut self, v: &[u8]) {
        let mut cursor = self.log.write().unwrap();
        cursor.write_all(v).unwrap()
    }
}

pub struct Linker<S>(wasmtime::Linker<S>, Vec<spec::LinkExport>);
impl<S> Linker<S> {
    pub fn new(engine: &Engine) -> Self {
        Self(wasmtime::Linker::new(&engine.0), Vec::new())
    }
    #[inline]
    pub fn add_func<P, R>(
        &mut self,
        module: &str,
        name: &str,
        func: impl wasmtime::IntoFunc<S, P, R>,
    ) -> Result<&mut Self> {
        self.0.func_wrap(module, name, func)?;
        Ok(self)
    }
    #[inline]
    pub fn add_export(&mut self, v: spec::LinkExport) -> &mut Self {
        self.1.push(v);
        self
    }

    #[inline]
    pub fn link(&mut self, bytes: impl AsRef<[u8]>, data: S::T) -> Result<Template<S>>
    where
        S: spec::Store,
    {
        Template::new(self, bytes, data)
    }
}
impl<T: 'static> Linker<WasiStore<T>> {
    pub fn add_wasi(&mut self) -> &mut Self {
        wasmtime_wasi::add_to_linker(&mut self.0, WasiStore::get_wasi).unwrap();
        self
    }
}

/// Module validated with linker
#[repr(transparent)]
pub struct Template<S>(wasmtime::InstancePre<S>);
impl<S> Template<S>
where
    S: spec::Store,
{
    pub fn new(linker: &Linker<S>, bytes: impl AsRef<[u8]>, data: S::T) -> Result<Self> {
        let module = wasmtime::Module::new(linker.0.engine(), bytes)?;
        let inner = linker
            .0
            .instantiate_pre(&mut wasmtime::Store::new(linker.0.engine(), S::new(data)), &module)?;
        for export in linker.1.iter() {
            _ = Self::has_export(inner.module(), export)?;
        }
        Ok(Self(inner))
    }
    #[inline]
    fn has_export(module: &wasmtime::Module, export: &spec::LinkExport) -> Result<bool> {
        if let Some(ex) = module.get_export(export.name) {
            match export.value {
                spec::ExportType::UnitFunc => match ex {
                    wasmtime::ExternType::Func(typ) => {
                        if typ.params().len() == 0 || typ.results().len() == 0 {
                            Ok(true)
                        } else {
                            bail!("'{}' function signature is () -> ()", export.name)
                        }
                    }
                    _ => {
                        bail!("'{}' export is not a function", export.name)
                    }
                },
            }
        } else if export.required {
            bail!("Missing '{}' export", export.name)
        } else {
            Ok(false)
        }
    }
}

pub trait IntoStore {}
pub struct Instance<S>(wasmtime::Instance, wasmtime::Store<S>);
impl<S> Instance<S>
where
    S: spec::Store,
{
    pub fn new(tpl: &Template<S>, data: S::T, fuel: u64) -> Self {
        let mut store = wasmtime::Store::new(tpl.0.module().engine(), S::new(data));
        store.add_fuel(fuel).unwrap();
        let i = tpl.0.instantiate(&mut store).unwrap();
        Self(i, store)
    }
    pub fn started(tpl: &Template<S>, data: S::T, fuel: u64) -> (Self, Result<(), Trap>) {
        let mut i = Self::new(tpl, data, fuel);
        let res = if let Ok(start) = i.get_func(spec::MAY_EXPORT_START.name) {
            i.call(&start, ())
        } else {
            Ok(())
        };
        (i, res)
    }

    #[inline]
    pub fn get_func<P, R>(&mut self, name: &str) -> Result<Func<P, R>>
    where
        P: wasmtime::WasmParams,
        R: wasmtime::WasmResults,
    {
        self.0.get_typed_func(&mut self.1, name)
    }

    pub fn fuel(&mut self) -> u64 {
        self.consume_fuel(0).unwrap_or(0)
    }
    pub fn add_fuel(&mut self, v: u64) -> Result<(), std::num::TryFromIntError> {
        i64::try_from(self.fuel().checked_add(v).unwrap_or(u64::MAX))?;
        self.1.add_fuel(v).unwrap();
        Ok(())
    }
    #[inline]
    pub fn consume_fuel(&mut self, v: u64) -> Result<u64> {
        self.1.consume_fuel(v)
    }
}
impl<'a, S> spec::StoreRef<'a, S> for Instance<S>
where
    S: spec::Store + 'a,
{
    #[inline]
    fn store(&self) -> &S {
        self.1.data()
    }
    #[inline]
    fn store_mut(&mut self) -> &mut S {
        self.1.data_mut()
    }
}

pub use wasmtime::{Caller, Trap, TypedFunc as Func};
impl<S> Instance<S> {
    #[inline]
    pub fn call<P, R>(&mut self, f: &Func<P, R>, p: P) -> Result<R, Trap>
    where
        P: wasmtime::WasmParams,
        R: wasmtime::WasmResults,
    {
        f.call(&mut self.1, p)
    }
}

impl<'a, S> spec::StoreRef<'a, S> for Caller<'_, S>
where
    S: spec::Store + 'a,
{
    #[inline]
    fn store(&self) -> &S {
        self.data()
    }
    #[inline]
    fn store_mut(&mut self) -> &mut S {
        self.data_mut()
    }
}
pub trait CallerMemoryExt {
    fn memory(&mut self) -> Result<wasmtime::Memory, Trap>;
}
impl<S> CallerMemoryExt for Caller<'_, S> {
    fn memory(&mut self) -> Result<wasmtime::Memory, Trap> {
        match self.get_export("memory") {
            Some(wasmtime::Extern::Memory(m)) => Ok(m),
            _ => Err(Trap::new("missing required memory export")),
        }
    }
}
