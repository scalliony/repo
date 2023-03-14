use engine::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{sync::mpsc, thread};

pub struct Client {
    commands: mpsc::Sender<Command>,
    events_rx: mpsc::Receiver<Event>,
    events_tx: mpsc::Sender<Event>,
    thread: Option<thread::JoinHandle<()>>,
    //TODO: move speed managment to bulb
    tick_ms: Arc<AtomicU64>,
}
impl Client {
    pub fn new() -> Self {
        let (commands_tx, commands_rx) = mpsc::channel();
        let (events_tx, events_rx) = mpsc::channel();

        let tick_ms = Arc::new(AtomicU64::new(DEFAULT_TICK_DURATION_MS));

        let evs = events_tx.clone();
        let mut game = GameState::new(
            move || commands_rx.try_recv().ok(),
            move |v: Event| {
                _ = evs.send(v);
            },
            false,
        );

        let thread_tick_ms = tick_ms.clone();
        let thread = thread::Builder::new()
            .name("game-master".into())
            .spawn(move || {
                while game.update() != State::Stopped {
                    //TODO: time step
                    thread::sleep(Duration::from_millis(
                        thread_tick_ms.load(Ordering::Relaxed),
                    ))
                }
            })
            .unwrap();

        Self {
            commands: commands_tx,
            events_rx,
            events_tx,
            thread: Some(thread),
            tick_ms,
        }
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            _ = thread.join();
        }
    }
}
impl super::super::Client for Client {
    #[inline]
    fn try_recv(&mut self) -> Option<Event> {
        self.events_rx.try_recv().ok()
    }
    #[inline]
    fn send(&mut self, c: Rpc) {
        //TODO: if tick_rate update tick_ms
        if let Some(cmd) = match c {
            Rpc::ChangeState(v) => Some(Command::ChangeState(v)),
            Rpc::Spawn(q) => Some(Command::Spawn(q)),
            Rpc::Map(q) => {
                let evs = self.events_tx.clone();
                Some(Command::Map(
                    q,
                    Promise::new(move |cr| {
                        _ = evs.send(Event::Cells(cr));
                    }),
                ))
            }
            Rpc::SetView { .. } => None,
            Rpc::Compile { cid, code } => {
                let evs = self.events_tx.clone();
                Some(Command::Compile(
                    code,
                    Promise::new(move |r| {
                        _ = evs.send(match r {
                            Ok(pid) => Event::ProgramAdd { cid, pid },
                            Err(err) => Event::CompileError { cid, err },
                        })
                    }),
                ))
            }
        } {
            _ = self.commands.send(cmd);
        }
    }
}
