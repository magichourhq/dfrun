use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dockerfile(content: &str, test_name: &str) -> (PathBuf, PathBuf) {
        // Create temp directory if it doesn't exist
        let temp_dir = PathBuf::from("temp");
        if !temp_dir.exists() {
            fs::create_dir(&temp_dir).expect("Failed to create temp directory");
        }

        // Create a test directory in the temp directory with unique name
        let test_dir = temp_dir.join(format!("test_{}", test_name));

        // Try to remove the directory if it exists, with retries
        if test_dir.exists() {
            let mut retries = 3;
            while retries > 0 {
                match fs::remove_dir_all(&test_dir) {
                    Ok(_) => break,
                    Err(e) => {
                        println!("Failed to remove directory, retrying... Error: {}", e);
                        thread::sleep(Duration::from_millis(100));
                        retries -= 1;
                    }
                }
            }
            if retries == 0 {
                panic!("Failed to remove existing test directory after multiple attempts");
            }
        }

        // Create the test directory
        fs::create_dir(&test_dir).expect("Failed to create test directory");

        // Create the Dockerfile
        let dockerfile_path = test_dir.join("Dockerfile");
        let mut file = File::create(&dockerfile_path).expect("Failed to create Dockerfile");
        file.write_all(content.as_bytes())
            .expect("Failed to write to Dockerfile");

        (test_dir, dockerfile_path)
    }

    fn cleanup_test_dir(test_dir: PathBuf) {
        let mut retries = 3;
        while retries > 0 {
            match fs::remove_dir_all(&test_dir) {
                Ok(_) => break,
                Err(e) => {
                    println!("Failed to remove directory, retrying... Error: {}", e);
                    thread::sleep(Duration::from_millis(100));
                    retries -= 1;
                }
            }
        }
        if retries == 0 {
            panic!("Failed to clean up test directory after multiple attempts");
        }
    }

    #[test]
    fn test_parse_arg_with_default() {
        let (test_dir, dockerfile_path) =
            create_test_dockerfile("ARG VERSION=1.0.0", "arg_default");
        println!("Dockerfile path: {:?}", dockerfile_path);

        let mut child = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        // Send empty input to use default value
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(b"\n").expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to wait on child");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command stderr: {}", stderr);

        assert!(output.status.success());

        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_parse_env() {
        let (test_dir, dockerfile_path) = create_test_dockerfile("ENV TEST_VAR=test_value", "env");
        println!("Dockerfile path: {:?}", dockerfile_path);

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command stderr: {}", stderr);

        assert!(output.status.success());

        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_parse_run_command() {
        let (test_dir, dockerfile_path) = create_test_dockerfile("RUN echo 'test'", "run");
        println!("Dockerfile path: {:?}", dockerfile_path);

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command stderr: {}", stderr);

        assert!(output.status.success());

        // Clean up
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_parse_add_url() {
        let (test_dir, dockerfile_path) =
            create_test_dockerfile("ADD https://example.com/file.txt ./file.txt", "add_url");
        println!("Dockerfile path: {:?}", dockerfile_path);

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command stderr: {}", stderr);

        assert!(output.status.success());

        // Clean up
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_workdir() {
        let dockerfile_content = r#"WORKDIR temp/test_workdir
RUN pwd
RUN mkdir new_folder && cd new_folder && touch file.txt
RUN pwd"#;

        let (test_dir, dockerfile_path) = create_test_dockerfile(dockerfile_content, "workdir");
        println!("Dockerfile path: {:?}", dockerfile_path);

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command stderr: {}", stderr);

        assert!(output.status.success());

        // Extract pwd outputs from stdout
        let lines: Vec<&str> = stdout.lines().collect();
        let pwd_outputs: Vec<&str> = lines.iter().map(|line| line.trim()).collect();

        // Verify both pwd commands show the same directory
        assert_eq!(pwd_outputs.len(), 2, "Expected two pwd outputs");
        assert_eq!(
            pwd_outputs[0], pwd_outputs[1],
            "pwd outputs should be the same: '{}' vs '{}'",
            pwd_outputs[0], pwd_outputs[1]
        );

        // Verify the file was created in the correct location
        let file_path = test_dir.join("new_folder/file.txt");
        println!("Checking file path: {:?}", file_path);
        assert!(
            file_path.exists(),
            "file.txt should exist in the correct path: {:?}",
            file_path
        );

        // Clean up
        cleanup_test_dir(test_dir);
    }
}
