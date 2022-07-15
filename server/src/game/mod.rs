mod dto;
mod generational;
mod wasm;
use anyhow::Result;
use chrono::Utc;
pub use dto::*;
use generational as gen;
use std::{thread, time::Duration};
use tokio::sync::{broadcast, mpsc};
use tracing::instrument;
use typed_index_collections::TiVec;

pub fn run() -> InterfaceRef {
    let (commands_tx, commands_rx) = mpsc::unbounded_channel();
    let (events_tx, events_rx) = broadcast::channel(128);

    let thread = thread::Builder::new()
        .name("game-master".into())
        .spawn(move || Game::new(commands_rx, events_tx).run())
        .unwrap();

    std::sync::Arc::new(Interface {
        commands: commands_tx,
        events: events_rx,
        thread: Some(thread),
    })
}
pub struct Interface {
    pub commands: mpsc::UnboundedSender<Command>,
    pub events: broadcast::Receiver<Event>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Drop for Interface {
    #[instrument(skip_all)]
    fn drop(&mut self) {
        if let Some(handle) = self.thread.take() {
            tracing::debug!("exit");
            _ = self.commands.send(Command::State(State::Stopped));
            handle.join().unwrap();
        }
        tracing::trace!("bye");
    }
}
pub type InterfaceRef = std::sync::Arc<Interface>;

type VM = wasm::Computer<BotState>;
struct Game {
    commands: mpsc::UnboundedReceiver<Command>,
    events: EventSender,

    state: State,
    counter: u32,

    vm: VM,
    programs: TiVec<ProgramId, Program>,
    bots: gen::Array<BotId, Bot>,
}
impl Game {
    fn new(commands: mpsc::UnboundedReceiver<Command>, events: broadcast::Sender<Event>) -> Self {
        let vm = VM::new();
        //TODO: vm.add_func("io", "log", func: impl IntoFunc<T, Params, Args>);

        Self {
            commands,
            events: EventSender(events),
            state: State::Running,
            counter: 0,
            vm,
            programs: TiVec::new(),
            bots: gen::Array::new(),
        }
    }

    fn run(mut self) {
        while self.state != State::Stopped {
            self.tick();
            thread::sleep(Duration::from_secs(2));
            self.counter += 1;
        }
    }

    #[inline]
    #[instrument(skip_all, fields(id = self.counter))]
    fn tick(&mut self) {
        self.events.send(Event::TickStart(self.counter.into(), Utc::now()));
        loop {
            self.commands();
            if self.state != State::Paused {
                break;
            }
            thread::sleep(Duration::from_millis(200));
        }
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
            let res = cpu.process.call(cpu.tick_func, ());
            self.events.log(&src, cpu.process.read_log());
            if let Err(trap) = res {
                self.events.send(src.err(err_trap("Trap during tick", trap)));
            }
        }

        self.events.send(Event::TickEnd);
    }

    #[inline]
    #[instrument(level = "debug", skip_all)]
    fn commands(&mut self) {
        let mut next_state = self.state;
        while let Ok(command) = self.commands.try_recv() {
            match command {
                Command::State(v) => {
                    if next_state != State::Stopped {
                        next_state = v
                    }
                }
                Command::Compile(code, cb) => {
                    _ = cb.send(
                        Program::new(code, &self.vm)
                            .map(|program| self.programs.push_and_get_key(program))
                            .map_err(|err| {
                                Error::new("Failed to compile", err.root_cause().to_string())
                            }),
                    )
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

    #[cold]
    #[inline]
    #[instrument(level = "trace", skip_all)]
    fn boot(
        bot: &mut Bot,
        programs: &mut TiVec<ProgramId, Program>,
        vm: &VM,
    ) -> Result<(), (String, Error)> {
        let program = programs.get_mut(bot.program).unwrap();
        let pre = program.compiled(vm).unwrap();
        let (mut process, res) = wasm::Process::instantiate(pre, BotState::default(), 10_000);
        let instance = match res {
            Ok(v) => v,
            Err(trap) => return Err((process.read_log(), err_trap("Trap during start", trap))),
        };
        let tick_func = process.get_func::<(), ()>(&instance, "tick").unwrap();
        bot.cpu = Some(BotCpu { process, tick_func });
        Ok(())
    }
}

struct EventSender(broadcast::Sender<Event>);
impl EventSender {
    fn send(&mut self, event: Event) {
        tracing::trace!(?event);
        _ = self.0.send(event);
    }
    fn log(&mut self, src: &BotSrc, log: String) {
        if let Some(log) = src.log(log) {
            self.send(log)
        }
    }
}

struct Program {
    inner: Option<wasm::Program<BotState>>,
    code: Bytes,
}
impl Program {
    fn new(code: Bytes, vm: &VM) -> Result<Self> {
        let mut s = Self { inner: None, code };
        _ = s.compiled(vm)?;
        Ok(s)
    }
    fn compiled(&mut self, vm: &VM) -> Result<&mut wasm::Program<BotState>> {
        if self.inner.is_none() {
            let program = wasm::Program::compile(vm, &self.code)
                .and_then(|p| p.has_flat_func("tick", true))?;
            self.inner = Some(program);
        }
        Ok(self.inner.as_mut().unwrap())
    }
}

struct Bot {
    program: ProgramId,
    cpu: Option<BotCpu>,
}
struct BotCpu {
    process: wasm::Process<BotState>,
    tick_func: wasm::Func<(), ()>,
}
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
