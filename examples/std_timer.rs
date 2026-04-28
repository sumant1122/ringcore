use std::time::{Duration, Instant};
use std::thread;

fn main() {
    let start = Instant::now();
    println!("Std Timer started: sleeping for 1 second...");
    thread::sleep(Duration::from_secs(1));
    println!("1 second passed at {:?}", start.elapsed());
    
    println!("Sleeping for 500ms...");
    thread::sleep(Duration::from_millis(500));
    println!("500ms passed. Done at {:?}", start.elapsed());
}
