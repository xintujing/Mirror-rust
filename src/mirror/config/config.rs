use serde::{Deserialize, Serialize};
use std::sync::{OnceLock, RwLock};

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub port: u16,
}

impl Config {
    fn config() -> &'static RwLock<config::Config> {
        static CONFIG: OnceLock<RwLock<config::Config>> = OnceLock::new();
        CONFIG.get_or_init(|| {
            let backend_data = config::Config::builder()
                .add_source(config::File::with_name("config"))
                .build()
                .unwrap();
            RwLock::new(backend_data)
        })
    }

    pub fn get_config() -> Config {
        Self::config()
            .read()
            .unwrap()
            .clone()
            .try_deserialize()
            .unwrap()
    }
}
