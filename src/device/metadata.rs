use std::cell::Cell;

#[derive(Debug)]
pub struct Metadata {
    pub hart_protocol_major_revision: Cell<u8>,
    pub device_revision_level: Cell<u8>,
    pub software_revision_level: Cell<u8>,
    pub number_of_preemble_bytes_in_response: Cell<u8>,
    pub configuration_change_counter: Cell<u16>,
}

impl Metadata {
    pub fn map_to_metadata(&self, hart_response: &[u8]) -> anyhow::Result<()> {
        if let Some(h) = hart_response.get(3) {
            self.hart_protocol_major_revision.set(*h);
        }

        if let Some(h) = hart_response.get(5) {
            self.device_revision_level.set(*h);
        }

        if let Some(h) = hart_response.get(6) {
            self.software_revision_level.set(*h);
        }

        if let Some(h) = hart_response.get(12) {
            self.number_of_preemble_bytes_in_response.set(*h);
        }

        if let Some(h) = hart_response.get(14..16) {
            self.configuration_change_counter
                .set(u16::from_be_bytes(TryInto::<[u8; 2]>::try_into(h).unwrap()));
        }

        Ok(())
    }
}
