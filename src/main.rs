use anyhow::Result;
use clap::Parser;
use xu4fan::FanController;

/// Simple program to control the CPU fan of the odroid XU4
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Hysteresis higher temperature
    #[clap(short, long, value_parser, default_value_t = 60.0)]
    high_temp: f32,

    /// Hysteresis lower temperature
    #[clap(short, long, value_parser, default_value_t = 50.0)]
    low_temp: f32,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut fc = FanController::new(
        args.high_temp,
        args.low_temp,
        std::path::Path::new("/sys/devices/virtual/thermal/cooling_device2/cur_state")
            .to_path_buf(),
        glob::glob("/sys/devices/virtual/thermal/thermal_zone*/temp")?
            .map(|x| {
                x.map(|p| p.to_path_buf())
                    .map_err(|e| anyhow::anyhow!("Cannot open path: {:?}", e))
            })
            .collect::<Result<Vec<std::path::PathBuf>>>()?,
    )?;

    loop {
        println!("It's currently {:?}C here", fc.get_mean_cpu_temp()?);
        fc.run_loop_once()?;
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
}
