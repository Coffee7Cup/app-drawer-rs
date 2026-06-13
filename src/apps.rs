use guido::widgets::ImageSource;
use niri_ipc::{Action, Request, socket};

use crate::error::Error;
use std::{env, fs, path::PathBuf};

#[derive(Debug, Clone)]
pub struct NiriApp {
    pub name: String,
    pub exec: String,
    pub icon: Option<String>,
    pub icon_path: Option<PathBuf>,
}

impl Default for NiriApp {
    fn default() -> Self {
        NiriApp {
            name: String::new(),
            exec: String::new(),
            icon: None,
            icon_path: None,
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

    pub fn get_apps() -> Vec<NiriApp> {
        let paths = Self::get_apps_paths().unwrap();
        let mut apps: Vec<NiriApp> = Vec::new();

        let home_dir = env::var("HOME")
            .map_err(|_| Error::InternalError("Cannot access Home".into()))
            .unwrap();
        let local_icon_path = format!("{}/.local/share/icons", home_dir);
        for path in paths {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut is_main = false;
            let mut app = NiriApp::default();
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
                                        let icon_path = linicon::lookup_icon(&icon)
                                            .with_search_paths(&[&local_icon_path]);

                                        //TODO: Here there are icons for apps but still fallling back to default icons because of the not matched size.
                                        app.icon = Some(icon);
                                        if let Ok(val) = icon_path {
                                            if let Some(icon) = val.with_size(64).next()
                                                && let Ok(icon2) = icon
                                            {
                                                app.icon_path = Some(icon2.path);
                                            }
                                        }
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

            apps.push(app);
        }

        apps.dedup_by(|a, b| a.name == b.name);

        apps
    }
}

impl NiriApp {
    pub fn open(&self) {
        println!("launching app: {}", self.name);
        let req = niri_ipc::Request::Action(Action::SpawnSh {
            command: self.exec.to_string(),
        });

        let mut socket = niri_ipc::socket::Socket::connect().unwrap();
        let res = socket.send(req);

        match res {
            // 1. Properly match against the error variant
            Err(e) => {
                println!("IPC communication error occurred: {:?}", e);
            }

            // 2. Look inside the successful response
            Ok(niri_res) => {
                // niri_res is likely a Result itself (hence your Ok(Ok(Handled)) output)
                match niri_res {
                    Ok(action_reply) => {
                        println!("Niri handled the action successfully: {:?}", action_reply);
                    }
                    Err(niri_error) => {
                        println!(
                            "Niri accepted the request but failed to run it: {:?}",
                            niri_error
                        );
                    }
                }
            }
        }
    }
}
