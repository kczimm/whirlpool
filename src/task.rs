//! A Task is the asynchronous computational unit that runs on the pool.

use std::{
    future::Future,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    task::{Context, Poll, Waker},
};

use futures::{
    future::BoxFuture,
    task::{waker_ref, ArcWake},
    FutureExt,
};
use parking_lot::Mutex;
use sorted_channel::Sender;

use crate::{message::Message, state::State};

pub struct TaskHandle {
    pub(crate) inner: Arc<Task>,
}

impl TaskHandle {
    pub fn set_priority(&self, priority: usize) {
        self.inner.set_priority(priority);
    }
}

impl Future for TaskHandle {
    type Output = ();

    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        if self.inner.get_state() == State::Finished {
            Poll::Ready(())
        } else {
            let waker = cx.waker().clone();
            *self.inner.waker.lock() = Some(waker);
            Poll::Pending
        }
    }
}

pub(crate) struct Task {
    priority: AtomicUsize,
    state: Mutex<State>,
    waker: Mutex<Option<Waker>>,
    future: Mutex<BoxFuture<'static, ()>>,
    tx: Sender<Message>,
}

impl Task {
    pub fn new<F>(priority: usize, f: F, tx: Sender<Message>) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let state = Mutex::new(State::Idle);
        let future = Mutex::new(f.boxed());
        Self {
            priority: AtomicUsize::new(priority),
            state,
            waker: Mutex::default(),
            future,
            tx,
        }
    }

    pub fn set_state(&self, state: State) {
        *self.state.lock() = state;
    }

    pub fn get_state(&self) -> State {
        *self.state.lock()
    }

    pub fn get_priority(&self) -> usize {
        self.priority.load(Ordering::Relaxed)
    }

    pub fn set_priority(&self, priority: usize) {
        self.priority.store(priority, Ordering::Relaxed);
    }

    pub fn run(arc_self: Arc<Self>) {
        arc_self.set_state(State::Active);
        let mut future = arc_self.future.lock();
        let waker = waker_ref(&arc_self);
        let mut cx = Context::from_waker(&waker);

        if future.poll_unpin(&mut cx).is_ready() {
            arc_self.set_state(State::Finished);
            if let Some(waker) = arc_self.waker.lock().take() {
                waker.wake();
            }
        } else {
            arc_self.set_state(State::Idle);
        }
    }
}

impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // Sending fails if there are no receivers. This would mean there are no more
        // threads in the pool and nothing we can do anyway.
        let _ = arc_self.tx.send(Message::Run(Arc::clone(arc_self)));
    }
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc::channel;

    use sorted_channel::sorted_channel;

    use super::*;

    #[test]
    fn run() {
        let (tx, rx) = channel();
        let value = 7;
        let task = Task::new(
            1,
            async move {
                tx.send(7).unwrap();
            },
            sorted_channel().0,
        );

        Task::run(Arc::new(task));

        assert_eq!(rx.recv(), Ok(value));
    }
}
