//! # RingCore
//! 
//! A minimalist async runtime powered by Linux's `io_uring`.
//! 
//! RingCore provides a transparent, "from-scratch" implementation of an asynchronous
//! runtime. It is designed for educational purposes and high-performance I/O tasks
//! where transparency over the underlying kernel operations is preferred.
//! 
//! ## Key Components
//! - `executor`: A single-threaded task scheduler and event loop.
//! - `ring`: Low-level abstraction over the `io_uring` submission and completion queues.
//! - `op`: Asynchronous operations (read, write, accept, etc.) that map to `io_uring` SQEs.
//! - `net`/`fs`: High-level wrappers for networking and file system operations.
//! 
//! ## Example: Simple Echo Server
//! ```no_run
//! use ringcore::{run, spawn, TcpListener};
//! 
//! fn main() {
//!     spawn(async {
//!         let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
//!         loop {
//!             let (stream, _) = listener.accept().await.unwrap();
//!             spawn(async move {
//!                 let mut buf = [0u8; 1024];
//!                 let n = stream.read(&mut buf).await.unwrap();
//!                 stream.write(&buf[..n]).await.unwrap();
//!             });
//!         }
//!     });
//!     run();
//! }
//! ```

#![warn(missing_docs)]

pub mod sys;
pub mod ring;
pub mod executor;
pub mod op;
pub mod net;
pub mod fs;

pub use executor::{run, spawn, init, RING};
pub use net::{TcpListener, TcpStream};
pub use fs::File;
