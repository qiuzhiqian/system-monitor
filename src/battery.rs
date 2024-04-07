use std::env::join_paths;

#[derive(Clone, Debug)]
pub struct Battery {
    pub name: String,
    manufacturer: String,
    model: String,
    serial_number: String,
    rtype: String,
}

impl Battery {
    pub fn new(path: &str) -> std::io::Result<Battery> {
        let path = std::path::Path::new(path);
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let manufacturer = crate::utils::read_line(&path.join("manufacturer").to_str().unwrap().to_string())?;
        let model = crate::utils::read_line(&path.join("model_name").to_str().unwrap().to_string())?;
        let serial_number = crate::utils::read_line(&path.join("serial_number").to_str().unwrap().to_string())?;
        let rtype = crate::utils::read_line(&path.join("type").to_str().unwrap().to_string())?;

        Ok(Battery{
            name,
            manufacturer,
            model,
            serial_number,
            rtype,
        })
    }

    pub fn capacity(&self) -> u32 {
        if let Ok(raw_val) = crate::utils::read_line(&format!("/sys/class/power_supply/{}/capacity", self.name)) {
            if let Ok(val) = raw_val.parse::<u32>() {
                return val
            } else {
                return 0;
            }
        }
        return 0
    }

    pub fn voltage_now(&self) -> u32 {
        if let Ok(raw_val) = crate::utils::read_line(&format!("/sys/class/power_supply/{}/voltage_now", self.name)) {
            if let Ok(val) = raw_val.parse::<u32>() {
                return val
            } else {
                return 0;
            }
        }
        return 0
    }

    pub fn current_now(&self) -> u32 {
        if let Ok(raw_val) = crate::utils::read_line(&format!("/sys/class/power_supply/{}/current_now", self.name)) {
            if let Ok(val) = raw_val.parse::<u32>() {
                return val
            } else {
                return 0;
            }
        }
        return 0
    }

    pub fn power_now(&self) -> u32 {
        if let Ok(raw_val) = crate::utils::read_line(&format!("/sys/class/power_supply/{}/power_now", self.name)) {
            if let Ok(val) = raw_val.parse::<u32>() {
                return val
            } else {
                return 0;
            }
        }
        return ( self.voltage_now() / 1000000 ) * (self.current_now() / 1000000);
    }

    pub fn status(&self) -> String {
        if let Ok(raw_val) = crate::utils::read_line(&format!("/sys/class/power_supply/{}/status", self.name)) {
            return raw_val;
        }
        return "".to_string();
    }
}

pub fn enumerate() -> Vec<Battery> {
    fn is_battery(entry: &walkdir::DirEntry) -> bool {
        let realpath = if entry.file_type().is_symlink() {
            let paths = std::fs::read_link(entry.path()).unwrap();
            std::path::Path::new("/sys/class/power_supply").join(paths).canonicalize().unwrap()
        } else if entry.file_type().is_dir() {
            entry.path().to_path_buf()
        } else {
            return false;
        };

        let path = realpath.join("type").to_str().unwrap().to_string();
        if let Ok(name) = crate::utils::read_line(&path) {
            let n = name.trim();
            if n == "Battery" || n == "UPS" {
                return true;
            }
        }

        return false;
    }

    let mut batterys = Vec::new();
    for entry in walkdir::WalkDir::new("/sys/class/power_supply")
            .sort_by_file_name()
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_battery(e)) {
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                if let Ok(battery) = Battery::new(&full_path) {
                    batterys.push(battery);
                }
    }
    
    return batterys
}