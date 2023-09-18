use anyhow::anyhow;
use serde::Serialize;
use std::cell::Cell;

// this is field device communication status
// also called response code,
// 1st byte of statuses
// which should be common for all vendors
// see https://library.fieldcommgroup.org/20183/TS20183/27.0/#page=138 Table A-2
#[derive(Debug, Serialize)]
pub struct FieldDeviceCommStatus {
    pub buffer_overflow: Cell<bool>,
    pub communication_failure: Cell<bool>,
    pub longitudinal_parity_error: Cell<bool>,
    pub framing_error: Cell<bool>,
    pub overrun_error: Cell<bool>,
    pub vertical_parity_error: Cell<bool>,
    pub communication_error: Cell<bool>,
}

impl FieldDeviceCommStatus {
    pub fn new() -> Self {
        Self {
            buffer_overflow: false.into(),
            communication_failure: false.into(),
            longitudinal_parity_error: false.into(),
            framing_error: false.into(),
            overrun_error: false.into(),
            vertical_parity_error: false.into(),
            communication_error: false.into(),
        }
    }

    pub fn map_to_comm_status(&self, hart_statuses: [u8; 2]) -> anyhow::Result<()> {
        let comm_statuses = match hart_statuses.first() {
            Some(d) => d,
            None => return Err(anyhow!("failed to get comm statuses")),
        };

        self.buffer_overflow.set((comm_statuses & 0x02) == 0x02);
        self.communication_failure
            .set((comm_statuses & 0x04) == 0x04);
        self.longitudinal_parity_error
            .set((comm_statuses & 0x08) == 0x08);
        self.framing_error.set((comm_statuses & 0x10) == 0x10);
        self.overrun_error.set((comm_statuses & 0x20) == 0x20);
        self.vertical_parity_error
            .set((comm_statuses & 0x40) == 0x40);
        self.communication_error.set((comm_statuses & 0x80) == 0x80);

        Ok(())
    }
}
