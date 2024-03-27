# opentelemetry-common-log

This is a project built with Rust and Cargo, mainly for handling OpenTelemetry and log related functionalities.

## Installation

First, you need to install Rust and Cargo. Then, you can clone and build this project with the following commands:

```bash
git clone https://github.com/username/opentelemetry-common-log.git
cd opentelemetry-common-log
cargo build
```

## Usage

In your Rust project, you can use this library as follows:

```rust
use opentelemetry_common_log::init_log;

fn main() {
    init_log("my_app", "info", "http://127.0.0.1:4317", true);
    // Your code here...
}
```

## Contributing

Any form of contribution is welcome! If you find any issues or have new feature suggestions, feel free to submit an issue or pull request.

## License

This project is licensed under the MIT License.

