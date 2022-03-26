use std::error;

use serde::Deserialize;

macro_rules! config_defaults {
    ($($name:ident -> $type:ty: $value:expr;)*) => {
    $(
        fn $name() -> $type {
            $value
        }
    )*
    }
}

config_defaults! {
    default_general_prefix -> String: "regite".to_string();
    default_general_graphite_address -> String: "localhost:2003".to_string();
}

#[derive(Debug, Default, Deserialize)]
pub struct Config {
    pub general: General,
    #[serde(default)]
    pub job: Vec<Job>,
}

#[derive(Debug, Default, Deserialize)]
pub struct General {
    #[serde(default = "default_general_prefix")]
    pub prefix: String,
    pub hostname: String,
    #[serde(default = "default_general_graphite_address")]
    pub graphite_address: String,
}

#[derive(Debug, Default, Deserialize)]
pub struct Job {
    pub interval: u64,
    pub command: String,
    pub regex: String,
    pub output: Vec<Output>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct Output {
    pub name: String,
    pub value: String,
}

pub fn load_config(file_path: &str) -> Result<Config, Box<dyn error::Error>> {
    let file_contents = std::fs::read_to_string(file_path)?;
    Ok(toml::from_str(&file_contents)?)
}
