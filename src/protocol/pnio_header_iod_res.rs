use super::{
    BlockHeaderType, PnioHeader, BLOCK_VERSION_HIGH, BLOCK_VERSION_LOW, IOD_PADDING, IOD_REQ_API,
    READ_MAX_COUNT,
};
use anyhow::anyhow;
use std::mem;
use uuid::Uuid;

#[derive(Debug, Copy, Clone)]
pub struct IodRes {
    pub block_header_type: [u8; 2],
    pub block_header_len: [u8; 2],
    pub block_header_version_high: [u8; 1],
    pub block_header_version_low: [u8; 1],
    pub seq_num: [u8; 2],
    pub ar_uuid: [u8; 16],
    pub api: [u8; 4],
    pub slot_num: [u8; 2],
    pub subslot_num: [u8; 2],
    pub padding1: [u8; 2],
    pub index: [u8; 2],
    pub record_data_len: [u8; 4],
    pub additional_value1: [u8; 2],
    pub additional_value2: [u8; 2],
    pub status: [u8; 4],
    pub padding2: [u8; 16],
}

impl IodRes {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        block_header_type: BlockHeaderType,
        seq_num: u16,
        ar_uuid: Uuid,
        slot_num: u16,
        subslot_num: u16,
        index: u16,
        status: [u8; 4],
        record_data_len: u32,
    ) -> Self {
        IodRes {
            block_header_type: (block_header_type as u16).to_be_bytes(),
            // TODO: what is this 4 bytes?
            block_header_len: (mem::size_of::<Self>() as u16 - 4).to_be_bytes(),
            block_header_version_high: BLOCK_VERSION_HIGH.to_be_bytes(),
            block_header_version_low: BLOCK_VERSION_LOW.to_be_bytes(),
            seq_num: seq_num.to_be_bytes(),
            ar_uuid: ar_uuid.as_bytes().to_owned(),
            api: IOD_REQ_API,
            slot_num: slot_num.to_be_bytes(),
            subslot_num: subslot_num.to_be_bytes(),
            padding1: [IOD_PADDING; 2],
            index: index.to_be_bytes(),
            record_data_len: record_data_len.to_be_bytes(),
            additional_value1: [0x00; 2],
            additional_value2: [0x00; 2],
            status,
            padding2: [IOD_PADDING; 16],
        }
    }
}

impl PnioHeader for IodRes {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        let mut v: Vec<u8> = vec![];

        v.extend(self.block_header_type);
        v.extend(self.block_header_len);
        v.extend(self.block_header_version_high);
        v.extend(self.block_header_version_low);
        v.extend(self.seq_num);
        v.extend(self.ar_uuid);
        v.extend(self.api);
        v.extend(self.slot_num);
        v.extend(self.subslot_num);
        v.extend(self.padding1);
        v.extend(self.index);
        v.extend(self.record_data_len);
        v.extend(self.additional_value1);
        v.extend(self.additional_value2);
        v.extend(self.status);
        v.extend(self.padding2);

        Ok(v)
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }

    fn get_max_count(&self) -> u32 {
        let block_header_type =
            BlockHeaderType::from_u16(u16::from_be_bytes(self.block_header_type));

        match block_header_type.unwrap() {
            BlockHeaderType::IodReadResType => READ_MAX_COUNT,
            BlockHeaderType::IodWriteResType => {
                // TODO: this is incorrect, should return corresponded
                // request's max count
                self.size() as u32 + u32::from_be_bytes(self.record_data_len)
            }
            _ => 0,
        }
    }

    fn get_actual_count(&self) -> u32 {
        let block_header_type =
            BlockHeaderType::from_u16(u16::from_be_bytes(self.block_header_type));

        match block_header_type.unwrap() {
            BlockHeaderType::IodReadResType => {
                self.size() as u32 + u32::from_be_bytes(self.record_data_len)
            }
            BlockHeaderType::IodWriteResType => self.size() as u32,
            _ => 0,
        }
    }

    fn get_args_length(&self) -> u32 {
        self.get_actual_count()
    }

    fn get_args_max(&self) -> Option<u32> {
        let block_header_type =
            BlockHeaderType::from_u16(u16::from_be_bytes(self.block_header_type));

        match block_header_type.unwrap() {
            BlockHeaderType::IodReadResType => None,
            BlockHeaderType::IodWriteResType => None,
            _ => None,
        }
    }
}

impl TryFrom<Vec<u8>> for IodRes {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let block_header_type_u16 =
            u16::from_be_bytes(*TryInto::<&[u8; 2]>::try_into(value.get(0..2).unwrap())?);

        let block_header_type = match BlockHeaderType::from_u16(block_header_type_u16) {
            Some(b) => b,
            None => return Err(anyhow!("failed to match block_header_type")),
        };

        let seq_num_u16 = u16::from_be_bytes(value[6..8].try_into()?);
        let ar_uuid = Uuid::from_bytes(value[8..24].try_into()?);
        let slot_num_u16 = u16::from_be_bytes(value[28..30].try_into()?);
        let subslot_num_u16 = u16::from_be_bytes(value[30..32].try_into()?);
        let index = match value.get(34..36) {
            Some(i) => u16::from_be_bytes(TryInto::<[u8; 2]>::try_into(i)?),
            None => return Err(anyhow!("failed to match IOD header index")),
        };

        let record_data_len_u16 = u32::from_be_bytes(value[36..40].try_into()?);

        let mut status = [0x00; 4];
        if block_header_type == BlockHeaderType::IodWriteResType {
            status = match value.get(45..49) {
                Some(s) => TryInto::<[u8; 4]>::try_into(s)?,
                None => return Err(anyhow!("failed to match status for IODWriteRes")),
            };
        };

        let iod_request = IodRes::new(
            block_header_type,
            seq_num_u16,
            ar_uuid,
            slot_num_u16,
            subslot_num_u16,
            index,
            status,
            record_data_len_u16,
        );

        Ok(iod_request)
    }
}
