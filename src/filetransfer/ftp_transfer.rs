//! ## Ftp_transfer
//!
//! `ftp_transfer` is the module which provides the implementation for the FTP/FTPS file transfer

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

// Dependencies
extern crate chrono;
extern crate ftp;
extern crate regex;

use super::{FileTransfer, FileTransferError, FileTransferErrorType};
use crate::fs::{FsDirectory, FsEntry, FsFile};
use crate::utils::lstime_to_systime;

// Includes
use ftp::openssl::ssl::{SslContext, SslMethod};
use ftp::{FtpError, FtpStream};
use regex::Regex;
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// ## FtpFileTransfer
///
/// Ftp file transfer struct
pub struct FtpFileTransfer {
    stream: Option<FtpStream>,
    ftps: bool,
}

impl FtpFileTransfer {
    /// ### new
    ///
    /// Instantiates a new `FtpFileTransfer`
    pub fn new(ftps: bool) -> FtpFileTransfer {
        FtpFileTransfer {
            stream: None,
            ftps: ftps,
        }
    }

    /// ### parse_list_line
    ///
    /// Parse a line of LIST command output and instantiates an FsEntry from it
    fn parse_list_line(&self, path: &Path, line: &str) -> Result<FsEntry, ()> {
        // Prepare list regex
        // NOTE: about this damn regex <https://stackoverflow.com/questions/32480890/is-there-a-regex-to-parse-the-values-from-an-ftp-directory-listing>
        lazy_static! {
            static ref LS_RE: Regex = Regex::new(r#"^([\-ld])([\-rwxs]{9})\s+(\d+)\s+(\w+)\s+(\w+)\s+(\d+)\s+(\w{3}\s+\d{1,2}\s+(?:\d{1,2}:\d{1,2}|\d{4}))\s+(.+)$"#).unwrap();
        }
        // Apply regex to result
        match LS_RE.captures(line) {
            // String matches regex
            Some(metadata) => {
                // NOTE: metadata fmt: (regex, file_type, permissions, link_count, uid, gid, filesize, mtime, filename)
                // Expected 7 + 1 (8) values: + 1 cause regex is repeated at 0
                if metadata.len() < 8 {
                    return Err(());
                }
                // Collect metadata
                // Get if is directory and if is symlink
                let (is_dir, is_symlink): (bool, bool) = match metadata.get(1).unwrap().as_str() {
                    "-" => (false, false),
                    "l" => (false, true),
                    "d" => (true, false),
                    _ => return Err(()), // Ignore special files
                };
                // Check string length (unix pex)
                if metadata.get(2).unwrap().as_str().len() < 9 {
                    return Err(());
                }
                // Get unix pex
                let unix_pex: (u8, u8, u8) = {
                    let owner_pex: u8 = {
                        let mut count: u8 = 0;
                        for (i, c) in metadata.get(2).unwrap().as_str()[0..3].chars().enumerate() {
                            match c {
                                '-' => {}
                                _ => {
                                    count = count
                                        + match i {
                                            0 => 4,
                                            1 => 2,
                                            2 => 1,
                                            _ => 0,
                                        }
                                }
                            }
                        }
                        count
                    };
                    let group_pex: u8 = {
                        let mut count: u8 = 0;
                        for (i, c) in metadata.get(2).unwrap().as_str()[3..6].chars().enumerate() {
                            match c {
                                '-' => {}
                                _ => {
                                    count = count
                                        + match i {
                                            0 => 4,
                                            1 => 2,
                                            2 => 1,
                                            _ => 0,
                                        }
                                }
                            }
                        }
                        count
                    };
                    let others_pex: u8 = {
                        let mut count: u8 = 0;
                        for (i, c) in metadata.get(2).unwrap().as_str()[6..9].chars().enumerate() {
                            match c {
                                '-' => {}
                                _ => {
                                    count = count
                                        + match i {
                                            0 => 4,
                                            1 => 2,
                                            2 => 1,
                                            _ => 0,
                                        }
                                }
                            }
                        }
                        count
                    };
                    (owner_pex, group_pex, others_pex)
                };
                // Parse mtime and convert to SystemTime
                let mtime: SystemTime = match lstime_to_systime(
                    metadata.get(7).unwrap().as_str(),
                    "%b %d %Y",
                    "%b %d %H:%M",
                ) {
                    Ok(t) => t,
                    Err(_) => return Err(()),
                };
                // Get uid
                let uid: Option<u32> = match metadata.get(4).unwrap().as_str().parse::<u32>() {
                    Ok(uid) => Some(uid),
                    Err(_) => None,
                };
                // Get gid
                let gid: Option<u32> = match metadata.get(5).unwrap().as_str().parse::<u32>() {
                    Ok(gid) => Some(gid),
                    Err(_) => None,
                };
                // Get filesize
                let filesize: usize = match metadata.get(6).unwrap().as_str().parse::<usize>() {
                    Ok(sz) => sz,
                    Err(_) => return Err(()),
                };
                let file_name: String = String::from(metadata.get(8).unwrap().as_str());
                let mut abs_path: PathBuf = PathBuf::from(path);
                let extension: Option<String> = match abs_path.as_path().extension() {
                    None => None,
                    Some(s) => Some(String::from(s.to_string_lossy())),
                };
                abs_path.push(file_name.as_str());
                // Return
                // Push to entries
                Ok(match is_dir {
                    true => FsEntry::Directory(FsDirectory {
                        name: file_name,
                        abs_path: abs_path,
                        last_change_time: mtime,
                        last_access_time: mtime,
                        creation_time: mtime,
                        readonly: false,
                        symlink: None,
                        user: uid,
                        group: gid,
                        unix_pex: Some(unix_pex),
                    }),
                    false => FsEntry::File(FsFile {
                        name: file_name,
                        abs_path: abs_path,
                        last_change_time: mtime,
                        last_access_time: mtime,
                        creation_time: mtime,
                        size: filesize,
                        ftype: extension,
                        readonly: false,
                        symlink: None,
                        user: uid,
                        group: gid,
                        unix_pex: Some(unix_pex),
                    }),
                })
            }
            None => Err(()),
        }
    }
}

impl FileTransfer for FtpFileTransfer {
    /// ### connect
    ///
    /// Connect to the remote server

