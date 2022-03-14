use std::error;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub general: General,
}

#[derive(Debug, Deserialize)]
pub struct General {
    pub hostname: String,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    let file_contents = std::fs::read_to_string(file_path)?;
    Ok(toml::from_str(&file_contents)?)
}
