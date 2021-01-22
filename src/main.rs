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

// External libs
use getopts::Options;
use std::env;

// Include
mod filetransfer;
mod fs;
mod host;

/// ### print_usage
///
/// Print usage

fn print_usage(opts: Options) {
    let brief = "Usage: gateway [Options]... Remote".to_string();
    print!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    //Program CLI options
    // TODO: insert opts here
    //Process options
    let mut opts = Options::new();
    // opts.optopt("c", "command", "Specify command to run. Shell returns after running the command", "<command>");
    // opts.optopt("C", "config", "Specify YAML configuration file", "<config>");
    // opts.optopt("l", "lang", "Specify shell language", "<ru|рус>");
    // opts.optopt("s", "shell", "Force the shell binary path", "</bin/bash>");
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
    // TODO: ...
    std::process::exit(0);
}
