//! ## FileTransferActivity
//!
//! `filetransfer_activiy` is the module which implements the Filetransfer activity, which is the main activity afterall

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

// This module is split into files, cause it's just too big
mod callbacks;
mod input;
mod layout;
mod misc;
mod session;

// Dependencies
extern crate chrono;
extern crate crossterm;
extern crate textwrap;
extern crate tui;
extern crate unicode_width;

// locals
use super::{Activity, Context};
use crate::filetransfer::FileTransferProtocol;

// File transfer
use crate::filetransfer::ftp_transfer::FtpFileTransfer;
use crate::filetransfer::scp_transfer::ScpFileTransfer;
use crate::filetransfer::sftp_transfer::SftpFileTransfer;
use crate::filetransfer::FileTransfer;
use crate::fs::FsEntry;

// Includes
use chrono::{DateTime, Local};
use crossterm::event::Event as InputEvent;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use std::collections::VecDeque;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tui::style::Color;

// Types
type DialogCallback = fn(&mut FileTransferActivity);
type OnInputSubmitCallback = fn(&mut FileTransferActivity, String);

/// ### FileTransferParams
///
/// Holds connection parameters for file transfers
pub struct FileTransferParams {
    pub address: String,
    pub port: u16,
    pub protocol: FileTransferProtocol,
    pub username: Option<String>,
    pub password: Option<String>,
}

/// ### InputField
///
/// Input field selected
#[derive(std::cmp::PartialEq)]
enum InputField {
    Explorer,
    Logs,
}

/// ### DialogYesNoOption
///
/// Current yes/no dialog option
#[derive(std::cmp::PartialEq, Clone)]
enum DialogYesNoOption {
    Yes,
    No,
}

/// ## PopupType
///
/// PopupType describes the type of popup
#[derive(Clone)]
enum PopupType {
    Alert(Color, String),                          // Block color; Block text
    Fatal(String),                                 // Must quit after being hidden
    FileInfo,                                      // Show info about current file
    Help,                                          // Show Help
    Input(String, OnInputSubmitCallback),          // Input description; Callback for submit
    Progress(String),                              // Progress block text
    Wait(String),                                  // Wait block text
    YesNo(String, DialogCallback, DialogCallback), // Yes, no callback
}

/// ## InputMode
///
/// InputMode describes the current input mode
/// Each input mode handle the input events in a different way
#[derive(Clone)]
enum InputMode {
    Explorer,
    Popup(PopupType),
}

/// ## FileExplorer
///
/// File explorer states
struct FileExplorer {
    pub wrkdir: PathBuf,         // Current directory
    pub index: usize,            // Selected file
    pub files: Vec<FsEntry>,     // Files in directory
    dirstack: VecDeque<PathBuf>, // Stack of visited directory (max 16)
}

impl FileExplorer {
    /// ### new
    ///
    /// Instantiates a new FileExplorer
    pub fn new() -> FileExplorer {
        FileExplorer {
            wrkdir: PathBuf::from("/"),
            index: 0,
            files: Vec::new(),
            dirstack: VecDeque::with_capacity(16),
        }
    }

    /// ### pushd
    ///
    /// push directory to stack
    pub fn pushd(&mut self, dir: &Path) {
        // Check if stack overflows the size
        if self.dirstack.len() + 1 > 16 {
            self.dirstack.pop_back(); // Start cleaning events from back
        }
        // Eventually push front the new record
        self.dirstack.push_front(PathBuf::from(dir));
    }

    /// ### popd
    ///
    /// Pop directory from the stack and return the directory
    pub fn popd(&mut self) -> Option<PathBuf> {
        self.dirstack.pop_front()
    }

    /// ### sort_files_by_name
    ///
    /// Sort explorer files by their name
    pub fn sort_files_by_name(&mut self) {
        self.files.sort_by_key(|x: &FsEntry| match x {
            FsEntry::Directory(dir) => dir.name.as_str().to_lowercase(),
            FsEntry::File(file) => file.name.as_str().to_lowercase(),
        });
    }
}

/// ## FileExplorerTab
///
/// File explorer tab
enum FileExplorerTab {
    Local,
    Remote,
}

/// ## LogLevel
///
/// Log level type
enum LogLevel {
    Error,
    Warn,
    Info,
}

/// ## LogRecord
///
/// Log record entry
struct LogRecord {
    pub time: DateTime<Local>,
    pub level: LogLevel,
    pub msg: String,
}

impl LogRecord {
    /// ### new
    ///
    /// Instantiates a new LogRecord
    pub fn new(level: LogLevel, msg: &str) -> LogRecord {
        LogRecord {
            time: Local::now(),
            level,
            msg: String::from(msg),
        }
    }
}

/// ### TransferStates
///
/// TransferStates contains the states related to the transfer process
struct TransferStates {
    pub progress: f64,        // Current read/write progress (percentage)
    pub started: Instant,     // Instant the transfer process started
    pub aborted: bool,        // Describes whether the transfer process has been aborted
    pub bytes_written: usize, // Bytes written during transfer
    pub bytes_total: usize,   // Total bytes to write
}

impl TransferStates {
    /// ### new
    ///
    /// Instantiates a new transfer states
    pub fn new() -> TransferStates {
        TransferStates {
            progress: 0.0,
            started: Instant::now(),
            aborted: false,
            bytes_written: 0,
            bytes_total: 0,
        }
    }

    /// ### reset
    ///
    /// Re-intiialize transfer states
    pub fn reset(&mut self) {
        self.progress = 0.0;
        self.started = Instant::now();
        self.aborted = false;
        self.bytes_written = 0;
        self.bytes_total = 0;
    }

