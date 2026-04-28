use ringring::{run, spawn, TcpListener, File};
use std::io;

async fn serve_file(stream: ringring::TcpStream, path: String) -> io::Result<()> {
    let mut file = File::open(path).await?;
    let mut buf = [0u8; 8192];
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 { break; }
        stream.write(&buf[..n]).await?;
    }
    Ok(())
}

fn main() {
    // Create a dummy file to serve
    std::fs::write("index.html", "<h1>Hello from io_uring!</h1>").unwrap();

    spawn(async {
        let listener = TcpListener::bind("127.0.0.1:8081").unwrap();
        println!("File server listening on http://127.0.0.1:8081");
        loop {
            let (stream, _) = listener.accept().await.unwrap();
            spawn(async move {
                if let Err(e) = serve_file(stream, "index.html".to_string()).await {
                    eprintln!("Serve error: {}", e);
                }
            });
        }
    });
    run();
}
