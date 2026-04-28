use tokio::time::{sleep, Duration, Instant};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    println!("Tokio Timer started: sleeping for 1 second...");
    sleep(Duration::from_secs(1)).await;
    println!("1 second passed at {:?}", start.elapsed());
    
    println!("Sleeping for 500ms...");
    sleep(Duration::from_millis(500)).await;
    println!("500ms passed. Done at {:?}", start.elapsed());
}
