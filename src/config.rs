use bitflags::bitflags;

bitflags!{
    struct PERMISSION: u32 {
        const READ = 1;
        const WRITE = 1 << 1;
    }
}

pub struct Config {
    node: String,
    value: Option<String>,
    permission: PERMISSION,
}

impl Config {
    fn new(node: &str) -> Config {
        Config {
            node: node.to_string(),
            value: None,
            permission: PERMISSION::READ,
        }
    }

    fn add_permission(mut self, perm: PERMISSION) -> Self {
        self.permission = self.permission | perm;
        self
    }

    fn update(&mut self, root: &str) {
        self.value  = match crate::utils::read_line(&format!("{}/{}", root, self.node)) {
            Ok(v) => Some(v),
            Err(e) => None,
        };
    }
}

pub struct Module {
    root: String,
    configs: Vec<Config>,
}

impl Module {
    pub fn update(&mut self) {
        for config in &mut self.configs {
            config.update(&self.root)
        }
    }
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 自定义格式，使得仅显示 `x` 和 `y` 的值。
        for config in &self.configs {
            writeln!(f, "+ {}/{}", self.root, config.node)?;
            match &config.value {
                Some(v) => writeln!(f, "{}", v)?,
                None => writeln!(f, "WARNING: can't read {}/{}", self.root, config.node)?,
            }
        }
        Ok(())
    }
}

pub fn enumerate() -> Vec<Module>{
    let mut modules = Vec::new();
    if let Ok(m) = enumerate_cpu() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_vm() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_fs() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_acpi() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_audio() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_graphics() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_kernel() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_power() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_scsihost() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_pci_device() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_block_device() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_pcie_aspm() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_net_wakeup() {
        modules.push(m);
    }

    if let Ok(m) = enumerate_usb_wakeup() {
        modules.push(m);
    }

    for module in &mut modules {
        module.update();
    }
    
    modules
}

fn enumerate_cpu() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/devices/system/cpu".to_string(),
        configs: Vec::new(),
    };

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
                let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_driver", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_governor", base_name)).add_permission(PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_available_governors", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_min_freq", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_max_freq", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/cpuinfo_min_freq", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/cpuinfo_max_freq", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/energy_performance_preference", base_name)).add_permission(PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/energy_performance_available_preferences", base_name)));
                module.configs.push(Config::new(&format!("{}/cpufreq/boost", base_name)));
                module.configs.push(Config::new(&format!("{}/power/energy_perf_bias", base_name)).add_permission(PERMISSION::WRITE));
    }

    module.configs.push(Config::new("intel_pstate/max_perf_pct"));
    module.configs.push(Config::new("intel_pstate/min_perf_pct"));
    module.configs.push(Config::new("intel_pstate/num_pstates"));
    module.configs.push(Config::new("intel_pstate/turbo_pct"));
    module.configs.push(Config::new("intel_pstate/no_turbo"));
    module.configs.push(Config::new("intel_pstate/hwp_dynamic_boost"));
    module.configs.push(Config::new("intel_pstate/status"));
    module.configs.push(Config::new("intel_pstate/energy_efficiency"));
    module.configs.push(Config::new("cpuidle/current_driver"));
    module.configs.push(Config::new("cpuidle/available_governors"));
    module.configs.push(Config::new("cpuidle/current_governor").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_vm() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/proc/sys/vm".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("laptop_mode").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("dirty_writeback_centisecs").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("dirty_expire_centisecs").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("dirty_ratio").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("dirty_background_ratio").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_fs() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/proc/sys/fs".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("xfs/age_buffer_centisecs").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("xfs/xfssyncd_centisecs").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_acpi() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/firmware/acpi".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("platform_profile_choices").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("platform_profile").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("pm_profile").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_audio() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/module".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("snd_hda_intel/parameters/power_save").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("snd_hda_intel/parameters/power_save_controller").add_permission(PERMISSION::WRITE));
    module.configs.push(Config::new("snd_ac97_codec/parameters/power_save").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_graphics() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/class/drm".to_string(),
        configs: Vec::new(),
    };
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
                let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                module.configs.push(Config::new(&format!("{}/device/power_dpm_force_performance_level", base_name)));
                module.configs.push(Config::new(&format!("{}/device/power_dpm_state", base_name)));
                module.configs.push(Config::new(&format!("{}/device/power_method", base_name)));
                module.configs.push(Config::new(&format!("{}/device/power_profile", base_name)));
                module.configs.push(Config::new(&format!("{}/gt_min_freq_mhz", base_name)));
                module.configs.push(Config::new(&format!("{}/gt_max_freq_mhz", base_name)));
                module.configs.push(Config::new(&format!("{}/gt_boost_freq_mhz", base_name)));
    }
    Ok(module)
}

