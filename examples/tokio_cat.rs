use tokio::fs::File;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use std::env;

async fn cat(path: String) -> io::Result<()> {
    let mut file = File::open(path).await?;
    let mut buf = [0u8; 4096];
    let mut stdout = io::stdout();
    loop {
        let n = file.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        stdout.write_all(&buf[..n]).await?;
    }
    stdout.flush().await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <file>", args[0]);
        return;
    }

    let path = args[1].clone();
    if let Err(e) = cat(path).await {
        eprintln!("Error: {}", e);
    }
}
