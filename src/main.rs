mod utils;
mod collector;
mod visualization;

mod cpu;
mod battery;

use walkdir;
use regex;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// get config
    Config{},
    /// collect data
    Collect{},
    /// visual data
    Visual{},
}

fn run_cmd_with_echo(cmd: &str, args: Vec<&str>) -> std::io::Result<()> {
    println!(">>> {} {}", cmd, args.join(" "));
    println!("{}", utils::run_cmd(cmd, args)?);
    Ok(())
}

fn read_line_with_echo(path: &str) {
    println!("+ {}", path);
    match utils::read_line(path) {
        Ok(v) => println!("{}", v),
        Err(e) => println!("ERROR: {} {}", path, e),
    };
}

fn read_all_wich_echo(path: &str) {
    println!("+ {}", path);
    match utils::read_all_line(path) {
        Ok(v) => println!("{}", v),
        Err(e) => println!("ERROR: {} {}", path, e),
    };
}

fn show_os_info() -> std::io::Result<()>{
    read_all_wich_echo("/etc/os-release");
    run_cmd_with_echo("uname", vec!["-a"])?;

    if let Ok(session_desktop) = std::env::var("XDG_SESSION_DESKTOP") {
        println!("XDG_SESSION_DESKTOP={}", session_desktop);
    }

    if let Ok(session_type) = std::env::var("XDG_SESSION_TYPE") {
        println!("XDG_SESSION_TYPE={}", session_type);
    }

    Ok(())
}

fn show_dim_info() {
    read_line_with_echo("/sys/class/dmi/id/product_version");
    read_line_with_echo("/sys/class/dmi/id/product_name");
    read_line_with_echo("/sys/class/dmi/id/sys_vendor");
    read_line_with_echo("/sys/class/dmi/id/bios_version");
    read_line_with_echo("/sys/class/dmi/id/ec_firmware_release");
}

fn show_cpu_info() {
    fn is_cpuname(entry: &walkdir::DirEntry) -> bool {
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
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_cpuname(e)) {
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                read_line_with_echo(&(full_path.clone()+"/cpufreq/scaling_driver"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/scaling_governor"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/scaling_available_governors"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/scaling_min_freq"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/scaling_max_freq"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/cpuinfo_min_freq"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/cpuinfo_max_freq"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/energy_performance_preference"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/energy_performance_available_preferences"));
                read_line_with_echo(&(full_path.clone()+"/cpufreq/boost"));
                read_line_with_echo(&(full_path.clone()+ "/power/energy_perf_bias"));
    }

    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/max_perf_pct");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/min_perf_pct");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/num_pstates");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/turbo_pct");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/no_turbo");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/hwp_dynamic_boost");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/status");
    read_line_with_echo("/sys/devices/system/cpu/intel_pstate/energy_efficiency");
    read_line_with_echo("/sys/devices/system/cpu/cpuidle/current_driver");
    read_line_with_echo("/sys/devices/system/cpu/cpuidle/available_governors");
    read_line_with_echo("/sys/devices/system/cpu/cpuidle/current_governor");
}

fn show_fs_info() {
    read_line_with_echo("/proc/sys/vm/laptop_mode");
    read_line_with_echo("/proc/sys/vm/dirty_writeback_centisecs");
    read_line_with_echo("/proc/sys/vm/dirty_expire_centisecs");
    read_line_with_echo("/proc/sys/vm/dirty_ratio");
    read_line_with_echo("/proc/sys/vm/dirty_background_ratio");

    read_line_with_echo("/proc/sys/fs/xfs/age_buffer_centisecs");
    read_line_with_echo("/proc/sys/fs/xfs/xfssyncd_centisecs");
}

fn show_platform_info() {
    read_line_with_echo("/sys/firmware/acpi/platform_profile_choices");
    read_line_with_echo("/sys/firmware/acpi/platform_profile");
    read_line_with_echo("/sys/firmware/acpi/pm_profile");
}

fn show_audio_info() {
    read_line_with_echo("/sys/module/snd_hda_intel/parameters/power_save");
    read_line_with_echo("/sys/module/snd_hda_intel/parameters/power_save_controller");
    read_line_with_echo("/sys/module/snd_ac97_codec/parameters/power_save");
}

