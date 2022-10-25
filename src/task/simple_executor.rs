use super::Task;
use alloc::collections::VecDeque;
use core::task::{
    Waker,
    RawWaker,
    RawWakerVTable,
    Context,
    Poll
};

// task_queue field of type VecDeque, which is basically a vector that allows fro push and pop
// operations on both ends. The idea behind using this type is that we insert new tasks through the
// spawn method at the end and pop the next task for execution from the front. This way, we get a
// simple FIFO queue.
pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}
impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
}

// RawWaker
// This type requires the programmer to explicity define a virtual method table(vtable) that specifies the
// functions that should be called when the RawWaker is cloned, woken, or dropped. The layout of
// this vtable is defined by the RawWakerVTable type. Each function receives a *const () argument,
// which is a type-erased pointer to some value. The reason for using a *const () pointer insetad
// of a proper reference is that the RawWaker type should be non-generic but still support
// arbitrary types. The pointer is provided by putting it into the data arugment of RawWaker::new,
// which just initialize a RawWaker. The Waker then uses this RawWaker to call the vtable functions
// with data.
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

impl SimpleExecutor {
    // Reapeatedly poll all queued tasks in a loop until all are done.
    // This is not efficient since it does not utilize the notifications of Waker.
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => self.task_queue.push_back(task)
            }
        }
    }
}
