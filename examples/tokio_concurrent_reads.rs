use tokio::fs;
use tokio::io::AsyncReadExt;
use std::time::Instant;

async fn read_file(id: usize) {
    let path = format!("tokio_temp_file_{}.txt", id);
    let mut file = fs::File::open(&path).await.unwrap();
    let mut buf = [0u8; 1024];
    file.read_exact(&mut buf).await.unwrap();
}

#[tokio::main]
async fn main() {
    // Setup 100 files
    for i in 0..100 {
        std::fs::write(format!("tokio_temp_file_{}.txt", i), vec![i as u8; 1024]).unwrap();
    }

    let start = Instant::now();
    let mut tasks = vec![];
    for i in 0..100 {
        tasks.push(tokio::spawn(read_file(i)));
    }

    for task in tasks {
        task.await.unwrap();
    }

    println!("Tokio concurrent: 100 tasks in {:?}", start.elapsed());

    // Cleanup
    for i in 0..100 {
        let _ = std::fs::remove_file(format!("tokio_temp_file_{}.txt", i));
    }
}
