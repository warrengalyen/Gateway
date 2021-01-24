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
use crate::filetransfer::FileTransferProtocol;
use crate::host::Localhost;
use crate::ui::activities::{
    auth_activity::AuthActivity,
    filetransfer_activity::FileTransferActivity, filetransfer_activity::FileTransferParams,
    Activity,
};
use crate::ui::context::Context;

// Namespaces
use std::thread::sleep;
use std::time::Duration;

/// ### NextActivity
///
/// NextActivity identified the next identity to run once the current has ended
pub enum NextActivity {
    Authentication,
    FileTransfer,
}

/// ### ActivityManager
///
/// The activity manager takes care of running activities and handling them until the application has ended
pub struct ActivityManager {
    context: Context,
    ftparams: Option<FileTransferParams>,
    interval: Duration,
}

impl ActivityManager {
    /// ### new
    ///
    /// Initializes a new Activity Manager
    pub fn new(
        local_dir: &PathBuf,
        interval: Duration,
    ) -> Result<ActivityManager, ()> {
        // Prepare Context
        let host: Localhost = match Localhost::new(local_dir.clone()) {
            Ok(h) => h,
            Err(_) => return Err(()),
        };
        let ctx: Context = Context::new(host);
        Ok(ActivityManager {
            context: ctx,
            ftparams: None,
            interval: interval,
        })
    }

    /// ### set_filetransfer_params
    ///
    /// Set file transfer params
    pub fn set_filetransfer_params(
        &mut self,
        address: String,
        port: u16,
        protocol: FileTransferProtocol,
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
    pub fn run(&mut self, launch_activity: NextActivity) {
        let mut current_activity: Option<NextActivity> = Some(launch_activity);
        loop {
            current_activity = match current_activity {
                Some(activity) => match activity {
                    NextActivity::Authentication => self.run_authentication(),
                    NextActivity::FileTransfer => self.run_filetransfer(),
                },
                None => break, // Exit
            }
        }
    }

    // Loops

    /// ### run_authentication
    ///
    /// Loop for Authentication activity.
    /// Returns when activity terminates.
    /// Returns the next activity to run
    fn run_authentication(&mut self) -> Option<NextActivity> {
        // Prepare activity
        let mut activity: AuthActivity = AuthActivity::new();
        // Prepare result
        let result: Option<NextActivity>;
        // Create activity
        activity.on_create(&mut self.context);
        loop {
            // Draw activity
            activity.on_draw(&mut self.context);
            // Check if has to be terminated
            if activity.quit {
                // Quit activities
                result = None;
                break;
            }
            if activity.submit {
                // User submitted, set next activity
                result = Some(NextActivity::FileTransfer);
                // Get params
                self.ftparams = Some(FileTransferParams {
                    address: activity.address.clone(),
                    port: activity.port.parse::<u16>().ok().unwrap(),
                    username: match activity.username.len() {
                        0 => None,
                        _ => Some(activity.username.clone()),
                    },
                    password: match activity.password.len() {
                        0 => None,
                        _ => Some(activity.password.clone()),
                    },
                    protocol: activity.protocol.clone(),
                });
                break;
            }
            // Sleep for ticks
            sleep(self.interval);
        }
        // Destroy activity
        activity.on_destroy(&mut self.context);
        result
    }

    /// ### run_filetransfer
    ///
    /// Loop for FileTransfer activity.
    /// Returns when activity terminates.
    /// Returns the next activity to run
    fn run_filetransfer(&mut self) -> Option<NextActivity> {
        if self.ftparams.is_none() {
            return Some(NextActivity::Authentication);
        }
        // Prepare activity
        let mut activity: FileTransferActivity =
            FileTransferActivity::new(self.ftparams.take().unwrap());
        // Prepare result
        let result: Option<NextActivity>;
        // Create activity
        activity.on_create(&mut self.context);
        loop {
            // Draw activity
            activity.on_draw(&mut self.context);
            // Check if has to be terminated
            if activity.quit {
                // Quit activities
                result = None;
                break;
            }
            if activity.disconnected {
                // User disconnected, set next activity to authentication
                result = Some(NextActivity::Authentication);
                break;
            }
            // Sleep for ticks
            sleep(self.interval);
        }
        // Destroy activity
        activity.on_destroy(&mut self.context);
        result
    }
}
