use ringcore::{run, spawn, TcpListener};
use std::io;

async fn handle_http(stream: ringcore::TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    let n = stream.read(&mut buf).await?;
    let request = String::from_utf8_lossy(&buf[..n]);
    
    if request.contains("\r\n\r\n") {
        let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: close\r\n\r\nHello, World!";
        stream.write(response.as_bytes()).await?;
    }
    Ok(())
}

fn main() {
    spawn(async {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        println!("HTTP server listening on http://127.0.0.1:8080");
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            spawn(async move {
                let _ = handle_http(stream).await;
            });
        }
    });
    run();
}
