use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::{OnceLock, RwLock};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub port: u16,
}

impl Config {
    fn config() -> &'static RwLock<config::Config> {
        static CONFIG: OnceLock<RwLock<config::Config>> = OnceLock::new();
        CONFIG.get_or_init(|| {
            // 判断文件是否存在
            if !Path::new("config.json").exists() {
                std::fs::write("config.json", {
                    let mut config = Config::default();
                    config.port = 7777;
                    serde_json::to_string_pretty(&config).unwrap()
                })
                    .unwrap();
            }
            let backend_data = config::Config::builder()
                .add_source(config::File::with_name("config.json"))
                .build()
                .unwrap();
            RwLock::new(backend_data)
        })
    }

    pub fn get_config() -> Self {
        Self::config()
            .read()
            .unwrap()
            .clone()
            .try_deserialize()
            .unwrap()
    }
}
