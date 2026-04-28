use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn handle_client(mut stream: tokio::net::TcpStream) {
    let mut buf = [0u8; 1024];
    if let Ok(n) = stream.read(&mut buf).await {
        let request = String::from_utf8_lossy(&buf[..n]);
        if request.contains("\r\n\r\n") {
            let response = "HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: close\r\n\r\nHello, World!";
            let _ = stream.write_all(response.as_bytes()).await;
        }
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Tokio HTTP server listening on http://127.0.0.1:8080");
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_client(stream));
        }
    }
}
