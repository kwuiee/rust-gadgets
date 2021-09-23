extern crate psutil;

use std::env;
use std::error::Error;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

use psutil::process::Process;

fn main() -> Result<(), Box<dyn Error>> {
    let pid: u32 = match env::args().nth(1) {
        Some(v) => v.parse()?,
        None => {
            eprintln!("Error: Required a pid argument.");
            process::exit(1);
        }
    };
    let p = Process::new(pid)?;
    let instant = Instant::now();
    while let Ok(v) = p.memory_info() {
        println!(
            "{}: rss {} bytes; vms: {} bytes.",
            instant.elapsed().as_secs_f32(),
            v.rss(),
            v.vms(),
        );
        thread::sleep(Duration::from_secs(5));
    }
    Ok(())
}
