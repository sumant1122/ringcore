use ringcore::{run, spawn, TcpStream};
use std::time::Instant;

async fn run_client(n_requests: usize) {
    let start = Instant::now();
    
    for _ in 0..n_requests {
        let stream = TcpStream::connect("127.0.0.1:8080").await.unwrap();
        let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write(request.as_bytes()).await.unwrap();
        
        let mut buf = [0u8; 1024];
        let _n = stream.read(&mut buf).await.unwrap();
    }
    
    let duration = start.elapsed();
    println!("RingCore client: {} requests in {:?}", n_requests, duration);
}

fn main() {
    spawn(async {
        run_client(100).await;
    });
    run();
}
