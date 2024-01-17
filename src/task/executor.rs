use alloc::collections::BTreeMap;
use crossbeam_queue::ArrayQueue;
use x86_64::instructions::interrupts;

use super::{Task, TaskId};
use alloc::sync::Arc;
use alloc::task::Wake;
use core::task::Waker;
use core::task::{Context, Poll};
pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    wakers: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            wakers: BTreeMap::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with the same id already exists!");
        }
        self.task_queue.push(task_id).expect("Task queue full");
    }

    pub fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            wakers,
        } = self;
        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue,
            };
            let waker = wakers
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));

            let mut cx = Context::from_waker(waker);
            match task.poll(&mut cx) {
                Poll::Ready(()) => {
                    tasks.remove(&task_id);
                    wakers.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();
        if self.task_queue.is_empty() {
            interrupts::enable_and_hlt();
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
        let task_waker = Self {
            task_id,
            task_queue,
        };

        Waker::from(Arc::new(task_waker))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("Task queue full");
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task()
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task()
    }
}
