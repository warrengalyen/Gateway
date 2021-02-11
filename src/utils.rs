//! ## Utils
//!
//! `utils` is the module which provides utilities of different kind

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
extern crate textwrap;
extern crate whoami;

use crate::filetransfer::FileTransferProtocol;

use chrono::format::ParseError;
use chrono::prelude::*;
use std::time::{Duration, SystemTime};

/// ### parse_remote_opt
///
/// Parse remote option string. Returns in case of success a tuple made of (address, port, protocol, username)
/// For ssh if username is not provided, current user will be used.
/// In case of error, message is returned
/// If port is missing default port will be used for each protocol
///     SFTP => 22
///     FTP => 21
/// The option string has the following syntax
/// [protocol]://[username]@{address}:[port]
/// The only argument which is mandatory is address
/// NOTE: possible strings
/// - 172.26.104.1
/// - root@172.26.104.1
/// - sftp://root@172.26.104.1
/// - sftp://172.26.104.1:4022
/// - sftp://172.26.104.1
/// - ...
///
pub fn parse_remote_opt(
    remote: &str,
) -> Result<(String, u16, FileTransferProtocol, Option<String>), String> {
    let mut wrkstr: String = remote.to_string();
    let address: String;
    let mut port: u16 = 22;
    let mut protocol: FileTransferProtocol = FileTransferProtocol::Sftp;
    let mut username: Option<String> = None;
    // Split string by '://'
    let tokens: Vec<&str> = wrkstr.split("://").collect();
    // If length is > 1, then token[0] is protocol
    match tokens.len() {
        1 => {}
        2 => {
            // Parse protocol
            match tokens[0] {
                "sftp" => {
                    // Set protocol to sftp
                    protocol = FileTransferProtocol::Sftp;
                    // Set port to default (22)
                    port = 22;
                }
                "scp" => {
                    // Set protocol to scp
                    protocol = FileTransferProtocol::Scp;
                    // Set port to default (22)
                    port = 22;
                }
                "ftp" => {
                    // Set protocol to fpt
                    protocol = FileTransferProtocol::Ftp(false);
                    // Set port to default (21)
                    port = 21;
                }
                "ftps" => {
                    // Set protocol to fpt
                    protocol = FileTransferProtocol::Ftp(true);
                    // Set port to default (21)
                    port = 21;
                }
                _ => return Err(format!("Unknown protocol '{}'", tokens[0])),
            }
            wrkstr = String::from(tokens[1]); // Wrkstr becomes tokens[1]
        }
        _ => return Err(String::from("Bad syntax")), // Too many tokens...
    }
    // Set username to default if sftp or scp
    if matches!(protocol, FileTransferProtocol::Sftp | FileTransferProtocol::Scp) {
        // Set username to current username
        username = Some(whoami::username());
    }
    // Split wrkstring by '@'
    let tokens: Vec<&str> = wrkstr.split('@').collect();
    match tokens.len() {
        1 => {}
        2 => {
            // Username is first token
            username = Some(String::from(tokens[0]));
            // Update wrkstr
            wrkstr = String::from(tokens[1]);
        }
        _ => return Err(String::from("Bad syntax")), // Too many tokens...
    }
    // Split wrkstring by ':'
    let tokens: Vec<&str> = wrkstr.split(':').collect();
    match tokens.len() {
        1 => {
            // Address is wrkstr
            address = wrkstr.clone();
        }
        2 => {
            // Address is first token
            address = String::from(tokens[0]);
            // Port is second str
            port = match tokens[1].parse::<u16>() {
                Ok(val) => val,
                Err(_) => {
                    return Err(format!(
                        "Port must be a number in range [0-65535], but is '{}'",
                        tokens[1]
                    ))
                }
            };
        }
        _ => return Err(String::from("Bad syntax")), // Too many tokens...
    }
    Ok((address, port, protocol, username))
}

