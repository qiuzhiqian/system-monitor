use std::{io::BufRead, rc::Rc};

use bitflags::bitflags;

bitflags!{
    #[derive(Clone, Debug)]
    #[derive(PartialEq)]
    struct PERMISSION: u32 {
        const READ = 1;
        const WRITE = 1 << 1;
    }
}

#[derive(Clone)]
pub struct Config {
    pub node: String,
    pub value: Option<String>,
    permission: PERMISSION,
    pub handler: Rc<Box<dyn Fn(&str) -> String>>,
}

impl Config {
    fn new(node: &str) -> Config {
        Self::new_with_handler(node, |s| {s.to_string()})
    }

    fn new_with_handler<F: 'static>(node: &str,f: F)  -> Config where
        F: Fn(&str) -> String {
        let value  = match crate::utils::read_line(node) {
            Ok(v) => Some(v),
            Err(e) => None,
        };

        Config {
            node: node.to_string(),
            value,
            permission: PERMISSION::READ,
            handler: Rc::new(Box::new(f)),
        }
    }

    fn add_permission(mut self, perm: PERMISSION) -> Self {
        self.permission = self.permission | perm;
        self
    }

    pub fn apply(&self) ->std::io::Result<()> {
        match &self.value {
            Some(val) => {
                let raw_val = (self.handler)(&val);
                crate::utils::write_line(&self.node, &raw_val).unwrap();
                Ok(())
            },
            None => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "val is none")),
        }
    }

    pub fn writeable(&self) -> bool {
        match &self.value {
            Some(_) => {
                (self.permission.clone() & PERMISSION::WRITE) == PERMISSION::WRITE
            },
            None => false,
        }
    }

    pub fn load(infile: &str) -> Vec<Self> {
        let file = std::fs::File::open(infile).unwrap();
        let mut f = std::io::BufReader::new(file);
        
        let mut configs = Vec::new();
        let mut start: bool = false;
        let mut config = Config::new("");
        let mut new_config = false;
        loop {
            let mut buf = String::new();
            let len = f.read_line(&mut buf).unwrap();
            if len == 0 {
                break;
            }

            let buf = buf.trim();

            if !start && buf.starts_with("----------") {
                start = true;
            } else if start && buf.starts_with("----------") {
                break;
            } else if start {
                if buf.is_empty() {
                    continue;
                } else if buf.starts_with("+ ") {
                    new_config = true;
                    config.node = buf[2..].to_string();
                } else if new_config {
                    new_config = false;
                    if buf.starts_with("ERROR:") || buf.starts_with("WARNING:") {
                        continue;
                    } else {
                        config.value = Some(buf.to_string());
                        configs.push(config.clone());
                    }
                }
            }
        }
        configs
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 自定义格式，使得仅显示 `x` 和 `y` 的值。
        writeln!(f, "+ {}", self.node)?;
        match &self.value {
            Some(v) => write!(f, "{}", v)?,
            None => write!(f, "WARNING: can't read {}", self.node)?,
        }
        Ok(())
    }
}

pub fn enumerate() -> Vec<Config> {
    let mut configs = Vec::new();

    configs.append(&mut enumerate_cpu());
    configs.append(&mut enumerate_vm());
    configs.append(&mut enumerate_fs());
    configs.append(&mut enumerate_acpi());
    configs.append(&mut enumerate_audio());
    configs.append(&mut enumerate_graphics());
    configs.append(&mut enumerate_kernel());
    configs.append(&mut enumerate_power());
    configs.append(&mut enumerate_scsihost());
    configs.append(&mut enumerate_pci_device());
    configs.append(&mut enumerate_block_device());
    configs.append(&mut enumerate_pcie_aspm());
    configs.append(&mut enumerate_net_wakeup());
    configs.append(&mut enumerate_usb_wakeup());
    
    configs
}

