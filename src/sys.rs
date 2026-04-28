use std::os::raw::{c_int, c_uint, c_void};

pub const SYS_IO_URING_SETUP: c_int = 425;
pub const SYS_IO_URING_ENTER: c_int = 426;

pub const IORING_OFF_SQ_RING: u64 = 0;
pub const IORING_OFF_CQ_RING: u64 = 0x8000000;
pub const IORING_OFF_SQES: u64 = 0x10000000;

pub const IORING_SETUP_SQPOLL: c_uint = 1 << 1;

// SQE Opcodes
pub const IORING_OP_READV: u8 = 1;
pub const IORING_OP_WRITEV: u8 = 2;
pub const IORING_OP_READ_FIXED: u8 = 4;
pub const IORING_OP_WRITE_FIXED: u8 = 5;
pub const IORING_OP_READ: u8 = 22;
pub const IORING_OP_WRITE: u8 = 23;
pub const IORING_OP_ACCEPT: u8 = 13;
pub const IORING_OP_TIMEOUT: u8 = 11;
pub const IORING_OP_CONNECT: u8 = 16;
pub const IORING_OP_CLOSE: u8 = 19;
pub const IORING_OP_OPENAT: u8 = 18;
pub const IORING_OP_SPLICE: u8 = 15;
pub const IORING_OP_ASYNC_CANCEL: u8 = 14;

// Flags
pub const IORING_ACCEPT_MULTISHOT: u32 = 1 << 0;
pub const IORING_SETUP_CQSIZE: u32 = 1 << 3;

// Syscalls
pub const SYS_IO_URING_REGISTER: c_int = 427;

pub const IORING_REGISTER_BUFFERS: c_uint = 0;
pub const IORING_UNREGISTER_BUFFERS: c_uint = 1;

pub const IORING_ENTER_GETEVENTS: c_uint = 1 << 0;

#[repr(C)]
#[derive(Default, Clone, Copy)]
pub struct io_uring_sqe {
    pub opcode: u8,    /* type of operation: e.g. IORING_OP_READ, IORING_OP_WRITE */
    pub flags: u8,     /* IOSQE_ flags: control behavior of this SQE (e.g. linked ops) */
    pub ioprio: u16,    /* ioprio for the request: sets priority of the operation */
    pub fd: i32,       /* file descriptor: the target of the operation */
    pub off: u64,      /* offset: file offset or pointer to extra data (like addrlen in accept) */
    pub addr: u64,     /* address: pointer to buffer or sockaddr */
    pub len: u32,      /* length: size of buffer or number of iovecs */
    pub union1: u32,   /* rw_flags: read/write flags (e.g. RWF_NOWAIT) */
    pub user_data: u64, /* user_data: custom ID returned in CQE to identify the request */
    pub union2: u64,   /* buf_index: index into fixed buffers or index for fixed files */
    pub __pad2: [u64; 2], /* padding: reserved for future expansion (total SQE size must be 64 bytes) */
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct io_uring_cqe {
    pub user_data: u64, /* user_data: matches the user_data field from the original SQE */
    pub res: i32,      /* res: result of the operation (e.g. bytes read, or -ERRNO) */
    pub flags: u32,    /* flags: additional information about the completion (e.g. multi-shot) */
}

#[repr(C)]
#[derive(Default)]
pub struct io_sqring_offsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub flags: u32,
    pub dropped: u32,
    pub array: u32,
    pub resv1: u32,
    pub resv2: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct io_cqring_offsets {
    pub head: u32,
    pub tail: u32,
    pub ring_mask: u32,
    pub ring_entries: u32,
    pub overflow: u32,
    pub cqes: u32,
    pub flags: u32,
    pub resv1: u32,
    pub resv2: u64,
}

#[repr(C)]
#[derive(Default)]
pub struct io_uring_params {
    pub sq_entries: u32,
    pub cq_entries: u32,
    pub flags: u32,
    pub sq_thread_cpu: u32,
    pub sq_thread_idle: u32,
    pub features: u32,
    pub wq_fd: u32,
    pub resv: [u32; 3],
    pub sq_off: io_sqring_offsets,
    pub cq_off: io_cqring_offsets,
}

#[repr(C)]
pub struct __kernel_timespec {
    pub tv_sec: i64,
    pub tv_nsec: i64,
}

pub unsafe fn io_uring_setup(entries: c_uint, p: *mut io_uring_params) -> c_int {
    unsafe { libc::syscall(SYS_IO_URING_SETUP as i64, entries, p) as c_int }
}

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