/// ### fmt_pex
///
/// Convert 3 bytes of permissions value into ls notation (e.g. rwx-wx--x)
pub fn fmt_pex(owner: u8, group: u8, others: u8) -> String {
    let mut mode: String = String::with_capacity(9);
    let read: u8 = (owner >> 2) & 0x1;
    let write: u8 = (owner >> 1) & 0x1;
    let exec: u8 = owner & 0x1;
    mode.push_str(match read {
        1 => "r",
        _ => "-",
    });
    mode.push_str(match write {
        1 => "w",
        _ => "-",
    });
    mode.push_str(match exec {
        1 => "x",
        _ => "-",
    });
    let read: u8 = (group >> 2) & 0x1;
    let write: u8 = (group >> 1) & 0x1;
    let exec: u8 = group & 0x1;
    mode.push_str(match read {
        1 => "r",
        _ => "-",
    });
    mode.push_str(match write {
        1 => "w",
        _ => "-",
    });
    mode.push_str(match exec {
        1 => "x",
        _ => "-",
    });
    let read: u8 = (others >> 2) & 0x1;
    let write: u8 = (others >> 1) & 0x1;
    let exec: u8 = others & 0x1;
    mode.push_str(match read {
        1 => "r",
        _ => "-",
    });
    mode.push_str(match write {
        1 => "w",
        _ => "-",
    });
    mode.push_str(match exec {
        1 => "x",
        _ => "-",
    });
    mode
}

/// ### instant_to_str
///
/// Format a `Instant` into a time string
pub fn time_to_str(time: SystemTime, fmt: &str) -> String {
    let datetime: DateTime<Local> = time.into();
    format!("{}", datetime.format(fmt))
}

/// ### fmt_millis
/// 
/// Format duration as {secs}.{millis}
pub fn fmt_millis(duration: Duration) -> String {
    let seconds: u128 = duration.as_millis() / 1000;
    let millis: u128 = duration.as_millis() % 1000;
    format!("{}.{:0width$}", seconds, millis, width = 3)
}

/// ### lstime_to_systime
///
/// Convert ls syntax time to System Time
/// ls time has two possible syntax:
/// 1. if year is current: %b %d %H:%M (e.g. Nov 5 13:46)
/// 2. else: %b %d %Y (e.g. Nov 5 2019)
pub fn lstime_to_systime(
    tm: &str,
    fmt_year: &str,
    fmt_hours: &str,
) -> Result<SystemTime, ParseError> {
    let datetime: NaiveDateTime = match NaiveDate::parse_from_str(tm, fmt_year) {
        Ok(date) => {
            // Case 2.
            // Return NaiveDateTime from NaiveDate with time 00:00:00
            date.and_hms(0, 0, 0)
        }
        Err(_) => {
            // Might be case 1.
            // We need to add Current Year at the end of the string
            let this_year: i32 = Utc::now().year();
            let date_time_str: String = format!("{} {}", tm, this_year);
            // Now parse
            match NaiveDateTime::parse_from_str(
                date_time_str.as_ref(),
                format!("{} %Y", fmt_hours).as_ref(),
            ) {
                Ok(dt) => dt,
                Err(err) => return Err(err),
            }
        }
    };
    // Convert datetime to system time
    let sys_time: SystemTime = SystemTime::UNIX_EPOCH;
    Ok(sys_time
        .checked_add(Duration::from_secs(datetime.timestamp() as u64))
        .unwrap_or(SystemTime::UNIX_EPOCH))
}

