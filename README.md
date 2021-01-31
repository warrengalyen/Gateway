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
  - [Installation ▶](#installation-)
    - [Cargo 🦀](#cargo-)
    - [Deb package 📦](#deb-package-)
    - [RPM Package 📦](#rpm-package-)
    - [Chocolatey 🍫](#chocolatey-)
    - [Brew 🍻](#brew-)
  - [Usage ❓](#usage-)
    - [Address argument](#address-argument)
      - [How Password can be provided](#how-password-can-be-provided)
  - [Keybindings ⌨](#keybindings-)
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

Get `deb` package from [HERE](https://github.com/warrengalyen/Gateway/releases/download/latest/gateway_0.1.0_amd64.deb)
or run `wget https://github.com/warrengalyen/Gateway/releases/download/latest/gateway_0.1.0_amd64.deb`

then install through dpkg:

```sh
dpkg -i gateway_*.deb
# Or even better with gdebi
gdebi gateway_*.deb
```

### RPM Package 📦

Get `rpm` package from [HERE](https://github.com/warrengalyen/Gateway/releases/download/latest/gateway-0.1.0-1.x86_64.rpm)
or run `wget https://github.com/warrengalyen/Gateway/releases/download/latest/gateway-0.1.0-1.x86_64.rpm`

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

Alternatively you can download the ZIP file from [HERE](https://github.com/warrengalyen/Gateway/releases/download/latest/gateway.0.1.0.nupkg)

and then with PowerShell started with administrator previleges, run:

```ps
choco install gateway -s .
```

---

## Usage ❓

Gateway can be started with the following options:

- `-P, --password <password>` if address is provided, password will be this argument
- `-v, --version` Print version info
- `-h, --help` Print help page

Gateway can be started in two different mode, if no extra arguments is provided, Gateway will show the authentication form, where the user will be able to provide the parameters required to connect to the remote peer.

Alternatively, the user can provide an address as argument to skip the authentication form and starting directly the connection to the remote server.

### Address argument

The address argument has the following syntax:

```txt
[protocol]://[username@]<address>[:port]
```

Let's see some example of this particular syntax, since it's very comfortable and you'll probably going to use this instead of the other one...

- Connect using default protocol (sftp) to 192.168.1.31, port is default for this protocol (22); username is current user's name

    ```sh
    gateway 192.168.1.31
    ```

- Connect using default protocol (sftp) to 192.168.1.31, port is default for this protocol (22); username is `root`

    ```sh
    gateway root@192.168.1.31
    ```

- Connect using scp to 192.168.1.31, port is 4022; username is `omar`

    ```sh
    gateway scp://omar@192.168.1.31:4022
    ```

#### How Password can be provided

You have probably noticed, that, when providing the address as argument, there's no way to provide the password.
Password can be basically provided through 3 ways when address argument is provided:

- `-P, --password` option: just use this CLI option providing the password. I strongly unrecommend this method, since it's very unsecure (since you might keep the password in the shell history)
- Via `sshpass`: you can provide password via `sshpass`, e.g. `sshpass -f ~/.ssh/topsecret.key gateway wgalyen@192.168.1.31`
- You will be prompted for it: if you don't use any of the previous methods, you will be prompted for the password, as happens with the more classics tools such as `scp`, `ssh`, etc.

---

## Keybindings ⌨

| Key           | Command                                               |
|---------------|-------------------------------------------------------|
| `<ESC>`       | Disconnect from remote; return to authentication page |
| `<TAB>`       | Switch between log tab and explorer                   |
| `<BACKSPACE>` | Go to previous directory in stack                     |
| `<RIGHT>`     | Move to remote explorer tab                           |
| `<LEFT>`      | Move to local explorer tab                            |
| `<UP>`        | Move up in selected list                              |
| `<DOWN>`      | Move down in selected list                            |
| `<PGUP>`      | Move up in selected list by 8 rows                    |
| `<PGDOWN>`    | Move down in selected list by 8 rows                  |
| `<ENTER>`     | Enter directory                                       |
| `<SPACE>`     | Upload / download selected file                       |
| `<CTRL+D>`    | Make directory                                        |
| `<CTRL+G>`    | Go to supplied path                                   |
| `<CTRL+H>`    | Show help                                             |
| `<CTRL+Q>`    | Quit Gateway                                          |
| `<CTRL+R>`    | Rename file                                           |
| `<CTRL+U>`    | Go to parent directory                                |
| `<CANC>`      | Delete file                                           |


---

## Known issues 🧻

- Ftp:
  - Time in explorer is `1 Jan 1970`, but shouldn't be: that's because chrono can't parse date in a different locale. So if your server has a locale different from the one on your machine, it won't be able to parse the date.
  - Some servers don't work: yes, some kinds of ftp server don't work correctly, sometimes it won't display any files in the directories, other times uploading files will fail. Up to date, `vsftpd` is the only one server which I saw working correctly with Gateway. Am I going to solve this? I'd like to, but it's not my fault at all. Unfortunately [rust-ftp](https://github.com/mattnenterprise/rust-ftp) is an abandoned project (up to 2020), indeed I had to patch many stuff by myself. I'll try to solve these issues, but it will take a long time.
- Sftp:
  - sftp is much slower than scp: Okay this is an annoying issue, and again: not my fault. It seems there is an issue with [ssh2-rs](https://github.com/alexcrichton/ssh2-rs) library. If you want to stay up to date with the status of this issue, subscribe to [this issue](https://github.com/alexcrichton/ssh2-rs/issues/206)

---

## Upcoming Features 🧪

- **File viewer**: possibility to show in a popup the file content from the explorer.
---

## Contributions 🤙🏻

Contributions are welcome! 😉

If you think you can contribute to Gateway, please follow [Gateway's contributions guide](CONTRIBUTING.md)

## Changelog ⏳

View Gateway's changelog [HERE](CHANGELOG.md)

---

## Powered by 🚀

Gateway is powered by these aweseome projects:

- [bytesize](https://github.com/hyunsik/bytesize)
- [crossterm](https://github.com/crossterm-rs/crossterm)
- [rpassword](https://github.com/conradkleinespel/rpassword)
- [ssh2-rs](https://github.com/alexcrichton/ssh2-rs)
- [textwrap](https://github.com/mgeisler/textwrap)
- [tui-rs](https://github.com/fdehau/tui-rs)
- [whoami](https://github.com/libcala/whoami)


---

## License 📃

Licensed under the GNU GPLv3 (the "License"); you may not use this file except in compliance with the License. You may obtain a copy of the License at

<http://www.gnu.org/licenses/gpl-3.0.txt>

Unless required by applicable law or agreed to in writing, software distributed under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the License for the specific language governing permissions and limitations under the License.

You can read the entire license [HERE](LICENSE)