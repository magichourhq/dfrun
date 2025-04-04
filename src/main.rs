use clap::{Arg, Command};
use colored::*;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::process::Command as ProcessCommand;

#[cfg(test)]
mod tests;

fn expand_vars(s: &str, vars_map: &HashMap<String, String>) -> String {
    let mut result = s.to_string();
    // Sort keys by length in descending order to handle nested variables correctly
    let mut keys: Vec<_> = vars_map.keys().collect();
    keys.sort_by(|a, b| b.len().cmp(&a.len()));

    // Expand all variables from vars_map
    for key in keys {
        if let Some(value) = vars_map.get(key) {
            result = result.replace(&format!("${}", key), value);
            result = result.replace(&format!("${{{}}}", key), value);
        }
    }
    result
}

fn main() {
    let matches = Command::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author("David Hu")
        .about("Runs a Dockerfile as a bash script")
        .arg(
            Arg::new("dockerfile")
                .short('f')
                .long("file")
                .value_name("DOCKERFILE")
                .help("Path to the Dockerfile. Default to Dockerfile in current directory.")
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
        println!(
            "{} {}",
            "DEBUG:".bright_blue().bold(),
            format!("Reading Dockerfile from: {}", dockerfile_path).bright_white()
        );
    }

    // Check if file exists first
    if fs::metadata(dockerfile_path).is_err() {
        eprintln!(
            "{} {}",
            "Error:".red().bold(),
            format!("Dockerfile not found at: {}", dockerfile_path).bright_white()
        );
        eprintln!(
            "{} {}",
            "Hint:".yellow().bold(),
            "Make sure the Dockerfile exists in the specified path or use -f/--file to specify a different path.".bright_white()
        );
        std::process::exit(1);
    }

    let file = match fs::File::open(dockerfile_path) {
        Ok(file) => file,
        Err(e) => {
            eprintln!(
                "{} {}",
                "Error:".red().bold(),
                format!("Failed to open Dockerfile: {}", e).bright_white()
            );
            std::process::exit(1);
        }
    };
    let reader = io::BufReader::new(file);

    let run_re = Regex::new(r"^RUN\s+(.*)").unwrap();
    let add_re = Regex::new(r"^ADD\s+(https?://\S+)\s+.*").unwrap();
    let env_re = Regex::new(r"^ENV\s+(\S+)\s+(.+)").unwrap();
    let arg_re = Regex::new(r"^ARG\s+([^=\s]+)(?:\s*=\s*(.+))?").unwrap();
    let workdir_re = Regex::new(r"^WORKDIR\s+(.+)").unwrap();

    let mut vars_map: HashMap<String, String> = HashMap::new();
    let mut run_command = String::new();
    let mut in_run_block = false;
    let dockerfile_dir = PathBuf::from(dockerfile_path)
        .canonicalize()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let mut workdir = dockerfile_dir.clone();

    // Create initial workdir if it doesn't exist
    if !workdir.exists() {
        fs::create_dir_all(&workdir).expect("Failed to create working directory");
    }

    // Change to the initial workdir
    env::set_current_dir(&workdir).expect("Failed to change to initial directory");

    for line in reader.lines() {
        let line = line.expect("Failed to read line").trim().to_string();
        if debug_enabled {
            println!(
                "{} {}",
                "DEBUG:".bright_blue().bold(),
                format!("Processing line: {}", line).bright_white()
            );
        }

        if in_run_block {
            if line.ends_with("\\") {
                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        "Action: Continuing multi-line RUN command".yellow()
                    );
                }
                if let Some(stripped) = line.strip_suffix("\\") {
                    run_command.push_str(stripped);
                    run_command.push(' ');
                }
            } else {
                run_command.push_str(&line);
                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        format!(
                            "Action: Executing multi-line command in {}: {}",
                            workdir.display(),
                            run_command
                        )
                        .green()
                    );
                }
                let current_dir = env::current_dir().unwrap();
                env::set_current_dir(&workdir).expect("Failed to change directory");
                let expanded_command = expand_vars(&run_command, &vars_map);
                let status = ProcessCommand::new("bash")
                    .arg("-c")
                    .arg(&expanded_command)
                    .status()
                    .expect("Failed to execute command");
                env::set_current_dir(current_dir).expect("Failed to restore directory");
                if !status.success() {
                    eprintln!("Command failed with status: {}", status);
                    std::process::exit(1);
                }
                run_command.clear();
                in_run_block = false;
            }
        } else if let Some(caps) = env_re.captures(&line) {
            let key = caps.get(1).unwrap().as_str();
            let value = caps.get(2).unwrap().as_str();
            if debug_enabled {
                println!(
                    "{} {}",
                    "DEBUG:".bright_blue().bold(),
                    format!("Action: Setting environment variable: {}={}", key, value).magenta()
                );
            }
            // First expand variables in the ENV value
            let expanded_value = expand_vars(value, &vars_map);
            if debug_enabled {
                println!(
                    "{} {}",
                    "DEBUG:".bright_blue().bold(),
                    format!("Action: Expanded value: {}={}", key, expanded_value).magenta()
                );
            }
            // Set both in environment and vars_map
            env::set_var(key, &expanded_value);
            vars_map.insert(key.to_string(), expanded_value);
        } else if let Some(caps) = workdir_re.captures(&line) {
            let dir = caps.get(1).unwrap().as_str();
            if debug_enabled {
                println!(
                    "{} {}",
                    "DEBUG:".bright_blue().bold(),
                    format!("Action: Setting working directory to: {}", dir).cyan()
                );
            }
            // First expand variables in the directory path
            let expanded_dir = expand_vars(dir, &vars_map);
            // If the path is absolute, use it as is, otherwise join with dockerfile directory
            workdir = if PathBuf::from(&expanded_dir).is_absolute() {
                PathBuf::from(&expanded_dir)
            } else {
                dockerfile_dir.join(&expanded_dir)
            };
            // Create directory if it doesn't exist
            if fs::metadata(&workdir).is_err() {
                fs::create_dir_all(&workdir).expect("Failed to create working directory");
            }
            // Actually change to the directory to verify it works
            env::set_current_dir(&workdir).expect("Failed to change directory");
            if debug_enabled {
                println!(
                    "{} {}",
                    "DEBUG:".bright_blue().bold(),
                    format!("Action: Changed to directory: {}", workdir.display()).cyan()
                );
            }
        } else if let Some(caps) = run_re.captures(&line) {
            let command = caps.get(1).unwrap().as_str();
            if command.ends_with("\\") {
                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        "Action: Starting multi-line RUN command".yellow()
                    );
                }
                if let Some(stripped) = command.strip_suffix("\\") {
                    run_command.push_str(stripped);
                    run_command.push(' ');
                }
                in_run_block = true;
            } else {
                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        format!(
                            "Action: Executing command in {}: {}",
                            workdir.display(),
                            command
                        )
                        .green()
                    );
                }
                let current_dir = env::current_dir().unwrap();
                env::set_current_dir(&workdir).expect("Failed to change directory");
                let expanded_command = expand_vars(command, &vars_map);
                let status = ProcessCommand::new("bash")
                    .arg("-c")
                    .arg(&expanded_command)
                    .status()
                    .expect("Failed to execute command");
                env::set_current_dir(current_dir).expect("Failed to restore directory");
                if !status.success() {
                    eprintln!("Command failed with status: {}", status);
                    std::process::exit(1);
                }
            }
        } else if let Some(caps) = add_re.captures(&line) {
            let url = caps.get(1).unwrap().as_str();
            if debug_enabled {
                println!(
                    "{} {}",
                    "DEBUG:".bright_blue().bold(),
                    format!("Action: Downloading from URL: {}", url).cyan()
                );
            }
            let current_dir = env::current_dir().unwrap();
            env::set_current_dir(&workdir).expect("Failed to change directory");
            let status = ProcessCommand::new("curl")
                .args(["-O", url])
                .status()
                .expect("Failed to execute curl");
            env::set_current_dir(current_dir).expect("Failed to restore directory");
            if !status.success() {
                eprintln!("Download failed with status: {}", status);
            }
        } else if let Some(caps) = arg_re.captures(&line) {
            let key = caps.get(1).unwrap().as_str().to_string();
            let default_value = caps.get(2).map(|v| v.as_str().to_string());
            if let Some(default) = default_value {
                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        format!("Action: Found ARG with default value: {}={}", key, default)
                            .yellow()
                    );
                }
                print!("Enter value for ARG {} (default: {}): ", key, default);
                io::stdout().flush().expect("Failed to flush stdout");
                let mut input = String::new();
                io::stdin()
                    .read_line(&mut input)
                    .expect("Failed to read input");
                let input = input.trim();
                let value = if input.is_empty() {
                    if debug_enabled {
                        println!(
                            "{} {}",
                            "DEBUG:".bright_blue().bold(),
                            "Action: Using default value".green()
                        );
                    }
                    default
                } else {
                    if debug_enabled {
                        println!(
                            "{} {}",
                            "DEBUG:".bright_blue().bold(),
                            format!("Action: Using provided value: {}", input).green()
                        );
                    }
                    input.to_string()
                };

                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        format!("Action: Setting ARG variable: {}={}", key, value).magenta()
                    );
                }
                vars_map.insert(key, value);
            } else {
                let env_value = env::var(&key).ok();
                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        format!("Action: Found ARG without default value: {}", key).yellow()
                    );
                    if let Some(ref val) = env_value {
                        println!(
                            "{} {}",
                            "DEBUG:".bright_blue().bold(),
                            format!("Action: Found environment value: {}", val).yellow()
                        );
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
                let value = if input.is_empty() {
                    if let Some(env_val) = env_value {
                        if debug_enabled {
                            println!(
                                "{} {}",
                                "DEBUG:".bright_blue().bold(),
                                "Action: Using environment value".green()
                            );
                        }
                        env_val
                    } else {
                        if debug_enabled {
                            println!(
                                "{} {}",
                                "DEBUG:".bright_blue().bold(),
                                "Action: No value provided".red()
                            );
                        }
                        eprintln!("Error: No value provided for ARG {}", key);
                        std::process::exit(1);
                    }
                } else {
                    if debug_enabled {
                        println!(
                            "{} {}",
                            "DEBUG:".bright_blue().bold(),
                            format!("Action: Using provided value: {}", input).green()
                        );
                    }
                    input.to_string()
                };

                if debug_enabled {
                    println!(
                        "{} {}",
                        "DEBUG:".bright_blue().bold(),
                        format!("Action: Setting ARG variable: {}={}", key, value).magenta()
                    );
                }
                vars_map.insert(key, value);
            };
        } else if !line.is_empty() && !line.starts_with('#') && debug_enabled {
            println!(
                "{} {}",
                "DEBUG:".bright_blue().bold(),
                format!("Original command: {}", line).bright_white()
            );
            println!(
                "{} {}",
                "DEBUG:".bright_blue().bold(),
                "Action: Ignoring unsupported instruction".red()
            );
        }
    }
}
