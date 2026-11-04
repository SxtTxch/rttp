use std::{fs::File};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub root: String,
    pub index: String,
    pub error_pages: std::collections::HashMap<u16, String>, 
}

#[derive(Debug, Deserialize)]
pub struct AppConfig {
    pub server: ServerConfig,
}

pub fn get_config() -> AppConfig {
    let file = File::open("config.yaml").expect("Failed to open config.yaml");
    serde_yaml::from_reader(file).expect("Failed to parse YAML")
}