fn show_graphics_info() {
    fn is_cardname(entry: &walkdir::DirEntry) -> bool {
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
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                read_line_with_echo(&(full_path.clone() + "/device/power_dpm_force_performance_level"));
                read_line_with_echo(&(full_path.clone() + "/device/power_dpm_state"));
                read_line_with_echo(&(full_path.clone() + "/device/power_method"));
                read_line_with_echo(&(full_path.clone() + "/device/power_profile"));
                read_line_with_echo(&(full_path.clone() + "/gt_min_freq_mhz"));
                read_line_with_echo(&(full_path.clone() + "/gt_max_freq_mhz"));
                read_line_with_echo(&(full_path.clone() + "/gt_boost_freq_mhz"));
    }
}

fn show_watchdog_info() {
    read_line_with_echo("/proc/sys/kernel/nmi_watchdog")
}

fn show_suspend_info() {
    read_line_with_echo("/sys/power/mem_sleep")
}

fn disk_info() {
    fn is_scsihost_name(entry: &walkdir::DirEntry) -> bool {
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
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  (e.file_type().is_dir() || e.file_type().is_symlink()) && is_scsihost_name(e)) {
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                read_line_with_echo(&(full_path.clone() + "/power/control"));
                read_line_with_echo(&(full_path.clone() + "/link_power_management_policy"));
    }

    for entry in walkdir::WalkDir::new("/sys/bus/pci/devices")
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                read_line_with_echo(&(full_path.clone() + "/power/control"));
    }

    for entry in walkdir::WalkDir::new("/sys/block")
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                if !entry.file_name().to_str().unwrap().starts_with("loop") {
                    let full_path =  entry.path().to_str().expect("is not path").to_string();
                    read_line_with_echo(&(full_path.clone() + "/device/power/control"));
                    read_line_with_echo(&(full_path.clone() + "/device/power/autosuspend_delay_ms"));
                    read_line_with_echo(&(full_path.clone() + "/queue/scheduler"));
                }
    }

    read_line_with_echo("/sys/module/pcie_aspm/parameters/policy")
}

fn wakeup_info() {
    fn is_usb_name(entry: &walkdir::DirEntry) -> bool {
        entry.file_name()
             .to_str()
             .map(|s| {
                let re = regex::Regex::new(r"^usb\d{1,3}$").unwrap();
                re.is_match(s)
             })
             .unwrap_or(false)
    }

    for entry in walkdir::WalkDir::new("/sys/class/net")
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e|  e.file_type().is_dir() || e.file_type().is_symlink()) {
                let name = entry.file_name().to_str().unwrap().to_string();
                if name != "lo" && !name.starts_with("docker") {
                    let full_path =  entry.path().to_str().expect("is not path").to_string();
                    read_line_with_echo(&(full_path.clone() + "/device/power/wakeup"));
                }
    }

    for entry in walkdir::WalkDir::new("/sys/bus/usb/devices")
            .max_depth(1)
            .into_iter()
            .filter_map(Result::ok)
            .filter(|e| (e.file_type().is_dir() || e.file_type().is_symlink()) && is_usb_name(e)) {
                let full_path =  entry.path().to_str().expect("is not path").to_string();
                read_line_with_echo(&(full_path.clone() + "/power/wakeup"));
                read_line_with_echo(&(full_path.clone() + "/link_power_management_policy"));
    }
}

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Command::Config{} => { 
                show_os_info()?;
                show_dim_info();
                show_cpu_info();
                show_fs_info();
                show_platform_info();
                show_audio_info();
                show_graphics_info();
                show_watchdog_info();
                show_suspend_info();
                disk_info();
                wakeup_info();
            },
            Command::Collect {  } => {
                let mut collectors: Vec<Box<dyn collector::Collector>> = Vec::new();
                let cpus = cpu::enumerate();
                //println!("{}",cpus.len());
                if cpus.len() > 0{
                    let c = Box::new(collector::CpuCollector::new(cpus, "cpufreq.csv")?);
                    collectors.push(c);
                }
                let bats = battery::enumerate();
                if bats.len() > 0 {
                    let c = Box::new(collector::CapacityCollector::new(bats, "capacity.csv")?);
                    collectors.push(c);
                }
                
                for i in 0..10 {
                    for c in &mut collectors {
                        c.update()?;
                    }
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }

                // do once at end
                for c in &mut collectors {
                    c.update()?;
                }
                let bat = battery::enumerate();
                println!("{:#?}", bat);
            },
            Command::Visual {  } => {
                //let cpus = cpu::enumerate_cpus();
                //for cpu in cpus {
                //    println!("cpu: {:?}", cpu);
                //}
                visualization::show_cpu().unwrap();
            }
        }
    }

    
    Ok(())
}
