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

fn main() -> io::Result<()> {
    spawn(async {
        let listener = match TcpListener::bind("127.0.0.1:8080") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind: {}", e);
                return;
            }
        };
        println!("HTTP server listening on http://127.0.0.1:8080");
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    spawn(async move {
                        if let Err(e) = handle_http(stream).await {
                            eprintln!("HTTP handle error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    eprintln!("Accept error: {}", e);
                    break;
                }
            }
        }
    });
    run();
    Ok(())
}
