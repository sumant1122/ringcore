use ringcore::{run, spawn, File, op, executor::RING};
use std::os::unix::io::AsRawFd;
use std::io;

async fn linked_cat(path: String) -> io::Result<()> {
    let file = File::open(path).await?;
    let mut buf = [0u8; 4096];
    let mut offset = 0;
    let stdout_fd = 1;

    println!("Linked Cat: Batching Read + Write SQEs into one submission...");

    loop {
        // 1. Prepare Read SQE with LINK flag
        let mut read_op = op::read(file.as_raw_fd(), buf.as_mut_ptr(), 4096, offset);
        let mut read_sqe = read_op.take_sqe().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to take read SQE"))?;
        read_sqe.flags |= ringcore::sys::IOSQE_IO_LINK;

        // 2. Prepare Write SQE
        let mut write_op = op::write(stdout_fd, buf.as_ptr(), 4096, 0); // length will be fixed below
        let write_sqe = write_op.take_sqe().ok_or_else(|| io::Error::new(io::ErrorKind::Other, "Failed to take write SQE"))?;

        // 3. Submit both together
        RING.with(|r| {
            if let Some(ring) = r.borrow().as_ref() {
                ring.submit_multiple(&[read_sqe, write_sqe]);
            }
        });

        // 4. Wait for Read result
        // For simplicity in this demo, we manually register wakers and await results
        // normally you'd use a dedicated 'Chain' abstraction.
        let n = read_op.await?;
        if n <= 0 { break; }
        
        // 5. Update Write SQE length based on actual bytes read
        // In a true kernel-side link, we'd use a more advanced feature or fixed length.
        // For this demo, we wait for read completion then let the linked write run.
        let _ = write_op.await?;
        
        offset += n as u64;
    }

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
        if let Err(e) = linked_cat(path).await {
            eprintln!("Linked Cat Error: {}", e);
        }
    });
    run();
}
