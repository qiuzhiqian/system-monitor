mod utils;
mod collector;
mod visualization;

mod config;
mod cpu;
mod battery;
mod thermal;
mod hwmon;

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
    Config{
        #[command(subcommand)]
        command: ConfigCommand,
    },
    /// collect data
    Collect{
        /// sum time to collect
        #[arg(short='t',long="time", default_value="120")]
        time: u32,
    },
    /// visual data
    Visual{},
}

#[derive(Subcommand)]
enum ConfigCommand {
    Show{},
    Apply{
        /// apply config from file
        #[arg(short='f',long="file")]
        file: String,
    }
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

fn main() -> std::io::Result<()> {
    let cli = Cli::parse();

    if let Some(cmd) = cli.command {
        match cmd {
            Command::Config{command} => { 
                match command {
                    ConfigCommand::Show {  } => {
                        show_os_info()?;
                        show_dim_info();
                        println!("--------------");
                        let configs = config::enumerate();
                        for config in configs {
                            println!("{}", config);
                        }
                        println!("--------------");

                        let bats = battery::enumerate();
                        if !bats.is_empty() {
                            println!(">> battery <<");
                            println!("{:#?}", bats);
                        }
                        
                        let thermals = thermal::enumerate();
                        if !thermals.is_empty() {
                            println!(">> thermal <<");
                            println!("{:#?}", thermals);
                        }
                        
                        let hwmons = hwmon::enumerate();
                        for hwmon in hwmons {
                            let fans = hwmon.fans();
                            let temps = hwmon.temps();
                            if !fans.is_empty() || !temps.is_empty() {
                                println!(">> hwmon name: {} <<", hwmon.name);
                                if !fans.is_empty() {
                                    println!("{:#?}", hwmon.fans());
                                }
                                if !temps.is_empty() {
                                    println!("{:#?}", hwmon.temps());
                                }
                            }
                        }
                    },
                    ConfigCommand::Apply { file } => {
                        let configs = config::Config::load(&file);
                        let local_configs = config::enumerate();

                        let mut need_change = Vec::new();
                        for config in &configs {
                            for local_config in &local_configs {
                                if config.node == local_config.node && local_config.value != None && config.value != local_config.value && local_config.writeable() {
                                    let mut config_new = local_config.clone();
                                    config_new.value = config.value.clone();
                                    need_change.push(config_new);
                                }
                            }
                        }

                        for config in need_change {
                            if let Some(val) = &config.value {
                                println!("apply: {} = {}", config.node, val);
                                if let Err(e) = config.apply() {
                                    println!("ERROR: apply {} failed {}", config.node, e)
                                }
                            }
                        }
                    }
                }
            },
            Command::Collect { time } => {
                let mut collectors: Vec<Box<dyn collector::Collector>> = Vec::new();
                let cpus = cpu::enumerate();
                //println!("{}",cpus.len());
                if cpus.len() > 0{
                    let c = Box::new(collector::CpuCollector::new(cpus, "cpufreq.csv")?);
                    collectors.push(c);
                }
                let bats = battery::enumerate();
                if bats.len() > 0 {
                    let c = Box::new(collector::CapacityCollector::new(bats.clone(), "capacity.csv", 5)?);
                    collectors.push(c);

                    let c = Box::new(collector::PowerCollector::new(bats, "power.csv")?);
                    collectors.push(c);
                }

                let thermals = thermal::enumerate();
                if thermals.len() > 0 {
                    let c = Box::new(collector::ThermalCollector::new(thermals, "thermal.csv")?);
                    collectors.push(c);
                }
                
                let count = (time + 4) / 5;
                'outer: for _ in 0..count {
                    for c in &mut collectors {
                        c.update()?;
                        if c.need_stop() {
                            break 'outer;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_secs(5));
                }

                // do once at end
                for c in &mut collectors {
                    c.update()?;
                }
            },
            Command::Visual {  } => {
                if let Err(e) = visualization::show_datas("cpufreq.csv", "cpufreq.svg", "show cpu freq chart") {
                    println!("WARNING: {}", e);
                }
                if let Err(e) = visualization::show_datas("capacity.csv", "capacity.svg", "show battery capacity chart") {
                    println!("WARNING: {}", e);
                }
                if let Err(e) = visualization::show_datas("power.csv", "power.svg", "show battery power chart") {
                    println!("WARNING: {}", e);
                }
                if let Err(e) = visualization::show_datas("thermal.csv", "thermal.svg", "show thermal chart") {
                    println!("WARNING: {}", e);
                }
            }
        }
    }
    
    Ok(())
}