fn enumerate_cpu() -> Vec<Config> {
    let mut configs = Vec::new();

    fn is_cpuname(entry: &walkdir::DirEntry) -> bool {
        if entry.path().to_str().unwrap() == "/sys/devices/system/cpu" {
            return false;
        }
        entry.file_name()
             .to_str()
             .map(|s| {
                let re = regex::Regex::new(r"^cpu\d{1,3}$").unwrap();
                re.is_match(s)
             })
             .unwrap_or(false)
    }

    for entry in walkdir::WalkDir::new("/sys/devices/system/cpu")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_cpuname(e)) {
                let fullpath = entry.path().to_str().unwrap();
                configs.push(Config::new(&format!("{}/cpufreq/scaling_driver", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/scaling_governor", fullpath)).add_permission(PERMISSION::WRITE));
                configs.push(Config::new(&format!("{}/cpufreq/scaling_available_governors", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/scaling_min_freq", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/scaling_max_freq", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/cpuinfo_min_freq", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/cpuinfo_max_freq", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/energy_performance_preference", fullpath)).add_permission(PERMISSION::WRITE));
                configs.push(Config::new(&format!("{}/cpufreq/energy_performance_available_preferences", fullpath)));
                configs.push(Config::new(&format!("{}/cpufreq/boost", fullpath)));
                configs.push(Config::new(&format!("{}/power/energy_perf_bias", fullpath)).add_permission(PERMISSION::WRITE));
    }

    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/max_perf_pct"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/min_perf_pct"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/num_pstates"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/turbo_pct"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/no_turbo"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/status"));
    configs.push(Config::new("/sys/devices/system/cpu/intel_pstate/energy_efficiency"));
    configs.push(Config::new("/sys/devices/system/cpu/cpuidle/current_driver"));
    configs.push(Config::new("/sys/devices/system/cpu/cpuidle/available_governors"));
    configs.push(Config::new("/sys/devices/system/cpu/cpuidle/current_governor").add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_vm() -> Vec<Config> {
    let mut configs = Vec::new();
    configs.push(Config::new("/proc/sys/vm/laptop_mode").add_permission(PERMISSION::WRITE));
    configs.push(Config::new("/proc/sys/vm/dirty_writeback_centisecs").add_permission(PERMISSION::WRITE));
    configs.push(Config::new("/proc/sys/vm/dirty_expire_centisecs").add_permission(PERMISSION::WRITE));
    configs.push(Config::new("/proc/sys/vm/dirty_ratio").add_permission(PERMISSION::WRITE));
    configs.push(Config::new("/proc/sys/vm/dirty_background_ratio").add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_fs() -> Vec<Config> {
    let mut configs = Vec::new();
    const ROOTPATH: &str = "/proc/sys/fs";
    configs.push(Config::new(&format!("{}/xfs/age_buffer_centisecs", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/xfs/xfssyncd_centisecs", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_acpi() -> Vec<Config> {
    let mut configs = Vec::new();
    const ROOTPATH: &str = "/sys/firmware/acpi";
    configs.push(Config::new(&format!("{}/platform_profile_choices", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/platform_profile", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/pm_profile", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_audio() -> Vec<Config> {
    let mut configs = Vec::new();
    const ROOTPATH: &str = "/sys/module";
    configs.push(Config::new(&format!("{}/snd_hda_intel/parameters/power_save", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/snd_hda_intel/parameters/power_save_controller", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/snd_ac97_codec/parameters/power_save", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_graphics() -> Vec<Config> {
    let mut configs = Vec::new();
    fn is_cardname(entry: &walkdir::DirEntry) -> bool {
        if entry.path().to_str().unwrap() == "/sys/class/drm" {
            return false;
        }
        let realpath = if entry.file_type().is_symlink() {
            let paths = std::fs::read_link(entry.path()).unwrap();
            std::path::Path::new("/sys/class/drm").join(paths).canonicalize().unwrap()
        } else if entry.file_type().is_dir() {
            entry.path().to_path_buf()
        } else {
            return false;
        };

        realpath.file_name()
             .map(|s| {
                let re = regex::Regex::new(r"^card\d{1,3}$").unwrap();
                re.is_match(s.to_str().unwrap())
             })
             .unwrap_or(false)
    }

    for entry in walkdir::WalkDir::new("/sys/class/drm")
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_cardname(e)) {
                let fullpath = entry.path().to_str().unwrap();
                configs.push(Config::new(&format!("{}/device/power_dpm_force_performance_level", fullpath)));
                configs.push(Config::new(&format!("{}/device/power_dpm_state", fullpath)));
                configs.push(Config::new(&format!("{}/device/power_method", fullpath)));
                configs.push(Config::new(&format!("{}/device/power_profile", fullpath)));
                configs.push(Config::new(&format!("{}/gt_min_freq_mhz", fullpath)));
                configs.push(Config::new(&format!("{}/gt_max_freq_mhz", fullpath)));
                configs.push(Config::new(&format!("{}/gt_boost_freq_mhz", fullpath)));
    }
    configs
}

fn enumerate_kernel() -> Vec<Config> {
    let mut configs = Vec::new();
    const ROOTPATH: &str = "/proc/sys/kernel";
    configs.push(Config::new(&format!("{}/nmi_watchdog", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/sched_rr_timeslice_ms", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/sched_rt_period_us", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs.push(Config::new(&format!("{}/sched_rt_runtime_us", ROOTPATH)).add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_power() -> Vec<Config> {
    const ROOTPATH: &str = "/sys/power";
    let mut configs = Vec::new();
    configs.push(Config::new_with_handler(&format!("{}/mem_sleep", ROOTPATH), |s| {
        let start = match  s.find("[") {
            Some(s) => s + 1,
            None => 0,
        };
        let end = match s.find("]") {
            Some(e) => e,
            None => s.len(),
        };

        s[start..end].to_string()
    }).add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_scsihost() -> Vec<Config> {
    let mut configs = Vec::new();
    const ROOTPATH: &str = "/sys/class/scsi_host";
    fn is_scsihost_name(entry: &walkdir::DirEntry) -> bool {
        if entry.path().to_str().unwrap() == "/sys/class/scsi_host" {
            return false;
        }
        entry.file_name()
             .to_str()
             .map(|s| {
                let re = regex::Regex::new(r"^host\d{1,3}$").unwrap();
                re.is_match(s)
             })
             .unwrap_or(false)
    }
    for entry in walkdir::WalkDir::new("/sys/class/scsi_host")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_scsihost_name(e)) {
                let fullpath = entry.path().to_str().unwrap();
                configs.push(Config::new(&format!("{}/power/control", fullpath)).add_permission(PERMISSION::WRITE));
                configs.push(Config::new(&format!("{}/link_power_management_policy", fullpath)).add_permission(PERMISSION::WRITE));
    }
    configs
}

fn enumerate_pci_device() ->  Vec<Config> {
    let mut configs = Vec::new();
    const ROOTPATH: &str = "/sys/bus/pci/devices";
    for entry in walkdir::WalkDir::new("/sys/bus/pci/devices")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if entry.path().to_str().unwrap() != "/sys/bus/pci/devices" {
                    let fullpath = entry.path().to_str().unwrap();
                    configs.push(Config::new(&format!("{}/power/control", fullpath)).add_permission(PERMISSION::WRITE));
                }
    }
    configs
}

fn enumerate_block_device() -> Vec<Config> {
    let mut configs = Vec::new();
    for entry in walkdir::WalkDir::new("/sys/block")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if entry.path().to_str().unwrap() != "/sys/block" {
                    if !entry.file_name().to_str().unwrap().starts_with("loop") {
                        let fullpath = entry.path().to_str().unwrap();
                        configs.push(Config::new(&format!("{}/device/power/control", fullpath)).add_permission(PERMISSION::WRITE));
                        configs.push(Config::new(&format!("{}/device/power/autosuspend_delay_ms", fullpath)).add_permission(PERMISSION::WRITE));
                        configs.push(Config::new_with_handler(&format!("{}/queue/scheduler", fullpath), |s| {
                            let start = match  s.find("[") {
                                Some(s) => s + 1,
                                None => 0,
                            };
                            let end = match s.find("]") {
                                Some(e) => e,
                                None => s.len(),
                            };
                    
                            s[start..end].to_string()
                        }).add_permission(PERMISSION::WRITE));
                    }
                }
    }
    configs
}

fn enumerate_pcie_aspm() -> Vec<Config> {
    let mut configs = Vec::new();
    // [default] performance powersave powersupersave
    configs.push(Config::new_with_handler("/sys/module/pcie_aspm/parameters/policy", |s| {
        let start = match  s.find("[") {
            Some(s) => s + 1,
            None => 0,
        };
        let end = match s.find("]") {
            Some(e) => e,
            None => s.len(),
        };

        s[start..end].to_string()
    }).add_permission(PERMISSION::WRITE));
    configs
}

fn enumerate_net_wakeup() -> Vec<Config> {
    let mut configs = Vec::new();

    for entry in walkdir::WalkDir::new("/sys/class/net")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if entry.path().to_str().unwrap() != "/sys/class/net" {
                    let name = entry.file_name().to_str().unwrap().to_string();
                    if name != "lo" && !name.starts_with("docker") {
                        let fullpath = entry.path().to_str().unwrap();
                        configs.push(Config::new(&format!("{}/device/power/wakeup", fullpath)).add_permission(PERMISSION::WRITE));
                    }
                }
    }
    configs
}

fn enumerate_usb_wakeup() -> Vec<Config> {
    let mut configs = Vec::new();
    fn is_usb_name(entry: &walkdir::DirEntry) -> bool {
        entry.file_name()
             .to_str()
             .map(|s| {
                let re = regex::Regex::new(r"^usb\d{1,3}$").unwrap();
                re.is_match(s)
             })
             .unwrap_or(false)
    }

    for entry in walkdir::WalkDir::new("/sys/bus/usb/devices")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| (e.file_type().is_dir() || e.file_type().is_symlink()) && is_usb_name(e)) {
                if entry.path().to_str().unwrap() != "/sys/bus/usb/devices" {
                    let fullpath = entry.path().to_str().unwrap();
                    configs.push(Config::new(&format!("{}/power/wakeup", fullpath)).add_permission(PERMISSION::WRITE));
                    configs.push(Config::new(&format!("{}/link_power_management_policy", fullpath)).add_permission(PERMISSION::WRITE));
                }
    }
    configs
}