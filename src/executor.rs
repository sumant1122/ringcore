//! The RingCore Task Executor.
//! 
//! This module implements a single-threaded executor that manages asynchronous tasks
//! and coordinates with the `io_uring` instance for I/O event notification.

use crate::ring::IoUring;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, RawWaker, RawWakerVTable, Waker};

thread_local! {
    /// The global `io_uring` instance for the current thread.
    pub static RING: RefCell<Option<IoUring>> = RefCell::new(IoUring::new(256).ok());
    /// Map of pending wakers indexed by user_data.
    pub static WAKERS: RefCell<HashMap<u64, Waker>> = RefCell::new(HashMap::new());
    /// Map of operation results indexed by user_data.
    pub static RESULTS: RefCell<HashMap<u64, VecDeque<i32>>> = RefCell::new(HashMap::new());
    /// Map indicating if an operation is multishot.
    pub static MULTISHOT: RefCell<HashMap<u64, bool>> = RefCell::new(HashMap::new());
    /// Queue of tasks ready to be polled.
    pub static TASKS: RefCell<VecDeque<Rc<Task>>> = const { RefCell::new(VecDeque::new()) };
}

/// Initialize the global `io_uring` instance with custom parameters.
pub fn init(entries: u32, flags: u32) -> std::io::Result<()> {
    let ring = IoUring::with_flags(entries, flags)?;
    RING.with(|r| {
        *r.borrow_mut() = Some(ring);
    });
    Ok(())
}

/// A handle to a spawned asynchronous task.
pub struct Task {
    /// The pinned future representing the task's work.
    pub future: RefCell<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Task {
    fn waker(self: &Rc<Self>) -> Waker {
        let ptr = Rc::into_raw(self.clone()) as *const ();
        unsafe { Waker::from_raw(RawWaker::new(ptr, &VTABLE)) }
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(
    |ptr| unsafe {
        let rc = Rc::from_raw(ptr as *const Task);
        let new_ptr = Rc::into_raw(rc.clone()) as *const ();
        std::mem::forget(rc);
        RawWaker::new(new_ptr, &VTABLE)
    },
    |ptr| {
        let rc = unsafe { Rc::from_raw(ptr as *const Task) };
        TASKS.with(|t| t.borrow_mut().push_back(rc));
    },
    |ptr| {
        let rc = unsafe { Rc::from_raw(ptr as *const Task) };
        TASKS.with(|t| t.borrow_mut().push_back(rc));
    },
    |ptr| unsafe {
        drop(Rc::from_raw(ptr as *const Task));
    },
);

/// Spawn a new task into the current thread's executor.
pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let task = Rc::new(Task {
        future: RefCell::new(Box::pin(future)),
    });
    TASKS.with(|t| t.borrow_mut().push_back(task));
}

/// Run the executor loop until all tasks have completed.
/// 
/// This function will block the current thread, polling tasks and waiting for
/// `io_uring` completions in a loop.
pub fn run() {
    loop {
        let mut progress = false;

        // 1. Poll ready tasks
        while let Some(task) = TASKS.with(|t| t.borrow_mut().pop_front()) {
            let waker = task.waker();
            let mut cx = Context::from_waker(&waker);
            let mut future = task.future.borrow_mut();
            if future.as_mut().poll(&mut cx).is_ready() {
                // Task finished
            }
            progress = true;
        }

        // 2. Submit SQEs
        let pending = RING.with(|r| {
            r.borrow()
                .as_ref()
                .map(|ring| ring.pending_sqes())
                .unwrap_or(0)
        });
        if pending > 0 {
            RING.with(|r| {
                if let Some(ring) = r.borrow().as_ref()
                    && let Err(e) = ring.enter(pending, 0)
                {
                    eprintln!("io_uring_enter submit error: {}", e);
                }
            });
        }

        // 3. Poll CQEs
        let completions = RING.with(|r| {
            r.borrow()
                .as_ref()
                .map(|ring| ring.poll_completions())
                .unwrap_or_default()
        });
        for cqe in completions {
            let is_multishot = MULTISHOT.with(|m| *m.borrow().get(&cqe.user_data).unwrap_or(&false));
            
            RESULTS.with(|r| {
                r.borrow_mut().entry(cqe.user_data).or_default().push_back(cqe.res);
            });

            if is_multishot {
                if let Some(waker) = WAKERS.with(|w| w.borrow().get(&cqe.user_data).cloned()) {
                    waker.wake();
                    progress = true;
                }
            } else {
                if let Some(waker) = WAKERS.with(|w| w.borrow_mut().remove(&cqe.user_data)) {
                    waker.wake();
                    progress = true;
                }
                MULTISHOT.with(|m| m.borrow_mut().remove(&cqe.user_data));
            }
        }

        // 4. Block if no progress
        if !progress {
            let pending_wakers = WAKERS.with(|w| w.borrow().len());
            let pending_tasks = TASKS.with(|t| t.borrow().len());
            
            if pending_wakers == 0 && pending_tasks == 0 {
                break;
            }
            
            // Wait for at least one completion
            RING.with(|r| {
                if let Some(ring) = r.borrow().as_ref() {
                    let to_submit = ring.pending_sqes();
                    if let Err(e) = ring.enter(to_submit, 1)
                        && e.kind() != std::io::ErrorKind::Interrupted
                    {
                        eprintln!("io_uring_enter wait error: {}", e);
                    }
                }
            });
        }
    }
}
