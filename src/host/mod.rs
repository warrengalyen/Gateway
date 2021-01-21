//! ## Host
//!
//! `host` is the module which provides functionalities to host file system

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

use std::fs::{self, File, Metadata, OpenOptions};
use std::path::{Path, PathBuf};
use std::time::SystemTime;
// Metadata ext
#[cfg(any(unix, macos, linux))]
extern crate users;
#[cfg(any(unix, macos, linux))]
use std::os::unix::fs::MetadataExt;
#[cfg(any(unix, macos, linux))]
use users::{get_group_by_gid, get_user_by_uid};

// Locals
use crate::fs::{FsDirectory, FsEntry, FsFile};

/// ## HostError
///
/// HostError provides errors related to local host file system
pub enum HostError {
    NoSuchFileOrDirectory,
    ReadonlyFile,
    DirNotAccessible,
    FileNotAccessible,
}

/// ## Localhost
///
/// Localhost is the entity which holds the information about the current directory and host.
/// It provides functions to navigate across the local host file system
pub struct Localhost {
    wrkdir: PathBuf,
    files: Vec<FsEntry>,
}

impl Localhost {
    /// ### new
    ///
    /// Instantiates a new Localhost struct
    pub fn new(wrkdir: PathBuf) -> Result<Localhost, HostError> {
        let mut host: Localhost = Localhost {
            wrkdir: wrkdir,
            files: Vec::new(),
        };
        // Check if dir exists
        if !host.file_exists(host.wrkdir.as_path()) {
            return Err(HostError::NoSuchFileOrDirectory);
        }
        // Retrieve files for provided path
        host.files = match host.scan_dir() {
            Ok(files) => files,
            Err(err) => return Err(err),
        };
        Ok(host)
    }

    /// ### pwd
    ///
    /// Print working directory
    pub fn pwd(&self) -> PathBuf {
        self.wrkdir.clone()
    }

    /// ### list_dir
    ///
    /// List files in current directory
    pub fn list_dir(&self) -> Vec<FsEntry> {
        self.files.clone()
    }

    /// ### change_wrkdir
    ///
    /// Change working directory with the new provided directory
    pub fn change_wrkdir(&mut self, new_dir: PathBuf) -> Result<PathBuf, HostError> {
        // Check whether directory exists
        if !self.file_exists(new_dir.as_path()) {
            return Err(HostError::NoSuchFileOrDirectory);
        }
        let prev_dir: PathBuf = self.wrkdir.clone(); // Backup location
                                                     // Update working directory

        self.wrkdir = new_dir;
        // Scan new directory
        self.files = match self.scan_dir() {
            Ok(files) => files,
            Err(err) => {
                // Restore directory
                self.wrkdir = prev_dir;
                return Err(err);
            }
        };
        Ok(self.wrkdir.clone())
    }

    /// ### open_file_read
    ///
    /// Open file for read
    pub fn open_file_read(&self, file: &Path) -> Result<File, HostError> {
        if !self.file_exists(file) {
            return Err(HostError::NoSuchFileOrDirectory);
        }
        match OpenOptions::new().create(false).read(true).write(false).open(file) {
            Ok(f) => Ok(f),
            Err(_) => Err(HostError::FileNotAccessible),
        }
    }

    /// ### open_file_write
    ///
    /// Open file for write
    pub fn open_file_write(&self, file: &Path) -> Result<File, HostError> {
        match OpenOptions::new().create(true).write(true).truncate(true).open(file) {
            Ok(f) => Ok(f),
            Err(_) => match self.file_exists(file) {
                true => Err(HostError::ReadonlyFile),
                false => Err(HostError::FileNotAccessible),
            },
        }
    }

    /// ### file_exists
    ///
    /// Returns whether provided file path exists
    fn file_exists(&self, path: &Path) -> bool {
        path.exists()
    }

