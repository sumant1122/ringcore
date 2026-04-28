use ringcore::{run, spawn, op, TcpListener, TcpStream};
use std::io;
use std::os::unix::io::AsRawFd;

async fn handle_client(stream: TcpStream) -> io::Result<()> {
    let mut buf = [0u8; 1024];
    let _n = stream.read(&mut buf).await?;
    let reply = format!("HTTP/1.1 200 OK\r\nContent-Length: 18\r\n\r\nMultishot worked!\n");
    stream.write(reply.as_bytes()).await?;
    Ok(())
}

async fn multishot_server() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8083")?;
    println!("Multishot Accept server listening on 127.0.0.1:8083");
    
    let mut accept_op = op::accept_multishot(listener.as_raw_fd());
    
    loop {
        // Each time we await accept_op, it gives us one new client from the CQE queue
        let client_fd = (&mut accept_op).await?;
        println!("New client fd: {}", client_fd);
        
        let stream = TcpStream::from_raw_fd(client_fd);
        
        spawn(async move {
            let _ = handle_client(stream).await;
        });
    }
}

fn main() {
    spawn(async {
        if let Err(e) = multishot_server().await {
            eprintln!("Multishot server error: {}", e);
        }
    });
    run();
}
