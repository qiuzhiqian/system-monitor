use std::fs::File;
use std::io::{BufReader, BufRead, Read};

pub fn read_line(path: &str) -> std::io::Result<String> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);

    let mut line = String::new();
    let _ = reader.read_line(&mut line)?;
    Ok(line.trim().to_string())
}

pub fn read_all_line(path: &str) -> std::io::Result<String> {
    let input = File::open(path)?;
    let mut reader = BufReader::new(input);

    let mut data = String::new();
    let _ = reader.read_to_string(&mut data)?;
    Ok(data)
}

pub fn run_cmd(cmd: &str, args: Vec<&str>) -> std::io::Result<String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()?;

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}