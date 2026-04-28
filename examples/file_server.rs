use ringcore::{run, spawn, TcpListener, File};
use std::io;

async fn serve_file(stream: ringcore::TcpStream, path: String) -> io::Result<()> {
    let mut file = File::open(path).await?;
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 { break; }
        stream.write(&buf[..n]).await?;
    }
    Ok(())
}

fn main() -> io::Result<()> {
    // Create a dummy file to serve
    std::fs::write("index.html", "<h1>Hello from io_uring!</h1>")?;

    spawn(async {
        let listener = match TcpListener::bind("127.0.0.1:8081") {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Failed to bind: {}", e);
                return;
            }
        };
        println!("File server listening on http://127.0.0.1:8081");
        loop {
            match listener.accept().await {
                Ok((stream, _)) => {
                    spawn(async move {
                        if let Err(e) = serve_file(stream, "index.html".to_string()).await {
                            eprintln!("Serve error: {}", e);
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
