use ringcore::{run, spawn, init, op, executor::RING};
use std::io;

async fn run_fixed_buffer_demo() -> io::Result<()> {
    // 1. Initialize runtime
    init(256, 0);

    // 2. Prepare a buffer
    let mut buffer = [0u8; 4096];
    let iovecs = [libc::iovec {
        iov_base: buffer.as_mut_ptr() as *mut _,
        iov_len: buffer.len(),
    }];

    // 3. Register the buffer with the kernel
    println!("Registering 4KB buffer with kernel...");
    RING.with(|r| r.borrow().register_buffers(iovecs.as_ptr(), 1))?;

    // 4. Use it to read from stdin (index 0)
    println!("Type something (using fixed buffer read):");
    let stdin_fd = 0;
    let n = op::read_fixed(stdin_fd, buffer.as_mut_ptr(), 4096, 0, 0).await?;
    
    println!("Read {} bytes into fixed buffer.", n);
    
    // 5. Use it to write to stdout
    println!("Echoing back (using fixed buffer write):");
    let stdout_fd = 1;
    let _ = op::write_fixed(stdout_fd, buffer.as_ptr(), n as u32, 0, 0).await?;

    Ok(())
}

fn main() {
    spawn(async {
        if let Err(e) = run_fixed_buffer_demo().await {
            eprintln!("Fixed buffer error: {}", e);
        }
    });
    run();
}
