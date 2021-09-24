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
    sys.refresh_all();
    let totalnt = sys.physical_core_count().unwrap_or(0);
    let totalmem = sys.total_memory();
    loop {
        println!(
            "cpu: {:.2}%*{}nt, memory: {}/{}KB.",
            sys.global_processor_info().cpu_usage(),
            totalnt,
            sys.used_memory(),
            totalmem,
        );
        thread::sleep(Duration::from_secs(5));
        sys.refresh_all();
    }
}
