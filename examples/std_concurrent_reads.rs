use std::fs;
use std::thread;
use std::time::Instant;
use std::io::Read;

fn read_file(id: usize) {
    let path = format!("temp_file_{}.txt", id);
    let mut file = fs::File::open(&path).unwrap();
    let mut buf = [0u8; 1024];
    file.read_exact(&mut buf).unwrap();
}

fn main() {
    // Setup 100 files
    for i in 0..100 {
        fs::write(format!("temp_file_{}.txt", i), vec![i as u8; 1024]).unwrap();
    }

    let start = Instant::now();
    let mut handles = vec![];
    for i in 0..100 {
        handles.push(thread::spawn(move || {
            read_file(i);
        }));
    }

    for handle in handles {
        handle.join().unwrap();
    }

    println!("Standard concurrent: 100 threads in {:?}", start.elapsed());

    // Cleanup
    for i in 0..100 {
        let _ = fs::remove_file(format!("temp_file_{}.txt", i));
    }
}
