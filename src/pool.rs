use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use futures::Future;
use sorted_channel::{sorted_channel, Sender};

use crate::{
    message::Message,
    task::{Task, TaskHandle},
    thread::Thread,
};

pub struct Pool {
    num_tasks: AtomicUsize,
    tx: Sender<Message>,
    // rx: Receiver<Message>,
    threads: Vec<Thread>,
}

impl Drop for Pool {
    fn drop(&mut self) {
        self.threads.iter().for_each(|_| {
            let _ = self.tx.send(Message::Close);
        });
    }
}

impl Pool {
    pub fn new(num_threads: usize) -> Self {
        let (tx, rx) = sorted_channel();
        let threads = (0..num_threads)
            .map(|_| {
                let rx = rx.clone();
                Thread::new(rx)
            })
            .collect();

        let num_tasks = AtomicUsize::new(0);

        Self {
            num_tasks,
            tx,
            threads,
        }
    }

    #[must_use]
    pub fn spawn<F>(&self, priority: usize, f: F) -> TaskHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let inner = Arc::new(Task::new(priority, f, self.tx.clone()));
        let task = Arc::clone(&inner);
        // TODO: heal the threads that have gone down.
        let _ = self.tx.send(Message::Run(task));

        self.num_tasks.fetch_add(1, Ordering::Relaxed);
        TaskHandle { inner }
    }
}

#[cfg(test)]
mod tests {
    use parking_lot::Mutex;

    use super::*;

    #[test]
    fn spawn() {
        let shared = Arc::new(Mutex::new(vec![]));
        let pool = Pool::new(2);
        let num_tasks = 10;
        let num_pushes = 10;

        for i in 0..num_tasks {
            let shared = Arc::clone(&shared);
            let _ = pool.spawn(i, async move {
                for _ in 0..num_pushes {
                    let mut data = shared.lock();
                    data.push(());
                }
            });
        }

        while shared.lock().len() != num_tasks * num_pushes {}
    }
}
