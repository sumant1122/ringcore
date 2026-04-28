use crate::executor::{RESULTS, RING, WAKERS, MULTISHOT};
use crate::sys::*;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

pub struct Op<T> {
    pub id: u64,
    pub(crate) sqe: Option<io_uring_sqe>,
    pub(crate) _marker: std::marker::PhantomData<T>,
}

impl<T> Op<T> {
    pub fn new(sqe: io_uring_sqe) -> Self {
        Self::with_multishot(sqe, false)
    }

    pub fn with_multishot(mut sqe: io_uring_sqe, multishot: bool) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        sqe.user_data = id;
        if multishot {
            MULTISHOT.with(|m| m.borrow_mut().insert(id, true));
        }
        Op {
            id,
            sqe: Some(sqe),
            _marker: std::marker::PhantomData,
        }
    }
}

impl<T> Future for Op<T> {
    type Output = io::Result<i32>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        if let Some(sqe) = this.sqe.take() {
            WAKERS.with(|w| w.borrow_mut().insert(this.id, cx.waker().clone()));
            let submitted = RING.with(|r| r.borrow().submit(sqe));
            if !submitted {
                this.sqe = Some(sqe);
                WAKERS.with(|w| w.borrow_mut().remove(&this.id));
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
        }

        let res = RESULTS.with(|r| {
            r.borrow_mut().get_mut(&this.id).and_then(|q| q.pop_front())
        });

        if let Some(res) = res {
            if res < 0 {
                Poll::Ready(Err(io::Error::from_raw_os_error(-res)))
            } else {
                Poll::Ready(Ok(res))
            }
        } else {
            WAKERS.with(|w| w.borrow_mut().insert(this.id, cx.waker().clone()));
            Poll::Pending
        }
    }
}

pub fn read(fd: i32, buf: *mut u8, len: u32, offset: u64) -> Op<()> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_READ;
    sqe.fd = fd;
    sqe.addr = buf as u64;
    sqe.len = len;
    sqe.off = offset;
    Op::new(sqe)
}

pub fn write(fd: i32, buf: *const u8, len: u32, offset: u64) -> Op<()> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_WRITE;
    sqe.fd = fd;
    sqe.addr = buf as u64;
    sqe.len = len;
    sqe.off = offset;
    Op::new(sqe)
}

pub fn accept(fd: i32, addr: *mut libc::sockaddr, addrlen: *mut libc::socklen_t) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_ACCEPT;
    sqe.fd = fd;
    sqe.addr = addr as u64;
    sqe.off = addrlen as u64;
    Op::new(sqe)
}

pub fn accept_multishot(fd: i32) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_ACCEPT;
    sqe.fd = fd;
    sqe.union1 = IORING_ACCEPT_MULTISHOT;
    Op::with_multishot(sqe, true)
}

pub fn timeout(ts: &mut __kernel_timespec) -> Op<()> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_TIMEOUT;
    sqe.addr = ts as *mut _ as u64;
    sqe.len = 1; // number of timespecs
    /*
     * SQE Fields for TIMEOUT:
     * opcode: IORING_OP_TIMEOUT (11)
     * addr: pointer to __kernel_timespec
     * len: 1 (specifying one timespec)
     * off: (optional) number of completions before timeout
     * user_data: unique ID to track completion in CQE
     */
    Op::new(sqe)
}

pub fn connect(fd: i32, addr: *const libc::sockaddr, addrlen: libc::socklen_t) -> Op<()> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_CONNECT;
    sqe.fd = fd;
    sqe.addr = addr as u64;
    sqe.off = addrlen as u64;
    Op::new(sqe)
}

pub fn openat(dirfd: i32, pathname: *const libc::c_char, flags: i32, mode: u32) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_OPENAT;
    sqe.fd = dirfd;
    sqe.addr = pathname as u64;
    sqe.union1 = flags as u32;
    sqe.len = mode;
    Op::new(sqe)
}

pub fn close(fd: i32) -> Op<()> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_CLOSE;
    sqe.fd = fd;
    Op::new(sqe)
}

pub fn splice(
    fd_in: i32,
    off_in: i64,
    fd_out: i32,
    off_out: i64,
    nbytes: u32,
    flags: u32,
) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_SPLICE;
    sqe.fd = fd_out;
    sqe.addr = fd_in as u64;
    sqe.off = off_out as u64;
    sqe.union1 = flags;
    sqe.len = nbytes;
    sqe.union2 = off_in as u64;
    Op::new(sqe)
}

pub fn cancel(user_data: u64) -> Op<()> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_ASYNC_CANCEL;
    sqe.addr = user_data;
    Op::new(sqe)
}

pub fn writev(fd: i32, iovecs: *const libc::iovec, nr: u32, offset: u64) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_WRITEV;
    sqe.fd = fd;
    sqe.addr = iovecs as u64;
    sqe.len = nr;
    sqe.off = offset;
    Op::new(sqe)
}

pub fn read_fixed(fd: i32, buf: *mut u8, len: u32, offset: u64, buf_index: u16) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_READ_FIXED;
    sqe.fd = fd;
    sqe.addr = buf as u64;
    sqe.len = len;
    sqe.off = offset;
    sqe.union2 = buf_index as u64;
    Op::new(sqe)
}

pub fn write_fixed(fd: i32, buf: *const u8, len: u32, offset: u64, buf_index: u16) -> Op<i32> {
    let mut sqe = io_uring_sqe::default();
    sqe.opcode = IORING_OP_WRITE_FIXED;
    sqe.fd = fd;
    sqe.addr = buf as u64;
    sqe.len = len;
    sqe.off = offset;
    sqe.union2 = buf_index as u64;
    Op::new(sqe)
}
