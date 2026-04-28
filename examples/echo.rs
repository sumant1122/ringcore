use ringring::{run, spawn, TcpListener};
use std::io;

async fn handle_client(stream: ringring::TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        stream.write(&buf[..n]).await?;
    }
    Ok(())
}

async fn server() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Echo server listening on 127.0.0.1:8080");

    loop {
        let (stream, addr) = listener.accept().await?;
        println!("Accepted connection from {}", addr);
        spawn(async move {
            if let Err(e) = handle_client(stream).await {
                eprintln!("Error handling client: {}", e);
            }
        });
    }
}

fn main() {
    spawn(async {
        if let Err(e) = server().await {
            eprintln!("Server error: {}", e);
        }
    });
    run();
}
