use serde::Deserialize;
use std::fs;
use std::path::Path;

pub struct ConfigManager {
    pub rss_limit: i32,
    pub idel_time_limit: i32,
    pub memory_change_rate: f32,
    pub reflush_time: u64,
    pub strategy: String
}

#[derive(Deserialize)]
struct Config {
    chrome_memory_manager: ChromeMemoryManager
}

#[derive(Deserialize)]
struct ChromeMemoryManager {
    rss_limit: i32,
    idel_time_limit: i32,
    memory_change_rate: f32,
    reflush_time: u64,
    strategy: String
}

impl ConfigManager {
    pub fn new(config_path: &Path) -> Self {
        let context = fs::read_to_string(config_path).expect("Failed to read config file");
        let config: Config = toml::from_str(&context).expect("Failed to parse config file");
        Self {  rss_limit: config.chrome_memory_manager.rss_limit, 
                idel_time_limit: config.chrome_memory_manager.idel_time_limit,
                memory_change_rate: config.chrome_memory_manager.memory_change_rate,
                reflush_time: config.chrome_memory_manager.reflush_time, 
                strategy:  config.chrome_memory_manager.strategy }
            }            
}