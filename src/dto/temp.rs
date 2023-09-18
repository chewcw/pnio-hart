use anyhow::Result;
use serde::Serialize;

use crate::device::{spec_comm_status::FieldDeviceCommStatus, spec_status::FieldDeviceStatus};

#[derive(Serialize)]
pub struct Temp<'a> {
    pub timestamp: &'a str,
    pub device_unique_name: &'a str,
    pub hart_device_name: &'a str,
    pub hart_command: u8,
    pub length: u8,
    pub data: &'a str,
}

impl Temp<'_> {
    pub fn map_to_string(hart_command: u8, bytes: &[u8]) -> Result<String> {
        // hart response code (1st byte)
        // available for every commands
        let field_device_comm_status = FieldDeviceCommStatus::new();
        field_device_comm_status
            .map_to_comm_status(TryInto::<[u8; 2]>::try_into(bytes.get(0..2).unwrap()).unwrap());

        // hart field device status (2nd byte)
        // available for every commands
        let field_device_status = FieldDeviceStatus::new();
        field_device_status
            .map_to_device_status(TryInto::<[u8; 2]>::try_into(bytes.get(2..4).unwrap()).unwrap());

        // command 48 -----------------------------------------------------------------
        if hart_command == 48 {
            let mut command_48_response = Command48ResponseDto {
                response_code: &field_device_comm_status,
                field_device_status: &field_device_status,
                hw_fw_error: (bytes[2] & 0b0000_0001) == 1,
                diag_alarm: (bytes[2] & 0b0000_0010) == 2,
                diag_warn: (bytes[2] & 0b0000_0100) == 4,
                sim_mode: (bytes[2] & 0b0000_1000) == 8,
                sensor_break_0: (bytes[2] & 0b0001_0000) == 16,
                ram_failure: (bytes[3] & 0b0000_0001) == 1,
                rom_failure: (bytes[3] & 0b0000_0010) == 2,
                sim_pressure: (bytes[24] & 0b0000_0001) == 1,
                sim_sensor_temperature: (bytes[24] & 0b0000_0010) == 2,
                sim_el_temperature: (bytes[24] & 0b0000_0100) == 4,
                watchdog_failed: (bytes[5] & 0b0000_1000) == 8,
                watchdog_triggered: (bytes[5] & 0b0001_0000) == 16,
                service_alarm: (bytes[8] & 0b0000_0001) == 1,
            };

            return Ok(serde_json::to_string(&command_48_response).unwrap());
        }

        // command 9  -----------------------------------------------------------------
        if hart_command == 9 {
            let mut command9_response = Command9ResponseDto {
                response_code: &field_device_comm_status,
                field_device_status: &field_device_status,
                device_variable_code: bytes[3],
                device_variable_classification: match bytes[4] {
                    v if v == 0x40 => "temperature",
                    v if v == 0x41 => "pressure",
                    _ => "unknown",
                },
                unit: match bytes[5] {
                    v if v == 0x20 => "celcius",
                    v if v == 0x08 => "mbar",
                    _ => "unknown",
                },
                value: f32::from_be_bytes(
                    TryInto::<[u8; 4]>::try_into(bytes.get(6..10).unwrap()).unwrap(),
                ),
            };

            return Ok(serde_json::to_string(&command9_response).unwrap());
        }

        // command 0 ------------------------------------------------------------------
        if hart_command == 0 {
            let mut command0_response = Command0ResponseDto {
                response_code: &field_device_comm_status,
                field_device_status: &field_device_status,
                device_type: "SITRANS P DS",
                hart_major_revision_number: *bytes.get(6).unwrap(),
                device_revision_level: *bytes.get(7).unwrap(),
                software_revision_level: *bytes.get(8).unwrap(),
                configuration_change_counter: u16::from_be_bytes(
                    TryInto::<[u8; 2]>::try_into(bytes.get(16..18).unwrap()).unwrap(),
                ),
                maintenance_required: false,
                device_variable_alert: false,
                critical_power_failure: false,
                failure: false,
                out_of_specification: false,
                function_check: false,
            };

            return Ok(serde_json::to_string(&command0_response).unwrap());
        }

        // command 14 -----------------------------------------------------------------
        if hart_command == 14 {
            let mut command14_response = Command14ResponseDto {
                response_code: &field_device_comm_status,
                field_device_status: &field_device_status,
                transducer_upper_limit: f32::from_be_bytes(
                    TryInto::<[u8; 4]>::try_into(bytes.get(6..10).unwrap()).unwrap(),
                ),
                transducer_lower_limit: f32::from_be_bytes(
                    TryInto::<[u8; 4]>::try_into(bytes.get(10..14).unwrap()).unwrap(),
                ),
            };

            return Ok(serde_json::to_string(&command14_response).unwrap());
        }

        Ok(String::from(""))
    }
}

#[derive(Serialize)]
pub struct Command48ResponseDto<'a> {
    pub response_code: &'a FieldDeviceCommStatus,
    pub field_device_status: &'a FieldDeviceStatus,
    pub hw_fw_error: bool,
    pub diag_alarm: bool,
    pub diag_warn: bool,
    pub sim_mode: bool,
    pub sensor_break_0: bool,
    pub ram_failure: bool,
    pub rom_failure: bool,
    pub sim_pressure: bool,
    pub sim_sensor_temperature: bool,
    pub sim_el_temperature: bool,
    pub watchdog_failed: bool,
    pub watchdog_triggered: bool,
    pub service_alarm: bool,
}

#[derive(Serialize)]
pub struct Command0ResponseDto<'a> {
    pub response_code: &'a FieldDeviceCommStatus,
    pub field_device_status: &'a FieldDeviceStatus,
    pub device_type: &'a str,
    pub hart_major_revision_number: u8,
    pub device_revision_level: u8,
    pub software_revision_level: u8,
    pub configuration_change_counter: u16,

    /// these bits come from extended device status byte (common table 16)
    pub maintenance_required: bool,
    pub device_variable_alert: bool,
    pub critical_power_failure: bool,
    pub failure: bool,
    pub out_of_specification: bool,
    pub function_check: bool,
}

#[derive(Serialize)]
pub struct Command9ResponseDto<'a> {
    pub response_code: &'a FieldDeviceCommStatus,
    pub field_device_status: &'a FieldDeviceStatus,
    pub device_variable_code: u8,
    pub device_variable_classification: &'a str,
    pub unit: &'a str,
    pub value: f32,
}

#[derive(Serialize)]
pub struct Command14ResponseDto<'a> {
    pub response_code: &'a FieldDeviceCommStatus,
    pub field_device_status: &'a FieldDeviceStatus,
    pub transducer_upper_limit: f32,
    pub transducer_lower_limit: f32,
}
