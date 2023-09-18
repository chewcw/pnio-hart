use anyhow::anyhow;
use clap::Parser;
use std::{path::PathBuf, str::FromStr};

#[derive(Parser, Debug)]
pub struct Cli {
    /// mode for working, see enum WorkingMode
    #[clap(short, long)]
    #[clap(default_value = "local")]
    pub mode: String,

    /// config file path, used in local mode
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    /// interval for each data collection
    #[clap(short, long)]
    #[clap(default_value_t = 30)]
    pub interval: u8,

    /// connection string of the Iot Edge module,
    /// used in iotedge mode, leave empty to use Iot Edge
    /// environment variables
    #[clap(long)]
    #[clap(default_value = "")]
    pub connection_string: String,

    // TODO: detect the ip address automatically?
    /// data collector ip address
    #[clap(short, long)]
    #[clap(default_value = "")]
    pub src_ip_address: String,
}

#[derive(Clone, Debug)]
pub enum WorkingMode {
    IotEdgeMode,
    LocalMode,
}

impl FromStr for WorkingMode {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "iotedge" => Ok(Self::IotEdgeMode),
            "local" => Ok(Self::LocalMode),
            _ => Err(anyhow!("only iotedge and local mode are supported")),
        }
    }
}
