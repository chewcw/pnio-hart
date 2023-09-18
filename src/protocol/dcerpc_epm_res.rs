use super::constant;
use anyhow::anyhow;
use std::net::Ipv4Addr;
use std::{mem, str};
use uuid::Uuid;

// DCE/RPC Endpoint Mapper response -------------------------------------------

#[derive(Debug)]
pub struct DceRpcEpmResponse<'a> {
    pub handle: String,
    _num_of_entries: u32,
    _actual_count: u32,
    pub entry: Box<Entry<'a>>,
}

impl<'a> TryFrom<&'a [u8]> for DceRpcEpmResponse<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // handle occupies 20 bytes
        let handle_arr = match value.get(0..20) {
            Some(h) => TryInto::<&[u8; 20]>::try_into(h).unwrap(),
            None => return Err(anyhow!("handle cannot be created".to_string())),
        };

        let handle = handle_arr
            .iter()
            .map(|h| format!("{:02x?}", h))
            .collect::<Vec<String>>()
            .join("");

        // num_of_entries occupies 4 bytes
        let num_of_entries = match value.get(20..24) {
            Some(n) => TryInto::<&[u8; 4]>::try_into(n).unwrap(),
            None => return Err(anyhow!("number of entries cannot be created".to_string())),
        };

        // max_count occupies 4 bytes, not used, put here for clarity
        if let Some(_max_count) = value.get(24..28) {};

        // offset occupies 4 bytes, not used, put here for clarity
        if let Some(_offset) = value.get(28..32) {};

        // actual_count occupies 4 bytes
        let actual_count = match value.get(32..36) {
            Some(a) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(a).unwrap()),
            None => return Err(anyhow!("actual count cannot be created".to_string())),
        };

        // TODO: the entry byte order is not confirmed.
        // I do not know how the bytes would look like if there were multiple entries.
        // so for now, only assume there is only one entry in box,
        // thus considering the remaining bytes belong that specific entry.
        let entry_arr = match value.get(36..) {
            Some(e) => TryInto::<&[u8]>::try_into(e).unwrap(),
            None => return Err(anyhow!("entries cannot be created".to_string())),
        };

        let entry = match Entry::try_from(entry_arr) {
            Ok(entry) => entry,
            Err(e) => return Err(e),
        };

        let dcerpc_epm_response = DceRpcEpmResponse {
            handle,
            _num_of_entries: u32::from_le_bytes(*num_of_entries),
            _actual_count: actual_count,
            entry: Box::new(entry),
        };

        Ok(dcerpc_epm_response)
    }
}

// DCE/RPC Endpoint Mapper response entries -----------------------------------

#[derive(Debug, Clone)]
pub struct Entry<'a> {
    pub object: Uuid,
    pub tower_pointer: TowerPointer<'a>,
}

impl<'a> TryFrom<&'a [u8]> for Entry<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        let object = match value.get(0..16) {
            Some(o) => TryInto::<&[u8; 16]>::try_into(o).unwrap(),
            None => return Err(anyhow!("this entry is empty".to_string())),
        };

        // TODO: unsure how the byte order looks like if there
        // were multiple entries, so for now, assume only one
        // entry and treat the rest of bytes as tower_pointer.
        let tower_pointer_arr = match value.get(16..) {
            Some(t) => t,
            None => todo!(),
        };

        let tower_pointer = match TowerPointer::try_from(tower_pointer_arr) {
            Ok(t) => t,
            Err(e) => return Err(e),
        };

        let entry = Entry {
            object: Uuid::from_bytes_le(*object),
            tower_pointer,
        };

        Ok(entry)
    }
}

// DCE/RPC Endpoint Mapper response entry tower pointer -----------------------

#[derive(Debug, Clone)]
pub struct TowerPointer<'a> {
    _annotation_offset: u32,
    _annotation_length: u32,
    pub annotation: &'a str,
    _length1: u32,
    _length2: u32,
    pub num_of_floors: u16,
    pub floors: Vec<TowerFloor>,
}

