<div align="center">
  <img alt="GitHub Workflow Status" src="https://img.shields.io/github/actions/workflow/status/kawaki-san/mirro-rs/ci.yml?label=build">
  <img alt="GitHub" src="https://img.shields.io/github/license/kawaki-san/mirro-rs">
</div>
<h3 align="center">$${\color{LightBlue}mirro-rs}$$</h3>
<p align="center">
A mirrorlist manager for Arch Linux systems
<br />
<a href="#usage"><strong>View usage examples »</strong></a>
<br />
<br />
<a href="https://github.com/kawaki-san/mirro-rs/issues">Report Bug</a>
·
<a href="https://github.com/kawaki-san/mirro-rs/issues">Request Feature</a>
</p>

![app](https://user-images.githubusercontent.com/70331483/209865273-81fcb593-c61d-46b9-9f2a-448da383782f.svg)

<p align="center">mirro-rs provides a TUI to help you better visualise managing your mirrorlist.</p>

## Features

- Sorting
  - Completion - The number of mirror checks (as a percentage) that have successfully connected and disconnected from the given URL. If this is below 100%, the mirror may be unreliable.
  - Score - It is currently calculated as (hours delay + average duration + standard deviation) / completion percentage. _Lower is better_.
  - Standard deviation - The standard deviation of the connect and retrieval time. A high standard deviation can indicate an unstable or overloaded mirror.
  - Delay - The mean value of last check − last sync for each check of this mirror URL. Due to the timing of mirror checks, any value under one hour should be viewed as ideal.
  - Rate - sort by download speed
- Filtering
  - Age
  - Country
  - ipv4, ipv6, isos
  - Protocol - `http`, `https` or `rsync`
  - Completion Percentage

## Getting Started

### Installation

`mirro-rs` is available in the AUR. If you're using `paru`:

```sh
paru -S mirro-rs-git
```

> **Note**
> By default, this enables [configuration](#configuration) through `toml` files. You should edit the `PKGBUILD` if you prefer another configuration format (or to disable configuration files altogether).

### Manual Compilation

- cargo

  You need to have `cargo` installed to build the application. The easiest way to set this up is installing `rustup`.

  ```sh
  pacman -S rustup
  ```

  Install a rust toolchain:

  ```sh
  rustup install stable
  ```

- git

  Clone the repository:

  ```sh
  git clone https://github.com/kawaki-san/mirro-rs
  ```

  You may then build the release target:

```sh
cargo build --release
```

### Usage

Pass the `-h` or `--help` flag to mirro-rs to view configuration parameters.
To preview `http` or `https` mirrors that were successfully synchronised in the last 24 hours and use `/home/user/mirrorlist` as an export location for the best (at max) 50:

```sh
mirro-rs --export 50 --protocols https --protocols http --age 24 --outfile "/home/user/mirrorlist"
```

To do the same but restrict the sources to be from France and the UK:

```sh
mirro-rs --export 50 --protocols https --protocols http --age 24 --outfile "/home/user/mirrorlist" -c France -c "United Kingdom"
```

#### Configuration

For convenience, mirro-rs optionally supports reading a configuration `[default: $XDG_CONFIG_HOME/mirro-rs/mirro-rs.toml]` for general preferences. If none is available, `[default: $XDG_CONFIG_HOME/mirro-rs.toml]` will be used. If both are available, the former takes priority.

For `toml` support:

```sh
cargo build --release --features toml
```

For `json` support:

```sh
cargo build --release --features json
```

Likewise, for `yaml` support:

```sh
cargo build --release --features yaml
```

> **Note**
> If you enable all configuration file features, if the configuration directory contains more than one valid file format, the order of priority goes from `toml` -> `json` -> `yaml`.

Sample configuration files are provided in the [example](example) folder.

A minimal `mirro-rs.toml` config file could look like:

```toml
cache-ttl = 24
timeout = 10
```

> **Note**
> Changing the configuration file at runtime will overwrite the parameters that were set as CLI arguments

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
