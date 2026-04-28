use tokio::fs::File;
use tokio::io::AsyncWriteExt;
use std::time::Instant;

#[tokio::main]
async fn main() {
    let mut file = File::create("tokio_app.log").await.unwrap();
    
    let line1 = "INFO: Starting application\n";
    let line2 = "DEBUG: Initializing ringcore runtime\n";
    let line3 = "WARN: No config file found, using defaults\n";
    
    let start = Instant::now();
    for _ in 0..1000 {
        // Tokio doesn't have a direct equivalent to writev that is as efficient as io_uring's
        // write_vectored on tokio File usually just calls a thread pool.
        file.write_all(line1.as_bytes()).await.unwrap();
        file.write_all(line2.as_bytes()).await.unwrap();
        file.write_all(line3.as_bytes()).await.unwrap();
    }
    file.flush().await.unwrap();
    
    println!("Tokio logger: 1000 batches in {:?}", start.elapsed());
}
