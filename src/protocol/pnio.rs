use super::{
    BlockHeaderType, Packet, PnioHeader, ArBlockRes,
    PnioHeaderEnum, IodRes,
};
use anyhow::anyhow;
use std::mem;

#[derive(Debug)]
pub struct Pnio {
    pub args_max: Option<u32>,   // only available in request
    pub status: Option<[u8; 4]>, // only available in response
    pub args_length: [u8; 4],
    pub max_count: [u8; 4],
    pub offset: [u8; 4],
    pub actual_count: [u8; 4],
    // this could be IODHeader for IO request or ARBlockRequest for connection request
    pub pnio_header: PnioHeaderEnum,
    // only available in request, user specified data containing HART command
    pub pnio_data: Option<Box<[u8]>>,
}

impl Pnio {
    pub fn new(
        status: Option<[u8; 4]>,
        pnio_header: PnioHeaderEnum,
        pnio_data: Option<Box<[u8]>>,
    ) -> Self {
        let max_count_u32 = pnio_header.get_max_count();
        let max_count = TryInto::<[u8; 4]>::try_into(max_count_u32.to_le_bytes()).unwrap();
        let args_length_u32 = pnio_header.get_args_length();
        let args_length = TryInto::<[u8; 4]>::try_into(args_length_u32.to_le_bytes()).unwrap();
        let actual_count_u32 = pnio_header.get_actual_count();
        let actual_count = TryInto::<[u8; 4]>::try_into(actual_count_u32.to_le_bytes()).unwrap();
        let args_max = pnio_header.get_args_max();

        Pnio {
            args_max,
            status,
            args_length,
            max_count,
            offset: [0x00; 4],
            actual_count,
            pnio_header,
            pnio_data,
        }
    }
}

impl Packet for Pnio {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        let mut v: Vec<u8> = vec![];

        v.extend(&self.args_max.unwrap().to_le_bytes());
        v.extend(&self.args_length);
        v.extend(&self.max_count);
        v.extend(&self.offset);
        v.extend(&self.actual_count);
        v.extend(&self.pnio_header.concat().unwrap());

        let data = match &self.pnio_data {
            Some(d) => &d[..],
            None => &[],
        };
        v.extend(data);

        Ok(v)
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl TryFrom<Vec<u8>> for Pnio {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        // actual_count includes size for pnio_header and data
        let actual_count = match value.get(16..20) {
            Some(a) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(a).unwrap()),
            None => return Err(anyhow!("failed to match pnio actual count")),
        };

        // header type
        let block_header_type = match value.get(20..22) {
            Some(b) => BlockHeaderType::from_u16(u16::from_be_bytes(<[u8; 2]>::try_from(b)?)),
            None => return Err(anyhow!("failed to match pnio block header type")),
        };

        // invalid header type
        if block_header_type.is_none() {
            return Err(anyhow!("pnio block header type is invalid"));
        }

        // response type has this status type
        // request type has args_max
        let mut status: Option<[u8; 4]> = None;
        if block_header_type.unwrap() == BlockHeaderType::IodReadResType
            || block_header_type.unwrap() == BlockHeaderType::IodWriteResType
        {
            status = match value.get(0..4) {
                Some(s) => Some(TryInto::<[u8; 4]>::try_into(s)?),
                None => return Err(anyhow!("failed to match pnio status")),
            };
        };

        // block length + 4 = pnio_header size
        let block_length = match value.get(22..24) {
            Some(b) => u16::from_be_bytes(TryInto::<[u8; 2]>::try_into(b)?),
            None => return Err(anyhow!("failed to match pnio block length")),
        };

        // pnio header, could be IODHeader or ARBlock
        // TODO: what is this 4 bytes?
        let pnio_header_end = 20 + block_length as usize + 4;
        let pnio_header_bytes = match value.get(20..pnio_header_end) {
            Some(bytes) => bytes,
            None => return Err(anyhow!("failed to match pnio header")),
        };

        // only deserialize pnio_header response type
        let pnio_header: PnioHeaderEnum = match block_header_type.unwrap() {
            BlockHeaderType::ArBlockResType => {
                let pnio_header_arblock_resp =
                    TryInto::<ArBlockRes>::try_into(pnio_header_bytes)?;
                PnioHeaderEnum::ArBlockRes(pnio_header_arblock_resp)
            }
            BlockHeaderType::IodReadResType | BlockHeaderType::IodWriteResType => {
                let pnio_header_iod_resp =
                    TryInto::<IodRes>::try_into(pnio_header_bytes.to_vec())?;
                PnioHeaderEnum::IodRes(pnio_header_iod_resp)
            }
            _ => panic!(
                "deserialization of pnio_header not working for header type other than response"
            ),
        };

