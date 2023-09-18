use super::sender::Sender;
use crate::config::Config;
use std::sync::RwLock;

pub struct Kafka {}

impl Kafka {
    pub fn new() -> Self {
        Self {}
    }
}

impl Sender for Kafka {
    fn setup(&self) -> anyhow::Result<()> {
        todo!()
    }

    fn send(&self, data: String) -> anyhow::Result<()> {
        todo!()
    }

    fn get_config(&self) -> &RwLock<Vec<Config>> {
        todo!()
    }
}
