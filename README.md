# Gateway

[![Build](https://github.com/warrengalyen/Gateway/workflows/Linux/badge.svg)](https://github.com/warrengalyen/Linux/actions)
[![Build](https://github.com/warrengalyen/Gateway/workflows/MacOS/badge.svg)](https://github.com/warrengalyen/MacOS/actions)
[![Build](https://github.com/warrengalyen/Gateway/workflows/Windows/badge.svg)](https://github.com/warrengalyen/Windows/actions)


Basically, WinSCP on a terminal

⚠ This project is **still under development**; installation instructions won't work until release ⚠

---

- [Gateway](#gateway)
  - [About Gateway 🖥](#about-gateway-)
    - [Why Gateway 🤔](#why-gateway-)
  - [Features 🎁](#features-)
  - [Usage ❓](#usage-)
  - [Installation ▶](#installation-)
    - [Cargo 🦀](#cargo-)
    - [Deb package 📦](#deb-package-)
    - [RPM Package 📦](#rpm-package-)
    - [Chocolatey 🍫](#chocolatey-)
    - [Brew 🍻](#brew-)
  - [Documentation 📚](#documentation-)
  - [Known issues 🧻](#known-issues-)
  - [Upcoming Features 🧪](#upcoming-features-)
  - [Contributions 🤙🏻](#contributions-)
  - [Changelog ⏳](#changelog-)
  - [License 📃](#license-)

---

## About Gateway 🖥

Gateway is basically a porting of WinSCP to terminal. So basically is a terminal utility with an TUI to connect to a remote server to retrieve and upload files and to interact with the local file system. It works both on **Linux**, **MacOS**, **UNIX** and **Windows** and supports SFTP, SCP, FTP and FTPS.

### Why Gateway 🤔

It happens very often to me when using SCP at work to forget the path of a file on a remote machine, which forces me then to connect through SSH, gather the file path and finally download it through SCP. I could use WinSCP, but I use Linux and I pratically use the terminal for everything, so I wanted something like WinSCP on my terminal.

## Features 🎁

- Different communication protocols
  - SFTP
  - SCP
  - FTP and FTPS
- Practical user interface to explore and operate on the remote and on the local machine file system
- Compatible with Windows, Linux, UNIX and MacOS
- Written in Rust
- Easy to extend with new file transfers protocols

## Usage ❓

Gateway can be started with the following options:

- `-v, --version` Print version info
- `-h, --help` Print help page

---
## Installation ▶

If you're considering to install Gateway I want to thank you 💛! I hope this project can be useful for you!  
If you want to contribute to this project, don't forget to check out our contribute guide. [Read More](CONTRIBUTING.md)

### Cargo 🦀

```sh
# Install gateway through cargo
cargo install gateway
```

### Deb package 📦

Get `deb` package from [HERE](https://github.com/warrengalyen/Gateway/releases/download/v0.1.0/gateway_0.1.0_amd64.deb)
or run `wget https://github.com/warrengalyen/Gateway/releases/download/v0.1.0/gateway_0.1.0_amd64.deb`

then install through dpkg:

```sh
dpkg -i gateway_*.deb
# Or even better with gdebi
gdebi gateway_*.deb
```

### RPM Package 📦

Get `rpm` package from [HERE](https://github.com/warrengalyen/Gateway/releases/download/v0.1.0/gateway-0.1.0-1.x86_64.rpm)
or run `wget https://github.com/warrengalyen/Gateway/releases/download/v0.1.0/gateway-0.1.0-1.x86_64.rpm`

then install through rpm:

```sh
rpm -U gateway_*.rpm
```

### Chocolatey 🍫

You can install Gateway on Windows using [chocolatey](https://chocolatey.org/)

Start PowerShell as administrator and run

```ps
choco install gateway
```

Alternatively you can download the ZIP file from [HERE](https://github.com/warrengalyen/Gateway/releases/download/v0.1.0/gateway.0.1.0.nupkg)

and then with PowerShell started with administrator previleges, run:

```ps
choco install gateway -s .
```

### Brew 🍻

---

## Known issues 🧻

- Ftp:
  - Time in explorer is `1 Jan 1970`, but shouldn't be: that's because chrono can't parse date in a different locale. So if your server has a locale different from the one on your machine, it won't be able to parse the date.

---

## Upcoming Features 🧪

- **File viewer**: possibility to show in a popup the file content from the explorer.
---

## Contributions 🤙🏻

Contributions are welcome! 😉

If you think you can contribute to Gateway, please follow [Gateway's contributions guide](CONTRIBUTING.md)

## Changelog ⏳

See the entire changelog [HERE](CHANGELOG.md)

---

## License 📃

Licensed under the GNU GPLv3 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

<http://www.gnu.org/licenses/gpl-3.0.txt>

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

You can read the entire license [HERE](LICENSE)