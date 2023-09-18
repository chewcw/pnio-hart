use serde::{self, Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Config {
    pub ip_address: String,
    /// port is the lookup port, actual port being used to communicate
    /// with the device is obtained through the program
    pub port: u16,
    pub hart_devices: Vec<ConfigHartDevice>,
    // device_name is the profinet device name i.e. the model used for searching the
    // device when performing the lookup, for example, `6ES7 155-6AU01-0BN0`
    pub device_name: String,
}

impl Config {
    pub fn serialize(&self) -> anyhow::Result<String> {
        let str = serde_yaml::to_string(&self)?;
        Ok(str)
    }

    pub fn deserialize(content: &str) -> anyhow::Result<Vec<Self>> {
        let config = serde_yaml::from_str::<Vec<Self>>(content)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip_address: "127.0.0.1".to_string(),
            port: 0,
            hart_devices: vec![ConfigHartDevice {
                slot_number: 0,
                subslot_number: 0,
                hart_commands: vec![],
                request_data_record_number: 80,
                response_data_record_number: 81,
                hart_device_name: "hart_device_name".to_string(),
            }],
            device_name: "device_name".to_string(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename(serialize = "hart_devices"))]
pub struct ConfigHartDevice {
    pub slot_number: u16,
    pub subslot_number: u16,
    /// hart_commands stores an list of hart command needed to be called.
    pub hart_commands: Vec<HartCommand>,
    /// request_data_record_number indicates the pnio data record for request each
    /// channel for the AI module, see manual for specific AI module for more info.
    pub request_data_record_number: u16,
    /// response_data_record_number indicates the pnio data record for response each
    /// channel for the AI module, see manual for specific AI module for more info.
    pub response_data_record_number: u16,
    /// hart_device_name is the hart device model
    pub hart_device_name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct HartCommand {
    pub number: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Box<[u8]>>,
}
