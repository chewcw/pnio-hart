use super::{
    constant::{self, BlockHeaderType},
    PnioHeader,
};
use std::mem;
use uuid::Uuid;

#[derive(Debug)]
pub struct ArBlockReq {
    pub block_header_type: [u8; 2],
    pub block_header_len: [u8; 2],
    pub block_header_version_high: [u8; 1],
    pub block_header_version_low: [u8; 1],
    pub ar_type: [u8; 2],
    pub ar_uuid: [u8; 16],
    pub session_key: [u8; 2],
    pub cm_initiator_mac: [u8; 6],
    pub cm_initiator_obj_uuid: [u8; 16],
    pub ar_props: [u8; 4],
    pub cm_initiator_act_timeout_factor: [u8; 2],
    pub cm_initiator_udprt_port: [u8; 2],
    pub station_name_len: [u8; 2],
    pub cm_initiator_station_name: [u8; 3],
}

impl ArBlockReq {
    pub fn new(ar_uuid: Uuid, session_key: u16, cm_initiator_obj_uuid: Uuid) -> Self {
        // TODO: what is this 4 bytes about?
        let block_header_len = (mem::size_of::<Self>() as u16 - 4).to_be_bytes();
        // TODO: how to get this value: "TBL"?
        let cm_initiator_station_name = TryInto::<[u8; 3]>::try_into("TBL".as_bytes()).unwrap();

        ArBlockReq {
            block_header_type: (BlockHeaderType::ArBlockReqType as u16).to_be_bytes(),
            block_header_len,
            block_header_version_high: constant::BLOCK_VERSION_HIGH.to_be_bytes(),
            block_header_version_low: constant::BLOCK_VERSION_LOW.to_be_bytes(),
            ar_type: constant::AR_TYPE.to_be_bytes(),
            ar_uuid: *ar_uuid.as_bytes(),
            session_key: session_key.to_be_bytes(),
            cm_initiator_mac: constant::CM_INITIATOR_MAC,
            cm_initiator_obj_uuid: *cm_initiator_obj_uuid.as_bytes(),
            ar_props: constant::AR_PROPS,
            cm_initiator_act_timeout_factor: constant::CM_INITIATOR_ACT_TIMEOUT_FACTOR
                .to_be_bytes(),
            cm_initiator_udprt_port: constant::CM_INITIATOR_UDPRT_PORT.to_be_bytes(),
            station_name_len: (cm_initiator_station_name.len() as u16).to_be_bytes(),
            cm_initiator_station_name,
        }
    }
}

impl PnioHeader for ArBlockReq {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        let mut v: Vec<u8> = vec![];

        v.extend(self.block_header_type);
        v.extend(self.block_header_len);
        v.extend(self.block_header_version_high);
        v.extend(self.block_header_version_low);
        v.extend(self.ar_type);
        v.extend(self.ar_uuid);
        v.extend(self.session_key);
        v.extend(self.cm_initiator_mac);
        v.extend(self.cm_initiator_obj_uuid);
        v.extend(self.ar_props);
        v.extend(self.cm_initiator_act_timeout_factor);
        v.extend(self.cm_initiator_udprt_port);
        v.extend(self.station_name_len);
        v.extend(self.cm_initiator_station_name);

        Ok(v)
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn get_max_count(&self) -> u32 {
        self.size() as u32
    }

    fn get_actual_count(&self) -> u32 {
        self.size() as u32
    }

    fn get_args_length(&self) -> u32 {
        self.size() as u32
    }

    fn get_args_max(&self) -> Option<u32> {
        Some(self.get_actual_count())
    }
}
