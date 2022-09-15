use crate::Game;
use bulb::dto::{Command, Event, State};

pub struct GameState<R, S> {
    game: Game<S>,
    commands: R,
    state: State,
}
impl<R, S> GameState<R, S>
where
    R: FnMut() -> Option<Command>,
    S: FnMut(Event) -> (),
{
    pub fn new(commands: R, events: S, paused: bool) -> Self {
        let state = if paused {
            tracing::warn!("game is paused");
            State::Paused
        } else {
            State::Running
        };
        Self { game: Game::new(events), commands, state }
    }

    pub fn update(&mut self) -> State {
        let prev_state = self.state;
        while let Some(cmd) = (self.commands)() {
            if let Command::ChangeState(state) = cmd {
                self.state = state;
            } else {
                self.game.apply(cmd);
            }
        }
        if self.state != prev_state {
            self.game.send(Event::StateChange(self.state));
        }
        self.game.tick();
        self.state
    }
}
