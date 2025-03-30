# dfrun - Dockerfile Runner

`dfrun` is a simple command-line tool that runs Dockerfile instructions directly as shell commands. 

It's useful for getting setup on a VM without docker. For example, when using Remote SSH with VSCode.

## Features

- 🚀 Runs Dockerfile commands directly in your shell
- 🔄 Supports multi-line RUN commands
- 📥 Handles ADD commands for downloading files
- 🌍 Sets ENV variables
- 💬 Interactive ARG prompts with default values
- 📁 Respects WORKDIR instructions
- 🐛 Debug mode for troubleshooting

## Installation

### From Releases
Download the latest release for your platform from the [releases page](https://github.com/magichourhq/dfrun/releases).

#### Linux/macOS
```bash
# Download and extract
tar -xzf dfrun-<target>.tar.gz
# Move to a directory in your PATH
sudo mv dfrun /usr/local/bin/
```

#### Windows
```powershell
# Extract the archive and add to your PATH
```

### From Source
```bash
cargo install --git https://github.com/magichourhq/dfrun
```

## Usage

Basic usage:
```bash
cd /dir/with/Dockerfile
dfrun
```

With a specific Dockerfile:
```bash
dfrun -f path/to/Dockerfile
```

With debug output:
```bash
dfrun -d
```

### Supported Dockerfile Instructions

- `RUN`: Executes shell commands
  ```dockerfile
  RUN echo "Hello, World!"
  ```

- `ENV`: Sets environment variables
  ```dockerfile
  ENV MY_VAR=value
  ```

- `ARG`: Prompts for values with optional defaults
  ```dockerfile
  ARG VERSION=1.0
  ARG USER
  ```

- `ADD`: Downloads files from URLs
  ```dockerfile
  ADD https://example.com/file.txt .
  ```

- `WORKDIR`: Changes working directory
  ```dockerfile
  WORKDIR /app
  ```

## Development

Requirements:
- Rust 1.70 or later
- Cargo

Setup:
```bash
# Clone the repository after forking
git clone https://github.com/magichourhq/dfrun
cd dfrun

# Install locally
cargo install --path .

# Build
cargo build

# Run tests
cargo test
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 