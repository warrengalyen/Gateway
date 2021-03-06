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

const GATEWAY_VERSION: &str = env!("CARGO_PKG_VERSION");
const GATEWAY_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");

// Crates
extern crate getopts;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate magic_crypt;
extern crate rpassword;

// External libs
use getopts::Options;
use std::env;
use std::path::PathBuf;
use std::time::Duration;

// Include
mod activity_manager;
mod bookmarks;
mod filetransfer;
mod fs;
mod host;
mod system;
mod ui;
mod utils;

// namespaces
use activity_manager::{ActivityManager, NextActivity};
use filetransfer::FileTransferProtocol;

/// ### print_usage
///
/// Print usage

fn print_usage(opts: Options) {
    let brief = String::from("Usage: gateway [options]... [protocol://user@address:port]");
    print!("{}", opts.usage(&brief));
    println!("\nPlease, report issues to <https://github.com/warrengalyen/Gateway>");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    //Program CLI options
    let mut address: Option<String> = None; // None
    let mut port: u16 = 22; // Default port
    let mut username: Option<String> = None; // Default username
    let mut password: Option<String> = None; // Default password
    let mut protocol: FileTransferProtocol = FileTransferProtocol::Sftp; // Default protocol
    let mut ticks: Duration = Duration::from_millis(10);
    //Process options
    let mut opts = Options::new();
    opts.optopt(
        "P",
        "password",
        "Provide password from CLI (use at your own risk)",
        "<password>",
    );
    opts.optopt("T", "ticks", "Set UI ticks; default 10ms", "<ms>");
    opts.optflag("v", "version", "");
    opts.optflag("h", "help", "Print this menu");
    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("{}", f.to_string());
            std::process::exit(255);
        }
    };
    // Help
    if matches.opt_present("h") {
        print_usage(opts);
        std::process::exit(255);
    }
    // Version
    if matches.opt_present("v") {
        eprintln!(
            "Gateway - {} - Developed by {}",
            GATEWAY_VERSION, GATEWAY_AUTHORS,
        );
        std::process::exit(255);
    }
    // Match password
    if let Some(passwd) = matches.opt_str("P") {
        password = Some(passwd);
    }
    // Match ticks
    if let Some(val) = matches.opt_str("T") {
        match val.parse::<usize>() {
            Ok(val) => ticks = Duration::from_millis(val as u64),
            Err(_) => {
                eprintln!("Ticks is not a number '{}'", val);
                print_usage(opts);
                std::process::exit(255);
            }
        }
    }
    // Check free args
    let extra_args: Vec<String> = matches.free;
    if let Some(remote) = extra_args.get(0) {
        // Parse address
        match utils::parser::parse_remote_opt(remote) {
            Ok((addr, portn, proto, user)) => {
                // Set params
                address = Some(addr);
                port = portn;
                protocol = proto;
                username = user;
            }
            Err(err) => {
                eprintln!("Bad address option: {}", err);
                print_usage(opts);
                std::process::exit(255);
            }
        }
    }
    // Get working directory
    let wrkdir: PathBuf = match env::current_dir() {
        Ok(dir) => dir,
        Err(_) => PathBuf::from("/"),
    };
    // Initialize client if necessary
    let mut start_activity: NextActivity = NextActivity::Authentication;
    if address.is_some() {
        if password.is_none() {
            // Ask password if unspecified
            password = match rpassword::read_password_from_tty(Some("Password: ")) {
                Ok(p) => {
                    if p.is_empty() {
                        None
                    } else {
                        Some(p)
                    }
                }
                Err(_) => {
                    eprintln!("Could not read password from prompt");
                    std::process::exit(255);
                }
            };
        }
        // In this case the first activity will be FileTransfer
        start_activity = NextActivity::FileTransfer;
    }
    // Create activity manager (and context too)
    let mut manager: ActivityManager = match ActivityManager::new(&wrkdir, ticks) {
        Ok(m) => m,
        Err(_) => {
            eprintln!("Invalid directory '{}'", wrkdir.display());
            std::process::exit(255);
        }
    };
    // Set file transfer params if set
    if let Some(address) = address {
        manager.set_filetransfer_params(address, port, protocol, username, password);
    }
    // Run
    manager.run(start_activity);
    // Then return
    std::process::exit(0);
}
