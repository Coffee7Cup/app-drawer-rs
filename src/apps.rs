use crate::error::Error;
use std::{env, fs, path::PathBuf};

#[derive(Default, Debug)]
pub struct App {
    pub name: String,
    pub exec: String,
    pub icon: String,
    pub icon_path: PathBuf,
}

impl App {
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

    pub fn get_apps() -> Result<Vec<App>, Error> {
        let paths = Self::get_apps_paths()?;
        let mut apps: Vec<App> = Vec::new();

        let home_dir =
            env::var("HOME").map_err(|_| Error::InternalError("Cannot access Home".into()))?;
        let local_icon_path = format!("{}/.local/share/icons", home_dir);
        let default_icon = PathBuf::from("/usr/share/appication/app-drawer/defaults/app-icon.png");

        for path in paths {
            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let mut is_main = false;
            let mut app = App::default();
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
                                    app.icon = value.trim().to_string();

                                    if app.icon.is_empty() {
                                        app.icon = "Default".to_string();
                                        app.icon_path = default_icon.clone();
                                    }

                                    let icon_path = linicon::lookup_icon(&app.icon)
                                        .with_search_paths(&[&local_icon_path]);

                                    if let Ok(val) = icon_path {
                                        if let Some(icon) = val.with_size(64).next() {
                                            if let Ok(icon2) = icon {
                                                app.icon_path = icon2.path;
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

        Ok(apps)
    }
}
