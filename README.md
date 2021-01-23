# Gateway

[![Lint](https://github.com/warrengalyen/Gateway/workflows/Lint/badge.svg)](https://github.com/warrengalyen/Lint/actions)
[![Build](https://github.com/warrengalyen/Gateway/workflows/Rust/badge.svg)](https://github.com/warrengalyen/Rust/actions)


Basically, WinSCP on a terminal

⚠ This project is **still under development**; installation instructions won't work until release ⚠

---

- [Gateway](#gateway)
  - [About Gateway](#about-gateway)
    - [Why Gateway](#why-gateway)
  - [Features](#features)
  - [Installation](#installation)
    - [Cargo](#cargo)
    - [Deb / Rpm](#deb--rpm)
    - [Usage](#usage)
  - [Documentation](#documentation)
  - [Known issues](#known-issues)
  - [Upcoming Features](#upcoming-features)
  - [Contributions](#contributions)
  - [Changelog](#changelog)
  - [License](#license)

---

## About Gateway

Gateway is basically a porting of WinSCP to terminal. So basically is a terminal tool with an UI to connect to a remote server to retrieve and upload files. It works both on Linux, MacOS and Windows (TODO: double check) and supports SFTP and FTPs.

### Why Gateway

It happens very often to me when using SCP at work to forget the path of a file on a remote machine, which forces me then to connect through SSH, gather the file path and finally download it through SCP. I could use WinSCP, but I use Linux and I pratically use the terminal for everything, so I wanted something like WinSCP on my terminal.

## Features

- Different communication protocols
  - SFTP
  - FTPS
- Practical user interface to explore the remote machine file system and to select the files to upload and download
- Written in Rust
- Easy to extend with new protocols

## Installation

If you're considering to install Gateway I want to thank you 💛! I hope this project can be useful for you!  
If you want to contribute to this project, don't forget to check out our contribute guide. [Read More](CONTRIBUTING.md)

### Cargo

```sh
# Install gateway through cargo
cargo install gateway
```

### Deb / Rpm

Coming soon

### Usage

Gateway can be started with the following options:

- `-v, --version` Print version info
- `-h, --help` Print help page

---

## Known issues

TODO:

---

## Upcoming Features

TODO:

---

## Contributions

Contributions are welcome! 😉

If you think you can contribute to Gateway, please follow [Gateway's contributions guide](CONTRIBUTING.md)

## Changelog

See the entire changelog [HERE](CHANGELOG.md)

---

## License

Licensed under the GNU GPLv3 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

<http://www.gnu.org/licenses/gpl-3.0.txt>

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

You can read the entire license [HERE](LICENSE)