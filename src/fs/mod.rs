//! ## Fs
//!
//! `fs` is the module which provides file system entities

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

extern crate bytesize;
#[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
extern crate users;

use crate::utils::{fmt_pex, time_to_str};

use bytesize::ByteSize;
use std::path::PathBuf;
use std::time::SystemTime;
#[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
use users::get_user_by_uid;

/// ## FsEntry
///
/// FsEntry represents a generic entry in a directory

#[derive(Clone, std::fmt::Debug)]
pub enum FsEntry {
    Directory(FsDirectory),
    File(FsFile),
}

/// ## FsDirectory
///
/// Directory provides an interface to file system directories

#[derive(Clone, std::fmt::Debug)]
pub struct FsDirectory {
    pub name: String,
    pub abs_path: PathBuf,
    pub last_change_time: SystemTime,
    pub last_access_time: SystemTime,
    pub creation_time: SystemTime,
    pub readonly: bool,
    pub symlink: Option<PathBuf>,       // UNIX only
    pub user: Option<u32>,              // UNIX only
    pub group: Option<u32>,             // UNIX only
    pub unix_pex: Option<(u8, u8, u8)>, // UNIX only
}

/// ### FsFile
///
/// FsFile provides an interface to file system files

#[derive(Clone, std::fmt::Debug)]
pub struct FsFile {
    pub name: String,
    pub abs_path: PathBuf,
    pub last_change_time: SystemTime,
    pub last_access_time: SystemTime,
    pub creation_time: SystemTime,
    pub size: usize,
    pub ftype: Option<String>, // File type
    pub readonly: bool,
    pub symlink: Option<PathBuf>,       // UNIX only
    pub user: Option<u32>,              // UNIX only
    pub group: Option<u32>,             // UNIX only
    pub unix_pex: Option<(u8, u8, u8)>, // UNIX only
}

