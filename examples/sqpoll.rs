use ringring::{run, spawn, init, TcpListener, sys};
use std::io;

async fn handle_client(stream: ringring::TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 { break; }
        stream.write(&buf[..n]).await?;
    }
    Ok(())
}

fn main() {
    // Initialize with SQPOLL
    println!("Initializing with IORING_SETUP_SQPOLL...");
    init(256, sys::IORING_SETUP_SQPOLL);

    spawn(async {
        let listener = TcpListener::bind("127.0.0.1:8082").unwrap();
        println!("SQPOLL Echo server listening on 127.0.0.1:8082");
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            spawn(async move {
                let _ = handle_client(stream).await;
            });
        }
    });
    run();
}
