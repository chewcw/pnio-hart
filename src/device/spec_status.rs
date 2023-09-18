use anyhow::anyhow;
use serde::Serialize;
use std::cell::Cell;

// this is field device statuses
// 2nd bytes of statuses
// which should be common for all vendors
// see https://library.fieldcommgroup.org/20183/TS20183/27.0/#page=138 Table A-1
#[derive(Debug, Serialize)]
pub struct FieldDeviceStatus {
    pub primary_variable_out_of_limits: Cell<bool>,
    pub non_primary_variable_out_of_limits: Cell<bool>,
    pub loop_current_saturated: Cell<bool>,
    pub loop_current_fixed: Cell<bool>,
    pub more_status_available: Cell<bool>,
    pub cold_start: Cell<bool>,
    pub configuration_changed: Cell<bool>,
    pub device_malfunction: Cell<bool>,
}

impl FieldDeviceStatus {
    pub fn new() -> Self {
        Self {
            primary_variable_out_of_limits: false.into(),
            non_primary_variable_out_of_limits: false.into(),
            loop_current_saturated: false.into(),
            loop_current_fixed: false.into(),
            more_status_available: false.into(),
            cold_start: false.into(),
            configuration_changed: false.into(),
            device_malfunction: false.into(),
        }
    }

    pub fn map_to_device_status(&self, hart_statuses: [u8; 2]) -> anyhow::Result<()> {
        let device_statuses = match hart_statuses.get(1) {
            Some(d) => d,
            None => return Err(anyhow!("failed to get device statuses")),
        };

        self.primary_variable_out_of_limits
            .set((device_statuses & 0x01) == 0x01);
        self.non_primary_variable_out_of_limits
            .set((device_statuses & 0x02) == 0x02);
        self.loop_current_saturated
            .set((device_statuses & 0x04) == 0x04);
        self.loop_current_fixed
            .set((device_statuses & 0x08) == 0x08);
        self.more_status_available
            .set((device_statuses & 0x10) == 0x10);
        self.cold_start.set((device_statuses & 0x20) == 0x20);
        self.configuration_changed
            .set((device_statuses & 0x40) == 0x40);
        self.device_malfunction
            .set((device_statuses & 0x80) == 0x80);

        Ok(())
    }
}
