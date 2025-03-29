use clap::{Arg, Command};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead};
use std::process::Command as ProcessCommand;

#[cfg(test)]
mod tests;

fn main() {
    let matches = Command::new("dockerfile-runner")
        .version("0.0.1")
        .author("David Hu")
        .about("Runs a Dockerfile as a bash script")
        .arg(
            Arg::new("dockerfile")
                .short('f')
                .long("file")
                .value_name("DOCKERFILE")
                .help("Path to the Dockerfile")
                .default_value("Dockerfile"),
        )
        .get_matches();

    let dockerfile_path = matches.get_one::<String>("dockerfile").unwrap();
    println!("📄 Reading Dockerfile from: {}", dockerfile_path);
    let file = fs::File::open(dockerfile_path).expect("Failed to open Dockerfile");
    let reader = io::BufReader::new(file);

    let run_re = Regex::new(r"^RUN\s+(.*)").unwrap();
    let add_re = Regex::new(r"^ADD\s+(https?://\S+)\s+.*").unwrap();
    let env_re = Regex::new(r"^ENV\s+(\S+)\s+(.+)").unwrap();
    let arg_re = Regex::new(r"^ARG\s+(\S+)(?:\s*=\s*(.+))?").unwrap();

    let mut args_map: HashMap<String, String> = HashMap::new();
    let mut run_command = String::new();
    let mut in_run_block = false;

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_string();
        println!("🔍 Processing line: {}", line);

        if in_run_block {
            if line.ends_with("\\") {
                println!("  ↪ Continuing multi-line RUN command");
                run_command.push_str(&line[..line.len() - 1]);
                run_command.push(' ');
            } else {
                run_command.push_str(&line);
                println!("🚀 Executing multi-line command: {}", run_command);
                ProcessCommand::new("bash")
                    .arg("-c")
                    .arg(&run_command)
                    .status()
                    .expect("Failed to execute command");
                run_command.clear();
                in_run_block = false;
            }
        } else if let Some(caps) = run_re.captures(&line) {
            let command = caps.get(1).unwrap().as_str();
            if command.ends_with("\\") {
                println!("  ↪ Starting multi-line RUN command");
                run_command.push_str(&command[..command.len() - 1]);
                run_command.push(' ');
                in_run_block = true;
            } else {
                println!("🚀 Executing command: {}", command);
                ProcessCommand::new("bash")
                    .arg("-c")
                    .arg(command)
                    .status()
                    .expect("Failed to execute command");
            }
        } else if let Some(caps) = add_re.captures(&line) {
            let url = caps.get(1).unwrap().as_str();
            println!("📥 Downloading from URL: {}", url);
            ProcessCommand::new("curl")
                .args(["-O", url])
                .status()
                .expect("Failed to execute curl");
        } else if let Some(caps) = env_re.captures(&line) {
            let key = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();
            println!("🔧 Setting environment variable: {}={}", key, value);
            env::set_var(key, value);
        } else if let Some(caps) = arg_re.captures(&line) {
            let key = caps.get(1).unwrap().as_str().to_string();
            let default_value = caps.get(2).map(|v| v.as_str().to_string());
            let value = if let Some(default) = default_value {
                println!("❓ ARG with default value: {}={}", key, default);
                println!("Enter value for ARG {} (default: {}): ", key, default);
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                let input = input.trim();
                if input.is_empty() {
                    println!("  ↪ Using default value");
                    default
                } else {
                    println!("  ↪ Using provided value: {}", input);
                    input.to_string()
                }
            } else {
                println!("❓ ARG without default value: {}", key);
                println!("Enter value for ARG {}: ", key);
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                let input = input.trim().to_string();
                println!("  ↪ Using provided value: {}", input);
                input
            };
            println!("🔧 Setting ARG variable: {}={}", key, value);
            args_map.insert(key, value);
        }
    }
}
