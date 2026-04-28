//! Low-level Linux `io_uring` bindings.

use std::os::raw::{c_int, c_uint, c_void};

/// Syscall number for `io_uring_setup`.
pub const SYS_IO_URING_SETUP: c_int = 425;
/// Syscall number for `io_uring_enter`.
pub const SYS_IO_URING_ENTER: c_int = 426;

/// Offset for mapping the submission queue ring.
pub const IORING_OFF_SQ_RING: u64 = 0;
/// Offset for mapping the completion queue ring.
pub const IORING_OFF_CQ_RING: u64 = 0x8000000;
/// Offset for mapping the submission queue entries.
pub const IORING_OFF_SQES: u64 = 0x10000000;

/// Setup flag: use a kernel thread to poll for SQEs.
pub const IORING_SETUP_SQPOLL: c_uint = 1 << 1;

// SQE Opcodes
/// Opcode: vectored read.
pub const IORING_OP_READV: u8 = 1;
/// Opcode: vectored write.
pub const IORING_OP_WRITEV: u8 = 2;
/// Opcode: read from a fixed buffer.
pub const IORING_OP_READ_FIXED: u8 = 4;
/// Opcode: write to a fixed buffer.
pub const IORING_OP_WRITE_FIXED: u8 = 5;
/// Opcode: standard read.
pub const IORING_OP_READ: u8 = 22;
/// Opcode: standard write.
pub const IORING_OP_WRITE: u8 = 23;
/// Opcode: accept a new connection.
pub const IORING_OP_ACCEPT: u8 = 13;
/// Opcode: register a timeout.
pub const IORING_OP_TIMEOUT: u8 = 11;
/// Opcode: connect to a remote address.
pub const IORING_OP_CONNECT: u8 = 16;
/// Opcode: close a file descriptor.
pub const IORING_OP_CLOSE: u8 = 19;
/// Opcode: open a file relative to a directory.
pub const IORING_OP_OPENAT: u8 = 18;
/// Opcode: splice two file descriptors.
pub const IORING_OP_SPLICE: u8 = 15;
/// Opcode: cancel an asynchronous operation.
pub const IORING_OP_ASYNC_CANCEL: u8 = 14;

// Flags
/// Accept flag: enable multishot mode.
pub const IORING_ACCEPT_MULTISHOT: u32 = 1 << 0;
/// Setup flag: specify a custom CQ size.
pub const IORING_SETUP_CQSIZE: u32 = 1 << 3;

// SQE Flags
/// SQE flag: use a fixed file descriptor.
pub const IOSQE_FIXED_FILE: u8 = 1 << 0;
/// SQE flag: drain all previous requests before starting.
pub const IOSQE_IO_DRAIN: u8 = 1 << 1;
/// SQE flag: link this operation with the next one.
pub const IOSQE_IO_LINK: u8 = 1 << 2;
/// SQE flag: hard-link this operation with the next one.
pub const IOSQE_IO_HARDLINK: u8 = 1 << 3;
/// SQE flag: start operation asynchronously.
pub const IOSQE_ASYNC: u8 = 1 << 4;
/// SQE flag: use a buffer from the provided pool.
pub const IOSQE_BUFFER_SELECT: u8 = 1 << 5;

// Syscalls
/// Syscall number for `io_uring_register`.
pub const SYS_IO_URING_REGISTER: c_int = 427;

/// Register opcode: register a set of buffers.
pub const IORING_REGISTER_BUFFERS: c_uint = 0;
/// Register opcode: unregister previously registered buffers.
pub const IORING_UNREGISTER_BUFFERS: c_uint = 1;

/// Enter flag: wait for events.
pub const IORING_ENTER_GETEVENTS: c_uint = 1 << 0;

