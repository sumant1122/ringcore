use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Instant;

async fn make_requests(n: usize) {
    for _ in 0..n {
        if let Ok(mut stream) = TcpStream::connect("127.0.0.1:8080").await {
            let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
            if stream.write_all(request.as_bytes()).await.is_ok() {
                let mut buf = [0u8; 1024];
                let _ = stream.read(&mut buf).await;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let n_tasks = 200;
    let reqs_per_task = 5;

    println!("Tokio Stress Client: Spawning {} concurrent tasks ({} total requests)...", n_tasks, n_tasks * reqs_per_task);
    
    let start = Instant::now();
    let mut handles = vec![];
    for _ in 0..n_tasks {
        handles.push(tokio::spawn(make_requests(reqs_per_task)));
    }

    for h in handles {
        let _ = h.await;
    }
    
    let duration = start.elapsed();
    println!("Tokio Stress Client: Finished in {:?}", duration);
}
