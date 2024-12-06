use crate::mirror::core::network_loop::stop_signal;
use crate::{log_error, log_info};
use notify::event::{DataChange, ModifyKind};
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::mpsc::channel;
use std::sync::{OnceLock, RwLock};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Config {
    pub port: u16,
}

impl Config {
    const CONFIG_FILE: &'static str = "config.json";
    fn config() -> &'static RwLock<config::Config> {
        static CONFIG: OnceLock<RwLock<config::Config>> = OnceLock::new();
        CONFIG.get_or_init(|| {
            // 判断文件是否存在
            if !Path::new(Self::CONFIG_FILE).exists() {
                std::fs::write(Self::CONFIG_FILE, {
                    let mut config = Config::default();
                    config.port = 7777;
                    serde_json::to_string_pretty(&config).unwrap()
                })
                    .unwrap();
            }

            let backend_data = config::Config::builder()
                .add_source(config::File::with_name(Self::CONFIG_FILE))
                .build()
                .unwrap();
            RwLock::new(backend_data)
        })
    }

    pub fn watch() {
        // Create a channel to receive the events.
        let (tx, rx) = channel();

        // Automatically select the best implementation for your platform.
        // You can also access each implementation directly e.g. INotifyWatcher.
        let mut watcher: RecommendedWatcher = Watcher::new(
            tx,
            notify::Config::default().with_poll_interval(Duration::from_secs(3)),
        )
            .unwrap();

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        watcher
            .watch(Path::new(Self::CONFIG_FILE), RecursiveMode::NonRecursive)
            .unwrap_or_else(|_| {});

        // This is a simple loop, but you may want to use more complex logic here,
        // for example to handle I/O.
        while let Ok(event) = rx.recv() {
            if *stop_signal() {
                break;
            }
            match event {
                Ok(Event {
                       kind: notify::event::EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                       ..
                   }) => {
                    match config::Config::builder()
                        .add_source(config::File::with_name(Self::CONFIG_FILE))
                        .build()
                    {
                        Ok(backend_data) => {
                            *Self::config().write().unwrap() = backend_data;
                            log_info!(format!("{} has been updated", Self::CONFIG_FILE));
                        }
                        Err(e) => {
                            log_error!(format!("watch error: {:?}", e));
                        }
                    }
                }
                Err(e) => {
                    log_error!(format!("watch error: {:?}", e));
                }
                _ => {}
            }
        }
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
