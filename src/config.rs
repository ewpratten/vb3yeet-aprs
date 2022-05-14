use std::io::Read;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub callsign: String,
    pub aprs_passcode: String,
}

impl Config {

    /// Load the configuration from a file
    pub fn load<P: AsRef<std::path::Path>>(path: P) -> Result<Self, std::io::Error> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}