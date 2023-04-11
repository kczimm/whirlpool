use core::fmt;
use std::{cmp::Ordering, sync::Arc};

use crate::task::Task;

pub(crate) enum Message {
    Run(Arc<Task>),
    Close,
}

impl fmt::Debug for Message {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Run(_) => write!(f, "Run"),
            Self::Close => write!(f, "Close"),
        }
    }
}

impl PartialEq for Message {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Message {}

impl PartialOrd for Message {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Message {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (self, other) {
            (Message::Run(a), Message::Run(b)) => a.get_priority().cmp(&b.get_priority()),
            (Message::Run(_), Message::Close) => Ordering::Less,
            (Message::Close, Message::Run(_)) => Ordering::Greater,
            (Message::Close, Message::Close) => Ordering::Equal,
        }
    }
}
