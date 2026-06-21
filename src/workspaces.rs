use crate::error::Error;

use niri_ipc::{
    Action, Request, Response, Window,
    socket::{self, Socket},
};

// the below can be made to use the socket and continuous event stream, but i guess i dont need that.
pub fn get_windows() -> Result<Vec<Window>, Error> {
    let mut socket = Socket::connect().map_err(|e| Error::NitiIpcError(e.to_string()))?;

    let reply = socket
        .send(Request::Windows)
        .map_err(|e| Error::NitiIpcError(e.to_string()))?;

    match reply {
        Ok(rs) => match rs {
            Response::Windows(win_list) => Ok(win_list),

            _ => Err(Error::NitiIpcError("cannot get windows".to_string())),
        },

        Err(e) => Err(Error::NitiIpcError(e)),
    }
}

pub fn focus_window(win: &Window) {
    let mut socket = match Socket::connect() {
        Ok(s) => s,
        Err(e) => {
            let _ = notify_rust::Notification::new()
                .body(&format!("Socket connection failed: {}", e))
                .summary("app-launcher")
                .show();
            return;
        }
    };

    let reply = match socket.send(Request::Action(Action::FocusWindow { id: win.id })) {
        Ok(r) => r,
        Err(e) => {
            let _ = notify_rust::Notification::new()
                .body(&format!("Failed to send IPC request: {}", e))
                .summary("app-launcher")
                .show();
            return;
        }
    };

    match reply {
        Ok(Response::Handled) | Ok(Response::FocusedWindow(_)) => {}
        _ => {
            let _ = notify_rust::Notification::new()
                .body("Niri refused to focus the window")
                .summary("app-launcher")
                .show();
        }
    }
}
