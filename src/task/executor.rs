use super::{
    Task,
    TaskId,
};
use alloc::{
    collections::BTreeMap,
    sync::Arc,
    task::Wake,
};
use core::task::{
    Waker,
    Context,
    Poll,
};
use crossbeam_queue::ArrayQueue;

// Instead of storing tasks in a VecDeque, we use the task_queue of task IDs and a BTreeMap named
// tasks that contains the actual Task instances. The map is indexed by the TaskId to allow
// efficient continuation of a specific task.
//
// The task_queue field is an ArrayQueue of task IDs, wrapped into the Arc type that implements
// reference counting. Reference counting makes it possible to share ownership of the value among
// multiple owners. It works by allocating the value on the heap and counting the number of active
// references to it. When the number of active references reaches zero, the value is no longer
// needed and can be deallocated.
//
// We use this Arc<ArrayQueue> type for the task_queue because it will be shared between the
// executor and wakers. The idea is that the wakers push the ID of the woken task to the queue. The
// executor sites on the receiving end of the queue, retrieves the woken tasks by their ID from the
// tasks map, and then runs them. The reason for using a fixed-size queue instead of an unbounded
// queue such as SegQueue is that interrupt handlers should not allocate on push to this queue.
//
// In addition to this task_queue and the tasks map, the Executor type has a waker_cache field that
// is also a map. This map caches the Waker of a task after it's creation. this has 2 reasons:
//  * it improves performance by reusing the same waker for multiple wake-ups of the same task
//    instead of creating a new waker each time.
//  * it ensures that reference-counted wakers are not deallocated inside interrupt hanlders
//    because it could head to deadlocks
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    waker_cache: BTreeMap<TaskId, Waker>,
}
impl Executor {
    // To create an Executor, we provide a simple new function. We choose a capacity of 100 for the
    // task_queue, which should be more than enoug for the foreseeable future. In case our system will
    // have more than 100 concurrent tasks at some point, we can easily increase this size.
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    // If there is already a task with the same ID in the map, the [BTreeMap::insert] method
    // returns it. This should never happen since each task has a unique ID, so we panic in this
    // case since it indicates a bug iin our code. Similarly, we panic when the task_queue is full
    // since this should nevner happen fi we choose a large enough queue size.
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task) .is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_task();
            self.sleep_if_idle();
        }
    }

    // The basic idea of this function is to loop over all tasks in the task_queue, create a waker
    // for each task, and then poll them. However, instead of adding pending tasks back to the end
    // of the task_queue, we let our TaskWaker implementation take care of adding woken tasks back
    // to the queue.
    fn run_ready_task(&mut self) {
        // destruct `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache.entry(task_id).or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remvoe it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    // We have to disable the interrupts before the if statment because interrupts can happen at
    // any time and that might be just after the if statment is passed and we halt even when we just got
    // an interrupt.
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{
            self,
            enable_and_hlt,
        };

        interrupts::disable();

        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}
impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    // We push the task_id to the referenced task_queue. Since modifications to the ArrayQueue type
    // only require a shared reference, we can implement this method on &self instead of &mut self.
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
}
// In order to use our TaskWaker type for polling futures, we need to convert it to a Waker
// instance first. This is required because the Future::poll method takes a Context instead of an
// argument, which can only be constructed from the Waker type. While we could do this by providing
// an implementation of the RawWaler type, it's both simpler and safer to instead implement the
// Arc-based Wake trait and then use the from implementations provided by std to construct Waker.
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
