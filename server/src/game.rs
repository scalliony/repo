pub use engine::*;
use std::{thread, time::Duration};
use tokio::sync::{broadcast, mpsc, oneshot};

pub fn run() -> InterfaceRef {
    let paused = std::env::var("GAME_PAUSED").map_or(false, |v| {
        v.parse().unwrap_or_else(|_| v.parse::<u8>().expect("Expect a bool for GAME_PAUSED") != 0)
    });
    let tick_time = Duration::from_millis(
        std::env::var("GAME_TICK_MS").map_or(DEFAULT_TICK_DURATION_MS, |v| {
            v.parse().expect("Expect an integer for GAME_TICK_MS")
        }),
    );

    let (commands_tx, mut commands_rx) = mpsc::unbounded_channel();
    let (events_tx, events_rx) = broadcast::channel(128);

    let mut game = Game::new(
        move || commands_rx.try_recv().ok(),
        move |v: Event| _ = events_tx.send(v),
        paused,
    );

    let thread = thread::Builder::new()
        .name("game-master".into())
        .spawn(move || {
            while game.update() != State::Stopped {
                thread::sleep(tick_time)
            }
        })
        .unwrap();

    std::sync::Arc::new(Interface {
        commands: commands_tx,
        events: events_rx,
        thread: Some(thread),
    })
}

pub fn compile_command(code: Bytes) -> (Command, oneshot::Receiver<CompileRes>) {
    let (tx, rx) = oneshot::channel();
    let cb = Promise::new(move |v| {
        _ = tx.send(v);
    });
    (Command::Compile(code, cb), rx)
}

pub struct Interface {
    pub commands: mpsc::UnboundedSender<Command>,
    pub events: broadcast::Receiver<Event>,
    thread: Option<thread::JoinHandle<()>>,
}
impl Drop for Interface {
    #[tracing::instrument(skip_all)]
    fn drop(&mut self) {
        if let Some(handle) = self.thread.take() {
            tracing::debug!("exit");
            _ = self.commands.send(Command::ChangeState(State::Stopped));
            handle.join().unwrap();
        }
        tracing::trace!("bye");
    }
}
pub type InterfaceRef = std::sync::Arc<Interface>;
