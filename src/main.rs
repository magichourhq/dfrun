use clap::{Arg, Command};
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
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
        .arg(
            Arg::new("debug")
                .short('d')
                .long("debug")
                .help("Enable debug logging")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let debug_enabled = matches.get_flag("debug");
    let dockerfile_path = matches.get_one::<String>("dockerfile").unwrap();

    if debug_enabled {
        println!("DEBUG: Reading Dockerfile from: {}", dockerfile_path);
    }
    let file = fs::File::open(dockerfile_path).expect("Failed to open Dockerfile");
    let reader = io::BufReader::new(file);

    let run_re = Regex::new(r"^RUN\s+(.*)").unwrap();
    let add_re = Regex::new(r"^ADD\s+(https?://\S+)\s+.*").unwrap();
    let env_re = Regex::new(r"^ENV\s+(\S+)\s+(.+)").unwrap();
    let arg_re = Regex::new(r"^ARG\s+([^=\s]+)(?:\s*=\s*(.+))?").unwrap();

    let mut args_map: HashMap<String, String> = HashMap::new();
    let mut run_command = String::new();
    let mut in_run_block = false;

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_string();
        if debug_enabled {
            println!("DEBUG: Processing line: {}", line);
        }

        if in_run_block {
            if line.ends_with("\\") {
                if debug_enabled {
                    println!("DEBUG: Action: Continuing multi-line RUN command");
                }
                run_command.push_str(&line[..line.len() - 1]);
                run_command.push(' ');
            } else {
                run_command.push_str(&line);
                if debug_enabled {
                    println!(
                        "DEBUG: Action: Executing multi-line command: {}",
                        run_command
                    );
                }
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
                if debug_enabled {
                    println!("DEBUG: Action: Starting multi-line RUN command");
                }
                run_command.push_str(&command[..command.len() - 1]);
                run_command.push(' ');
                in_run_block = true;
            } else {
                if debug_enabled {
                    println!("DEBUG: Action: Executing command: {}", command);
                }
                ProcessCommand::new("bash")
                    .arg("-c")
                    .arg(command)
                    .status()
                    .expect("Failed to execute command");
            }
        } else if let Some(caps) = add_re.captures(&line) {
            let url = caps.get(1).unwrap().as_str();
            if debug_enabled {
                println!("DEBUG: Action: Downloading from URL: {}", url);
            }
            ProcessCommand::new("curl")
                .args(["-O", url])
                .status()
                .expect("Failed to execute curl");
        } else if let Some(caps) = env_re.captures(&line) {
            let key = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();
            if debug_enabled {
                println!(
                    "DEBUG: Action: Setting environment variable: {}={}",
                    key, value
                );
            }
            env::set_var(key, value);
        } else if let Some(caps) = arg_re.captures(&line) {
            let key = caps.get(1).unwrap().as_str().to_string();
            let default_value = caps.get(2).map(|v| v.as_str().to_string());
            let value = if let Some(default) = default_value {
                if debug_enabled {
                    println!(
                        "DEBUG: Action: Found ARG with default value: {}={}",
                        key, default
                    );
                }
                print!("Enter value for ARG {} (default: {}): ", key, default);
                io::stdout().flush().expect("Failed to flush stdout");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                let input = input.trim();
                if input.is_empty() {
                    if debug_enabled {
                        println!("DEBUG: Action: Using default value");
                    }
                    default
                } else {
                    if debug_enabled {
                        println!("DEBUG: Action: Using provided value: {}", input);
                    }
                    input.to_string()
                }
            } else {
                // Check if there's an environment variable with this name
                let env_value = env::var(&key).ok();
                if debug_enabled {
                    println!("DEBUG: Action: Found ARG without default value: {}", key);
                    if let Some(ref val) = env_value {
                        println!("DEBUG: Action: Found environment value: {}", val);
                    }
                }
                print!(
                    "Enter value for ARG {}{}: ",
                    key,
                    env_value
                        .as_ref()
                        .map_or("".to_string(), |v| format!(" (default: {})", v))
                );
                io::stdout().flush().expect("Failed to flush stdout");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                let input = input.trim();
                if input.is_empty() && env_value.is_some() {
                    if debug_enabled {
                        println!("DEBUG: Action: Using environment value");
                    }
                    env_value.unwrap()
                } else if input.is_empty() {
                    if debug_enabled {
                        println!("DEBUG: Action: Using empty value");
                    }
                    String::new()
                } else {
                    if debug_enabled {
                        println!("DEBUG: Action: Using provided value: {}", input);
                    }
                    input.to_string()
                }
            };
            if debug_enabled {
                println!("DEBUG: Action: Setting ARG variable: {}={}", key, value);
            }
            args_map.insert(key, value);
        } else if !line.is_empty() && !line.starts_with('#') {
            // Only log if line isn't empty and isn't a comment
            if debug_enabled {
                println!("DEBUG: Original command: {}", line);
                println!("DEBUG: Action: Ignoring unsupported instruction");
            }
        }
    }
}
