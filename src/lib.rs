//! A priority-based asynchronous executor.
//!
//! # Examples
//! ```rust
//! let pool = Builder::new().build();
//!
//! let (tx, rx) = some_async_channel();
//!
//! let priority = 1;
//! pool.spawn(priority, async move {
//!     tx.send(7).await;
//! });
//!
//! let priority = 2;
//! pool.spawn(priority, async move {
//!     let value = rx.recv().await;
//!     println!("{value}");
//! });
//! ```

pub(crate) mod message;
pub mod pool;
pub(crate) mod state;
pub mod task;
pub(crate) mod thread;
