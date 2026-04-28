use std::fs::File;
use std::io::{Write, IoSlice};
use std::time::Instant;

fn main() {
    let file = File::create("std_app.log").unwrap();
    let mut writer = std::io::BufWriter::new(file);
    
    let line1 = "INFO: Starting application\n";
    let line2 = "DEBUG: Initializing ringring runtime\n";
    let line3 = "WARN: No config file found, using defaults\n";
    
    let start = Instant::now();
    for _ in 0..1000 {
        let slices = [
            IoSlice::new(line1.as_bytes()),
            IoSlice::new(line2.as_bytes()),
            IoSlice::new(line3.as_bytes()),
        ];
        writer.get_ref().write_vectored(&slices).unwrap();
    }
    
    println!("Standard logger: 1000 batches in {:?}", start.elapsed());
}
