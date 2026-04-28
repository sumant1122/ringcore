//! Asynchronous `io_uring` operations.
//! 
//! This module contains the `Op` future and functions to create various
//! asynchronous operations.

use crate::executor::{RESULTS, RING, WAKERS, MULTISHOT};
use crate::sys::*;
use std::future::Future;
use std::io;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::task::{Context, Poll};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

/// A future representing an asynchronous `io_uring` operation.
pub struct Op<T> {
    /// The unique identifier for this operation.
    pub id: u64,
    pub(crate) sqe: Option<io_uring_sqe>,
    pub(crate) _marker: std::marker::PhantomData<T>,
}

impl<T> Op<T> {
    /// Create a new `Op` from an SQE.
    pub fn new(sqe: io_uring_sqe) -> Self {
        Self::with_multishot(sqe, false)
    }

    /// Extract the SQE from this Op, leaving None in its place.
    /// This is useful for manual submission or linking.
    pub fn take_sqe(&mut self) -> Option<io_uring_sqe> {
        self.sqe.take()
    }

    /// Create a new `Op` from an SQE and specify if it's a multishot operation.
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

    /// Link this operation to the next one submitted to the ring.
    /// The next operation will only start if this one succeeds.
    pub fn link(mut self) -> Self {
        if let Some(ref mut sqe) = self.sqe {
            sqe.flags |= IOSQE_IO_LINK;
        }
        self
    }
}

impl<T> Future for Op<T> {
    type Output = io::Result<i32>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = unsafe { self.get_unchecked_mut() };
        if let Some(sqe) = this.sqe.take() {
            WAKERS.with(|w| w.borrow_mut().insert(this.id, cx.waker().clone()));
            let submitted = RING.with(|r| {
                if let Some(ring) = r.borrow().as_ref() {
                    ring.submit(sqe)
                } else {
                    false
                }
            });
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

/// Create a read operation.
pub fn read(fd: i32, buf: *mut u8, len: u32, offset: u64) -> Op<()> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_READ,
        fd,
        addr: buf as u64,
        len,
        off: offset,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a write operation.
pub fn write(fd: i32, buf: *const u8, len: u32, offset: u64) -> Op<()> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_WRITE,
        fd,
        addr: buf as u64,
        len,
        off: offset,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create an accept operation.
pub fn accept(fd: i32, addr: *mut libc::sockaddr, addrlen: *mut libc::socklen_t) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_ACCEPT,
        fd,
        addr: addr as u64,
        off: addrlen as u64,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a multishot accept operation.
pub fn accept_multishot(fd: i32) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_ACCEPT,
        fd,
        union1: IORING_ACCEPT_MULTISHOT,
        ..Default::default()
    };
    Op::with_multishot(sqe, true)
}

/// Create a timeout operation.
pub fn timeout(ts: &mut __kernel_timespec) -> Op<()> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_TIMEOUT,
        addr: ts as *mut _ as u64,
        len: 1,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a connect operation.
pub fn connect(fd: i32, addr: *const libc::sockaddr, addrlen: libc::socklen_t) -> Op<()> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_CONNECT,
        fd,
        addr: addr as u64,
        off: addrlen as u64,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create an openat operation.
pub fn openat(dirfd: i32, pathname: *const libc::c_char, flags: i32, mode: u32) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_OPENAT,
        fd: dirfd,
        addr: pathname as u64,
        union1: flags as u32,
        len: mode,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a close operation.
pub fn close(fd: i32) -> Op<()> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_CLOSE,
        fd,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a splice operation.
pub fn splice(
    fd_in: i32,
    off_in: i64,
    fd_out: i32,
    off_out: i64,
    nbytes: u32,
    flags: u32,
) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_SPLICE,
        fd: fd_out,
        addr: fd_in as u64,
        off: off_out as u64,
        union1: flags,
        len: nbytes,
        union2: off_in as u64,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create an async cancel operation.
pub fn cancel(user_data: u64) -> Op<()> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_ASYNC_CANCEL,
        addr: user_data,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a writev (vectored write) operation.
pub fn writev(fd: i32, iovecs: *const libc::iovec, nr: u32, offset: u64) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_WRITEV,
        fd,
        addr: iovecs as u64,
        len: nr,
        off: offset,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a read_fixed operation.
pub fn read_fixed(fd: i32, buf: *mut u8, len: u32, offset: u64, buf_index: u16) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_READ_FIXED,
        fd,
        addr: buf as u64,
        len,
        off: offset,
        union2: buf_index as u64,
        ..Default::default()
    };
    Op::new(sqe)
}

/// Create a write_fixed operation.
pub fn write_fixed(fd: i32, buf: *const u8, len: u32, offset: u64, buf_index: u16) -> Op<i32> {
    let sqe = io_uring_sqe {
        opcode: IORING_OP_WRITE_FIXED,
        fd,
        addr: buf as u64,
        len,
        off: offset,
        union2: buf_index as u64,
        ..Default::default()
    };
    Op::new(sqe)
}