/// A Submission Queue Entry.
#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct io_uring_sqe {
    /// Operation opcode.
    pub opcode: u8,
    /// SQE flags.
    pub flags: u8,
    /// Priority for the request.
    pub ioprio: u16,
    /// File descriptor.
    pub fd: i32,
    /// Offset for the operation.
    pub off: u64,
    /// Address (pointer) for the operation.
    pub addr: u64,
    /// Length of the buffer or number of iovecs.
    pub len: u32,
    /// Union field: read/write flags.
    pub union1: u32,
    /// User data returned in the CQE.
    pub user_data: u64,
    /// Union field: buffer index.
    pub union2: u64,
    /// Padding.
    pub __pad2: [u64; 2],
}

/// A Completion Queue Entry.
#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct io_uring_cqe {
    /// User data from the SQE.
    pub user_data: u64,
    /// Result of the operation (bytes or -errno).
    pub res: i32,
    /// Completion flags.
    pub flags: u32,
}

/// Offsets for the submission queue ring members.
#[repr(C)]
#[derive(Default)]
pub struct io_sqring_offsets {
    /// Offset to the head pointer.
    pub head: u32,
    /// Offset to the tail pointer.
    pub tail: u32,
    /// Offset to the ring mask.
    pub ring_mask: u32,
    /// Offset to the number of ring entries.
    pub ring_entries: u32,
    /// Offset to the flags.
    pub flags: u32,
    /// Offset to the dropped count.
    pub dropped: u32,
    /// Offset to the index array.
    pub array: u32,
    /// Reserved.
    pub resv1: u32,
    /// Reserved.
    pub resv2: u64,
}

/// Offsets for the completion queue ring members.
#[repr(C)]
#[derive(Default)]
pub struct io_cqring_offsets {
    /// Offset to the head pointer.
    pub head: u32,
    /// Offset to the tail pointer.
    pub tail: u32,
    /// Offset to the ring mask.
    pub ring_mask: u32,
    /// Offset to the number of ring entries.
    pub ring_entries: u32,
    /// Offset to the overflow count.
    pub overflow: u32,
    /// Offset to the CQEs array.
    pub cqes: u32,
    /// Offset to the flags.
    pub flags: u32,
    /// Reserved.
    pub resv1: u32,
    /// Reserved.
    pub resv2: u64,
}

/// Parameters for `io_uring_setup`.
#[repr(C)]
#[derive(Default)]
pub struct io_uring_params {
    /// Number of SQ entries.
    pub sq_entries: u32,
    /// Number of CQ entries.
    pub cq_entries: u32,
    /// Setup flags.
    pub flags: u32,
    /// CPU for SQ thread.
    pub sq_thread_cpu: u32,
    /// Idle time for SQ thread.
    pub sq_thread_idle: u32,
    /// Features supported by the kernel.
    pub features: u32,
    /// WQ file descriptor.
    pub wq_fd: u32,
    /// Reserved.
    pub resv: [u32; 3],
    /// SQ offsets.
    pub sq_off: io_sqring_offsets,
    /// CQ offsets.
    pub cq_off: io_cqring_offsets,
}

/// Kernel timespec structure.
#[repr(C)]
pub struct __kernel_timespec {
    /// Seconds.
    pub tv_sec: i64,
    /// Nanoseconds.
    pub tv_nsec: i64,
}

/// Low-level wrapper for `io_uring_setup` syscall.
/// 
/// # Safety
/// The caller must ensure that `p` is a valid pointer to an `io_uring_params` structure.
pub unsafe fn io_uring_setup(entries: c_uint, p: *mut io_uring_params) -> c_int {
    unsafe { libc::syscall(SYS_IO_URING_SETUP as i64, entries, p) as c_int }
}

/// Low-level wrapper for `io_uring_enter` syscall.
/// 
/// # Safety
/// The caller must ensure that `sig` is either null or a valid pointer to a `sigset_t`.
pub unsafe fn io_uring_enter(
    fd: c_int,
    to_submit: c_uint,
    min_complete: c_uint,
    flags: c_uint,
    sig: *const c_void,
) -> c_int {
    unsafe {
        libc::syscall(
            SYS_IO_URING_ENTER as i64,
            fd,
            to_submit,
            min_complete,
            flags,
            sig,
            std::mem::size_of::<libc::sigset_t>(),
        ) as c_int
    }
}
