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
        let test_dir = temp_dir.join(format!(
            "test_{}_{}",
            test_name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        ));

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
            create_test_dockerfile("ARG VERSION=1.0.0", "arg_with_default");
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
        let (test_dir, dockerfile_path) = create_test_dockerfile(
            "ADD https://example.com/file.txt ./temp/file.txt",
            "add_url",
        );
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
RUN mkdir new_folder && cd new_folder
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

        let lines: Vec<&str> = stdout.lines().collect();
        let pwd_outputs: Vec<&str> = lines.iter().map(|line| line.trim()).collect();

        assert_eq!(pwd_outputs.len(), 2, "Expected two pwd outputs");
        assert_eq!(
            pwd_outputs[0], pwd_outputs[1],
            "pwd outputs should be the same: '{}' vs '{}'",
            pwd_outputs[0], pwd_outputs[1]
        );

        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_arg_env_interaction() {
        let dockerfile_content = r#"ARG VERSION=1.0.0
ENV APP_VERSION=$VERSION
ENV BUILD_TYPE=release
RUN echo "Building version $APP_VERSION in $BUILD_TYPE mode"
RUN echo "VERSION=$VERSION" > version.txt
RUN echo "APP_VERSION=$APP_VERSION" >> version.txt
RUN echo "BUILD_TYPE=$BUILD_TYPE" >> version.txt"#;

        let (test_dir, dockerfile_path) = create_test_dockerfile(dockerfile_content, "arg_env");
        println!("Dockerfile path: {:?}", dockerfile_path);

        let mut child = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn command");

        // Send empty input to use default value for ARG
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(b"\n").expect("Failed to write to stdin");
        }

        let output = child.wait_with_output().expect("Failed to wait on child");

        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("Command output: {}", stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        println!("Command stderr: {}", stderr);

        assert!(output.status.success());

        // Verify the version.txt file was created and contains the correct values
        let version_file = PathBuf::from("version.txt");
        println!("Checking version file path: {:?}", version_file);
        assert!(version_file.exists(), "version.txt should exist");

        let version_content =
            fs::read_to_string(&version_file).expect("Failed to read version.txt");
        println!("Version file content: {}", version_content);

        let lines: Vec<&str> = version_content.lines().collect();
        assert_eq!(lines.len(), 3, "version.txt should have 3 lines");

        // Verify each line contains the expected value
        assert!(
            lines.iter().any(|line| *line == "VERSION=1.0.0"),
            "version.txt should contain VERSION=1.0.0"
        );
        assert!(
            lines.iter().any(|line| *line == "APP_VERSION=1.0.0"),
            "version.txt should contain APP_VERSION=1.0.0"
        );
        assert!(
            lines.iter().any(|line| *line == "BUILD_TYPE=release"),
            "version.txt should contain BUILD_TYPE=release"
        );

        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_env_with_equals_syntax() {
        // Test ENV with KEY=VALUE syntax (no space)
        let dockerfile_content = r#"ENV MY_VAR=hello_world
RUN echo "MY_VAR=$MY_VAR" > env_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "env_equals_syntax");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let env_file = PathBuf::from("env_test.txt");
        assert!(env_file.exists(), "env_test.txt should exist");

        let content = fs::read_to_string(&env_file).expect("Failed to read env_test.txt");
        assert!(
            content.contains("MY_VAR=hello_world"),
            "env_test.txt should contain MY_VAR=hello_world, got: {}",
            content
        );

        fs::remove_file(env_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_env_with_space_syntax() {
        // Test ENV with KEY VALUE syntax (space separated)
        let dockerfile_content = r#"ENV MY_VAR hello_world
RUN echo "MY_VAR=$MY_VAR" > env_space_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "env_space_syntax");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let env_file = PathBuf::from("env_space_test.txt");
        assert!(env_file.exists(), "env_space_test.txt should exist");

        let content = fs::read_to_string(&env_file).expect("Failed to read env_space_test.txt");
        assert!(
            content.contains("MY_VAR=hello_world"),
            "env_space_test.txt should contain MY_VAR=hello_world, got: {}",
            content
        );

        fs::remove_file(env_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_env_variable_expansion() {
        // Test that ENV expands variables from ARG
        let dockerfile_content = r#"ARG BASE_VERSION=2.0.0
ENV FULL_VERSION=${BASE_VERSION}-stable
RUN echo "FULL_VERSION=$FULL_VERSION" > expand_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "env_var_expansion");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("expand_test.txt");
        assert!(test_file.exists(), "expand_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read expand_test.txt");
        assert!(
            content.contains("FULL_VERSION=2.0.0-stable"),
            "expand_test.txt should contain FULL_VERSION=2.0.0-stable, got: {}",
            content
        );

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_env_overwrite() {
        // Test that ENV can overwrite a previous ENV value
        let dockerfile_content = r#"ENV VERSION=1.0.0
ENV VERSION=${VERSION}-updated
RUN echo "VERSION=$VERSION" > overwrite_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "env_overwrite");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("overwrite_test.txt");
        assert!(test_file.exists(), "overwrite_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read overwrite_test.txt");
        assert!(
            content.contains("VERSION=1.0.0-updated"),
            "overwrite_test.txt should contain VERSION=1.0.0-updated, got: {}",
            content
        );

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_multiline_run() {
        // Test multi-line RUN commands with backslash continuation
        let dockerfile_content = r#"RUN echo "line1" > multiline_test.txt && \
    echo "line2" >> multiline_test.txt && \
    echo "line3" >> multiline_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "multiline_run");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("multiline_test.txt");
        assert!(test_file.exists(), "multiline_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read multiline_test.txt");
        let lines: Vec<&str> = content.lines().collect();
        assert_eq!(lines.len(), 3, "multiline_test.txt should have 3 lines");
        assert_eq!(lines[0], "line1");
        assert_eq!(lines[1], "line2");
        assert_eq!(lines[2], "line3");

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_arg_from_environment() {
        // Test that ARG picks up value from environment variable
        let dockerfile_content = r#"ARG MY_ARG
RUN echo "MY_ARG=$MY_ARG" > arg_env_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "arg_from_env");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .env("MY_ARG", "from_environment")
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("arg_env_test.txt");
        assert!(test_file.exists(), "arg_env_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read arg_env_test.txt");
        assert!(
            content.contains("MY_ARG=from_environment"),
            "arg_env_test.txt should contain MY_ARG=from_environment, got: {}",
            content
        );

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_unsupported_instructions_ignored() {
        // Test that unsupported instructions are ignored without error
        let dockerfile_content = r#"FROM ubuntu:22.04
COPY . /app
EXPOSE 8080
CMD ["echo", "done"]
LABEL maintainer="test"
USER nobody
VOLUME /data
RUN echo "success" > unsupported_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "unsupported_instructions");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("unsupported_test.txt");
        assert!(test_file.exists(), "unsupported_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read unsupported_test.txt");
        assert!(
            content.contains("success"),
            "unsupported_test.txt should contain 'success', got: {}",
            content
        );

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_comments_ignored() {
        // Test that comments are properly ignored
        let dockerfile_content = r#"# This is a comment
ARG VERSION=1.0.0
# Another comment
ENV APP_VERSION=$VERSION
# Comment before RUN
RUN echo "VERSION=$APP_VERSION" > comment_test.txt"#;

        let (test_dir, dockerfile_path) =
            create_test_dockerfile(dockerfile_content, "comments_ignored");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("comment_test.txt");
        assert!(test_file.exists(), "comment_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read comment_test.txt");
        assert!(
            content.contains("VERSION=1.0.0"),
            "comment_test.txt should contain VERSION=1.0.0, got: {}",
            content
        );

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }

    #[test]
    fn test_nested_variable_expansion_in_run() {
        // Test that bash correctly expands nested variables in RUN
        let dockerfile_content = r#"ARG PREFIX=app
ARG SUFFIX=prod
RUN export COMBINED="${PREFIX}_${SUFFIX}" && echo "COMBINED=$COMBINED" > nested_test.txt"#;

        let (test_dir, dockerfile_path) = create_test_dockerfile(dockerfile_content, "nested_vars");

        let output = Command::new("cargo")
            .args(["run", "--", "-f", dockerfile_path.to_str().unwrap()])
            .output()
            .expect("Failed to execute command");

        assert!(output.status.success());

        let test_file = PathBuf::from("nested_test.txt");
        assert!(test_file.exists(), "nested_test.txt should exist");

        let content = fs::read_to_string(&test_file).expect("Failed to read nested_test.txt");
        assert!(
            content.contains("COMBINED=app_prod"),
            "nested_test.txt should contain COMBINED=app_prod, got: {}",
            content
        );

        fs::remove_file(test_file).ok();
        cleanup_test_dir(test_dir);
    }
}
