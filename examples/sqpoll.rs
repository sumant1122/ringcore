use ringcore::{run, spawn, init, TcpListener, sys};
use std::io;

async fn handle_client(stream: ringcore::TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 { break; }
        stream.write(&buf[..n]).await?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Initialize with SQPOLL
    println!("Initializing with IORING_SETUP_SQPOLL...");
    init(256, sys::IORING_SETUP_SQPOLL)?;

    spawn(async {
        let listener = match TcpListener::bind("127.0.0.1:8082") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind: {}", e);
                return;
            }
        };
        println!("SQPOLL Echo server listening on 127.0.0.1:8082");
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    spawn(async move {
                        if let Err(e) = handle_client(stream).await {
                            eprintln!("Echo handle error: {}", e);
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
