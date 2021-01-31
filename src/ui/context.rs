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
extern crate crossterm;
extern crate tui;

// Locals
use super::input::InputHandler;
use crate::host::Localhost;

// Includes
use crossterm::execute;
use crossterm::event::DisableMouseCapture;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{stdout, Stdout, Write};
use tui::backend::CrosstermBackend;
use tui::Terminal;

/// ## Context
///
/// Context holds data structures used by the ui
pub struct Context {
    pub local: Localhost,
    pub(crate) input_hnd: InputHandler,
    pub(crate) terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl Context {
    /// ### new
    ///
    /// Instantiates a new Context
    pub fn new(local: Localhost) -> Context {
        // Create terminal
        let mut stdout = stdout();
        assert!(execute!(stdout, EnterAlternateScreen).is_ok());
        Context {
            local: local,
            input_hnd: InputHandler::new(),
            terminal: Terminal::new(CrosstermBackend::new(stdout)).unwrap()
        }
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        // Re-enable terminal stuff
        let _ = execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        drop(self);
    }
}

/*
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
*/
