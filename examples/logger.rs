use ringcore::{run, spawn, File, op};
use std::os::unix::io::AsRawFd;
use std::time::Instant;

async fn logger(n: usize) -> std::io::Result<()> {
    let file = File::create("app.log").await?;
    
    let line1 = "INFO: Starting application\n";
    let line2 = "DEBUG: Initializing ringcore runtime\n";
    let line3 = "WARN: No config file found, using defaults\n";
    
    let iovecs = [
        libc::iovec { iov_base: line1.as_ptr() as *mut _, iov_len: line1.len() },
        libc::iovec { iov_base: line2.as_ptr() as *mut _, iov_len: line2.len() },
        libc::iovec { iov_base: line3.as_ptr() as *mut _, iov_len: line3.len() },
    ];
    
    println!("Writing {} batches of logs using writev...", n);
    let start = Instant::now();
    for _ in 0..n {
        let _ = op::writev(file.as_raw_fd(), iovecs.as_ptr(), 3, 0).await?;
    }
    println!("RingCore logger: {} batches in {:?}", n, start.elapsed());
    
    Ok(())
}

fn main() {
    spawn(async {
        if let Err(e) = logger(1000).await {
            eprintln!("Logger error: {}", e);
        }
    });
    run();
}
