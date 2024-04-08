#[derive(Clone, Debug)]
pub struct Thermal {
    pub name: String,
    rtype: String,
    mode: String,
    available_policies: String,
    policy: String,
}

impl Thermal {
    pub fn new(path: &str) -> std::io::Result<Self> {
        let path = std::path::Path::new(path);
        let name = path.file_name().unwrap().to_str().unwrap().to_string();
        let rtype = crate::utils::read_line(&path.join("type").to_str().unwrap().to_string())?;
        let mode = if let Ok(v) = crate::utils::read_line(&path.join("mode").to_str().unwrap().to_string()) {
            v
        } else {
            "".to_string()
        };
        let available_policies = crate::utils::read_line(&path.join("available_policies").to_str().unwrap().to_string())?;
        let policy = crate::utils::read_line(&path.join("policy").to_str().unwrap().to_string())?;

        Ok(Thermal{
            name,
            rtype,
            mode,
            available_policies,
            policy,
        })
    }
    pub fn temp(&self) -> u32 {
        if let Ok(raw_val) = crate::utils::read_line(&format!("/sys/class/thermal/{}/temp", self.name)) {
            if let Ok(val) = raw_val.parse::<u32>() {
                return val
            } else {
                return 0;
            }
        }
        return 0;
    }
}

pub fn enumerate() -> Vec<Thermal> {
    fn is_thermal(entry: &walkdir::DirEntry) -> bool {
        let realpath = if entry.file_type().is_symlink() {
            let paths = std::fs::read_link(entry.path()).unwrap();
            std::path::Path::new("/sys/class/thermal/").join(paths).canonicalize().unwrap()
        } else if entry.file_type().is_dir() {
            entry.path().to_path_buf()
        } else {
            return false;
        };

        let re = regex::Regex::new(r"^thermal_zone\d{1,3}$").unwrap();
        re.is_match(realpath.file_name().unwrap().to_str().unwrap())
    }

    let mut thermals = Vec::new();
    for entry in walkdir::WalkDir::new("/sys/class/thermal")
            .sort_by_file_name()
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_thermal(e)) {
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                if let Ok(thermal) = Thermal::new(&full_path) {
                    thermals.push(thermal);
                }
    }
    
    thermals
}