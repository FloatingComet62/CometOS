use crate::{
    println,
    print,
};
use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use core::{
    pin::Pin,
    task::{
        Poll,
        Context
    },
};
use futures_util::{
    stream::{
        Stream,
        StreamExt,
    },
    task::AtomicWaker,
};
use pc_keyboard::{
    layouts::Us104Key,
    DecodedKey,
    HandleControl,
    Keyboard,
    ScancodeSet1
};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();
static WAKER: AtomicWaker = AtomicWaker::new();

// Called by the keyboard interrupt hanlder
/// Must not block or allocate.
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    // The purpose of _private field is to prevent construction of the struct from outside of the
    // module. This makes the new function the only way to construct the type.
    _private: (),
}
impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100)).expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}

impl Stream for ScancodeStream {
    type Item = u8;

    // We first use the OnceCell::try_get method to get a reference to the initialized scancode
    // queue. This should never fail since we initialize the queue in the `new` function, so we can
    // safely use the expect method to panic if it's not initialized. We then optimistically try to
    // pop from the queue and retrun Poll::Ready when it succeeds. This way, we can avoid the
    // performance overhead of registering a waker when the queue is not empty. Next, we use the
    // ArrayQueue::pop method to try to get the next element from the queue. If it succeeds, we
    // return the scancode wrapped in Poll::Read(Some(...)). If it failes, it means that the queue
    // is potentially empty. Only potentially because the interrupt handler might have filled the
    // queue asynchronously immediately after the check. Since this race condition can occur again
    // fro the next check, we need to register the Waker in the WAKER static before the second
    // check. This way, a wakeup might happen before we return Poll::Pending, but it is guaranteed
    // that we get a wakeup for any scancodes pushed after the check.
    fn poll_next(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE.try_get().expect("not initialized");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(&context.waker());
        match queue.pop() {
            Ok(scancode) => Poll::Ready(Some(scancode)),
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}

pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(Us104Key, ScancodeSet1, HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}
