use ringcore::{run, spawn, init, op, executor::RING, File};
use std::os::unix::io::AsRawFd;
use std::time::Instant;

async fn bench_fixed(path: String) -> std::io::Result<()> {
    let file = File::open(path).await?;
    let mut buffer = [0u8; 16384]; // 16KB buffer
    let iovecs = [libc::iovec {
        iov_base: buffer.as_mut_ptr() as *mut _,
        iov_len: buffer.len(),
    }];

    // Register buffer
    RING.with(|r| r.borrow().register_buffers(iovecs.as_ptr(), 1))?;

    let start = Instant::now();
    let mut offset = 0;
    let fd = file.as_raw_fd();
    
    loop {
        let n = op::read_fixed(fd, buffer.as_mut_ptr(), 16384, offset, 0).await?;
        if n == 0 { break; }
        offset += n as u64;
    }

    println!("RingCore Fixed Buffer: Read {} bytes in {:?}", offset, start.elapsed());
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return;
    }
    let path = args[1].clone();

    spawn(async move {
        bench_fixed(path).await.unwrap();
    });
    run();
}
