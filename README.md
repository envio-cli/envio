<div align="center">
  <a href="https://irusa.org/middle-east/palestine/">
    <img src="assets/palestine-banner.png" width="800px">
  </a>
</div>

<div align="center">
  <img src="assets/icon.svg" width="200px">
  <h1>envio</h1>
</div>

<div align="center">
  <h2>A secure command-line tool for managing environment variables</h2>
  <a href="https://github.com/humblepenguinn/envio/workflows/CICD.yml">
    <img src="https://github.com/humblepenguinn/envio/actions/workflows/CICD.yml/badge.svg" alt="CICD">
  </a>
  <a href="https://crates.io/crates/envio">
    <img src="https://img.shields.io/crates/v/envio.svg" alt="Version info">
  </a>
</div>

<div align="center" style="margin: 24px 0;">
  <img src="assets/demo.svg" alt="Demo" width="80%">
</div>


## About

`envio` is a command-line tool for securely managing environment variables. It allows users to create encrypted profiles containing environment variables for a specific project or use case. The tool provides various operations to manage these profiles, such as loading them into terminal sessions or running programs with the specified environment variables.

Some key features of `envio` include:

- **Encrypt** profiles using different encryption methods
- **Load** profiles into your terminal sessions
- **Run** programs with your profiles
- **Import** profiles stored on the internet
- **Export** profiles to a plain text file

## Installation

Pre-built binaries are available on the [releases page](https://github.com/envio-cli/envio/releases).

### Linux

**Arch Linux**

Use your favorite AUR helper to install `envio`:

```bash
paru -S envio      # or envio-bin for pre-built binary
```

**Debian/Ubuntu**

```bash
sudo dpkg -i envio_<version>_<arch>.deb
```

### macOS

```bash
brew install envio
```

### Windows

Download the MSI installer or zip archive from the [releases page](https://github.com/envio-cli/envio/releases).

## Usage

See the [Usage Guide](docs/usage.md) for detailed instructions on how to use the tool.

## Contributing

Take a look at the [Contributing Guide](CONTRIBUTING.md) for more information.

## License

`envio` is available under the terms of either the MIT License or the Apache License 2.0, at your option.

See the [LICENSE-MIT](LICENSE-MIT) and [LICENSE-APACHE](LICENSE-APACHE) files for license details.
