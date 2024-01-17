use alloc::boxed::Box;
use core::task::Context;
use core::{future::Future, pin::Pin, task::Poll};

pub mod executor;
pub mod keyboard;
pub mod simple_executor;

pub struct Task {
    id: TaskId,
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future),
        }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}

use core::sync::atomic::{AtomicU64, Ordering};
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);
impl TaskId {
    fn new() -> Self {
        static ATOMIC_ID: AtomicU64 = AtomicU64::new(0);
        Self(ATOMIC_ID.fetch_add(1, Ordering::Relaxed))
    }
}
