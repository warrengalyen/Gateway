/*
*
*   Copyright (C) 2021 Warren Galyen
*
* 	This file is part of "Gateway"
*
*   Gateway is free software: you can redistribute it and/or modify
*   it under the terms of the GNU General Public License as published by
*   the Free Software Foundation, either version 3 of the License, or
*   (at your option) any later version.
*
*   Gateway is distributed in the hope that it will be useful,
*   but WITHOUT ANY WARRANTY; without even the implied warranty of
*   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
*   GNU General Public License for more details.
*
*   You should have received a copy of the GNU General Public License
*   along with Gateway.  If not, see <http://www.gnu.org/licenses/>.
*
*/

// Deps
extern crate bytesize;
extern crate content_inspector;
extern crate crossterm;

// Locals
use super::{FileTransferActivity, FsEntry, InputMode, LogLevel, PopupType};
use crate::utils::fmt_millis;

// Ext
use bytesize::ByteSize;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;
use tui::style::Color;

impl FileTransferActivity {
    /// ### connect
    ///
    /// Connect to remote
    pub(super) fn connect(&mut self) {
        // Connect to remote
        match self.client.connect(
            self.params.address.clone(),
            self.params.port,
            self.params.username.clone(),
            self.params.password.clone(),
        ) {
            Ok(welcome) => {
                if let Some(banner) = welcome {
                    // Log welcome
                    self.log(
                        LogLevel::Info,
                        format!(
                            "Established connection with '{}': \"{}\"",
                            self.params.address, banner
                        )
                        .as_ref(),
                    );
                }
                // Set state to explorer
                self.input_mode = InputMode::Explorer;
                self.reload_remote_dir();
            }
            Err(err) => {
                // Set popup fatal error
                self.input_mode = InputMode::Popup(PopupType::Fatal(format!("{}", err)));
            }
        }
    }

    /// ### disconnect
    ///
    /// disconnect from remote
    pub(super) fn disconnect(&mut self) {
        // Show popup disconnecting
        self.input_mode = InputMode::Popup(PopupType::Alert(
            Color::Red,
            String::from("Disconnecting from remote..."),
        ));
        // Disconnect
        let _ = self.client.disconnect();
        // Quit
        self.disconnected = true;
    }

    /// ### disconnect_and_quit
    ///
    /// disconnect from remote and then quit
    pub(super) fn disconnect_and_quit(&mut self) {
        self.disconnect();
        self.quit = true;
    }

    /// ### reload_remote_dir
    ///
    /// Reload remote directory entries
    pub(super) fn reload_remote_dir(&mut self) {
        // Get current entries
        if let Ok(pwd) = self.client.pwd() {
            self.remote_scan(pwd.as_path());
        }
    }

