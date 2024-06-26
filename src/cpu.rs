
use std::fs::File;
use std::io::BufRead;

static ROOTPATH: &str = "/sys/devices/system/cpu";

#[derive(Clone, Debug)]
pub struct CPU {
    number: u32,
    vendor: String,
    family: String,
    model: String,
    core_id: u32,
    physical_package_id: u32,
    scaling_driver: String,
    scaling_governor: String,
    scaling_min_freq: u32,
    scaling_max_freq: u32,
}

impl CPU {
    pub fn default() -> CPU {
        CPU{number: 0,vendor: "".to_string(),
            family: "".to_string(),
            model: "".to_string(),
            core_id: 0,
            physical_package_id: 0,
            scaling_driver: "".to_string(),
            scaling_governor: "".to_string(), 
            scaling_min_freq: 0, 
            scaling_max_freq: 0}
    }
    pub fn tag(&self) -> String {
        return format!("{}:{}:{}", self.physical_package_id, self.core_id, self.number).to_string();
    }

    pub fn freq(&self) -> u32 {
        if let Ok(freq_val) = crate::utils::read_line(&format!("{}/cpu{}/cpufreq/scaling_cur_freq", ROOTPATH, self.number)) {
            if let Ok(val) = freq_val.parse::<u32>() {
                return val
            } else {
                return 0;
            }
        }
        return 0
    }
}

pub fn new_cpu(number: u32, vendor: &str, family: &str, model: &str) -> std::io::Result<CPU> {
    let core_str = crate::utils::read_line(&format!("{}/cpu{}/topology/core_id", ROOTPATH, number))?;
    let core_id = core_str.parse::<u32>().map_err(|e| {std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())})?;

    let physical_package = crate::utils::read_line(&format!("{}/cpu{}/topology/physical_package_id", ROOTPATH, number))?;
    let physical_package_id = physical_package.parse::<u32>().map_err(|e| {std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())})?;

    let scaling_driver = crate::utils::read_line(&format!("{}/cpu{}/cpufreq/scaling_driver", ROOTPATH, number))?;
    let scaling_governor = crate::utils::read_line(&format!("{}/cpu{}/cpufreq/scaling_governor", ROOTPATH, number))?;

    let scaling_min_freq_str = crate::utils::read_line(&format!("{}/cpu{}/cpufreq/scaling_min_freq", ROOTPATH, number))?;
    let scaling_min_freq = scaling_min_freq_str.parse::<u32>().map_err(|e| {std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())})?;

    let scaling_max_freq_str = crate::utils::read_line(&format!("{}/cpu{}/cpufreq/scaling_max_freq", ROOTPATH, number))?;
    let scaling_max_freq = scaling_max_freq_str.parse::<u32>().map_err(|e| {std::io::Error::new(std::io::ErrorKind::InvalidData, e.to_string())})?;

    Ok(CPU{
        number,
        vendor: vendor.to_string(),
        family: family.to_string(),
        model: model.to_string(),
        core_id,
        physical_package_id,
        scaling_driver,
        scaling_governor,
        scaling_min_freq,
        scaling_max_freq,
    })
}

pub fn enumerate() -> Vec<CPU> {
    let mut cpus = Vec::<CPU>::new();
    if let Ok(input) = File::open("/proc/cpuinfo") {
        let reader = std::io::BufReader::new(input);

        //let mut cpu = CPU::default();
        let mut vendor = String::new();
        let mut number = 0;
        let mut family = String::new();
        let mut model = String::new();
        let mut found = false;
        for line in reader.lines() {
            let line = line.unwrap().trim().to_string();
            
            if line.starts_with("vendor_id\t") {
                //
                let v: Vec<&str> = line.split(":").collect();
                if v.len() > 1 {
                    vendor = v[1].trim().to_string();
                }
            } else if line.starts_with("processor\t") {
                let v: Vec<&str> = line.split(":").collect();
                if v.len() > 1 {
                    number = v[1].trim().parse::<u32>().unwrap();
                    found = true;
                }
            } else if line.starts_with("cpu family\t") {
                let v: Vec<&str> = line.split(":").collect();
                if v.len() > 1 {
                    family = v[1].trim().to_string();
                }

            } else if line.starts_with("model\t") {
                let v: Vec<&str> = line.split(":").collect();
                if v.len() > 1 {
                    model = v[1].trim().to_string();
                }
            }

            if line.is_empty() && found {
                if let Ok(cpu) = new_cpu(number, &vendor, &family, &model) {
                    cpus.push(cpu);
                }
                found = false;
            }
        }

        if found {
            if let Ok(cpu) = new_cpu(number, &vendor, &family, &model) {
                cpus.push(cpu);
            }
            found = false;
        }
    }

    return cpus;
}