impl<'a> TryFrom<&'a [u8]> for TowerPointer<'a> {
    type Error = anyhow::Error;

    fn try_from(value: &'a [u8]) -> Result<Self, Self::Error> {
        // annotation_offset occupies 4 bytes
        let annotation_offset = match value.get(4..8) {
            // TODO: not sure if this is little endian
            Some(a) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(a).unwrap()),
            None => {
                return Err(anyhow!(
                    "tower pointer annotation offset cannot be created".to_string()
                ))
            }
        };

        // annotation_length occupies 4 bytes
        let annotation_length = match value.get(8..12) {
            Some(a) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(a).unwrap()),
            None => {
                return Err(anyhow!(
                    "tower pointer annotation length cannot be created".to_string()
                ))
            }
        };

        // annotation dynamically occupies <annotation_length> bytes
        let annotation_start = 12;
        let annotation_end = annotation_start + (annotation_length as usize);
        let annotation = match value.get(annotation_start..annotation_end) {
            Some(a) => str::from_utf8(a).unwrap(),
            None => {
                return Err(anyhow!(
                    "tower pointer annotation cannot be created".to_string()
                ))
            }
        };

        // length1 occupies 4 bytes
        let length1_end = annotation_end + 4;
        let length1 = match value.get(annotation_end..length1_end) {
            Some(l) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(l).unwrap()),
            None => return Err(anyhow!("tower point length1 cannot be created".to_string())),
        };

        // length2 occupies 4 bytes
        let length2_end = length1_end + 4;
        let length2 = match value.get(length1_end..length2_end) {
            Some(l) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(l).unwrap()),
            None => return Err(anyhow!("tower point length2 cannot be created".to_string())),
        };

        // num of floors occupies 2 bytes
        let num_of_floors_end = length2_end + 2;
        let num_of_floors = match value.get(length2_end..num_of_floors_end) {
            Some(n) => u16::from_le_bytes(*TryInto::<&[u8; 2]>::try_into(n).unwrap()),
            None => return Err(anyhow!("num of floors cannot be created".to_string())),
        };

        // construct floors
        let mut floors_start = num_of_floors_end;
        let mut floors: Vec<TowerFloor> = vec![];
        for i in 0..num_of_floors {
            // uuid type floor (in the order of bytes)
            // lhs_length (2 bytes)
            // protocol (1 byte) <-- start of data with lhs_length
            // uuid
            // version <-- end of data with lhs_length
            // rhs_length (2 bytes)
            // version minor <-- data with rhs_length

            // other type (in the order of bytes)
            // lhs_length (2 bytes)
            // protocol (1 byte) <-- data with lhs_length
            // rhs_length (2 bytes)
            // data <-- data with rhs_length

            // lhs_length occupies 2 bytes
            let lhs_length = match value.get(floors_start..floors_start + 2) {
                Some(l) => u16::from_le_bytes(*TryInto::<&[u8; 2]>::try_into(l).unwrap()),
                None => todo!(),
            };

            // protocol occupies 1 byte
            let protocol_start = floors_start + mem::size_of_val(&lhs_length);
            let protocol_end = protocol_start + 1;
            let protocol_u8 = match value.get(protocol_start..protocol_end) {
                Some(p) => u8::from_le_bytes(*TryInto::<&[u8; 1]>::try_into(p).unwrap()),
                None => return Err(anyhow!("tower floor protocol cannot be created".to_string())),
            };
            let protocol = match constant::TowerFloorProtocol::from_u8(protocol_u8) {
                Some(p) => p,
                None => {
                    let err = format!(
                        "tower floor \"{}\" protocol \"{}\" not found",
                        i + 1,
                        protocol_u8
                    );
                    dbg!("{}", err);
                    continue;
                }
            };

            // rhs_length
            // lhs_length occupies 2 bytes
            let rhs_start = floors_start + mem::size_of_val(&lhs_length) + (lhs_length as usize);
            // rhs_length occupies 2 bytes
            let rhs_end = rhs_start + 2;
            let rhs_length = match value.get(rhs_start..rhs_end) {
                Some(r) => u16::from_le_bytes(*TryInto::<&[u8; 2]>::try_into(r).unwrap()),
                None => {
                    let err = format!("tower floor \"{}\" rhs length cannot be created", i + 1);
                    dbg!("{}", err);
                    continue;
                }
            };

            // payload (some data come after lhs_length, some after rhs_length)
            let mut uuid: Option<Uuid> = None;
            let mut udp_port: Option<u16> = None;
            let mut ipv4: Option<Ipv4Addr> = None;
            match protocol {
                constant::TowerFloorProtocol::Uuid => {
                    let uuid_start = protocol_end;
                    let uuid_end = protocol_end + 16; // uuid always occupies 16 bytes
                    let uuid_arr_u8 = value.get(uuid_start..uuid_end).unwrap();
                    uuid = Some(Uuid::from_bytes_le(
                        *TryInto::<&[u8; 16]>::try_into(uuid_arr_u8).unwrap(),
                    ));
                }
                constant::TowerFloorProtocol::Udp => {
                    let udp_start = rhs_end;
                    let udp_end = udp_start + 2; // udp port always occupies 2 bytes
                    let udp_arr_u8 = value.get(udp_start..udp_end).unwrap();
                    udp_port = Some(u16::from_be_bytes(
                        *TryInto::<&[u8; 2]>::try_into(udp_arr_u8).unwrap(),
                    ));
                }
                constant::TowerFloorProtocol::Ip => {
                    let ip_start = rhs_end;
                    let ip_end = ip_start + 4; // ipv4 always occupies 4 bytes
                    let ipv4_arr_u8 = value.get(ip_start..ip_end).unwrap();
                    let ipv4_arr_u8_4bytes = TryInto::<&[u8; 4]>::try_into(ipv4_arr_u8).unwrap();
                    ipv4 = Some(Ipv4Addr::from(*ipv4_arr_u8_4bytes));
                }
                _ => (),
            };

            floors.push(TowerFloor {
                _lhs_length: lhs_length,
                _rhs_length: rhs_length,
                protocol,
                uuid,
                udp_port,
                ipv4,
            });

            // iterate next floor
            // a floor always occupies lhs_length + rhs_length + 2 + 2
            // where lhs_length and rhs_length occupies 2 bytes respectively
            floors_start = floors_start + (lhs_length as usize) + (rhs_length as usize) + 2 + 2;
        }

        let tower_pointer = TowerPointer {
            _annotation_offset: annotation_offset,
            _annotation_length: annotation_length,
            annotation,
            _length1: length1,
            _length2: length2,
            num_of_floors,
            floors,
        };

        Ok(tower_pointer)
    }
}

