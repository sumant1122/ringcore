use ringcore::{run, spawn, TcpStream};
use std::time::Instant;
use std::io;

async fn run_client(n_requests: usize) -> io::Result<()> {
    let start = Instant::now();
    
    for _ in 0..n_requests {
        let stream = TcpStream::connect("127.0.0.1:8080").await?;
        let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write(request.as_bytes()).await?;
        
        let mut buf = [0u8; 1024];
        let _n = stream.read(&mut buf).await?;
    }
    
    let duration = start.elapsed();
    println!("RingCore client: {} requests in {:?}", n_requests, duration);
    Ok(())
}

fn main() -> io::Result<()> {
    spawn(async {
        if let Err(e) = run_client(100).await {
            eprintln!("Client error: {}", e);
        }
    });
    run();
    Ok(())
}
