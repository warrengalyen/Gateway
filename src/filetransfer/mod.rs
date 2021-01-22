//! ## FileTransfer
//!
//! `filetransfer` is the module which provides the trait file transfers must implement and the different file transfers

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

use std::path::{Path, PathBuf};
use std::fs::File;

use crate::fs::FsEntry;

// Transfers
pub mod sftp_transfer;

// Types
type ProgressCallback = fn(bytes_written: usize, size: usize);

/// ## FileTransferProtocol
///
/// This enum defines the different transfer protocol available in Gateway

#[derive(PartialEq, Clone)]
pub enum FileTransferProtocol {
    Scp,
    Sftp,
    Ftps,
}

/// ## FileTransferError
///
/// FileTransferError defines the possible errors available for a file transfer

#[derive(PartialEq, Clone)]
pub enum FileTransferError {
    AuthenticationFailed,
    BadAddress,
    ConnectionError,
    DirStatFailed,
    FileCreateDenied,
    FileReadonly,
    IoErr(std::io::Error),
    NoSuchFileOrDirectory,
    ProtocolError,
    UninitializedSession,
    UnknownError,
}

/// ## FileTransfer
///
/// File transfer trait must be implemented by all the file transfers and defines the method used by a generic file transfer

pub trait FileTransfer {
    /// ### connect
    ///
    /// Connect to the remote server

    fn connect(
        &mut self,
        address: String,
        port: usize,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<(), FileTransferError>;

    /// ### disconnect
    ///
    /// Disconnect from the remote server

    fn disconnect(&mut self) -> Result<(), FileTransferError>;

    /// ### pwd
    ///
    /// Print working directory

    fn pwd(&self) -> Result<PathBuf, FileTransferError>;

    /// ### change_dir
    ///
    /// Change working directory

    fn change_dir(&mut self, dir: &Path) -> Result<PathBuf, FileTransferError>;

    /// ### list_dir
    ///
    /// List directory entries

    fn list_dir(&self, path: &Path) -> Result<Vec<FsEntry>, FileTransferError>;

    /// ### mkdir
    ///
    /// Make directory
    fn mkdir(&self, dir: String) -> Result<(), FileTransferError>;

    /// ### remove
    ///
    /// Remove a file or a directory
    fn remove(&self, file: FsEntry) -> Result<(), FileTransferError>;

    /// ### send_file
    ///
    /// Send file to remote
    /// File name is referred to the name of the file as it will be saved
    /// Data contains the file data
    fn send_file(&self, file_name: &Path, file: &mut File, prog_cb: Option<ProgressCallback>) -> Result<(), FileTransferError>;

    /// ### recv_file
    ///
    /// Receive file from remote with provided name
    fn recv_file(&self, file_name: &Path, dest_file: &mut File, prog_cb: Option<ProgressCallback>) -> Result<(), FileTransferError>;
}