// DCE/RPC Endpoint Mapper response entry tower floor -------------------------

#[derive(Debug, Clone)]
pub struct TowerFloor {
    _lhs_length: u16,
    _rhs_length: u16,
    pub protocol: constant::TowerFloorProtocol,
    pub uuid: Option<Uuid>,
    pub udp_port: Option<u16>,
    pub ipv4: Option<Ipv4Addr>,
}

// test -----------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::*;

    fn get_et200sp_response_packet() -> Vec<u8> {
        let hex_string = "00000000290000000000\
                          00108000ec1c5d4d5497\
                          01000000010000000000\
                          0000010000000000a0de\
                          976cd111827100010313\
                          002a0300000000000000\
                          40000000455432303053\
                          50202020202020202020\
                          20202020202020202020\
                          36455337203135352d36\
                          415530312d30424e3020\
                          20202020203420562020\
                          34202032202030004b00\
                          00004b00000005001300\
                          0d0100a0de976cd11182\
                          7100a02442df7d010002\
                          00000013000d045d888a\
                          eb1cc9119fe808002b10\
                          48600200020000000100\
                          0a020000000100080200\
                          c0040100090400000000\
                          007000000000";
        hex::decode(hex_string).unwrap()
    }

    fn get_empty_response_packet() -> Vec<u8> {
        let hex_string = "00000000000000000000\
                          00000000000000000000\
                          00000000010000000000\
                          000000000000d6a0c916";

        hex::decode(hex_string).unwrap()
    }

    #[test]
    // correct packet should map to all respective structs
    fn try_from_tower_pointer_should_return_correctly() {
        let full_packet = &get_et200sp_response_packet()[..];

        let tower_pointer_packet = &get_et200sp_response_packet()[52..];
        let tower_pointer = TowerPointer::try_from(tower_pointer_packet).unwrap();

        assert_eq!(5, tower_pointer.floors.len());
        assert_eq!(
            Uuid::parse_str("dea00001-6c97-11d1-8271-00a02442df7d").unwrap(),
            tower_pointer.floors[0].uuid.unwrap()
        );

        let dcerpc_epm_response = DceRpcEpmResponse::try_from(full_packet).unwrap();
        assert_eq!(
            "0000000029000000000000108000ec1c5d4d5497",
            dcerpc_epm_response.handle
        );

        assert_eq!(
            Uuid::parse_str("dea00000-6c97-11d1-8271-00010313002a").unwrap(),
            dcerpc_epm_response.entry.object
        );

        // TODO: more tests
    }

    #[test]
    // if there is error, just return and ignore this packet
    fn test_try_from_dcerpc_epm_response_should_return_error() {
        let full_packet = &get_empty_response_packet()[..];

        assert!(DceRpcEpmResponse::try_from(full_packet).is_err());
    }
}