fn enumerate_kernel() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/proc/sys/kernel".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("nmi_watchdog").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_power() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/power".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("mem_sleep").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_scsihost() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/class/scsi_host".to_string(),
        configs: Vec::new(),
    };
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
                let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                module.configs.push(Config::new(&format!("{}/power/control", base_name)).add_permission(PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/link_power_management_policy", base_name)).add_permission(PERMISSION::WRITE));
    }
    Ok(module)
}

fn enumerate_pci_device() ->  std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/bus/pci/devices".to_string(),
        configs: Vec::new(),
    };
    for entry in walkdir::WalkDir::new("/sys/bus/pci/devices")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if entry.path().to_str().unwrap() != "/sys/bus/pci/devices" {
                    let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                    module.configs.push(Config::new(&format!("{}/power/control", base_name)).add_permission(PERMISSION::WRITE));
                }
    }
    Ok(module)
}

fn enumerate_block_device() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/block".to_string(),
        configs: Vec::new(),
    };
    for entry in walkdir::WalkDir::new("/sys/block")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if entry.path().to_str().unwrap() != "/sys/block" {
                    if !entry.file_name().to_str().unwrap().starts_with("loop") {
                        let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                        module.configs.push(Config::new(&format!("{}/device/power/control", base_name)).add_permission(PERMISSION::WRITE));
                        module.configs.push(Config::new(&format!("{}/device/power/autosuspend_delay_ms", base_name)).add_permission(PERMISSION::WRITE));
                        module.configs.push(Config::new(&format!("{}/queue/scheduler", base_name)).add_permission(PERMISSION::WRITE));
                    }
                }
    }
    Ok(module)
}

fn enumerate_pcie_aspm() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/module/pcie_aspm".to_string(),
        configs: Vec::new(),
    };
    module.configs.push(Config::new("parameters/policy").add_permission(PERMISSION::WRITE));
    Ok(module)
}

fn enumerate_net_wakeup() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/class/net".to_string(),
        configs: Vec::new(),
    };

    for entry in walkdir::WalkDir::new("/sys/class/net")
            .max_depth(1)
            .sort_by_file_name()
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if entry.path().to_str().unwrap() != "/sys/class/net" {
                    let name = entry.file_name().to_str().unwrap().to_string();
                    if name != "lo" && !name.starts_with("docker") {
                        let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                        module.configs.push(Config::new(&format!("{}/device/power/wakeup", base_name)).add_permission(PERMISSION::WRITE));
                    }
                }
    }
    Ok(module)
}

fn enumerate_usb_wakeup() -> std::io::Result<Module> {
    let mut module = Module {
        root: "/sys/bus/usb/devices".to_string(),
        configs: Vec::new(),
    };
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
                    let base_name = entry.path().file_name().unwrap().to_str().unwrap().to_string();
                    module.configs.push(Config::new(&format!("{}/power/wakeup", base_name)).add_permission(PERMISSION::WRITE));
                    module.configs.push(Config::new(&format!("{}/link_power_management_policy", base_name)).add_permission(PERMISSION::WRITE));
                }
    }
    Ok(module)
}