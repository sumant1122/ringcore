use ringcore::{run, spawn, File};
use std::time::Instant;

async fn read_file(id: usize) {
    let path = format!("ring_temp_file_{}.txt", id);
    let mut file = File::open(&path).await.unwrap();
    let mut buf = [0u8; 1024];
    let _n = file.read(&mut buf).await.unwrap();
}

fn main() {
    // Setup 100 files
    for i in 0..100 {
        std::fs::write(format!("ring_temp_file_{}.txt", i), vec![i as u8; 1024]).unwrap();
    }

    let start = Instant::now();
    for i in 0..100 {
        spawn(async move {
            read_file(i).await;
        });
    }

    run();
    println!("RingCore concurrent: 100 ops in {:?}", start.elapsed());

    // Cleanup
    for i in 0..100 {
        let _ = std::fs::remove_file(format!("ring_temp_file_{}.txt", i));
    }
}
