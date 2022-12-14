use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
    sync::atomic::{
        AtomicI64,
        Ordering,
    },
};
use alloc::boxed::Box;

pub mod keyboard;
pub mod executor;

// wrapper around a pinned, heap-allocated, and dynamically dispatched future with the empty type
// as output.
//
// dyn keyword indicates that we store a trait object in the Box. This means that the methods on
// the future are dynamically dispatched, allowing different types of futures to be stored in the
// Task type. This is important because each async fn has its own type and we want to be able to
// create multiple different tasks.
//
// Pin<Box> type ensures that a value can't be moved in memory by placing it on the heap and
// preventing the creation of &mut references to it. This is important because futures generated by
// async/await might be self-referential, i.e., contain pointers to themselves that would be
// invalidated when the future is moved.
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(i64);
impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicI64 = AtomicI64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
