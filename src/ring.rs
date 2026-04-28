//! Low-level `io_uring` abstraction.

use crate::sys::*;
use std::io;
use std::ptr;
use std::sync::atomic::{AtomicU32, Ordering};

/// An abstraction over a Linux `io_uring` instance.
pub struct IoUring {
    fd: i32,
    _sq_ptr: *mut u8,
    _cq_ptr: *mut u8,
    sqes_ptr: *mut io_uring_sqe,
    
    sq_head_ptr: *const AtomicU32,
    sq_tail_ptr: *mut AtomicU32,
    sq_mask: u32,
    sq_entries: u32,
    sq_array: *mut u32,

    cq_head_ptr: *mut AtomicU32,
    cq_tail_ptr: *const AtomicU32,
    cq_mask: u32,
    cqes: *mut io_uring_cqe,
}

impl IoUring {
    /// Create a new `IoUring` with the specified number of entries.
    pub fn new(entries: u32) -> io::Result<Self> {
        Self::with_flags(entries, 0)
    }

    /// Create a new `IoUring` with specified entries and flags.
    pub fn with_flags(entries: u32, flags: u32) -> io::Result<Self> {
        let mut params = io_uring_params {
            flags,
            ..Default::default()
        };
        let fd = unsafe { io_uring_setup(entries, &mut params) };
        if fd < 0 {
            return Err(io::Error::last_os_error());
        }

        let sq_size = params.sq_off.array + params.sq_entries * std::mem::size_of::<u32>() as u32;
        let cq_size = params.cq_off.cqes + params.cq_entries * std::mem::size_of::<io_uring_cqe>() as u32;

        unsafe {
            let sq_ptr = libc::mmap(
                ptr::null_mut(),
                sq_size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                fd,
                IORING_OFF_SQ_RING as i64,
            );
            if sq_ptr == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            let cq_ptr = libc::mmap(
                ptr::null_mut(),
                cq_size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                fd,
                IORING_OFF_CQ_RING as i64,
            );
            if cq_ptr == libc::MAP_FAILED {
                return Err(io::Error::last_os_error());
            }

            let sqes_size = params.sq_entries * std::mem::size_of::<io_uring_sqe>() as u32;
            let sqes_ptr = libc::mmap(
                ptr::null_mut(),
                sqes_size as usize,
                libc::PROT_READ | libc::PROT_WRITE,
                libc::MAP_SHARED | libc::MAP_POPULATE,
                fd,
                IORING_OFF_SQES as i64,
            ) as *mut io_uring_sqe;
            if sqes_ptr == libc::MAP_FAILED as *mut io_uring_sqe {
                return Err(io::Error::last_os_error());
            }

            Ok(IoUring {
                fd,
                _sq_ptr: sq_ptr as *mut u8,
                _cq_ptr: cq_ptr as *mut u8,
                sqes_ptr,
                sq_head_ptr: sq_ptr.add(params.sq_off.head as usize) as *const AtomicU32,
                sq_tail_ptr: sq_ptr.add(params.sq_off.tail as usize) as *mut AtomicU32,
                sq_mask: *(sq_ptr.add(params.sq_off.ring_mask as usize) as *const u32),
                sq_entries: params.sq_entries,
                sq_array: sq_ptr.add(params.sq_off.array as usize) as *mut u32,
                cq_head_ptr: cq_ptr.add(params.cq_off.head as usize) as *mut AtomicU32,
                cq_tail_ptr: cq_ptr.add(params.cq_off.tail as usize) as *const AtomicU32,
                cq_mask: *(cq_ptr.add(params.cq_off.ring_mask as usize) as *const u32),
                cqes: cq_ptr.add(params.cq_off.cqes as usize) as *mut io_uring_cqe,
            })
        }
    }

    /// Submit a single SQE to the ring.
    /// Returns `true` if submitted successfully, `false` if the queue is full.
    pub fn submit(&self, sqe: io_uring_sqe) -> bool {
        unsafe {
            let tail = (*self.sq_tail_ptr).load(Ordering::Relaxed);
            let head = (*self.sq_head_ptr).load(Ordering::Acquire);
            
            if tail - head >= self.sq_entries {
                return false;
            }

            let index = tail & self.sq_mask;
            *self.sqes_ptr.add(index as usize) = sqe;
            *self.sq_array.add(index as usize) = index;
            
            (*self.sq_tail_ptr).store(tail + 1, Ordering::Release);
            true
        }
    }

    /// Submit multiple SQEs to the ring.
    /// Returns the number of SQEs successfully submitted.
    pub fn submit_multiple(&self, sqes: &[io_uring_sqe]) -> usize {
        unsafe {
            let mut tail = (*self.sq_tail_ptr).load(Ordering::Relaxed);
            let head = (*self.sq_head_ptr).load(Ordering::Acquire);
            
            let mut count = 0;
            for sqe in sqes {
                if tail - head >= self.sq_entries {
                    break;
                }
                let index = tail & self.sq_mask;
                *self.sqes_ptr.add(index as usize) = *sqe;
                *self.sq_array.add(index as usize) = index;
                tail += 1;
                count += 1;
            }
            
            if count > 0 {
                (*self.sq_tail_ptr).store(tail, Ordering::Release);
            }
            count
        }
    }

    /// Get the number of SQEs currently waiting in the queue.
    pub fn pending_sqes(&self) -> u32 {
        unsafe {
            let tail = (*self.sq_tail_ptr).load(Ordering::Relaxed);
            let head = (*self.sq_head_ptr).load(Ordering::Acquire);
            tail - head
        }
    }

    /// Register fixed buffers with the ring.
    /// 
    /// # Safety
    /// The caller must ensure that `iovecs` points to a valid array of `libc::iovec`
    /// and that the buffers described by them remain valid for the duration of the registration.
    pub unsafe fn register_buffers(&self, iovecs: *const libc::iovec, nr: u32) -> io::Result<()> {
        let res = unsafe {
            libc::syscall(
                SYS_IO_URING_REGISTER as i64,
                self.fd,
                IORING_REGISTER_BUFFERS,
                iovecs,
                nr,
            )
        };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(())
        }
    }

    /// Enter the kernel to submit SQEs and/or wait for completions.
    pub fn enter(&self, to_submit: u32, min_complete: u32) -> io::Result<u32> {
        let res = unsafe {
            io_uring_enter(
                self.fd,
                to_submit,
                min_complete,
                if min_complete > 0 { IORING_ENTER_GETEVENTS } else { 0 },
                ptr::null(),
            )
        };
        if res < 0 {
            Err(io::Error::last_os_error())
        } else {
            Ok(res as u32)
        }
    }

    /// Poll for any available completions in the CQ.
    pub fn poll_completions(&self) -> Vec<io_uring_cqe> {
        let mut completions = Vec::new();
        unsafe {
            let mut head = (*self.cq_head_ptr).load(Ordering::Relaxed);
            let tail = (*self.cq_tail_ptr).load(Ordering::Acquire);

            while head != tail {
                let index = head & self.cq_mask;
                completions.push(*self.cqes.add(index as usize));
                head += 1;
            }
            (*self.cq_head_ptr).store(head, Ordering::Release);
        }
        completions
    }
}

impl Drop for IoUring {
    fn drop(&mut self) {
        unsafe {
            // In a real implementation we'd need to keep track of mmap sizes
            // For this minimal one, we just close the fd.
            libc::close(self.fd);
        }
    }
}