    /// ### set_progress
    ///
    /// Calculate progress percentage based on current progress
    pub fn set_progress(&mut self, w: usize, sz: usize) {
        self.bytes_written = w;
        self.bytes_total = sz;
        let mut prog: f64 = ((self.bytes_written as f64) * 100.0) / (self.bytes_total as f64);
        // Check value
        if prog > 100.0 {
            prog = 100.0;
        } else if prog < 0.0 {
            prog = 0.0;
        }
        self.progress = prog;
    }

    /// ### byte_per_second
    ///
    /// Calculate bytes per second
    pub fn bytes_per_second(&self) -> u64 {
        // bytes_written : elapsed_secs = x : 1
        let elapsed_secs: u64 = self.started.elapsed().as_secs();
        match elapsed_secs {
            0 => 0, // NOTE: would divide by 0 :D
            _ => self.bytes_written as u64 / elapsed_secs,
        }
    }
}

impl Default for TransferStates {
    fn default() -> Self {
        Self::new()
    }
}

/// ## FileTransferActivity
///
/// FileTransferActivity is the data holder for the file transfer activity
pub struct FileTransferActivity {
    pub disconnected: bool,           // Has disconnected from remote?
    pub quit: bool,                   // Has quit term scp?
    context: Option<Context>,         // Context holder
    params: FileTransferParams,       // FT connection params
    client: Box<dyn FileTransfer>,    // File transfer client
    local: FileExplorer,              // Local File explorer state
    remote: FileExplorer,             // Remote File explorer state
    tab: FileExplorerTab,             // Current selected tab
    log_index: usize,                 // Current log index entry selected
    log_records: VecDeque<LogRecord>, // Log records
    log_size: usize,                  // Log records size (max)
    input_mode: InputMode,            // Current input mode
    input_field: InputField,          // Current selected input mode
    input_txt: String,                // Input text
    choice_opt: DialogYesNoOption,    // Dialog popup selected option
    transfer: TransferStates,         // Transfer states
}

impl FileTransferActivity {
    /// ### new
    ///
    /// Instantiates a new FileTransferActivity
    pub fn new(params: FileTransferParams) -> FileTransferActivity {
        let protocol: FileTransferProtocol = params.protocol;
        FileTransferActivity {
            disconnected: false,
            quit: false,
            context: None,
            client: match protocol {
                FileTransferProtocol::Sftp => Box::new(SftpFileTransfer::new()),
                FileTransferProtocol::Ftp(ftps) => Box::new(FtpFileTransfer::new(ftps)),
                FileTransferProtocol::Scp => Box::new(ScpFileTransfer::new()),
            },
            params,
            local: FileExplorer::new(),
            remote: FileExplorer::new(),
            tab: FileExplorerTab::Local,
            log_index: 0,
            log_records: VecDeque::with_capacity(256), // 256 events is enough I guess
            log_size: 256,                             // Must match with capacity
            input_mode: InputMode::Explorer,
            input_field: InputField::Explorer,
            input_txt: String::new(),
            choice_opt: DialogYesNoOption::Yes,
            transfer: TransferStates::default(),
        }
    }
}

/**
 * Activity Trait
 * Keep it clean :)
 * Use methods instead!
 */

impl Activity for FileTransferActivity {
    /// ### on_create
    ///
    /// `on_create` is the function which must be called to initialize the activity.
    /// `on_create` must initialize all the data structures used by the activity
    fn on_create(&mut self, context: Context) {
        // Set context
        self.context = Some(context);
        // Clear terminal
        let _ = self.context.as_mut().unwrap().terminal.clear();
        // Put raw mode on enabled
        let _ = enable_raw_mode();
        // Set working directory
        let pwd: PathBuf = self.context.as_ref().unwrap().local.pwd();
        // Get files at current wd
        self.local_scan(pwd.as_path());
        self.local.wrkdir = pwd;
    }

    /// ### on_draw
    ///
    /// `on_draw` is the function which draws the graphical interface.
    /// This function must be called at each tick to refresh the interface
    fn on_draw(&mut self) {
        // Should ui actually be redrawned?
        let mut redraw: bool = false;
        // Context must be something
        if self.context.is_none() {
            return;
        }
        let is_explorer_mode: bool = matches!(self.input_mode, InputMode::Explorer);
        // Check if connected
        if !self.client.is_connected() && is_explorer_mode {
            // Set init state to connecting popup
            self.input_mode = InputMode::Popup(PopupType::Wait(format!(
                "Connecting to {}:{}...",
                self.params.address, self.params.port
            )));
            // Force ui draw
            self.draw();
            // Connect to remote
            self.connect();
            // Redraw
            redraw = true;
        }
        // Handle input events (if false, becomes true; otherwise remains true)
        redraw |= self.read_input_event();
        // @! draw interface
        if redraw {
            self.draw();
        }
    }

    /// ### on_destroy
    ///
    /// `on_destroy` is the function which cleans up runtime variables and data before terminating the activity.
    /// This function must be called once before terminating the activity.
    fn on_destroy(&mut self) -> Option<Context> {
        // Disable raw mode
        let _ = disable_raw_mode();
        // Disconnect client
        if self.client.is_connected() {
            let _ = self.client.disconnect();
        }
        // Clear terminal and return
        match self.context.take() {
            Some(mut ctx) => {
                let _ = ctx.terminal.clear();
                Some(ctx)
            }
            None => None,
        }
    }
}
