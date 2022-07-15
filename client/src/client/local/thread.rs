use engine::*;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{sync::mpsc, thread};

pub struct Client {
    commands: mpsc::Sender<Command>,
    events: mpsc::Receiver<Event>,
    thread: Option<thread::JoinHandle<()>>,
    //TODO: move speed managment to bulb
    tick_ms: Arc<AtomicU64>,
}
impl Client {
    pub fn new() -> Self {
        let (commands_tx, commands_rx) = mpsc::channel();
        let (events_tx, events_rx) = mpsc::channel();

        let tick_ms = Arc::new(AtomicU64::new(DEFAULT_TICK_DURATION_MS));

        let mut game = Game::new(
            move || commands_rx.try_recv().ok(),
            move |v: Event| {
                _ = events_tx.send(v);
            },
            false,
        );

        let thread_tick_ms = tick_ms.clone();
        let thread = thread::Builder::new()
            .name("game-master".into())
            .spawn(move || {
                while game.update() != State::Stopped {
                    //TODO: time step
                    thread::sleep(Duration::from_millis(thread_tick_ms.load(Ordering::Relaxed)))
                }
            })
            .unwrap();

        Self { commands: commands_tx, events: events_rx, thread: Some(thread), tick_ms }
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
        self.events.try_recv().ok()
    }
    #[inline]
    fn send(&mut self, c: Command) {
        //TODO: if tick_rate update tick_ms
        _ = self.commands.send(c);
    }
}
