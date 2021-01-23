//! ## ActivityManager
//!
//! `activity_manager` is the module which provides run methods and handling for activities

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

use std::path::PathBuf;

// Deps
use crate::filetransfer::FileTransfer;
use crate::host::Localhost;
use crate::ui::activities::{auth_activity::AuthActivity, auth_activity::ScpProtocol, Activity};
use crate::ui::context::Context;

/// ### NextActivity
///
/// NextActivity identified the next identity to run once the current has ended
pub enum NextActivity {
    Authentication,
    FileTransfer,
}

/// ### FileTransferParams
///
/// Holds connection parameters for file transfers
struct FileTransferParams {
    address: String,
    port: u16,
    protocol: ScpProtocol,
    username: Option<String>,
    password: Option<String>,
}

/// ### ActivityManager
///
/// The activity manager takes care of running activities and handling them until the application has ended
pub struct ActivityManager {
    context: Context,
    ftparams: Option<FileTransferParams>,
}

impl ActivityManager {
    /// ### new
    ///
    /// Initializes a new Activity Manager
    pub fn new(client: Box<dyn FileTransfer>, local_dir: &PathBuf) -> Result<ActivityManager, ()> {
        // Prepare Context
        let host: Localhost = match Localhost::new(local_dir.clone()) {
            Ok(h) => h,
            Err(_) => return Err(()),
        };
        let ctx: Context = Context::new(client, host);
        Ok(ActivityManager {
            context: ctx,
            ftparams: None,
        })
    }

    /// ### set_filetransfer_params
    ///
    /// Set file transfer params
    pub fn set_filetransfer_params(
        &mut self,
        address: String,
        port: u16,
        protocol: ScpProtocol,
        username: Option<String>,
        password: Option<String>,
    ) {
        self.ftparams = Some(FileTransferParams {
            address: address,
            port: port,
            protocol: protocol,
            username: username,
            password: password,
        });
    }

    /// ### run
    ///
    ///
    /// Loop for activity manager. You need to provide the activity to start with
    /// Returns the exitcode
    pub fn run(&mut self, launch_activity: NextActivity) -> i32 {
        0
    }
}
