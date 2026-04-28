use ringring::{run, spawn, File, op};
use std::io;
use std::rc::Rc;
use std::cell::Cell;
use std::os::unix::io::AsRawFd;

async fn tee(file1_path: String, file2_path: String) -> io::Result<()> {
    let f1 = Rc::new(File::create(file1_path).await?);
    let f2 = Rc::new(File::create(file2_path).await?);
    
    let stdin_fd = 0;
    let mut buf = [0u8; 4096];
    
    loop {
        let n = op::read(stdin_fd, buf.as_mut_ptr(), 4096, 0).await?;
        if n == 0 {
            break;
        }
        
        let done_count = Rc::new(Cell::new(0));
        
        let c1 = done_count.clone();
        let fd1 = f1.as_raw_fd();
        let b1 = buf.as_ptr();
        spawn(async move {
            let _ = op::write(fd1, b1, n as u32, 0).await;
            c1.set(c1.get() + 1);
        });

        let c2 = done_count.clone();
        let fd2 = f2.as_raw_fd();
        let b2 = buf.as_ptr();
        spawn(async move {
            let _ = op::write(fd2, b2, n as u32, 0).await;
            c2.set(c2.get() + 1);
        });

        // Wait for both to finish (busy wait for simplicity in this demo, 
        // or we could use a better sync primitive if we had one)
        while done_count.get() < 2 {
            // Yield to executor
            op::timeout(&mut ringring::sys::__kernel_timespec { tv_sec: 0, tv_nsec: 1000 }).await.unwrap();
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        return;
    }
    let f1 = args[1].clone();
    let f2 = args[2].clone();
    spawn(async move {
        if let Err(e) = tee(f1, f2).await {
            eprintln!("Tee error: {}", e);
        }
    });
    run();
}
