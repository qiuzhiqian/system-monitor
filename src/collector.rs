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
    pub fn update(&mut self) -> std::io::Result<()>{
        let mut record = vec![1234.to_string()];
        if self.cpus.len() > 0 {
            for cpu in &self.cpus {
                record.push(cpu.freq().to_string());
            }
        }
        self.writer.write_record(record)?;
        self.writer.flush()
    }
}