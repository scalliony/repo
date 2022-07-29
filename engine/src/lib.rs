mod dto;
mod generational;
use chrono::Utc;
pub use dto::*;
use generational as gen;
use sys::{wasm, Result};
use tracing::instrument;
use typed_index_collections::TiVec;

type VM = wasm::Linker<BotStore>;
pub struct Game<R, S> {
    commands: R,
    events: EventSender<S>,

    state: State,
    counter: u32,
    in_tick: bool,

    vm: VM,
    programs: TiVec<ProgramId, Program>,
    bots: gen::Array<BotId, Bot>,
}
impl<R, S> Game<R, S>
where
    R: FnMut() -> Option<Command>,
    S: FnMut(Event),
{
    pub fn new(commands: R, events: S, paused: bool) -> Self {
        let state = if paused {
            tracing::warn!("game is paused");
            State::Paused
        } else {
            State::Running
        };

        let mut vm = VM::new(&wasm::Engine::new());
        vm.add_wasi().add_export(wasm::spec::MAY_EXPORT_START.clone());
        vm.add_export(wasm::spec::LinkExport {
            name: "tick",
            required: true,
            value: wasm::spec::ExportType::UnitFunc,
        });
        //TODO: .add_func("io", "log", func: impl IntoFunc<T, Params, Args>);

        Self {
            commands,
            events: EventSender(events),
            state,
            counter: 0,
            in_tick: false,
            vm,
            programs: TiVec::new(),
            bots: gen::Array::new(),
        }
    }

    pub fn update(&mut self) -> State {
        if self.state == State::Stopped {
            return State::Stopped;
        }

        self.receive();
        if self.state != State::Running {
            return State::Paused;
        }

        self.tick();
        self.in_tick = false;
        self.counter += 1;

        return self.state;
    }

    #[inline]
    #[instrument(skip_all, fields(id = self.counter))]
    fn tick(&mut self) {
        self.with_tick();

        for (id, bot) in self.bots.iter_mut() {
            let span = tracing::debug_span!("bot", id = gen::I::from(id));
            let _span_guard = span.enter();

            let src = BotSrc { id };

            if bot.cpu.is_none() {
                //TODO: check fuel
                if let Err((log, err)) = Self::boot(bot, &mut self.programs, &self.vm) {
                    self.events.log(&src, log);
                    self.events.send(src.err(err));
                    continue;
                }
            }
            let cpu = bot.cpu.as_mut().unwrap();

            // Tick
            tracing::trace!("fuel {}", cpu.process.fuel());
            let res = cpu.process.call(&cpu.tick, ());
            self.events.log(&src, cpu.process.store().read_log());
            if let Err(trap) = res {
                self.events.send(src.err(err_trap("Trap during tick", trap)));
            }
        }

        self.events.send(Event::TickEnd);
        tracing::debug!("done");
    }

    #[inline]
    #[instrument(level = "debug", name = "commands", skip_all)]
    fn receive(&mut self) {
        let mut next_state = self.state;
        while let Some(command) = (self.commands)() {
            match command {
                Command::State(v) => {
                    if next_state != State::Stopped {
                        next_state = v
                    }
                }
                Command::Compile(code, cb) => {
                    let res = Program::new(code, &self.vm)
                        .map(|program| self.programs.push_and_get_key(program))
                        .map_err(|err| {
                            Error::new("Failed to compile", err.root_cause().to_string())
                        });
                    cb.resolve(res)
                }
                Command::Spawn(id) => {
                    let i: usize = id.into();
                    if i < self.programs.len() {
                        self.bots.insert(Bot { cpu: None, program: id });
                        //TODO: event
                        //MAYBE: return id
                    } //MAYBE: return not found
                }
            }
        }

        if next_state != self.state {
            tracing::warn!(state = ?next_state);
            self.state = next_state;
            self.events.send(Event::State(next_state));
        }
    }

    #[inline]
    fn with_tick(&mut self) {
        if !self.in_tick {
            self.events.send(Event::TickStart(self.counter.into(), Utc::now()));
            self.in_tick = false;
        }
    }

    #[cold]
    #[inline]
    #[instrument(level = "trace", skip_all)]
    fn boot(
        bot: &mut Bot,
        programs: &mut TiVec<ProgramId, Program>,
        vm: &VM,
    ) -> Result<(), (String, Error)> {
        let program = programs.get_mut(bot.program).unwrap();
        let tpl = program.compiled(vm).unwrap();
        let (mut process, res) = wasm::Instance::started(tpl, BotState::default(), 10_000);
        if let Err(trap) = res {
            return Err((process.store().read_log(), err_trap("Trap during start", trap)))
        }
        let tick = process.get_func::<(), ()>("tick").unwrap();
        bot.cpu = Some(BotCpu { process, tick });
        Ok(())
    }
}

struct EventSender<S>(S);
impl<S> EventSender<S>
where
    S: FnMut(Event),
{
    fn send(&mut self, event: Event) {
        tracing::trace!(?event);
        self.0(event);
    }
    fn log(&mut self, src: &BotSrc, log: String) {
        if let Some(log) = src.log(log) {
            self.send(log)
        }
    }
}

struct Program {
    inner: Option<wasm::Template<BotStore>>,
    code: Bytes,
}
impl Program {
    fn new(code: Bytes, vm: &VM) -> Result<Self> {
        let mut s = Self { inner: None, code };
        s.compile(vm)?;
        Ok(s)
    }

    fn compiled(&mut self, vm: &VM) -> Result<&mut wasm::Template<BotStore>> {
        if self.inner.is_none() {
            self.compile(vm)?;
        }
        Ok(self.inner.as_mut().unwrap())
    }
    #[cold]
    #[inline]
    #[instrument(level = "trace", skip_all)]
    fn compile(&mut self, vm: &VM) -> Result<()> {
        debug_assert!(self.inner.is_none());
        let tpl = wasm::Template::new(vm, &self.code, BotState::default())?;
        self.inner = Some(tpl);
        Ok(())
    }
}

struct Bot {
    program: ProgramId,
    cpu: Option<BotCpu>,
}
struct BotCpu {
    process: wasm::Instance<BotStore>,
    tick: wasm::Func<(), ()>,
}
type BotStore = wasm::WasiStore<BotState>;
struct BotState {}
impl Default for BotState {
    fn default() -> Self {
        Self {}
    }
}

#[cold]
fn err_trap(ctx: &'static str, trap: wasm::Trap) -> Error {
    Error::new(ctx, trap.to_string())
}
