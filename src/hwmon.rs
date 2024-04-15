// /sys/class/hwmon/hwmon
use std::collections::HashMap;

static ROOTPATH: &str = "/sys/class/hwmon";

#[derive(Clone, Debug)]
pub struct Hwmon {
    pub node: String,
    pub name: String,
}

impl Hwmon {
    pub fn new(path: &str) -> std::io::Result<Hwmon> {
        let path = std::path::Path::new(path);
        let node = path.file_name().unwrap().to_str().unwrap().to_string();
        let name = crate::utils::read_line(&path.join("name").to_str().unwrap().to_string())?;

        Ok(Hwmon{
            node,
            name,
        })
    }

    pub fn fans(&self) -> HashMap<String, u32> {
        let mut node_vals :HashMap<String, u32>= HashMap::new();
        for entry in walkdir::WalkDir::new(format!("{}/{}", ROOTPATH, self.node))
                .sort_by_file_name()
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e|  (e.file_type().is_file() || e.file_type().is_symlink())) {
                    let re = regex::Regex::new(r"^fan\d{1,3}_input$").unwrap();
                    if re.is_match(entry.path().file_name().unwrap().to_str().unwrap()) {
                        if let Ok(raw_val) = crate::utils::read_line(entry.path().to_str().unwrap()) {
                            if let Ok(val) = raw_val.parse::<u32>() {
                                let key = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                                node_vals.insert(key, val);
                            }
                        }
                    }
        }

        node_vals
    }

    pub fn temps(&self) -> HashMap<String, u32> {
        let mut node_vals: HashMap<String, u32> = HashMap::new();
        for entry in walkdir::WalkDir::new(format!("{}/{}", ROOTPATH, self.node))
                .sort_by_file_name()
                .max_depth(1)
                .into_iter()
                .filter_map(Result::ok)
                .filter(|e|  (e.file_type().is_file() || e.file_type().is_symlink())) {
                    //println!("path: {:#?}", entry.path());
                    let re = regex::Regex::new(r"^temp(?<num>\d{1,3})_input$").unwrap();
                    if let Some(caps) = re.captures(entry.path().file_name().unwrap().to_str().unwrap()) {
                        let num = caps["num"].to_string();

                        
                        let val = if let Ok(raw_val) = crate::utils::read_line(entry.path().to_str().unwrap()) {
                            if let Ok(val) = raw_val.parse::<u32>() {
                                val
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                        let key = if let Ok(val) = crate::utils::read_line(&format!("{}/{}/temp{}_label", ROOTPATH, self.node, num)) {
                            val
                        } else {
                            format!("temp{}_label", num)
                        };
                        node_vals.insert(key, val);
                    }
        }

        node_vals
    }
}

pub fn enumerate() -> Vec<Hwmon> {
    let mut hwmons = Vec::new();
    for entry in walkdir::WalkDir::new(ROOTPATH)
            .sort_by_file_name()
            .max_depth(1) 
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink())){
                if entry.path().to_str().unwrap() != ROOTPATH {
                    //println!("node: {:#?}", entry.path());
                    let re = regex::Regex::new(r"^hwmon\d{1,3}$").unwrap();
                    if re.is_match(entry.path().file_name().unwrap().to_str().unwrap().trim()) {
                        if let Ok(hwmon) = Hwmon::new(entry.path().to_str().unwrap().trim()) {
                            hwmons.push(hwmon)
                        }
                    }
                }
    }
    
    hwmons
}