    /// ### scan_dir
    ///
    /// Get content of the current directory as a list of fs entry (Windows)
    #[cfg(any(unix, macos, linux))]
    fn scan_dir(&self) -> Result<Vec<FsEntry>, HostError> {
        let entries = match std::fs::read_dir(self.wrkdir.as_path()) {
            Ok(e) => e,
            Err(_) => return Err(HostError::DirNotAccessible),
        };
        let mut fs_entries: Vec<FsEntry> = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let path: PathBuf = entry.path();
                let attr: Metadata = fs::metadata(path.clone()).unwrap();
                // Get user stuff
                let user: Option<String> = match get_user_by_uid(attr.uid()) {
                    Some(user) => Some(String::from(user.name().to_str().unwrap_or(""))),
                    None => None,
                };
                let group: Option<String> = match get_group_by_gid(attr.gid()) {
                    Some(group) => Some(String::from(group.name().to_str().unwrap_or(""))),
                    None => None,
                };
                let file_name: String =
                    String::from(path.file_name().unwrap().to_str().unwrap_or(""));
                // Match dir / file
                fs_entries.push(match path.is_dir() {
                    true => {
                        // Is dir
                        FsEntry::Directory(FsDirectory {
                            name: file_name,
                            last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                            creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                            readonly: attr.permissions().readonly(),
                            symlink: match fs::read_link(path) {
                                Ok(p) => Some(p),
                                Err(_) => None,
                            },
                            user: user,
                            group: group,
                            unix_pex: Some(self.u32_to_mode(attr.mode())),
                        })
                    }
                    false => {
                        // Is File
                        let extension: Option<String> = match path.extension() {
                            Some(s) => Some(String::from(s.to_str().unwrap_or(""))),
                            None => None,
                        };
                        FsEntry::File(FsFile {
                            name: file_name,
                            last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                            creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                            readonly: attr.permissions().readonly(),
                            size: attr.len() as usize,
                            ftype: extension,
                            symlink: match fs::read_link(path) {
                                Ok(p) => Some(p),
                                Err(_) => None,
                            },
                            user: user,
                            group: group,
                            unix_pex: Some(self.u32_to_mode(attr.mode())),
                        })
                    }
                });
            }
        }
        Ok(fs_entries)
    }

    /// ### scan_dir
    ///
    /// Get content of the current directory as a list of fs entry (Windows)
    #[cfg(target_os = "windows")]
    #[cfg(not(tarpaulin_include))]
    fn scan_dir(&self) -> Result<Vec<FsEntry>, HostError> {
        let entries = match std::fs::read_dir(self.wrkdir.as_path()) {
            Ok(e) => e,
            Err(_) => return Err(HostError::DirNotAccessible),
        };
        let mut fs_entries: Vec<FsEntry> = Vec::new();
        for entry in entries {
            if let Ok(entry) = entry {
                let path: PathBuf = entry.path();
                let attr: Metadata = fs::metadata(path.clone()).unwrap();
                let file_name: String =
                    String::from(path.file_name().unwrap().to_str().unwrap_or(""));
                fs_entries.push(match path.is_dir() {
                    true => {
                        // Is dir
                        FsEntry::Directory(FsDirectory {
                            name: file_name,
                            last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                            creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                            readonly: attr.permissions().readonly(),
                            symlink: None,
                            user: None,
                            group: None,
                            unix_pex: None,
                        })
                    }
                    false => {
                        // Is File
                        let extension: Option<String> = match path.extension() {
                            Some(s) => Some(String::from(s.to_str().unwrap_or(""))),
                            None => None,
                        };
                        FsEntry::File(FsFile {
                            name: file_name,
                            last_change_time: attr.modified().unwrap_or(SystemTime::UNIX_EPOCH),
                            last_access_time: attr.accessed().unwrap_or(SystemTime::UNIX_EPOCH),
                            creation_time: attr.created().unwrap_or(SystemTime::UNIX_EPOCH),
                            readonly: attr.permissions().readonly(),
                            size: attr.len() as usize,
                            ftype: extension,
                            symlink: None,
                            user: None,
                            group: None,
                            unix_pex: None,
                        })
                    }
                });
            }
        }
        Ok(fs_entries)
    }

    /// ### u32_to_mode
    ///
    /// Return string with format xxxxxx to tuple of permissions (user, group, others)
    #[cfg(any(unix, macos, linux))]
    fn u32_to_mode(&self, mode: u32) -> (u8, u8, u8) {
        let user: u8 = ((mode >> 6) & 0x7) as u8;
        let group: u8 = ((mode >> 3) & 0x7) as u8;
        let others: u8 = (mode & 0x7) as u8;
        (user, group, others)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use std::io::Write;
    use std::fs::File;

    #[cfg(any(unix, macos, linux))]
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_new() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        assert_eq!(host.wrkdir, PathBuf::from("/bin"));
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from("/bin").as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.files.len(), counter);
    }

    #[test]
    #[cfg(target_os = "windows")]
    fn test_host_localhost_new() {
        let host: Localhost = Localhost::new(PathBuf::from("C:\\")).ok().unwrap();
        assert_eq!(host.wrkdir, PathBuf::from("C:\\"));
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from("C:\\").as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.files.len(), counter);
    }

    #[test]
    #[should_panic]
    fn test_host_localhost_new_bad() {
        Localhost::new(PathBuf::from("/omargabber/123/345"))
            .ok()
            .unwrap();
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_pwd() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        assert_eq!(host.pwd(), PathBuf::from("/bin"));
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_list_files() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from("/bin").as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.list_dir().len(), counter);
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_change_dir() {
        let mut host: Localhost = Localhost::new(PathBuf::from("/dev")).ok().unwrap();
        let new_dir: PathBuf = PathBuf::from("/dev");
        assert!(host.change_wrkdir(new_dir.clone()).is_ok());
        // Verify new files
        // Scan dir
        let entries = std::fs::read_dir(PathBuf::from(new_dir).as_path()).unwrap();
        let mut counter: usize = 0;
        for _ in entries {
            counter = counter + 1;
        }
        assert_eq!(host.files.len(), counter);
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    #[should_panic]
    fn test_host_localhost_change_dir_failed() {
        let mut host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        let new_dir: PathBuf = PathBuf::from("/omar/gabber/123/456");
        assert!(host.change_wrkdir(new_dir.clone()).is_ok());
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_open_read() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        // Create temp file
        let file: tempfile::NamedTempFile = create_sample_file();
        assert!(host.open_file_read(file.path()).is_ok());
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    #[should_panic]
    fn test_host_localhost_open_read_err_no_such_file() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        assert!(host
            .open_file_read(PathBuf::from("/bin/foo-bar-test-omar-123-456-789.txt").as_path())
            .is_ok());
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_open_read_err_not_accessible() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        let file: tempfile::NamedTempFile = create_sample_file();
        //let mut perms = fs::metadata(file.path())?.permissions();
        fs::set_permissions(file.path(), PermissionsExt::from_mode(0o222)).unwrap();
        //fs::set_permissions(file.path(), perms)?;
        assert!(host.open_file_read(file.path()).is_err());
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_open_write() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        // Create temp file
        let file: tempfile::NamedTempFile = create_sample_file();
        assert!(host.open_file_write(file.path()).is_ok());
    }

    #[test]
    #[cfg(any(unix, macos, linux))]
    fn test_host_localhost_open_write_err() {
        let host: Localhost = Localhost::new(PathBuf::from("/bin")).ok().unwrap();
        let file: tempfile::NamedTempFile = create_sample_file();
        //let mut perms = fs::metadata(file.path())?.permissions();
        fs::set_permissions(file.path(), PermissionsExt::from_mode(0o444)).unwrap();
        //fs::set_permissions(file.path(), perms)?;
        assert!(host.open_file_write(file.path()).is_err());
    }

    /// ### create_sample_file
    ///
    /// Create a sample file
    #[cfg(any(unix, macos, linux))]
    fn create_sample_file() -> tempfile::NamedTempFile {
        // Write
        let mut tmpfile: tempfile::NamedTempFile = tempfile::NamedTempFile::new().unwrap();
        write!(
            tmpfile,
            "Lorem ipsum dolor sit amet, consectetur adipiscing elit.\nMauris ultricies consequat eros,\nnec scelerisque magna imperdiet metus.\n"
        )
        .unwrap();
        tmpfile
    }

    #[cfg(any(unix, macos, linux))]
    #[test]
    fn test_host_localhost_symlinks() {
        let tmpdir: tempfile::TempDir = tempfile::TempDir::new().unwrap();
        // Create sample file
        assert!(File::create(format!("{}/foo.txt", tmpdir.path().display()).as_str()).is_ok());
        // Create symlink
        assert!(symlink(format!("{}/foo.txt", tmpdir.path().display()), format!("{}/bar.txt", tmpdir.path().display())).is_ok());
        // Get dir
        let host: Localhost = Localhost::new(PathBuf::from(tmpdir.path())).ok().unwrap();
        let files: Vec<FsEntry> = host.list_dir();
        println!("Entries {:?}", files);
        // Verify files
        let foo_file: &FsEntry = files.get(0).unwrap();
        match foo_file {
            FsEntry::File(foo_file) => {
                assert_eq!(foo_file.name, String::from("foo.txt"));
                assert!(foo_file.symlink.is_none());
            },
            _ => panic!("expected entry 0 to be file: {:?}", foo_file)
        };
        // Verify simlink
        let bar_file: &FsEntry = files.get(1).unwrap();
        match bar_file {
            FsEntry::File(bar_file) => {
                assert_eq!(bar_file.name, String::from("bar.txt"));
                assert_eq!(*bar_file.symlink.as_ref().unwrap(), PathBuf::from(format!("{}/foo.txt", tmpdir.path().display())));
            },
            _ => panic!("expected entry 1 to be file: {:?}", bar_file)
        };
    }
}
