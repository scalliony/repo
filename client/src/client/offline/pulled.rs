use engine::*;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

//TODO: replace with fn_trait
type R = Box<dyn FnMut() -> Option<Command>>;
type S = Box<dyn FnMut(Event)>;
pub struct Client {
    game: Game<R, S>,
    store: ClientStore,
    tick_acc_ms: f32,
    //TODO: move speed managment to bulb
    tick_ms: u64,
}
impl Client {
    pub fn new() -> Self {
        let mut store = ClientStore::default();

        let receiver = store.commands.clone();
        let sender = store.events.clone();
        let receive: R = Box::new(move || receiver.borrow_mut().pop_front());
        let send: S = Box::new(move |v: Event| sender.borrow_mut().push_back(v));
        let game = Game::new(receive, send, false);

        Self {
            game,
            store,
            tick_acc_ms: 0.,
            tick_ms: DEFAULT_TICK_DURATION_MS,
        }
    }
}
impl super::super::Client for Client {
    #[inline]
    fn update(&mut self) {
        self.tick_acc_ms += macroquad::prelude::get_frame_time() * 1000.0;
        if self.tick_acc_ms as u64 >= self.tick_ms {
            //MAYBE: overflow limiter
            self.tick_acc_ms -= self.tick_ms as f32;
            self.game.update();
        }
    }
    #[inline]
    fn try_recv(&mut self) -> Option<Event> {
        self.store.events.borrow_mut().pop_front()
    }
    #[inline]
    fn send(&mut self, c: Rpc) {
        //TODO: if tick_rate update tick_ms
        if let Some(cmd) = match c {
            Rpc::ChangeState(v) => Some(Command::ChangeState(v)),
            Rpc::Spawn(q) => Some(Command::Spawn(q)),
            Rpc::Map(q) => {
                let evs = self.store.events.clone();
                Some(Command::Map(
                    q,
                    Promise::new(move |cr| {
                        evs.borrow_mut().push_back(Event::Cells(cr));
                    }),
                ))
            }
            Rpc::SetView { .. } => None,
        } {
            self.store.commands.borrow_mut().push_back(cmd);
        }
    }
}

#[derive(Default)]
struct ClientStore {
    commands: Rc<RefCell<VecDeque<Command>>>,
    events: Rc<RefCell<VecDeque<Event>>>,
}
