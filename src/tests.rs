use crate::*;
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_dockerfile(content: &str) -> (PathBuf, PathBuf) {
        // Create a test directory in the current directory
        let test_dir = PathBuf::from("temp");
        if test_dir.exists() {
            fs::remove_dir_all(&test_dir).expect("Failed to remove existing test directory");
        }
        fs::create_dir(&test_dir).expect("Failed to create test directory");

        // Create the Dockerfile
        let dockerfile_path = test_dir.join("Dockerfile");
        let mut file = File::create(&dockerfile_path).expect("Failed to create Dockerfile");
        file.write_all(content.as_bytes())
            .expect("Failed to write to Dockerfile");

        (test_dir, dockerfile_path)
    }

    #[test]
    fn test_parse_arg_with_default() {
        let (test_dir, dockerfile_path) = create_test_dockerfile("ARG VERSION=1.0.0");
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

        // Clean up
        fs::remove_dir_all(test_dir).expect("Failed to clean up test directory");
    }

    #[test]
    fn test_parse_env() {
        let (test_dir, dockerfile_path) = create_test_dockerfile("ENV TEST_VAR=test_value");
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
        fs::remove_dir_all(test_dir).expect("Failed to clean up test directory");
    }

    #[test]
    fn test_parse_run_command() {
        let (test_dir, dockerfile_path) = create_test_dockerfile("RUN echo 'test'");
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
        fs::remove_dir_all(test_dir).expect("Failed to clean up test directory");
    }

    #[test]
    fn test_parse_add_url() {
        let (test_dir, dockerfile_path) =
            create_test_dockerfile("ADD https://example.com/file.txt ./file.txt");
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
        fs::remove_dir_all(test_dir).expect("Failed to clean up test directory");
    }
}