        // pnio data, only available in read response type and write request type
        let mut pnio_data: Option<Box<[u8]>> = None;
        if block_header_type.unwrap() == BlockHeaderType::IodReadResType
            || block_header_type.unwrap() == BlockHeaderType::IodWriteReqType
        {
            // TODO: what is this 4 bytes?
            // this is supposed to be the size of the pnio_data
            let pnio_data_length = actual_count as usize - (block_length as usize + 4);
            // let pnio_data_end = pnio_header_end + pnio_data_length;
            // this is the real size of the pnio_data
            // let real_data_length = value.len() - pnio_header_end;
            // if the real size is fewer than target size
            // then append 0x00 to the end
            pnio_data = match value.get(pnio_header_end..) {
                Some(p) => {
                    let mut new_p = p.to_vec();
                    // TODO: why size of the data coming back is not pnio_data_length?
                    new_p.resize(pnio_data_length, 0x00);
                    Some(new_p.into_boxed_slice())
                }
                None => return Err(anyhow!("failed to match pnio_data")),
            };
        }

        let pnio = Pnio::new(status, pnio_header, pnio_data);

        Ok(pnio)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn try_from_pnio_connect_response() {
        let bytes = hex::decode(
            "00000000220000003d00\
             000000000000220000008\
             101001e01000006f4162d\
             be951d4041b5839b57a3b\
             ed95e0001ec1c5d4d5497\
             8892",
        )
        .unwrap();

        let pnio_result = TryInto::<Pnio>::try_into(bytes);
        assert_eq!(pnio_result.is_ok(), true);

        let pnio = pnio_result.unwrap();
        if let PnioHeaderEnum::ArBlockRes(pnio_header) = pnio.pnio_header {
            assert_eq!(
                pnio_header.cm_responder_mac_address,
                [0xec, 0x1c, 0x5d, 0x4d, 0x54, 0x97]
            );
        }
    }

    #[test]
    fn try_from_pnio_read_response() {
        let bytes = hex::decode(
            "0000000030010000300001000000000030010000\
             8009003c01000023f4162dbe951d4041b5839b57\
             a3bed95e000000000001000100000051000000f0\
             0000000000000000000000000000000000000000\
             000000000400068000130000fe2a0b0505030638\
             003fcc78050c0269009e2082082002066cfa00c5\
             ba000000000000009a0000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             0000000000000000000000000000000000000000\
             00000000",
        )
        .unwrap();

        let pnio_result = TryInto::<Pnio>::try_into(bytes);
        assert_eq!(pnio_result.is_ok(), true);

        let pnio = pnio_result.unwrap();
        if let PnioHeaderEnum::IodRes(pnio_header) = pnio.pnio_header {
            assert_eq!(
                Uuid::from_slice(&pnio_header.ar_uuid).unwrap(),
                Uuid::parse_str("f4162dbe951d4041b5839b57a3bed95e").unwrap(),
            );
        }
    }

    #[test]
    fn try_from_pnio_write_response() {
        let bytes = hex::decode(
            "00000000400000004b00\
            00000000000040000000\
            8008003c01000024f416\
            2dbe951d4041b5839b57\
            a3bed95e000000000001\
            00010000005000000000\
            00000000000000000000\
            00000000000000000000\
            00000000",
        )
        .unwrap();

        let pnio_result = TryInto::<Pnio>::try_into(bytes);
        assert_eq!(pnio_result.is_ok(), true);

        let pnio = pnio_result.unwrap();
        if let PnioHeaderEnum::IodRes(pnio_header) = pnio.pnio_header {
            assert_eq!(pnio_header.status, [0x00; 4]);
            assert_eq!(u16::from_be_bytes(pnio_header.block_header_len), 60);
            assert_eq!(
                Uuid::from_slice(&pnio_header.ar_uuid).unwrap(),
                Uuid::parse_str("f4162dbe-951d-4041-b583-9b57a3bed95e").unwrap()
            )
        };
    }
}
