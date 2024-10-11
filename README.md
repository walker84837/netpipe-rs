# netpipe: a rust port of netcat

> [!WARNING]
> This project is currently under heavy development.

## Installation

Ensure you have the Rust compiler needed to compile this project. If you dislike building from a source, you can use the precompiled binaries.

``` console
cargo build --release
```

## Usage

``` console
.\netcat [OPTIONS] <file> --address <address> --port <port>
```

### Options

- `<file>`: Path to the file to be transferred (contents must be UTF-8).
- `--address <address>`: IP address for the connection (IPv4 and IPv6 are supported).
- `--port <port>`: Port for the connection (0-65536).
- `-v, --version <version>`: IP version (default is 4).

### Examples

Example of transferring a file to IPv4 address 192.168.1.100 on port 8080

``` console
.\netpipe file.txt --address 192.168.1.100 --port 8080
```

Example of transferring a file to IPv6 address [2001:db8::1] on port 8080

``` console
.\netpipe file.txt --address 2001:db8::1 --port 8080 --version 6
```

## Support

For help or issues, please [create an issue](https://github.com/walker88437/netpipe/issues).

## Roadmap

- [ ] Add support for binary files
- [ ] Add support for piping file contents
- [ ] Add listening support
- [ ] Integrate UDP support
- [ ] Implement other features present in GNU Netcat

## Contributing

Contributions are welcome! Please follow the guidelines in its [file](CONTRIBUTING.md).

## Authors and Acknowledgment

Sole maintainer: [walker84837](https://github.com/walker84837)

## License

Dual licensed under [MIT](LICENSE_MIT.md) and [Apache 2.0](LICENSE_APACHE.md) licenses.

## Project Status

As I'm the sole maintainer of this project, development is relatively slow, so
the project is not actively maintained. Feel free to step in as a maintainer if
interested.
