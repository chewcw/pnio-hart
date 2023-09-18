use std::sync::RwLock;
use crate::config::Config;

pub trait Sender {
    fn setup(&self) -> anyhow::Result<()>;
    fn send(&self, data: String) -> anyhow::Result<()>;
    fn get_config(&self) -> &RwLock<Vec<Config>>;
}