    /// ### filetransfer_send
    ///
    /// Send fs entry to remote.
    /// If dst_name is Some, entry will be saved with a different name.
    /// If entry is a directory, this applies to directory only
    pub(super) fn filetransfer_send(
        &mut self,
        entry: &FsEntry,
        curr_remote_path: &Path,
        dst_name: Option<String>,
    ) {
        // Write popup
        let file_name: String = match entry {
            FsEntry::Directory(dir) => dir.name.clone(),
            FsEntry::File(file) => file.name.clone(),
        };
        self.input_mode = InputMode::Popup(PopupType::Wait(format!("Uploading \"{}\"", file_name)));
        // Draw
        self.draw();
        // Get remote path
        let mut remote_path: PathBuf = PathBuf::from(curr_remote_path);
        let remote_file_name: PathBuf = match dst_name {
            Some(s) => PathBuf::from(s.as_str()),
            None => PathBuf::from(file_name.as_str()),
        };
        remote_path.push(remote_file_name);
        // Match entry
        match entry {
            FsEntry::File(file) => {
                // Upload file
                // Try to open local file
                match self
                    .context
                    .as_ref()
                    .unwrap()
                    .local
                    .open_file_read(file.abs_path.as_path())
                {
                    Ok(mut fhnd) => match self.client.send_file(file, remote_path.as_path()) {
                        Ok(mut rhnd) => {
                            // Write file
                            let file_size: usize =
                                fhnd.seek(std::io::SeekFrom::End(0)).unwrap_or(0) as usize;
                            // rewind
                            if let Err(err) = fhnd.seek(std::io::SeekFrom::Start(0)) {
                                self.log_and_alert(
                                    LogLevel::Error,
                                    format!("Could not rewind local file: {}", err),
                                );
                            }
                            // Write remote file
                            let mut total_bytes_written: usize = 0;
                            // Set input state to popup progress
                            self.input_mode = InputMode::Popup(PopupType::Progress(format!(
                                "Uploading \"{}\"",
                                file_name
                            )));
                            // Reset transfer states
                            self.transfer.reset();
                            let mut last_progress_val: f64 = 0.0;
                            let mut last_input_event_fetch: Instant = Instant::now();
                            // While the entire file hasn't been completely written,
                            // Or filetransfer has been aborted
                            while total_bytes_written < file_size && !self.transfer.aborted {
                                // Handle input events (each 500ms)
                                if last_input_event_fetch.elapsed().as_millis() >= 500 {
                                    // Read events
                                    self.read_input_event();
                                    // Reset instant
                                    last_input_event_fetch = Instant::now();
                                }
                                // Read till you can
                                let mut buffer: [u8; 65536] = [0; 65536];
                                match fhnd.read(&mut buffer) {
                                    Ok(bytes_read) => {
                                        total_bytes_written += bytes_read;
                                        if bytes_read == 0 {
                                            continue;
                                        } else {
                                            let mut buf_start: usize = 0;
                                            while buf_start < bytes_read {
                                                // Write bytes
                                                match rhnd.write(&buffer[buf_start..bytes_read]) {
                                                    Ok(bytes) => {
                                                        buf_start += bytes;
                                                    }
                                                    Err(err) => {
                                                        self.log_and_alert(
                                                            LogLevel::Error,
                                                            format!(
                                                                "Could not write remote file: {}",
                                                                err
                                                            ),
                                                        );
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(err) => {
                                        self.log_and_alert(
                                            LogLevel::Error,
                                            format!("Could not read local file: {}", err),
                                        );
                                        break;
                                    }
                                }
                                // Increase progress
                                self.transfer.set_progress(total_bytes_written, file_size);
                                // Draw only if a significant progress has been made (performance improvement)
                                if last_progress_val < self.transfer.progress - 1.0 {
                                    // Draw
                                    self.draw();
                                    last_progress_val = self.transfer.progress;
                                }
                            }
                            // Finalize stream
                            if let Err(err) = self.client.on_sent(rhnd) {
                                self.log(
                                    LogLevel::Warn,
                                    format!("Could not finalize remote stream: \"{}\"", err)
                                        .as_str(),
                                );
                            }
                            self.log(
                                LogLevel::Info,
                                format!(
                                    "Saved file \"{}\" to \"{}\" (took {} seconds; at {}/s)",
                                    file.abs_path.display(),
                                    remote_path.display(),
                                    fmt_millis(self.transfer.started.elapsed()),
                                    ByteSize(self.transfer.bytes_per_second()),
                                )
                                .as_ref(),
                            );
                        }
                        Err(err) => {
                            self.log_and_alert(
                                LogLevel::Error,
                                format!(
                                    "Failed to upload file \"{}\": {}",
                                    file.abs_path.display(),
                                    err
                                ),
                            );
                        }
                    },
                    Err(err) => {
                        // Report error
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Failed to open file \"{}\": {}",
                                file.abs_path.display(),
                                err
                            ),
                        );
                    }
                }
            }
            FsEntry::Directory(dir) => {
                // Create directory on remote
                match self.client.mkdir(remote_path.as_path()) {
                    Ok(_) => {
                        self.log(
                            LogLevel::Info,
                            format!("Created directory \"{}\"", remote_path.display()).as_ref(),
                        );
                        // Get files in dir
                        match self
                            .context
                            .as_ref()
                            .unwrap()
                            .local
                            .scan_dir(dir.abs_path.as_path())
                        {
                            Ok(entries) => {
                                // Iterate over files
                                for entry in entries.iter() {
                                    // If aborted; break
                                    if self.transfer.aborted {
                                        break;
                                    }
                                    // Send entry; name is always None after first call
                                    self.filetransfer_send(&entry, remote_path.as_path(), None);
                                }
                            }
                            Err(err) => {
                                self.log_and_alert(
                                    LogLevel::Error,
                                    format!(
                                        "Could not scan directory \"{}\": {}",
                                        dir.abs_path.display(),
                                        err
                                    ),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Failed to create directory \"{}\": {}",
                                remote_path.display(),
                                err
                            ),
                        );
                    }
                }
            }
        }
        // Scan dir on remote
        if let Ok(path) = self.client.pwd() {
            self.remote_scan(path.as_path());
        }
        // If aborted; show popup
        if self.transfer.aborted {
            // Log abort
            self.log_and_alert(
                LogLevel::Warn,
                format!("Upload aborted for \"{}\"!", entry.get_abs_path().display()),
            );
            // Set aborted to false
            self.transfer.aborted = false;
        } else {
            // @! Successful
            // Eventually, Reset input mode to explorer (if input mode is wait or progress)
            if let InputMode::Popup(ptype) = &self.input_mode {
                if matches!(ptype, PopupType::Wait(_) | PopupType::Progress(_)) {
                    self.input_mode = InputMode::Explorer
                }
            }
        }
    }

    /// ### filetransfer_recv
    ///
    /// Recv fs entry from remote.
    /// If dst_name is Some, entry will be saved with a different name.
    /// If entry is a directory, this applies to directory only
    pub(super) fn filetransfer_recv(
        &mut self,
        entry: &FsEntry,
        local_path: &Path,
        dst_name: Option<String>,
    ) {
        // Write popup
        let file_name: String = match entry {
            FsEntry::Directory(dir) => dir.name.clone(),
            FsEntry::File(file) => file.name.clone(),
        };
        self.input_mode =
            InputMode::Popup(PopupType::Wait(format!("Downloading \"{}\"...", file_name)));
        // Draw
        self.draw();
        // Match entry
        match entry {
            FsEntry::File(file) => {
                // Get local file
                let mut local_file_path: PathBuf = PathBuf::from(local_path);
                let local_file_name: String = match dst_name {
                    Some(n) => n,
                    None => file.name.clone(),
                };
                local_file_path.push(local_file_name.as_str());
                // Try to open local file
                match self
                    .context
                    .as_ref()
                    .unwrap()
                    .local
                    .open_file_write(local_file_path.as_path())
                {
                    Ok(mut local_file) => {
                        // Download file from remote
                        match self.client.recv_file(file) {
                            Ok(mut rhnd) => {
                                // Set popup progress
                                self.input_mode = InputMode::Popup(PopupType::Progress(format!(
                                    "Downloading \"{}\"...",
                                    file_name
                                )));
                                let mut total_bytes_written: usize = 0;
                                // Reset transfer states
                                self.transfer.reset();
                                // Write local file
                                let mut last_progress_val: f64 = 0.0;
                                let mut last_input_event_fetch: Instant = Instant::now();
                                // While the entire file hasn't been completely read,
                                // Or filetransfer has been aborted
                                while total_bytes_written < file.size && !self.transfer.aborted {
                                    // Handle input events (each 500 ms)
                                    if last_input_event_fetch.elapsed().as_millis() >= 500 {
                                        // Read events
                                        self.read_input_event();
                                        // Reset instant
                                        last_input_event_fetch = Instant::now();
                                    }
                                    // Read till you can
                                    let mut buffer: [u8; 65536] = [0; 65536];
                                    match rhnd.read(&mut buffer) {
                                        Ok(bytes_read) => {
                                            total_bytes_written += bytes_read;
                                            if bytes_read == 0 {
                                                continue;
                                            } else {
                                                let mut buf_start: usize = 0;
                                                while buf_start < bytes_read {
                                                    // Write bytes
                                                    match local_file
                                                        .write(&buffer[buf_start..bytes_read])
                                                    {
                                                        Ok(bytes) => buf_start += bytes,
                                                        Err(err) => {
                                                            self.log_and_alert(
                                                                LogLevel::Error,
                                                                format!(
                                                                "Could not write local file: {}",
                                                                err
                                                            ),
                                                            );
                                                            break;
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                        Err(err) => {
                                            self.log_and_alert(
                                                LogLevel::Error,
                                                format!("Could not read remote file: {}", err),
                                            );
                                            break;
                                        }
                                    }
                                    // Set progress
                                    self.transfer.set_progress(total_bytes_written, file.size);
                                    // Draw only if a significant progress has been made (performance improvement)
                                    if last_progress_val < self.transfer.progress - 1.0 {
                                        // Draw
                                        self.draw();
                                        last_progress_val = self.transfer.progress;
                                    }
                                }
                                // Finalize stream
                                if let Err(err) = self.client.on_recv(rhnd) {
                                    self.log(
                                        LogLevel::Warn,
                                        format!("Could not finalize remote stream: \"{}\"", err)
                                            .as_str(),
                                    );
                                }
                                // Apply file mode to file
                                #[cfg(any(
                                    target_os = "unix",
                                    target_os = "macos",
                                    target_os = "linux"
                                ))]
                                if let Some(pex) = file.unix_pex {
                                    if let Err(err) = self
                                        .context
                                        .as_ref()
                                        .unwrap()
                                        .local
                                        .chmod(local_file_path.as_path(), pex)
                                    {
                                        self.log(
                                            LogLevel::Error,
                                            format!(
                                                "Could not apply file mode {:?} to \"{}\": {}",
                                                pex,
                                                local_file_path.display(),
                                                err
                                            )
                                            .as_ref(),
                                        );
                                    }
                                }
                                // Log
                                self.log(
                                    LogLevel::Info,
                                    format!(
                                        "Saved file \"{}\" to \"{}\" (took {} seconds; at {}/s)",
                                        file.abs_path.display(),
                                        local_file_path.display(),
                                        fmt_millis(self.transfer.started.elapsed()),
                                        ByteSize(self.transfer.bytes_per_second()),
                                    )
                                    .as_ref(),
                                );
                            }
                            Err(err) => {
                                self.log_and_alert(
                                    LogLevel::Error,
                                    format!(
                                        "Failed to download file \"{}\": {}",
                                        file.abs_path.display(),
                                        err
                                    ),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        // Report error
                        self.log_and_alert(
                            LogLevel::Error,
                            format!(
                                "Failed to open local file for write \"{}\": {}",
                                local_file_path.display(),
                                err
                            ),
                        );
                    }
                }
            }
            FsEntry::Directory(dir) => {
                // Get dir name
                let mut local_dir_path: PathBuf = PathBuf::from(local_path);
                match dst_name {
                    Some(name) => local_dir_path.push(name),
                    None => local_dir_path.push(dir.name.as_str()),
                }
                // Create directory on local
                match self
                    .context
                    .as_mut()
                    .unwrap()
                    .local
                    .mkdir_ex(local_dir_path.as_path(), true)
                {
                    Ok(_) => {
                        // Apply file mode to directory
                        #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
                        if let Some(pex) = dir.unix_pex {
                            if let Err(err) = self
                                .context
                                .as_ref()
                                .unwrap()
                                .local
                                .chmod(local_dir_path.as_path(), pex)
                            {
                                self.log(
                                    LogLevel::Error,
                                    format!(
                                        "Could not apply file mode {:?} to \"{}\": {}",
                                        pex,
                                        local_dir_path.display(),
                                        err
                                    )
                                    .as_ref(),
                                );
                            }
                        }
                        self.log(
                            LogLevel::Info,
                            format!("Created directory \"{}\"", local_dir_path.display()).as_ref(),
                        );
                        // Get files in dir
                        match self.client.list_dir(dir.abs_path.as_path()) {
                            Ok(entries) => {
                                // Iterate over files
                                for entry in entries.iter() {
                                    // If transfer has been aborted; break
                                    if self.transfer.aborted {
                                        break;
                                    }
                                    // Receive entry; name is always None after first call
                                    // Local path becomes local_dir_path
                                    self.filetransfer_recv(&entry, local_dir_path.as_path(), None);
                                }
                            }
                            Err(err) => {
                                self.log_and_alert(
                                    LogLevel::Error,
                                    format!(
                                        "Could not scan directory \"{}\": {}",
                                        dir.abs_path.display(),
                                        err
                                    ),
                                );
                            }
                        }
                    }
                    Err(err) => {
                        self.log(
                            LogLevel::Error,
                            format!(
                                "Failed to create directory \"{}\": {}",
                                local_dir_path.display(),
                                err
                            )
                            .as_ref(),
                        );
                    }
                }
            }
        }
        // Reload directory on local
        self.local_scan(local_path);
        // if aborted; show alert
        if self.transfer.aborted {
            // Log abort
            self.log_and_alert(
                LogLevel::Warn,
                format!(
                    "Download aborted for \"{}\"!",
                    entry.get_abs_path().display()
                ),
            );
            // Reset aborted to false
            self.transfer.aborted = false;
        } else {
            // Eventually, Reset input mode to explorer
            self.input_mode = InputMode::Explorer;
        }
    }

    /// ### local_scan
    ///
    /// Scan current local directory
    pub(super) fn local_scan(&mut self, path: &Path) {
        match self.context.as_ref().unwrap().local.scan_dir(path) {
            Ok(files) => {
                self.local.files = files;
                // Set index; keep if possible, otherwise set to last item
                self.local.index = match self.local.files.get(self.local.index) {
                    Some(_) => self.local.index,
                    None => match self.local.files.len() {
                        0 => 0,
                        _ => self.local.files.len() - 1,
                    },
                };
                // Sort files
                self.local.sort_files_by_name();
            }
            Err(err) => {
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not scan current directory: {}", err),
                );
            }
        }
    }

    /// ### remote_scan
    ///
    /// Scan current remote directory
    pub(super) fn remote_scan(&mut self, path: &Path) {
        match self.client.list_dir(path) {
            Ok(files) => {
                self.remote.files = files;
                // Set index; keep if possible, otherwise set to last item
                self.remote.index = match self.remote.files.get(self.remote.index) {
                    Some(_) => self.remote.index,
                    None => match self.remote.files.len() {
                        0 => 0,
                        _ => self.remote.files.len() - 1,
                    },
                };
                // Sort files
                self.remote.sort_files_by_name();
            }
            Err(err) => {
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not scan current directory: {}", err),
                );
            }
        }
    }

    /// ### local_changedir
    ///
    /// Change directory for local
    pub(super) fn local_changedir(&mut self, path: &Path, push: bool) {
        // Get current directory
        let prev_dir: PathBuf = self.context.as_ref().unwrap().local.pwd();
        // Change directory
        match self
            .context
            .as_mut()
            .unwrap()
            .local
            .change_wrkdir(PathBuf::from(path))
        {
            Ok(_) => {
                self.log(
                    LogLevel::Info,
                    format!("Changed directory on local: {}", path.display()).as_str(),
                );
                // Reload files
                self.local_scan(path);
                // Reset index
                self.local.index = 0;
                // Push prev_dir to stack
                if push {
                    self.local.pushd(prev_dir.as_path())
                }
            }
            Err(err) => {
                // Report err
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not change working directory: {}", err),
                );
            }
        }
    }

    pub(super) fn remote_changedir(&mut self, path: &Path, push: bool) {
        // Get current directory
        match self.client.pwd() {
            Ok(prev_dir) => {
                // Change directory
                match self.client.change_dir(path) {
                    Ok(_) => {
                        self.log(
                            LogLevel::Info,
                            format!("Changed directory on remote: {}", path.display()).as_str(),
                        );
                        // Update files
                        self.remote_scan(path);
                        // Reset index
                        self.remote.index = 0;
                        // Push prev_dir to stack
                        if push {
                            self.remote.pushd(prev_dir.as_path())
                        }
                    }
                    Err(err) => {
                        // Report err
                        self.log_and_alert(
                            LogLevel::Error,
                            format!("Could not change working directory: {}", err),
                        );
                    }
                }
            }
            Err(err) => {
                // Report err
                self.log_and_alert(
                    LogLevel::Error,
                    format!("Could not change working directory: {}", err),
                );
            }
        }
    }

    /// ### edit_file
    ///
    /// Edit a file on localhost
    pub(super) fn edit_file(&mut self, path: &Path) {
        // Read first 2048 bytes or less from file to check if it is textual
        match OpenOptions::new().read(true).open(path) {
            Ok(mut f) => {
                // Read
                let mut buff: [u8; 2048] = [0; 2048];
                match f.read(&mut buff) {
                    Ok(_) => {
                        if content_inspector::inspect(&buff).is_binary() {
                            self.log_and_alert(
                                LogLevel::Error,
                                format!("Could not open file in editor: file is binary"),
                            );
                        }
                    }
                    Err(err) => {
                        self.log_and_alert(
                            LogLevel::Error,
                            format!("Could not read file: {}", err),
                        );
                        return;
                    }
                }
            }
            Err(err) => {
                self.log_and_alert(LogLevel::Error, format!("Could not read file: {}", err));
                return;
            }
        }
        // Put input mode back to normal
        let _ = disable_raw_mode();
        // Open editor
        if let Err(err) = edit::edit_file(path) {
            self.log_and_alert(LogLevel::Error, format!("Could not open editor: {}", err));
        }
        // Re-enable raw mode
        let _ = enable_raw_mode();
    }
}
