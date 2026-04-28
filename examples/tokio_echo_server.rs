use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

async fn handle_client(mut stream: tokio::net::TcpStream) {
    let mut buf = [0u8; 1024];
    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                if stream.write_all(&buf[..n]).await.is_err() {
                    break;
                }
            }
            Err(_) => break,
        }
    }
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
    println!("Tokio Echo server listening on 127.0.0.1:8080");
    loop {
        if let Ok((stream, _)) = listener.accept().await {
            tokio::spawn(handle_client(stream));
        }
    }
}