    fn connect(
        &mut self,
        address: String,
        port: u16,
        username: Option<String>,
        password: Option<String>,
    ) -> Result<(), FileTransferError> {
        // Get stream
        let mut stream: FtpStream = match FtpStream::connect(format!("{}:{}", address, port)) {
            Ok(stream) => stream,
            Err(err) => {
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    format!("{}", err),
                ))
            }
        };
        // If SSL, open secure session
        if self.ftps {
            let mut ctx = SslContext::builder(SslMethod::tls()).unwrap();
            let ctx = ctx.build();
            if let Err(err) = stream.into_secure(ctx) {
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::SslError,
                    format!("{}", err),
                ));
            }
        }
        // If username / password...
        if let Some(username) = username {
            if let Err(err) = stream.login(
                username.as_str(),
                match password {
                    Some(pwd) => pwd.as_ref(),
                    None => "",
                },
            ) {
                return Err(FileTransferError::new_ex(
                    FileTransferErrorType::AuthenticationFailed,
                    format!("{}", err),
                ));
            }
        }
        // Set stream
        self.stream = Some(stream);
        // Return OK
        Ok(())
    }

    /// ### disconnect
    ///
    /// Disconnect from the remote server

    fn disconnect(&mut self) -> Result<(), FileTransferError> {
        match self.stream {
            Some(stream) => match stream.quit() {
                Ok(_) => Ok(()),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    format!("{}", err),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### is_connected
    ///
    /// Indicates whether the client is connected to remote
    fn is_connected(&self) -> bool {
        match self.stream {
            Some(_) => true,
            None => false,
        }
    }

    /// ### pwd
    ///
    /// Print working directory

    fn pwd(&self) -> Result<PathBuf, FileTransferError> {
        match self.stream {
            Some(stream) => match stream.pwd() {
                Ok(path) => Ok(PathBuf::from(path.as_str())),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    format!("{}", err),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### change_dir
    ///
    /// Change working directory

    fn change_dir(&mut self, dir: &Path) -> Result<PathBuf, FileTransferError> {
        match self.stream {
            Some(stream) => match stream.cwd(&dir.to_string_lossy()) {
                Ok(_) => Ok(PathBuf::from(dir)),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::ConnectionError,
                    format!("{}", err),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### list_dir
    ///
    /// List directory entries

    fn list_dir(&self, path: &Path) -> Result<Vec<FsEntry>, FileTransferError> {
        match self.stream {
            Some(stream) => match stream.list(Some(&path.to_string_lossy())) {
                Ok(entries) => {
                    // Prepare result
                    let mut result: Vec<FsEntry> = Vec::with_capacity(entries.len());
                    // Iterate over entries
                    for entry in entries.iter() {
                        if let Ok(file) = self.parse_list_line(path, entry) {
                            result.push(file);
                        }
                    }
                    Ok(result)
                }
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::DirStatFailed,
                    format!("{}", err),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### mkdir
    ///
    /// Make directory
    fn mkdir(&self, dir: &Path) -> Result<(), FileTransferError> {
        match self.stream {
            Some(stream) => match stream.mkdir(&dir.to_string_lossy()) {
                Ok(_) => Ok(()),
                Err(err) => Err(FileTransferError::new_ex(
                    FileTransferErrorType::DirStatFailed,
                    format!("{}", err),
                )),
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### remove
    ///
    /// Remove a file or a directory
    fn remove(&self, fsentry: &FsEntry) -> Result<(), FileTransferError> {
        match self.stream {
            Some(stream) => match fsentry {
                // Match fs entry...
                FsEntry::File(file) => {
                    // Remove file directly
                    match stream.rm(file.name.as_ref()) {
                        Ok(_) => Ok(()),
                        Err(err) => Err(FileTransferError::new_ex(
                            FileTransferErrorType::DirStatFailed,
                            format!("{}", err),
                        )),
                    }
                }
                FsEntry::Directory(dir) => {
                    // Get directory files
                    match self.list_dir(dir.abs_path.as_path()) {
                        Ok(files) => {
                            // Remove recursively files
                            for file in files.iter() {
                                if let Err(err) = self.remove(&file) {
                                    return Err(FileTransferError::new_ex(
                                        FileTransferErrorType::DirStatFailed,
                                        format!("{}", err),
                                    ));
                                }
                            }
                            // Once all files in directory have been deleted, remove directory
                            match stream.rmdir(dir.name.as_str()) {
                                Ok(_) => Ok(()),
                                Err(err) => Err(FileTransferError::new_ex(
                                    FileTransferErrorType::DirStatFailed,
                                    format!("{}", err),
                                )),
                            }
                        }
                        Err(err) => Err(FileTransferError::new_ex(
                            FileTransferErrorType::DirStatFailed,
                            format!("{}", err),
                        )),
                    }
                }
            },
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### rename
    ///
    /// Rename file or a directory
    fn rename(&self, file: &FsEntry, dst: &Path) -> Result<(), FileTransferError> {
        match self.stream {
            Some(stream) => {
                // Get name
                let src_name: String = match file {
                    FsEntry::Directory(dir) => dir.name.clone(),
                    FsEntry::File(file) => file.name.clone(),
                };
                let dst_name: PathBuf = match dst.file_name() {
                    Some(p) => PathBuf::from(p),
                    None => {
                        return Err(FileTransferError::new_ex(
                            FileTransferErrorType::FileCreateDenied,
                            String::from("Invalid destination name"),
                        ))
                    }
                };
                // Only names are supported
                match stream.rename(src_name.as_str(), &dst_name.as_path().to_string_lossy()) {
                    Ok(_) => Ok(()),
                    Err(err) => Err(FileTransferError::new_ex(
                        FileTransferErrorType::DirStatFailed,
                        format!("{}", err),
                    )),
                }
            }
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### stat
    ///
    /// Stat file and return FsEntry
    fn stat(&self, path: &Path) -> Result<FsEntry, FileTransferError> {
        match self.stream {
            Some(stream) => Err(FileTransferError::new(
                FileTransferErrorType::UnsupportedFeature,
            )),
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### send_file
    ///
    /// Send file to remote
    /// File name is referred to the name of the file as it will be saved
    /// Data contains the file data
    /// Returns file and its size
    fn send_file(&self, file_name: &Path) -> Result<Box<dyn Write>, FileTransferError> {
        match self.stream {
            Some(stream) => {}
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }

    /// ### recv_file
    ///
    /// Receive file from remote with provided name
    /// Returns file and its size
    fn recv_file(&self, file_name: &Path) -> Result<(Box<dyn Read>, usize), FileTransferError> {
        match self.stream {
            Some(stream) => {}
            None => Err(FileTransferError::new(
                FileTransferErrorType::UninitializedSession,
            )),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_filetransfer_ftp_new() {
        let ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        assert_eq!(ftp.ftps, false);
        assert!(ftp.stream.is_none());
        // FTPS
        let ftp: FtpFileTransfer = FtpFileTransfer::new(true);
        assert_eq!(ftp.ftps, true);
        assert!(ftp.stream.is_none());
    }

    #[test]
    fn test_filetransfer_ftp_parse_list_line() {
        let ftp: FtpFileTransfer = FtpFileTransfer::new(false);
        // Simple file
        let fs_entry: FsEntry = ftp
            .parse_list_line(
                PathBuf::from("/tmp").as_path(),
                "-rw-rw-r-- 1 root  dialout  8192 Nov 5 2018 omar.txt",
            )
            .ok()
            .unwrap();
        if let FsEntry::File(file) = fs_entry {
            assert_eq!(file.abs_path, PathBuf::from("/tmp/omar.txt"));
            assert_eq!(file.name, String::from("omar.txt"));
            assert_eq!(file.size, 8192);
            assert_eq!(file.symlink, None);
            assert_eq!(file.user, None);
            assert_eq!(file.group, None);
            assert_eq!(file.unix_pex.unwrap(), (6, 6, 4));
            assert_eq!(file.last_access_time, 1541376000);
            assert_eq!(file.last_change_time, 1541376000);
            assert_eq!(file.creation_time, 1541376000);
        } else {
            panic!("Expected file, got directory");
        }
        // Simple file with number as gid, uid
        let fs_entry: FsEntry = ftp
            .parse_list_line(
                PathBuf::from("/tmp").as_path(),
                "-rwxr-xr-x 1 0  9  4096 Nov 5 16:32 omar.txt",
            )
            .ok()
            .unwrap();
        if let FsEntry::File(file) = fs_entry {
            assert_eq!(file.abs_path, PathBuf::from("/tmp/omar.txt"));
            assert_eq!(file.name, String::from("omar.txt"));
            assert_eq!(file.size, 8192);
            assert_eq!(file.symlink, None);
            assert_eq!(file.user, Some(0));
            assert_eq!(file.group, Some(9));
            assert_eq!(file.unix_pex.unwrap(), (7, 5, 5));
            assert_eq!(file.last_access_time, 1604593920);
            assert_eq!(file.last_change_time, 1604593920);
            assert_eq!(file.creation_time, 1604593920);
        } else {
            panic!("Expected file, got directory");
        }
        // Directory
        let fs_entry: FsEntry = ftp
            .parse_list_line(
                PathBuf::from("/tmp").as_path(),
                "drwxrwxr-x 1 0  9  4096 Nov 5 2018 docs",
            )
            .ok()
            .unwrap();
        if let FsEntry::Directory(dir) = fs_entry {
            assert_eq!(dir.abs_path, PathBuf::from("/tmp/docs"));
            assert_eq!(dir.name, String::from("docs"));
            assert_eq!(dir.symlink, None);
            assert_eq!(dir.user, Some(0));
            assert_eq!(dir.group, Some(9));
            assert_eq!(dir.unix_pex.unwrap(), (7, 7, 5));
            assert_eq!(dir.last_access_time, 1541376000);
            assert_eq!(dir.last_change_time, 1541376000);
            assert_eq!(dir.creation_time, 1541376000);
            assert_eq!(dir.readonly, false);
        } else {
            panic!("Expected directory, got directory");
        }
        // Error
        assert!(ftp
            .parse_list_line(
                PathBuf::from("/").as_path(),
                "drwxrwxr-x 1 0  9  Nov 5 2018 docs"
            )
            .is_err());
    }
}
