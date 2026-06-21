use std::path::PathBuf;

use crate::error::Error;

#[derive(Clone, Default)]
pub struct Congif {
    pub window_width: u32,
    pub window_height: u32,
    pub trigger_area: u32,
    pub window_thickness: u32,
    pub window_length: u32,
    pub icon_dim: f32,
    pub icon_gap: f32,
    pub cache_file: PathBuf,
}

impl Congif {
    pub fn new() -> Result<Self, Error> {
        let home = std::env::var("HOME").map_err(|e| Error::InternalError(e.to_string()))?;

        Ok(Congif {
            window_width: 800,
            window_height: 100,
            trigger_area: 5,
            window_thickness: 100,
            window_length: 600,
            icon_dim: 90.0,
            icon_gap: 5.0,
            cache_file: PathBuf::from(home)
                .join(".cache")
                .join("app-launcher")
                .join("app-launch.cache"),
        })
    }
}
