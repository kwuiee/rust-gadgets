extern crate env_logger;
extern crate humantime;
extern crate log;
extern crate sysinfo;

use std::error::Error;
use std::io::Write;
use std::thread;
use std::time::{Duration, SystemTime};

use env_logger::Env;
use log::warn;
use sysinfo::{ProcessorExt, RefreshKind, System, SystemExt};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(Env::default().default_filter_or("warn"))
        .format(|buf, record| {
            writeln!(
                buf,
                "{} - {}",
                humantime::format_rfc3339_seconds(
                    SystemTime::now() + Duration::from_secs(8 * 3600)
                ),
                record.args()
            )
        })
        .init();
    let refresh = RefreshKind::new();
    refresh.with_cpu();
    refresh.with_memory();
    let mut sys = System::new_with_specifics(refresh);
    sys.refresh_all();
    let totalcpu = sys.processors().len();
    let totalmem = sys.total_memory();
    loop {
        warn!(
            "cpu: {:.2}%*{}, memory: {}/{}KB.",
            sys.global_processor_info().cpu_usage(),
            totalcpu,
            sys.used_memory(),
            totalmem,
        );
        thread::sleep(Duration::from_secs(5));
        sys.refresh_all();
    }
}