/// align_text_center
///
/// Align text to center for a given width
pub fn align_text_center(text: &str, width: u16) -> String {
    let indent_size: usize = match (width as usize) >= text.len() {
        // NOTE: The check prevents underflow
        true => (width as usize - text.len()) / 2,
        false => 0,
    };
    textwrap::indent(
        text,
        (0..indent_size).map(|_| " ").collect::<String>().as_str(),
    )
    .trim_end()
    .to_string()
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_utils_parse_remote_opt() {
        // Base case
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("172.26.104.1"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 22);
        assert_eq!(result.2, FileTransferProtocol::Sftp);
        assert!(result.3.is_some());
        // User case
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("root@172.26.104.1"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 22);
        assert_eq!(result.2, FileTransferProtocol::Sftp);
        assert_eq!(result.3.unwrap(), String::from("root"));
        // User + port
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("root@172.26.104.1:8022"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 8022);
        assert_eq!(result.2, FileTransferProtocol::Sftp);
        assert_eq!(result.3.unwrap(), String::from("root"));
        // Port only
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("172.26.104.1:4022"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 4022);
        assert_eq!(result.2, FileTransferProtocol::Sftp);
        assert!(result.3.is_some());
        // Protocol
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("ftp://172.26.104.1"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 21); // Fallback to ftp default
        assert_eq!(result.2, FileTransferProtocol::Ftp(false));
        assert!(result.3.is_none()); // Doesn't fall back
        // Protocol
        let result: (String, u16, FileTransferProtocol, Option<String>) =
        parse_remote_opt(&String::from("sftp://172.26.104.1"))
            .ok()
            .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 22); // Fallback to sftp default
        assert_eq!(result.2, FileTransferProtocol::Sftp);
        assert!(result.3.is_some()); // Doesn't fall back
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("scp://172.26.104.1"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 22); // Fallback to scp default
        assert_eq!(result.2, FileTransferProtocol::Scp);
        assert!(result.3.is_some()); // Doesn't fall back
        // Protocol + user
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("ftps://anon@172.26.104.1"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 21); // Fallback to ftp default
        assert_eq!(result.2, FileTransferProtocol::Ftp(true));
        assert_eq!(result.3.unwrap(), String::from("anon"));
        // All together now
        let result: (String, u16, FileTransferProtocol, Option<String>) =
            parse_remote_opt(&String::from("ftp://anon@172.26.104.1:8021"))
                .ok()
                .unwrap();
        assert_eq!(result.0, String::from("172.26.104.1"));
        assert_eq!(result.1, 8021); // Fallback to ftp default
        assert_eq!(result.2, FileTransferProtocol::Ftp(false));
        assert_eq!(result.3.unwrap(), String::from("anon"));

        // bad syntax
        assert!(parse_remote_opt(&String::from("://172.26.104.1")).is_err()); // Missing protocol
        assert!(parse_remote_opt(&String::from("omar://172.26.104.1")).is_err()); // Bad protocol
        assert!(parse_remote_opt(&String::from("172.26.104.1:abc")).is_err()); // Bad port
    }

    #[test]
    fn test_utils_fmt_pex() {
        assert_eq!(fmt_pex(7, 7, 7), String::from("rwxrwxrwx"));
        assert_eq!(fmt_pex(7, 5, 5), String::from("rwxr-xr-x"));
        assert_eq!(fmt_pex(6, 6, 6), String::from("rw-rw-rw-"));
        assert_eq!(fmt_pex(6, 4, 4), String::from("rw-r--r--"));
        assert_eq!(fmt_pex(6, 0, 0), String::from("rw-------"));
        assert_eq!(fmt_pex(0, 0, 0), String::from("---------"));
        assert_eq!(fmt_pex(4, 4, 4), String::from("r--r--r--"));
        assert_eq!(fmt_pex(1, 2, 1), String::from("--x-w---x"));
    }

    #[test]
    fn test_utils_time_to_str() {
        let system_time: SystemTime = SystemTime::from(SystemTime::UNIX_EPOCH);
        assert_eq!(
            time_to_str(system_time, "%Y-%m-%d"),
            String::from("1970-01-01")
        );
    }

    #[test]
    fn test_utils_lstime_to_systime() {
        // Good cases
        assert_eq!(
            lstime_to_systime("Nov 5 16:32", "%b %d %Y", "%b %d %H:%M")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1604593920)
        );
        assert_eq!(
            lstime_to_systime("Dec 2 21:32", "%b %d %Y", "%b %d %H:%M")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1606944720)
        );
        assert_eq!(
            lstime_to_systime("Nov 5 2018", "%b %d %Y", "%b %d %H:%M")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1541376000)
        );
        assert_eq!(
            lstime_to_systime("Mar 18 2018", "%b %d %Y", "%b %d %H:%M")
                .ok()
                .unwrap()
                .duration_since(SystemTime::UNIX_EPOCH)
                .ok()
                .unwrap(),
            Duration::from_secs(1521331200)
        );
        // bad cases
        assert!(lstime_to_systime("Oma 31 2018", "%b %d %Y", "%b %d %H:%M").is_err());
        assert!(lstime_to_systime("Feb 31 2018", "%b %d %Y", "%b %d %H:%M").is_err());
        assert!(lstime_to_systime("Feb 15 25:32", "%b %d %Y", "%b %d %H:%M").is_err());
    }

    #[test]
    fn test_utils_align_text_center() {
        assert_eq!(
            align_text_center("hello world!", 24),
            String::from("      hello world!")
        );
        // Bad case
        assert_eq!(
            align_text_center("hello world!", 8),
            String::from("hello world!")
        );
    }

    #[test]
    fn test_utils_fmt_millis() {
        assert_eq!(fmt_millis(Duration::from_millis(2048)), String::from("2.048"));
        assert_eq!(fmt_millis(Duration::from_millis(8192)), String::from("8.192"));
        assert_eq!(fmt_millis(Duration::from_millis(18192)), String::from("18.192"));
    }
}