impl std::fmt::Display for FsEntry {
    /// ### fmt_ls
    ///
    /// Format File Entry as `ls` does
    #[cfg(any(target_os = "unix", target_os = "macos", target_os = "linux"))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FsEntry::Directory(dir) => {
                // Create mode string
                let mut mode: String = String::with_capacity(10);
                let file_type: char = match dir.symlink {
                    Some(_) => 'l',
                    None => 'd',
                };
                mode.push(file_type);
                match dir.unix_pex {
                    None => mode.push_str("?????????"),
                    Some((owner, group, others)) => {
                        mode.push_str(fmt_pex(owner, group, others).as_str())
                    }
                }
                // Get username
                let username: String = match dir.user {
                    Some(uid) => match get_user_by_uid(uid) {
                        Some(user) => user.name().to_string_lossy().to_string(),
                        None => uid.to_string(),
                    },
                    None => String::from("0"),
                };
                // Get group
                /*
                let group: String = match dir.group {
                    Some(gid) => match get_group_by_gid(gid) {
                        Some(group) => group.name().to_string_lossy().to_string(),
                        None => gid.to_string(),
                    },
                    None => String::from("0"),
                };
                */
                // Get byte size
                let size: String = String::from("4096");
                // Get date
                let datetime: String = time_to_str(dir.last_change_time, "%b %d %Y %H:%M");
                // Set file name (or omit if too long)
                let dir_name: String = match dir.name.len() >= 24 {
                    false => dir.name.clone(),
                    true => format!("{}...", &dir.name.as_str()[0..20]),
                };
                write!(
                    f,
                    "{:24}\t{:12}\t{:12}\t{:9}\t{:17}",
                    dir_name, mode, username, size, datetime
                )
            }
            FsEntry::File(file) => {
                // Create mode string
                let mut mode: String = String::with_capacity(10);
                let file_type: char = match file.symlink {
                    Some(_) => 'l',
                    None => '-',
                };
                mode.push(file_type);
                match file.unix_pex {
                    None => mode.push_str("?????????"),
                    Some((owner, group, others)) => {
                        mode.push_str(fmt_pex(owner, group, others).as_str())
                    }
                }
                // Get username
                let username: String = match file.user {
                    Some(uid) => match get_user_by_uid(uid) {
                        Some(user) => user.name().to_string_lossy().to_string(),
                        None => uid.to_string(),
                    },
                    None => String::from("0"),
                };
                // Get group
                /*
                let group: String = match file.group {
                    Some(gid) => match get_group_by_gid(gid) {
                        Some(group) => group.name().to_string_lossy().to_string(),
                        None => gid.to_string(),
                    },
                    None => String::from("0"),
                };
                */
                // Get byte size
                let size: ByteSize = ByteSize(file.size as u64);
                // Get date
                let datetime: String = time_to_str(file.last_change_time, "%b %d %Y %H:%M");
                // Set file name (or omit if too long)
                let file_name: String = match file.name.len() >= 24 {
                    false => file.name.clone(),
                    true => format!("{}...", &file.name.as_str()[0..20]),
                };
                write!(
                    f,
                    "{:24}\t{:12}\t{:12}\t{:9}\t{:17}",
                    file_name, mode, username, size, datetime
                )
            }
        }
    }

    /// ### fmt_ls
    ///
    /// Format File Entry as `ls` does
    #[cfg(target_os = "windows")]
    #[cfg(not(tarpaulin_include))]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FsEntry::Directory(dir) => {
                // Create mode string
                let mut mode: String = String::with_capacity(10);
                let file_type: char = match dir.symlink {
                    Some(_) => 'l',
                    None => 'd',
                };
                mode.push(file_type);
                match dir.unix_pex {
                    None => mode.push_str("?????????"),
                    Some((owner, group, others)) => {
                        mode.push_str(fmt_pex(owner, group, others).as_str())
                    }
                }
                // Get username
                let username: String = match dir.user {
                    Some(uid) => uid.to_string(),
                    None => String::from("0"),
                };
                // Get group
                /*
                let group: String = match dir.group {
                    Some(gid) => gid.to_string(),
                    None => String::from("0"),
                };
                */
                // Get byte size
                let size: String = String::from("4096");
                // Get date
                let datetime: String = time_to_str(dir.last_change_time, "%b %d %Y %H:%M");
                // Set file name (or omit if too long)
                let dir_name: String = match dir.name.len() >= 24 {
                    false => dir.name.clone(),
                    true => format!("{}...", &dir.name.as_str()[0..20]),
                };
                write!(
                    f,
                    "{:24}\t{:12}\t{:12}\t{:9}\t{:17}",
                    dir_name, mode, username, size, datetime
                )
            }
            FsEntry::File(file) => {
                // Create mode string
                let mut mode: String = String::with_capacity(10);
                let file_type: char = match file.symlink {
                    Some(_) => 'l',
                    None => '-',
                };
                mode.push(file_type);
                match file.unix_pex {
                    None => mode.push_str("?????????"),
                    Some((owner, group, others)) => {
                        mode.push_str(fmt_pex(owner, group, others).as_str())
                    }
                }
                // Get username
                let username: String = match file.user {
                    Some(uid) => uid.to_string(),
                    None => String::from("0"),
                };
                // Get group
                /*
                let group: String = match file.group {
                    Some(gid) => gid.to_string(),
                    None => String::from("0"),
                };
                */
                // Get byte size
                let size: ByteSize = ByteSize(file.size as u64);
                // Get date
                let datetime: String = time_to_str(file.last_change_time, "%b %d %Y %H:%M");
                // Set file name (or omit if too long)
                let file_name: String = match file.name.len() >= 24 {
                    false => file.name.clone(),
                    true => format!("{}...", &file.name.as_str()[0..20]),
                };
                write!(
                    f,
                    "{:24}\t{:12}\t{:12}\t{:9}\t{:17}",
                    file_name, mode, username, size, datetime
                )
            }
        }
    }
}
