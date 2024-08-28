use serde_derive::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

#[derive(Debug, Deserialize)]
pub struct Config {
    pub mqtt: Mqtt,
    pub i2c: I2C,
    pub ios: IOs,
}

#[derive(Debug, Deserialize)]
pub struct Mqtt {
    pub connection: Connection,
    pub credentials: Credentials,
}

#[derive(Debug, Deserialize)]
pub struct I2C {
    #[serde(default = "default_i2c_path")]
    pub device: String,
}

fn default_i2c_path() -> String {
    "/dev/i2c-1".to_string()
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

#[derive(Debug, Deserialize)]
pub struct IOs {
    pub inputs: Vec<IO>,
    pub outputs: Vec<IO>,
}

#[derive(Debug, Deserialize)]
pub struct IO {
    pub address: u8,
    pub chip: String,
    #[serde(default)]
    pub pins: Vec<String>,
}

pub trait ConfigParser {
    fn parse_config(self) -> Result<Config>;
}

impl ConfigParser for &str {
    fn parse_config(self) -> Result<Config> {
        let config: Config = serde_yaml::from_str(self)
            .with_context(|| format!("Failed to parse YAML document: \n{}", self))?;

        // TODO perform input validation
        // - IOs must have unique names
        Ok(config)
    }
}

pub fn read_config_file<P: AsRef<Path>>(file_path: P) -> Result<String> {
    let mut file = File::open(&file_path).with_context(|| {
        format!(
            "Failed to open config file: {}",
            file_path.as_ref().display()
        )
    })?;
    let mut content = String::new();
    file.read_to_string(&mut content).with_context(|| {
        format!(
            "Failed to read config file: {}",
            file_path.as_ref().display()
        )
    })?;
    Ok(content)
}

impl ConfigParser for PathBuf {
    fn parse_config(self) -> Result<Config> {
        let yaml_content = read_config_file(&self)?;
        yaml_content.as_str().parse_config()
    }
}

impl ConfigParser for &Path {
    fn parse_config(self) -> Result<Config> {
        let yaml_content = read_config_file(self)?;
        yaml_content.as_str().parse_config()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_tracked_config_success() {
        let current_file = Path::new(file!());
        let parent_dir = current_file.parent().unwrap();
        let yaml_file_path = parent_dir.join("../config/config.yaml");
        println!("Derived config file path: {:?}", yaml_file_path);
        let result = yaml_file_path.parse_config();

        if let Err(e) = &result {
            println!("Error parsing config: {:?}", e);
        }

        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_config_success() {
        let yaml_str = r#"
mqtt:
  connection:
    host: "localhost"
    port: 1883
    id: "mqtt_client"
  credentials:
    user: "user"
    password: "pass"
i2c:
  device: /dev/i2c-0
ios:
  inputs:
  - address: 0x20
    chip: PCF8574
    pins: []
  outputs:
  - address: 0x21
    chip: MAX7311
    pins:
      - one
      - two
      - three
  - address: 0x22
    chip: PCF8574
"#;

        let result = yaml_str.parse_config();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert_eq!(config.mqtt.connection.host, "localhost");
        assert_eq!(config.mqtt.connection.port, 1883);
        assert_eq!(config.mqtt.connection.id, "mqtt_client");
        assert_eq!(config.mqtt.credentials.user, "user");
        assert_eq!(config.mqtt.credentials.password, "pass");
    }

    #[test]
    fn test_parse_config_failure() {
        let invalid_yaml_str = r#"
mqtt:
  connection:
    host: "localhost"
    port: "not_a_number"
    id: "mqtt_client"
  credentials:
    user: "user"
    password: "pass"
"#;

        let result = invalid_yaml_str.parse_config();
        assert!(result.is_err());

        if let Err(error) = result {
            let error_msg = format!("{:#}", error);
            assert!(
                error_msg.contains("Failed to parse YAML document"),
                "Unexpected error message: {}",
                error_msg
            );
            assert!(
                error_msg.contains("invalid type: string \"not_a_number\", expected u16"),
                "Unexpected error message: {}",
                error_msg
            );
        }
    }
}
