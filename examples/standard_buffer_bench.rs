use ringcore::{run, spawn, op, File};
use std::os::unix::io::AsRawFd;
use std::time::Instant;

async fn bench_standard(path: String) -> std::io::Result<()> {
    let file = File::open(path).await?;
    let mut buffer = [0u8; 16384]; // 16KB buffer
    
    let start = Instant::now();
    let mut offset = 0;
    let fd = file.as_raw_fd();
    
    loop {
        let n = op::read(fd, buffer.as_mut_ptr(), 16384, offset).await?;
        if n == 0 { break; }
        offset += n as u64;
    }

    println!("RingCore Standard Buffer: Read {} bytes in {:?}", offset, start.elapsed());
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
        bench_standard(path).await.unwrap();
    });
    run();
}
