#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(improper_ctypes)]
#![allow(unused)]

mod cli;
mod client;
mod config;
mod device;
mod dto;
mod protocol;
mod transport;

use crate::{cli::WorkingMode, client::worker::Worker};
use anyhow::anyhow;
use clap::Parser;
use cli::Cli;
use client::{iotedge::IotEdge, kafka::Kafka, sender::Sender};
use config::Config;
use signal_hook::{
    consts::{SIGHUP, SIGINT, SIGTERM},
    iterator::Signals,
};
use std::{
    fs::File,
    io::{BufReader, Read},
    net::Ipv4Addr,
    str::FromStr,
    sync::{Arc, RwLock},
    thread, time,
};

fn main() -> anyhow::Result<()> {
    env_logger::init();

    let args = Cli::parse();

    let mut signals = Signals::new([SIGTERM, SIGINT, SIGHUP])?;
    thread::spawn(move || {
        for _ in signals.forever() {
            log::info!("received shutdown request");
            std::process::exit(0);
        }
    });

    // connection string, skip if not using
    let connection_string = match args.connection_string {
        conn_str if conn_str.is_ascii() => Some(conn_str),
        _ => None,
    };

    // egress clients
    let mut iotedge_client;
    let kafka_client;
    let sender: &dyn Sender;

    // working mode
    let working_mode = WorkingMode::from_str(&args.mode)?;
    match working_mode {
        WorkingMode::IotEdgeMode => {
            iotedge_client = IotEdge::new(connection_string)?;
            // register module twin update callback
            iotedge_client.set_module_twin_callback();

            sender = &iotedge_client;
        }
        WorkingMode::LocalMode => {
            let config_file_path = args.config.ok_or(anyhow!(
                "config file path is required in local mode, check --help"
            ))?;

            let config_file = File::open(config_file_path)?;
            let mut buf_reader = BufReader::new(config_file);
            let mut content = String::new();
            buf_reader.read_to_string(&mut content)?;

            let local_file_config = Config::deserialize(content.as_str())?;
            let _config = Arc::new(RwLock::new(local_file_config));
            kafka_client = Kafka::new();

            sender = &kafka_client;
        }
    }

    let src_ip_address = args.src_ip_address.parse::<Ipv4Addr>()?;
    let mut worker = Worker::new(sender);
    loop {
        worker.evaluate(src_ip_address);
        worker.read();
        log::info!("sleep for {} seconds waiting for next loop", args.interval);
        thread::sleep(time::Duration::from_secs(args.interval as u64));
    }
}
