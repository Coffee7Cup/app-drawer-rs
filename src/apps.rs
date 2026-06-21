use guido::prelude::{RwSignal, expect_context};
use guido::widgets::ImageSource;
use niri_ipc::socket::Socket;
use niri_ipc::{Action, Request, socket};
use std::fmt::{Write, write};

use crate::config::Congif;
use crate::init_app::AccessCache;
use crate::{app_launcher, error::Error};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct NiriApp {
    //TODO: i guess i can remove the file_name safely
    pub file_name: String,
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<PathBuf>,
    pub terminal: bool,
}

impl Default for NiriApp {
    fn default() -> Self {
        NiriApp {
            file_name: String::new(),
            name: String::new(),
            exec: String::new(),
            icon: None,
            icon_path: None,
            terminal: false,
        }
    }
}

impl NiriApp {
    pub fn get_apps_paths() -> Result<Vec<PathBuf>, Error> {
        let xdg_data_str =
            env::var("XDG_DATA_DIRS").unwrap_or_else(|_| "/usr/local/share:/usr/share".to_string());

        let mut data_dirs: Vec<PathBuf> = env::split_paths(&xdg_data_str).collect();

        let home_dir =
            env::var("HOME").map_err(|_| Error::InternalError("Cannot access Home".into()))?;

        let xdg_home_str =
            env::var("XDG_DATA_HOME").unwrap_or_else(|_| format!("{}/.local/share", home_dir));

        data_dirs.push(PathBuf::from(xdg_home_str));

        let mut desktop_files: Vec<PathBuf> = Vec::new();

        for mut path in data_dirs {
            path.push("applications");

            if !path.is_dir() {
                continue;
            }

            if let Ok(entry) = path.read_dir() {
                for ent in entry.flatten() {
                    let pat = ent.path();
                    if pat.is_file() && pat.extension().and_then(|s| s.to_str()) == Some("desktop")
                    {
                        desktop_files.push(pat);
                    }
                }
            }
        }

        desktop_files.sort();
        desktop_files.dedup();
        Ok(desktop_files)
    }

    pub fn get_apps() -> Result<Arc<[Arc<NiriApp>]>, Error> {
        let paths = Self::get_apps_paths()?;
        let mut apps: Vec<Arc<NiriApp>> = Vec::new();
        let mut seen_names = HashSet::new();

        for path in paths {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut is_main = false;
            let mut app = NiriApp::default();
            if let Some(file_name) = path.file_prefix() {
                app.file_name = file_name.to_string_lossy().to_string();
            };
            let mut skip_app = false;

            for line in content.lines() {
                let trimmed_line = line.trim();

                match trimmed_line {
                    "[Desktop Entry]" => {
                        is_main = true;
                    }
                    s if s.starts_with('[') && is_main => {
                        break;
                    }
                    s if is_main => {
                        if let Some((key, value)) = s.split_once('=') {
                            match key.trim() {
                                "Name" => {
                                    app.name = value.trim().to_string();
                                }
                                "Icon" => {
                                    let icon = value.trim().to_string();

                                    if !icon.is_empty() {
                                        let icon_path = lookup_icon(&icon);
                                        app.icon = Some(icon);
                                        app.icon_path = icon_path;
                                    }
                                }
                                "Exec" => {
                                    app.exec = value.trim().to_string();
                                }
                                "NoDisplay" => {
                                    if value.trim() == "true" {
                                        skip_app = true;
                                        break;
                                    }
                                }
                                "Terminal" => {
                                    if value.trim() == "true" {
                                        app.terminal = true
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                    _ => {}
                }
            }

            if skip_app || app.name.is_empty() || app.exec.is_empty() {
                continue;
            }

            if seen_names.insert(app.name.clone()) {
                apps.push(Arc::new(app));
            }
        }

        // Convert Vec<Rc<NiriApp>> into Rc<[Rc<NiriApp>]> to match return signature
        Ok(Arc::from(apps.into_boxed_slice()))
    }
}

impl NiriApp {
    pub fn open(&self) -> Result<(), Error> {
        let conf = expect_context::<RwSignal<AccessCache>>();

        conf.update(|c| {
            c.increment(&self.name);
        });

        println!("launching app: {}", self.name);

        let mut command = String::new();

        if self.terminal {
            let _term = get_term();
            if let Some(term) = _term {
                println!("term: {}", term);
                write!(command, "{} ", term);
            }
        }

        write!(command, "{}", self.exec);
        println!(" command : {}", command);

        let req = niri_ipc::Request::Action(Action::SpawnSh { command: command });

        let mut socket =
            niri_ipc::socket::Socket::connect().map_err(|e| Error::NitiIpcError(e.to_string()))?;
        let res = socket.send(req);

        match res {
            Err(_) => {
                notify_rust::Notification::new()
                    .summary("app_launcher")
                    .body("Failed to open app ");
            }

            Ok(niri_res) => {
                if niri_res.is_err() {
                    notify_rust::Notification::new()
                        .summary("app_launcher")
                        .body("Failed to open app <- niri responded");
                }
            }
        }
        Ok(())
    }
}

pub fn lookup_icon(id: &str) -> Option<PathBuf> {
    if let Some(path) = freedesktop_icons::lookup(id).with_size(64).find() {
        return Some(path);
    }

    if let Some(path) = linicon::lookup_icon(id)
        .with_size(512)
        .filter_map(|result| result.ok())
        .next()
    {
        return Some(path.path);
    }

    None
}

pub fn get_term() -> Option<String> {
    let all_terms = [
        "kitty",
        "alacritty",
        "ghostty",
        "wezterm",
        "foot",
        "gnome-terminal",
        "konsole",
        "xfce4-terminal",
        "mate-terminal",
        "terminator",
        "termite",
        "st",
        "urxvt",
        "xterm",
    ];

    if let Ok(path_var) = env::var("PATH") {
        let paths: Vec<_> = env::split_paths(&path_var).collect();

        for term in all_terms {
            for entry in &paths {
                if entry.join(term).exists() {
                    return Some(term.to_string());
                }
            }
        }
    }

    None
}
