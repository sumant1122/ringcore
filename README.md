# RingRing: A Minimalist io_uring Async Runtime

RingRing is a "from scratch" implementation of an asynchronous Rust runtime powered by Linux's `io_uring`. It demonstrates the fundamental mechanics of async/await by bridging the gap between high-level `Future` traits and low-level kernel submission queues.

## Architecture

The project is structured into four distinct layers:

### 1. Ring Abstraction (`src/sys.rs`, `src/ring.rs`)
This layer handles the "raw" interaction with the Linux kernel.
- **Syscalls:** Manually invokes `SYS_IO_URING_SETUP` and `SYS_IO_URING_ENTER` via `libc`.
- **Memory Mapping:** Uses `mmap` to map the kernel's Submission Queue (SQ) and Completion Queue (CQ) directly into the process's address space.
- **Synchronization:** Uses `std::sync::atomic` to manage the head and tail pointers of the rings, ensuring thread-safe (or in this case, single-threaded but re-entrant) access to the shared kernel memory.

### 2. Op Futures (`src/op.rs`)
This layer translates `io_uring` operations into Rust `Future`s.
- **Lazy Submission:** An `Op` future only writes its Submission Queue Entry (SQE) to the ring when it is first polled.
- **Waker Management:** Each operation is assigned a unique `user_data` ID. Before returning `Poll::Pending`, the future stores its `Waker` in a global map.
- **Completion:** When the executor finds a Completion Queue Entry (CQE) with a matching ID, it retrieves the result and triggers the `Waker`.

### 3. Executor (`src/executor.rs`)
The "heart" of the runtime.
- **Task Queue:** A `VecDeque` of tasks ready to be polled.
- **Run Loop:** 
    1. Polls all ready tasks.
    2. Enters the ring to submit pending SQEs and harvest CQEs.
    3. Wakes tasks associated with finished CQEs.
    4. Blocks on `io_uring_enter` (with `min_complete=1`) if there is no work to do, effectively putting the thread to sleep until the kernel notifies it of a completion.

### 4. Resource Handles (`src/net.rs`)
High-level, idiomatic wrappers for system resources.
- **Thread-Local Storage:** Uses a `thread_local!` macro to provide access to the `IoUring` instance, allowing `TcpListener` and `TcpStream` to submit operations without carrying around references or handles.
- **Async API:** Provides `async fn` methods for `accept`, `read`, and `write` that feel like standard Rust networking but use the underlying `io_uring` ops.

## Requirements
- **OS:** Linux 5.10+ (for stable `IORING_OP_ACCEPT` support).
- **Architecture:** x86_64.
- **Dependencies:** `libc` and `std` only.

## Why io_uring is faster?
In a standard threaded model, every connection incurs the overhead of a thread stack and context switching. More importantly, every `read` and `write` is a separate system call that triggers a user-to-kernel mode switch.

With `io_uring`:
- **System Call Elision:** Multiple operations (SQEs) can be submitted with a single `io_uring_enter` call.
- **Zero-Copy (Potential):** While this minimal runtime uses buffers, `io_uring` supports registered buffers for even higher performance.
- **Single-Threaded Efficiency:** We handle thousands of connections on a single thread without the overhead of thread management.

## Benchmarks

We compared RingRing against the Rust Standard Library and Tokio on a Debian 13 (Kernel 6.12) system.

### 1. File I/O (Cat 100MB File)
Sequential read and write performance.

| Runtime | Real Time | System Time |
|---------|-----------|-------------|
| **Standard (`std::fs`)** | 0.057s | 0.016s |
| **Tokio (epoll + thread pool)** | 0.461s | 0.376s |
| **RingRing (`io_uring`)** | **0.088s** | **0.036s** |

*Note: RingRing significantly outperforms Tokio on file I/O because it uses true asynchronous kernel operations instead of a blocking thread pool.*

### 2. Networking (100 HTTP Requests)
Total time for 100 sequential GET requests.

| Server / Client | Std Client | Tokio Client | RingRing Client |
|-----------------|------------|--------------|-----------------|
| **Std Server** | 12.8ms | 13.1ms | 13.4ms |
| **Tokio Server** | 17.8ms | 14.9ms | 10.4ms |
| **RingRing Server** | **9.9ms** | **15.8ms** | **7.5ms** |

*RingRing shows the lowest latency for high-frequency networking tasks by minimizing context switches and system call overhead.*

## Examples

Run any example using `cargo run --example <name>`.

### Tier 1: Proving the Runtime
- **Echo Server:** `cargo run --example echo`
  - Chained Accept -> Read -> Write.
- **Async Cat:** `cargo run --example cat -- <file>`
  - File I/O in isolation.
- **Async Timer:** `cargo run --example timer`
  - Non-I/O task parking and waking.
- **Ping Pong:** `cargo run --example ping_pong`
  - Task synchronization over a `socketpair`.

### Tier 2: Showing the Async Model
- **Concurrent Reads:** `cargo run --example concurrent_downloads`
  - Submitting 100 SQEs simultaneously.
- **Tee Utility:** `cargo run --example tee -- <file1> <file2>`
  - Fan-out writes to multiple files.
- **Timeout Race:** `cargo run --example timeout_race`
  - Demonstrates operation cancellation (`IORING_OP_ASYNC_CANCEL`).

### Tier 3: Real Workloads
- **HTTP Server:** `cargo run --example http_server`
  - High-concurrency "Hello World" benchmark.
- **File Server:** `cargo run --example file_server`
  - Serving static files over TCP.
- **Logger:** `cargo run --example logger`
  - Batch writes using Scatter-Gather (`writev`).

### Tier 4: Advanced Features
- **SQPOLL:** `sudo cargo run --example sqpoll`
  - Kernel-side SQ polling (requires `sudo` for `CAP_SYS_ADMIN`).
- **Fixed Buffers:** `cargo run --example fixed_buffers`
  - Pre-registered kernel buffers for zero-copy.
- **Multishot Accept:** `cargo run --example multishot_accept`
  - One SQE generating infinite connection CQEs.

## Internal SQE/CQE Mapping
Every `io_uring_sqe` field and `io_uring_cqe` field is documented in `src/sys.rs` to explain how the kernel interprets the submission and reports results.
