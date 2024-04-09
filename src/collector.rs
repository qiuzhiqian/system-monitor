pub trait Collector {
    fn update(&mut self) -> std::io::Result<()>;
    fn need_stop(&self) -> bool;
}

pub struct CpuCollector {
    cpus: Vec<crate::cpu::CPU>,
    writer: csv::Writer<std::fs::File>,
}

impl CpuCollector {
    pub fn new(cpus: Vec<crate::cpu::CPU>,file: &str) -> std::io::Result<Self> {
        let mut writer = csv::Writer::from_path(file)?;
        let mut header = vec!["timestamp".to_string()];
        for cpu in &cpus {
            header.push(cpu.tag().clone());
        }
        writer.write_record(header)?;
        Ok(Self { cpus, writer })
    }
}

impl Collector for CpuCollector {
    fn update(&mut self) -> std::io::Result<()>{
        let since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");

        // 以秒为单位
        let timestamp = since_epoch.as_secs();

        let mut record = vec![timestamp.to_string()];
        if self.cpus.len() > 0 {
            for cpu in &self.cpus {
                record.push(cpu.freq().to_string());
            }
        }
        self.writer.write_record(record)?;
        self.writer.flush()
    }

    fn need_stop(&self) -> bool {
        false
    }
}

pub struct CapacityCollector {
    batterys: Vec<crate::battery::Battery>,
    writer: csv::Writer<std::fs::File>,
    last_capacity: u32,
}

impl CapacityCollector {
    pub fn new(batterys: Vec<crate::battery::Battery>,file: &str) -> std::io::Result<Self> {
        let mut writer = csv::Writer::from_path(file)?;
        let mut header = vec!["timestamp".to_string()];
        for battery in &batterys {
            header.push(battery.name.clone());
        }
        writer.write_record(header)?;
        Ok(Self { batterys, writer, last_capacity:0 })
    }
}

impl Collector for CapacityCollector {
    fn update(&mut self) -> std::io::Result<()>{
        let since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");

        // 以秒为单位
        let timestamp = since_epoch.as_secs();

        let mut record = vec![timestamp.to_string()];
        if self.batterys.len() > 0 {
            for battery in &self.batterys {
                record.push(battery.capacity().to_string());
                self.last_capacity = battery.capacity();
            }
        }
        self.writer.write_record(record)?;
        self.writer.flush()
    }

    fn need_stop(&self) -> bool {
        self.last_capacity <= 5
    }
}

pub struct PowerCollector {
    batterys: Vec<crate::battery::Battery>,
    writer: csv::Writer<std::fs::File>,
}

impl PowerCollector {
    pub fn new(batterys: Vec<crate::battery::Battery>,file: &str) -> std::io::Result<Self> {
        let mut writer = csv::Writer::from_path(file)?;
        let mut header = vec!["timestamp".to_string()];
        for battery in &batterys {
            header.push(battery.name.clone());
        }
        writer.write_record(header)?;
        Ok(Self { batterys, writer })
    }
}

impl Collector for PowerCollector {
    fn update(&mut self) -> std::io::Result<()>{
        let since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");

        // 以秒为单位
        let timestamp = since_epoch.as_secs();

        let mut record = vec![timestamp.to_string()];
        if self.batterys.len() > 0 {
            for battery in &self.batterys {
                record.push(battery.power_now().to_string());
            }
        }
        self.writer.write_record(record)?;
        self.writer.flush()
    }

    fn need_stop(&self) -> bool {
        false
    }
}

pub struct ThermalCollector {
    thermals: Vec<crate::thermal::Thermal>,
    writer: csv::Writer<std::fs::File>,
}

impl ThermalCollector {
    pub fn new(thermals: Vec<crate::thermal::Thermal>,file: &str) -> std::io::Result<Self> {
        let mut writer = csv::Writer::from_path(file)?;
        let mut header = vec!["timestamp".to_string()];
        for thermal in &thermals {
            header.push(thermal.name.clone());
        }
        writer.write_record(header)?;
        Ok(Self { thermals, writer })
    }
}

impl Collector for ThermalCollector {
    fn update(&mut self) -> std::io::Result<()>{
        let since_epoch = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
            .expect("Time went backwards");

        // 以秒为单位
        let timestamp = since_epoch.as_secs();

        let mut record = vec![timestamp.to_string()];
        if self.thermals.len() > 0 {
            for thermal in &self.thermals {
                record.push(thermal.temp().to_string());
            }
        }
        self.writer.write_record(record)?;
        self.writer.flush()
    }

    fn need_stop(&self) -> bool {
        false
    }
}