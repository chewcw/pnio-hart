use super::{
    constant::BlockHeaderType, PnioHeader, AR_TYPE, BLOCK_VERSION_HIGH, BLOCK_VERSION_LOW,
};
use anyhow::anyhow;
use std::mem;
use uuid::Uuid;

#[derive(Debug)]
pub struct ArBlockRes {
    pub block_header_type: [u8; 2],
    pub block_header_len: [u8; 2],
    pub block_header_version_high: [u8; 1],
    pub block_header_version_low: [u8; 1],
    pub ar_type: [u8; 2],
    pub ar_uuid: [u8; 16],
    pub session_key: [u8; 2],
    pub cm_responder_mac_address: [u8; 6],
    pub cm_responder_udpport: [u8; 2],
}

impl ArBlockRes {
    pub fn new(
        block_header_type: BlockHeaderType,
        ar_uuid: Uuid,
        session_key: u16,
        cm_responder_mac_address: [u8; 6],
        cm_responder_udpport: [u8; 2],
    ) -> Self {
        Self {
            block_header_type: (block_header_type as u16).to_be_bytes(),
            // TODO: what is this 4 bytes?
            block_header_len: (mem::size_of::<Self>() as u16 - 4).to_be_bytes(),
            block_header_version_high: BLOCK_VERSION_HIGH.to_be_bytes(),
            block_header_version_low: BLOCK_VERSION_LOW.to_be_bytes(),
            ar_type: AR_TYPE.to_be_bytes(),
            ar_uuid: ar_uuid.as_bytes().to_owned(),
            session_key: u16::to_be_bytes(session_key),
            cm_responder_mac_address,
            cm_responder_udpport,
        }
    }
}

impl PnioHeader for ArBlockRes {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        let mut v: Vec<u8> = vec![];

        v.extend(self.block_header_type);
        v.extend(self.block_header_len);
        v.extend(self.block_header_version_high);
        v.extend(self.block_header_version_low);
        v.extend(self.ar_type);
        v.extend(self.ar_uuid);
        v.extend(self.session_key);
        v.extend(self.cm_responder_mac_address);
        v.extend(self.cm_responder_udpport);

        Ok(v)
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn get_max_count(&self) -> u32 {
        // TODO: this is incorrect, should return corresponded
        // request's max count
        self.size() as u32
    }

    fn get_actual_count(&self) -> u32 {
        self.size() as u32
    }

    fn get_args_length(&self) -> u32 {
        self.size() as u32
    }

    fn get_args_max(&self) -> Option<u32> {
        None
    }
}

impl TryFrom<&[u8]> for ArBlockRes {
    type Error = anyhow::Error;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let block_header_type_u16 =
            u16::from_be_bytes(*TryInto::<&[u8; 2]>::try_into(value.get(0..2).unwrap()).unwrap());
        let block_header_type = match BlockHeaderType::from_u16(block_header_type_u16) {
            Some(b) => b,
            None => return Err(anyhow!("failed to match block_header_type")),
        };

        let _: [u8; 2] = match value.get(6..8) {
            Some(a) => a.try_into()?,
            None => return Err(anyhow!("failed to match ar_type")),
        };

        let ar_uuid = match value.get(8..24) {
            Some(a) => Uuid::from_slice(a)?,
            None => return Err(anyhow!("failed to match ARUUID")),
        };

        let session_key: u16 = match value.get(24..26) {
            Some(s) => u16::from_be_bytes(TryInto::<[u8; 2]>::try_into(s)?),
            None => return Err(anyhow!("failed to match session_key")),
        };

        let cm_responder_mac_address: [u8; 6] = match value.get(26..32) {
            Some(c) => c.try_into()?,
            None => return Err(anyhow!("failed to match cm_responder_mac_address")),
        };

        let cm_responder_udpport = match value.get(32..34) {
            Some(c) => c.try_into()?,
            None => return Err(anyhow!("failed to match cm_responder_udpport")),
        };

        let arblock_request = Self::new(
            block_header_type,
            ar_uuid,
            session_key,
            cm_responder_mac_address,
            cm_responder_udpport,
        );

        Ok(arblock_request)
    }
}
