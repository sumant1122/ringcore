use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let start = Instant::now();
    let n_requests = 100;
    
    for _ in 0..n_requests {
        let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
        let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write_all(request.as_bytes()).await?;
        
        let mut buf = [0u8; 1024];
        let _n = stream.read(&mut buf).await?;
    }
    
    let duration = start.elapsed();
    println!("Tokio client: {} requests in {:?}", n_requests, duration);
    Ok(())
}
