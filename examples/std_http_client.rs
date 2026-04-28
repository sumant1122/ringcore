use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Instant;

fn main() {
    let start = Instant::now();
    let n_requests = 100;
    
    for _ in 0..n_requests {
        let mut stream = TcpStream::connect("127.0.0.1:8080").unwrap();
        let request = "GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n";
        stream.write_all(request.as_bytes()).unwrap();
        
        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();
    }
    
    let duration = start.elapsed();
    println!("Standard client: {} requests in {:?}", n_requests, duration);
}
