use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

fn handle_client(mut stream: std::net::TcpStream) {
    let mut buf = [0u8; 1024];
    if let Ok(n) = stream.read(&mut buf) {
        let request = String::from_utf8_lossy(&buf[..n]);
        if request.contains("\r\n\r\n") {
            let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: close\r\n\r\nHello, World!";
            let _ = stream.write_all(response.as_bytes());
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
    println!("Standard HTTP server listening on http://127.0.0.1:8080");
    for stream in listener.incoming() {
        if let Ok(stream) = stream {
            thread::spawn(|| {
                handle_client(stream);
            });
        }
    }
}
