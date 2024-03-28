# opentelemetry-common-log

This is a project built with Rust and Cargo, mainly for handling OpenTelemetry and log related functionalities.  
Support for OpenTelemetry tracing and logging is provided, otlp jaeger and datadog are supported as tracing backends.

## Installation

First, you need to install Rust and Cargo. Then, you can clone and build this project with the following commands:

```bash
cargo add --git https://github.com/Starry9/opentelemetry-common-log --tag v0.1.0
```
The default tracing backend is otlp, if you want to use jaeger or datadog, you can add the corresponding feature flags:

```bash
cargo add --git https://github.com/Starry9/opentelemetry-common-log --tag v0.1.0 --features "jaeger"
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

