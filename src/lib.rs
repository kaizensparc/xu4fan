use std::io::{Read, Seek, Write};

use anyhow::Result;

fn read_temperature(file: &mut std::fs::File) -> Result<f32> {
    let mut buf: Vec<u8> = vec![];
    file.rewind()?;
    file.read_to_end(&mut buf)?;
    std::str::from_utf8(&buf)?
        .trim()
        .parse::<u32>()
        .map_err(|e| anyhow::anyhow!("Cannot read temperature: {:?}", e))
        .map(|i| i as f32 / 1000.0f32)
}

pub struct FanController {
    fan: std::fs::File,
    temperatures: Vec<std::fs::File>,
    high_temp: f32,
    low_temp: f32,
}

impl FanController {
    pub fn new(
        high_temp_deg: f32,
        low_temp_deg: f32,
        fan_path: std::path::PathBuf,
        temperatures: Vec<std::path::PathBuf>,
    ) -> Result<Self> {
        Ok(FanController {
            fan: std::fs::File::options()
                .write(true)
                .read(false)
                .open(fan_path)
                .map_err(|e| anyhow::anyhow!("Could not open fan file: {:?}", e))?,
            temperatures: temperatures
                .into_iter()
                .map(|p| {
                    std::fs::File::options()
                        .write(false)
                        .read(true)
                        .open(p)
                        .map_err(|e| anyhow::anyhow!("Could not open temperature file: {:?}", e))
                })
                .collect::<Result<Vec<std::fs::File>>>()?,
            high_temp: high_temp_deg,
            low_temp: low_temp_deg,
        })
    }

    pub fn get_mean_cpu_temp(&mut self) -> Result<f32> {
        let temps = self
            .temperatures
            .iter_mut()
            .map(|mut f| read_temperature(&mut f))
            .collect::<Result<Vec<f32>>>()?;

        Ok(temps.iter().sum::<f32>() / temps.len() as f32)
    }

    pub fn set_fan_off(&mut self) -> Result<()> {
        self.fan
            .write_all(&vec!['0' as u8])
            .map_err(|e| anyhow::anyhow!("Could not turn off fan: {:?}", e))
    }

    pub fn set_fan_on(&mut self) -> Result<()> {
        self.fan
            .write_all(&vec!['1' as u8])
            .map_err(|e| anyhow::anyhow!("Could not turn on fan: {:?}", e))
    }

    pub fn run_loop_once(&mut self) -> Result<()> {
        let cpu_temp = self.get_mean_cpu_temp()?;

        if cpu_temp > self.high_temp {
            self.set_fan_on()
        } else if cpu_temp < self.low_temp {
            self.set_fan_off()
        } else {
            Ok(())
        }
    }
}
