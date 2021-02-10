# Changelog

- [Changelog](#changelog)
  - [0.2.0](#020)
  - [0.1.4](#014)
  - [0.1.3](#012)
  - [0.1.1](#011)
  - [0.1.0](#010)

---

## 0.2.0

- **Bookmarks**
  - Bookmarks and recent connections are now displayed in the home page
  - Bookmarks are saved at
    - Linux: `/home/alice/.config/gateway/bookmarks.toml`
    - Windows: `C:\Users\Alice\AppData\Roaming\gateway\bookmarks.toml`
    - MacOS: `/Users/Alice/Library/Application Support/gateway/bookmarks.toml`
- Bugfix:
  - File mode of file on remote is now reported on local file after being downloaded (unix, linux, macos only)

## 0.1.4

- Enhancements:
  - File transfer:
    - Read buffer is now 65536 bytes long
  - File explorer:
    - Fixed color mismatch in local explorer
    - Explorer tabs have now 70% of layout height, while logging area is 30%
    - Highlight selected entry in tabs, only when the tab is active
  - Auth page:
    - align popup text to center
  - Keybindings:
    - `L`: Refresh directory content
- Bugfix:
  - Fixed memory vulnerability in Windows version

## 0.1.3

- General performance and code improvements
- Improved symlinks management
- Possibility to abort file transfers
- Enhancements:
  - File explorer:
    - When file index is at the end of the list, moving down will set the current index to the first element and viceversa.
    - Selected file has now colorful background, instead of foreground, for a better readability.
- Keybindings:
  - `E`: Delete file (Same as `DEL`); added because some keyboards don't have `DEL` (hey, that's my MacBook Air's keyboard!)
  - `Ctrl+C`: Abort transfer process

## 0.1.1

- enhancements:
  - password prompt: ask before performing terminal clear
  - file explorer:
    - file names are now sorted ignoring capital letters
    - file names longer than 23, are now cut to 20 and followed by `...`
    - paths which exceed tab size in explorer are omitted with the following format `ANCESTOR[1]/.../PARENT/DIRNAME`
- keybindings:
  - `I`: show info about selected file or directory
  - Removed `CTRL`; just use keys now.
- bugfix:
  - prevent panic in set_progress, for progress values `> 100.0 or < 0.0`
  - Fixed FTP get, which didn't finalize the reader
- dependencies:
  - updated `textwrap` to `0.13.0`

## 0.1.0

- First release