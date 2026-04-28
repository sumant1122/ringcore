use ringcore::{run, spawn, File};
use std::time::Instant;
use std::io;

async fn read_file(id: usize) -> io::Result<()> {
    let path = format!("ring_temp_file_{}.txt", id);
    let mut file = File::open(&path).await?;
    let mut buf = [0u8; 1024];
    let _n = file.read(&mut buf).await?;
    Ok(())
}

fn main() -> io::Result<()> {
    // Setup 100 files
    for i in 0..100 {
        std::fs::write(format!("ring_temp_file_{}.txt", i), vec![i as u8; 1024])?;
    }

    let start = Instant::now();
    for i in 0..100 {
        spawn(async move {
            if let Err(e) = read_file(i).await {
                eprintln!("Error reading file {}: {}", i, e);
            }
        });
    }

    run();
    println!("RingCore concurrent: 100 ops in {:?}", start.elapsed());

    // Cleanup
    for i in 0..100 {
        let _ = std::fs::remove_file(format!("ring_temp_file_{}.txt", i));
    }
    
    Ok(())
}
