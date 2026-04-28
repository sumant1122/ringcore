use crate::ring::IoUring;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

thread_local! {
    pub static RING: RefCell<IoUring> = RefCell::new(IoUring::new(256).expect("Failed to init io_uring"));
    pub static WAKERS: RefCell<HashMap<u64, Waker>> = RefCell::new(HashMap::new());
    pub static RESULTS: RefCell<HashMap<u64, VecDeque<i32>>> = RefCell::new(HashMap::new());
    pub static MULTISHOT: RefCell<HashMap<u64, bool>> = RefCell::new(HashMap::new());
    pub static TASKS: RefCell<VecDeque<Rc<Task>>> = RefCell::new(VecDeque::new());
}

pub fn init(entries: u32, flags: u32) {
    RING.with(|r| {
        *r.borrow_mut() = IoUring::with_flags(entries, flags).expect("Failed to re-init io_uring");
    });
}

pub struct Task {
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

pub fn spawn<F>(future: F)
where
    F: Future<Output = ()> + 'static,
{
    let task = Rc::new(Task {
        future: RefCell::new(Box::pin(future)),
    });
    TASKS.with(|t| t.borrow_mut().push_back(task));
}

pub fn run() {
    loop {
        let mut progress = false;

        // 1. Poll ready tasks
        while let Some(task) = TASKS.with(|t| t.borrow_mut().pop_front()) {
            let waker = task.waker();
            let mut cx = Context::from_waker(&waker);
            let mut future = task.future.borrow_mut();
            if let Poll::Ready(_) = future.as_mut().poll(&mut cx) {
                // Task finished
            }
            progress = true;
        }

        // 2. Submit SQEs
        let pending = RING.with(|r| r.borrow().pending_sqes());
        if pending > 0 {
            if let Err(e) = RING.with(|r| r.borrow().enter(pending, 0)) {
                eprintln!("io_uring_enter submit error: {}", e);
            }
        }

        // 3. Poll CQEs
        let completions = RING.with(|r| r.borrow().poll_completions());
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
            let to_submit = RING.with(|r| r.borrow().pending_sqes());
            // eprintln!("DEBUG: blocking on enter, pending_wakers={}, pending_tasks={}, to_submit={}", pending_wakers, pending_tasks, to_submit);
            if let Err(e) = RING.with(|r| r.borrow().enter(to_submit, 1)) {
                if e.kind() != std::io::ErrorKind::Interrupted {
                    eprintln!("io_uring_enter wait error: {}", e);
                }
            }
        }
    }
}
