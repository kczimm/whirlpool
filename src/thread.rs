use std::{
    sync::Arc,
    thread::{self, JoinHandle},
};

use parking_lot::Mutex;
use sorted_channel::Receiver;

use crate::{message::Message, task::Task, state::State};

pub(crate) struct Thread {
    state: Arc<Mutex<State>>,
    rx: Receiver<Message>,
    handle: JoinHandle<()>,
}

impl Thread {
    /// Create a new [`Thread`] that processes [`Message`]s and updates it's
    /// [`State`].
    pub fn new(rx: Receiver<Message>) -> Self {
        let state = Arc::new(Mutex::new(State::Idle));
        let handle = Self::spawn(Arc::clone(&state), rx.clone());
        Self { state, rx, handle }
    }

    /// Start a new system thread if the underlying thread has finished. Returns
    /// `true` if a spawn occurred, `false` otherwise.
    pub fn heal(&mut self) -> bool {
        if self.handle.is_finished() {
            let state = Arc::clone(&self.state);
            let rx = self.rx.clone();
            drop(std::mem::replace(&mut self.handle, Self::spawn(state, rx)));
            true
        } else {
            false
        }
    }

    pub fn get_state(&self) -> State {
        *self.state.lock()
    }

    pub fn join(self) {
        let _ = self.handle.join();
    }

    fn spawn(state: Arc<Mutex<State>>, rx: Receiver<Message>) -> JoinHandle<()> {
        thread::spawn(move || {
            *state.lock() = State::Idle;
            while let Ok(Message::Run(task)) = rx.recv() {
                *state.lock() = State::Active;
                Task::run(task);
                *state.lock() = State::Idle;
            }
            *state.lock() = State::Finished;
        })
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    use sorted_channel::sorted_channel;

    #[test]
    fn heal() {
        let (tx, rx) = sorted_channel();
        let mut t = Thread::new(rx);
        assert_eq!(t.get_state(), State::Idle);
        tx.send(Message::Close).unwrap();
        thread::sleep(Duration::from_millis(1));
        assert_eq!(t.get_state(), State::Finished);
        assert!(t.heal());
        thread::sleep(Duration::from_millis(1));
        assert_eq!(t.get_state(), State::Idle);
        tx.send(Message::Close).unwrap();
        t.join();
    }
}
