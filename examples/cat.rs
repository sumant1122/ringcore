use ringcore::{run, spawn, File};
use std::env;
use std::io;

async fn cat(path: String) -> io::Result<()> {
    let mut file = File::open(path).await?;
    let mut buf = [0u8; 4096];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        // Write to stdout (fd 1) using raw write
        let stdout_fd = 1;
        let mut written = 0;
        while written < n {
            // Use offset 0 for stdout (which is a pipe/char device when redirected)
            let res = ringcore::op::write(stdout_fd, buf[written..n].as_ptr(), (n - written) as u32, 0).await?;
            written += res as usize;
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return;
    }

    let path = args[1].clone();
    spawn(async move {
        if let Err(e) = cat(path).await {
            eprintln!("Error: {}", e);
        }
    });
    run();
}
