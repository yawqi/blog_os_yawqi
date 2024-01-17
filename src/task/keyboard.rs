use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use pc_keyboard::{layouts::Us104Key, HandleControl, KeyEvent, Keyboard, ScancodeSet1};

static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

use crate::{print, println};

pub(crate) fn push_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if queue.push(scancode).is_err() {
            println!("WARNING! The scancode queue is full! Dropping keyboard events.");
        } else {
            WAKER.wake();
        }
    } else {
        println!("WARNING! The scancode queue hasn't been initialized!");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE
            .try_init_once(|| ArrayQueue::new(100))
            .expect("The Scancode Queue should only be initialzed once.");
        Self { _private: () }
    }
}

use core::{
    pin::Pin,
    task::{Context, Poll},
};
use futures_util::task::AtomicWaker;
use futures_util::{stream::Stream, StreamExt};

static WAKER: AtomicWaker = AtomicWaker::new();

impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("The queue hasn't been initialized yet");

        if let Ok(scancode) = queue.pop() {
            return Poll::Ready(Some(scancode));
        }

        WAKER.register(cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            _ => Poll::Pending,
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
                    pc_keyboard::DecodedKey::RawKey(k) => print!("{:?}", k),
                    pc_keyboard::DecodedKey::Unicode(ch) => print!("{}", ch),
                }
            }
        }
    }
}
