use std::fmt::Write;
use std::fs::OpenOptions;
use std::fs::{self, File};
use std::io::Write as IoWrite;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use guido::prelude::expect_context;

use crate::config::{self, Congif};
use crate::error::Error;

pub fn app_config() -> Result<Congif, Error> {
    let home =
        std::env::var("HOME").map_err(|_| Error::InternalError("Cannot find home".to_string()))?;

    let config_file = PathBuf::from(home)
        .join(".config")
        .join("app_launcher")
        .join("conf");

    if !config_file.exists() || !config_file.is_file() {
        println!("Config_file do not exists \nFalling back to default config");
        return Congif::new();
    }

    let file = fs::read_to_string(config_file).map_err(|e| Error::InternalError(e.to_string()))?;

    let mut conf = Congif::default();

    for line in file.lines() {
        let trimmed_line = line.trim();
        if let Some((value, key)) = trimmed_line.split_once("=") {
            match value {
                "window_width" => conf.window_width = key.parse().map_err(|_| Error::ParseError)?,
                "window_height" => {
                    conf.window_height = key.parse().map_err(|_| Error::ParseError)?
                }
                "trigger_area" => conf.trigger_area = key.parse().map_err(|_| Error::ParseError)?,
                "cache_file" => {
                    conf.cache_file = {
                        let cache_file = PathBuf::from(key);
                        if cache_file.exists() {
                            cache_file
                        } else {
                            return Err(Error::ParseError);
                        }
                    }
                }

                "icon_dim" => {
                    conf.icon_dim = {
                        let dim = match key.parse::<f32>() {
                            Ok(val) => val,
                            Err(_) => return Err(Error::ParseError),
                        };

                        if conf.window_height > (dim as u32) {
                            dim
                        } else {
                            return Err(Error::ParseError);
                        }
                    }
                }
                "icon_gap" => conf.icon_gap = key.parse().map_err(|_| Error::ParseError)?,
                _ => {}
            }
        }
    }
    Ok(conf)
}

#[derive(Default, Clone, PartialEq)]
pub struct AccessCache {
    pub cache: Vec<(String, u8)>,
}

impl AccessCache {
    pub fn increment(&mut self, app_name: &str) {
        let mut found = false;
        let stop_dec: [u8; 3] = [15, 12, 8];

        for ele in self.cache.iter_mut() {
            if ele.0 == app_name {
                found = true;
                if ele.1 < 20 {
                    ele.1 += 1;
                }
            } else {
                if !stop_dec.contains(&ele.1) && ele.1 > 4 {
                    ele.1 -= 1;
                }
            }
        }
        if !found {
            self.cache.push((app_name.to_string(), 1));
        }

        self.cache
            .sort_by_key(|&(_, value)| std::cmp::Reverse(value));

        println!("{:?}", self.cache);

        let clo = self.clone();

        Self::write_to_cache_file(clo);
    }

    fn write_to_cache_file(ac: AccessCache) {
        let cf = expect_context::<Rc<Congif>>();

        let mut content = String::new();

        for ele in ac.cache {
            let _ = writeln!(content, "{}={}", ele.0, ele.1);
        }

        let conf = (*cf).clone();

        std::thread::spawn(move || {
            let cfile = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .open(&conf.cache_file);

            let Ok(mut f) = cfile else { return };
            if let Err(e) = f.write_all(content.as_bytes()) {
                eprintln!("Failed to write data bytes to cache: {}", e);
            }
        });
    }

    pub fn read_cache_file(conf: Rc<Congif>) -> Result<AccessCache, Error> {
        let path = &conf.cache_file;

        if !path.exists() {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }

            println!("Cache file missing, initializing fresh empty instance.");
            return Ok(AccessCache::default());
        }

        let cache_file =
            fs::read_to_string(conf.cache_file.clone()).map_err(|_| Error::CacheParse)?;

        let mut cache = AccessCache::default();

        for line in cache_file.lines() {
            let trimmed = line.trim();
            if let Some((key, value)) = trimmed.split_once("=") {
                let val = value.parse::<u8>();

                if let Ok(v) = val {
                    cache.cache.push((key.to_string(), v));
                }
            }
        }

        cache.cache.sort_by_key(|&(_, value)| value);

        Ok(cache)
    }

    pub fn del_from_cache(&mut self, app_name: &str) {
        self.cache.retain(|app| app.0 != app_name);
        self.cache.sort_by_key(|&(_, value)| value);

        let clo = self.clone();

        std::thread::spawn(move || {
            Self::write_to_cache_file(clo);
        });
    }

    pub fn i_am(&self) -> &Self {
        self
    }
}
