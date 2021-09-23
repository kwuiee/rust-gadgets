extern crate sysinfo;

use std::error::Error;
use std::thread;
use std::time::Duration;

use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};

fn main() -> Result<(), Box<dyn Error>> {
    let refresh = RefreshKind::new();
    refresh.with_cpu();
    refresh.with_memory();
    let mut sys = System::new_with_specifics(refresh);
    loop {
        sys.refresh_all();
        println!(
            "cpu: {}%, memory: {}KB.",
            sys.global_processor_info().cpu_usage(),
            sys.used_memory(),
        );
        thread::sleep(Duration::from_secs(5));
    }
}
