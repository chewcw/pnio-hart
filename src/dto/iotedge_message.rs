use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct IotedgeMessageDto<'a> {
    pub timestamp: &'a str,
    pub device_unique_name: &'a str,
    pub hart_device_name: &'a str,
    pub hart_command: u8,
    pub length: u8,
    pub bytes: &'a [u8],
}
