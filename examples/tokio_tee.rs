use tokio::fs::File;
use tokio::io::{self, AsyncWriteExt};
use std::env;

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: {} <file1> <file2>", args[0]);
        return Ok(());
    }

    let mut f1 = File::create(&args[1]).await?;
    let mut f2 = File::create(&args[2]).await?;
    
    let mut buf = [0u8; 4096];
    let _stdin = io::stdin();
    
    loop {
        // Standard stdin isn't easily async without blocking or special crates,
        // but for this demo we'll use a simple read loop.
        let n = match tokio::task::spawn_blocking(move || {
            let mut stdin = std::io::stdin();
            let mut buf = [0u8; 4096];
            let n = std::io::Read::read(&mut stdin, &mut buf);
            (n, buf)
        }).await.map_err(|e| io::Error::new(io::ErrorKind::Other, e))? {
            (Ok(0), _) => break,
            (Ok(n), b) => {
                buf[..n].copy_from_slice(&b[..n]);
                n
            }
            (Err(e), _) => return Err(e),
        };

        // Write to both concurrently
        let w1 = f1.write_all(&buf[..n]);
        let w2 = f2.write_all(&buf[..n]);
        tokio::try_join!(w1, w2)?;
    }
    
    Ok(())
}
