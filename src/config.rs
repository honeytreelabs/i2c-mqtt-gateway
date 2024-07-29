use serde_derive::Deserialize;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use anyhow::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mqtt: Mqtt,
}

#[derive(Debug, Deserialize)]
pub struct Mqtt {
    pub connection: Connection,
    pub credentials: Credentials,
}

#[derive(Debug, Deserialize)]
pub struct Connection {
    pub host: String,
    pub port: u16,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Credentials {
    pub user: String,
    pub password: String,
}

pub fn parse_config<P: AsRef<Path>>(file_path: P) -> Result<Config> {
    let file = File::open(file_path)?;

    let reader = BufReader::new(file);

    let config: Config = serde_yaml::from_reader(reader)?;

    Ok(config)
}
