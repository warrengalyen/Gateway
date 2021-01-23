//! ## Context
//!
//! `Context` is the module which provides all the functionalities related to the UI data holder, called Context

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
use super::input::InputHandler;
use crate::filetransfer::FileTransfer;
use crate::host::Localhost;

/// ## Context
///
/// Context holds data structures used by the ui
pub struct Context {
    pub scp_client: Box<dyn FileTransfer>,
    pub local: Localhost,
    pub(crate) input_hnd: InputHandler,
}

impl Context {
    /// ### new
    ///
    /// Instantiates a new Context
    pub fn new(scp_client: Box<dyn FileTransfer>, local: Localhost) -> Context {
        Context {
            scp_client: scp_client,
            local: local,
            input_hnd: InputHandler::new(),
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Disconnect client
        let _ = self.scp_client.disconnect();
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::filetransfer::sftp_transfer::SftpFileTransfer;
    use std::path::PathBuf;

    #[test]
    fn test_ui_context_new() {
        // Prepare stuff
        Context::new(
            build_sftp_client(),
            Localhost::new(PathBuf::from("/")).ok().unwrap(),
        );
    }

    fn build_sftp_client() -> Box<dyn FileTransfer> {
        let mut sftp_client: SftpFileTransfer = SftpFileTransfer::new();
        // Connect to remote
        assert!(sftp_client
            .connect(
                String::from("test.rebex.net"),
                22,
                Some(String::from("demo")),
                Some(String::from("password"))
            )
            .is_ok());
        Box::new(sftp_client)
    }
}