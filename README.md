# PiRelay

A high-performance file transfer client written in Rust for sending and receiving files through a relay server. PiRelay supports both small and large file transfers with automatic chunking for files larger than 5MB.

## Why PiRelay?

PiRelay was designed to simplify peer-to-peer file transfers through a centralized relay server. It provides:

- **Efficient large file handling**: Automatically chunks files >5MB into 1MB pieces for reliable transfer
- **Directory support**: Send entire directories recursively
- **Flexible recipient targeting**: Send to specific users or receive from specific senders
- **Mailbox functionality**: Receive files from a general mailbox when sender is unspecified
- **HTTPS support**: Secure transfers using the reqwest client
- **Async operations**: Built on Tokio for non-blocking I/O

## Features

- Send files and directories to specific recipients
- Receive files from specific senders or from a general mailbox
- Automatic file chunking for large files (>5MB)
- Multipart form data support for small files
- Configurable connection settings (URL, IP, port, hostname)
- Debug mode for troubleshooting
- Custom output directory for received files
- Configuration file support via JSON

## Dependencies

PiRelay requires the following Rust dependencies:

- `reqwest` 0. - HTTP client with JSON and multipart support
- `tokio` 1.0 - Async runtime
- `serde` 1.0 - Serialization framework
- `serde_json` 1.0 - JSON serialization
- `dotenvy` 0.15.7 - Environment variable management

## Installation

### Prerequisites

- Rust 1.70 or later (uses edition 2021)
- Cargo package manager

### Build from source

```bash
# Clone the repository
git clone <repository-url>
cd PiRelay

# Build the project
cargo build --release

# The binary will be available at target/release/pirelay
```

### Install locally

```bash
cargo install --path .
```

## Usage

### Basic Syntax

```bash
pirelay <METHOD> [OPTIONS]
```

### Methods

- `SEND` - Send files to recipients
- `RECEIVE` - Receive files from senders

### Options

| Option | Description |
|--------|-------------|
| `-d, --debug` | Enable debug mode |
| `-s=<path>, --src=<path>` | Output directory for received files |
| `-conf=<path>, --config=<path>` | Path to config JSON file |
| `-c=(url,ip,port,hostname), --conn=(...)` | Connection settings |
| `--send=recipient=(file1,file2,...)` | Send files to recipient |
| `--receive=(sender1,sender2,...)` | Receive from specific senders |

## Examples

### Send a single file

```bash
pirelay SEND --send=alice=(document.pdf)
```

### Send multiple files to a recipient

```bash
pirelay SEND --send=alice=(file1.txt,file2.pdf,image.png)
```

### Send a directory

```bash
pirelay SEND --send=bob=(./my_folder)
```

### Send to multiple recipients

```bash
pirelay SEND --send=alice=(file.txt) --send=bob=(document.pdf)
```

### Receive from specific senders

```bash
pirelay RECEIVE --receive=(alice,bob)
```

### Receive from mailbox (all senders)

```bash
pirelay RECEIVE
```

### Receive to specific directory

```bash
pirelay RECEIVE -s=./downloads
```

### With debug mode

```bash
pirelay SEND --send=alice=(file.txt) -d
```

### With custom connection settings

```bash
pirelay SEND --send=alice=(file.txt) -c=(https://relay.example.com,192.168.1.100,5623,relay-server)
```

### With config file

```bash
pirelay SEND -conf=config.json --send=alice=(file.txt)
```

## Configuration File

You can provide a JSON configuration file using the `--config` or `-conf` option. The config file should contain valid JSON that will be sent to the relay server.

Example config.json:
```json
{
  "max_file_size": 104857600,
  "allowed_extensions": [".txt", ".pdf", ".png"],
  "compression": true
}
```

## Connection Settings

The connection can be configured via the `--conn` or `-c` option with the following format:

```
-c=(url,ip,port,hostname)
```

- `url`: Base URL for the relay server (e.g., `https://relay.example.com`)
- `ip`: IP address of the relay server
- `port`: Port number (default: 5623)
- `hostname`: Server hostname

## File Transfer Details

### Small Files (≤5MB)
Small files are sent using multipart form data in a single request.

### Large Files (>5MB)
Large files are automatically split into 1MB chunks and sent sequentially. Each chunk includes:
- File ID
- Chunk index
- Total chunk count
- Sender username
- Relative path

### Directory Transfers
When sending a directory, PiRelay recursively traverses the directory structure and maintains the relative path for each file.

## API Endpoints

PiRelay communicates with a relay server using the following endpoints:

- `POST /config` - Send configuration to the server
- `POST /send/{recipient}/{file_id}` - Send small file
- `POST /send/{recipient}/{file_id}/{chunk_index}` - Send file chunk
- `GET /receive/` - Receive from mailbox
- `GET /receive/{sender}` - Receive from specific sender

## Environment Variables

PiRelay uses the `USER` environment variable to identify the sender. If not set, it defaults to "ANONYM".

## Development

### Run tests

```bash
cargo test
```

### Run in debug mode

```bash
cargo run -- SEND --send=alice=(test.txt) -d
```

### Build with optimizations

```bash
cargo build --release
```

## Docker Support

A `docker-compose.yml` file is included for containerized deployment.

```bash
docker-compose up
```

## License

This project is licensed under GPL-3.0-only. See the [LICENSE](LICENSE) file for details.

## Troubleshooting

### Connection Issues
- Ensure the relay server is running and accessible
- Check firewall settings for the specified port
- Verify connection settings (IP, port, hostname)

### File Transfer Failures
- Check file permissions
- Ensure sufficient disk space
- Use debug mode (`-d`) for detailed error messages

### Build Errors
- Ensure Rust version is 1.70 or later
- Run `cargo update` to update dependencies
- Check that all dependencies in `Cargo.toml` are available

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

## Makefile

A Makefile is provided for common operations:

```bash
make build    # Build the project
make run      # Run the project
make test     # Run tests
make clean    # Clean build artifacts
```
