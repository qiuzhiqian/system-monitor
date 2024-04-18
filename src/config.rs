use bitflags::bitflags;

bitflags!{
    struct PERMISSION: u32 {
        const READ = 1;
        const WRITE = 1 << 1;
    }
}

struct Config {
    node: String,
    value: String,
    permission: PERMISSION,
}

impl Config {
    fn new(node: &str, perm: PERMISSION) -> Config {
        Config {
            node: node.to_string(),
            value: "".to_string(),
            permission: perm,
        }
    }
}

struct Module {
    root: String,
    configs: Vec<Config>,
}

impl std::fmt::Display for Module {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // 自定义格式，使得仅显示 `x` 和 `y` 的值。
        for config in &self.configs {
            writeln!(f, "+ {}/{}", self.root, config.node)?;
            writeln!(f, "{}", config.value)?;
        }
        Ok(())
    }
}

fn enumerate() -> Vec<Module>{
    let module = Vec::new();

    module
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
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_driver", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_governor", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_available_governors", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_min_freq", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/scaling_max_freq", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/cpuinfo_min_freq", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/cpuinfo_max_freq", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/energy_performance_preference", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/energy_performance_available_preferences", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/cpufreq/boost", base_name), PERMISSION::READ | PERMISSION::WRITE));
                module.configs.push(Config::new(&format!("{}/power/energy_perf_bias", base_name), PERMISSION::READ | PERMISSION::WRITE));
    }

    module.configs.push(Config::new("intel_pstate/max_perf_pct", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/min_perf_pct", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/num_pstates", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/turbo_pct", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/no_turbo", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/hwp_dynamic_boost", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/status", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("intel_pstate/energy_efficiency", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("cpuidle/current_driver", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("cpuidle/available_governors", PERMISSION::READ | PERMISSION::WRITE));
    module.configs.push(Config::new("cpuidle/current_governor", PERMISSION::READ | PERMISSION::WRITE));
    Ok(module)
}