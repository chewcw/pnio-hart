use super::{constant, Packet};
use anyhow::anyhow;
use std::mem;
use uuid::Uuid;

#[derive(Debug)]
pub struct DceRpcPacket {
    pub version: [u8; 1],
    pub packet_type: [u8; 1],
    pub flags1: [u8; 1],
    pub flags2: [u8; 1],
    pub data_representation: [u8; 3],
    pub serial_high: [u8; 1],
    pub obj_uuid: [u8; 16],
    pub interface: [u8; 16],
    pub activity: [u8; 16],
    pub server_boot_time: [u8; 4],
    pub interface_ver: [u8; 4],
    pub seq_num: [u8; 4],
    pub opnum: [u8; 2],
    pub interface_hint: [u8; 2],
    pub activity_hint: [u8; 2],
    pub fragment_len: [u8; 2],
    pub fragment_num: [u8; 2],
    pub auth_proto: [u8; 1],
    pub serial_low: [u8; 1],
    pub data: Box<[u8]>, // this is actually the PNIO packet
}

impl DceRpcPacket {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        packet_type: constant::PacketType,
        obj_uuid: Uuid,
        interface: Uuid,
        interface_ver: constant::InterfaceVersion,
        activity: Uuid,
        seq_num: u32,
        opnum: constant::OpNum,
        // this could be PNIO packet, or DCE/RPC endpoint mapper packet
        data: Box<[u8]>,
    ) -> Self {
        let packet_type_u8 = (packet_type as u8).to_be_bytes();
        let op_num_u16 = (opnum as u16).to_le_bytes();
        let fragment_len = (data.len() as u16).to_le_bytes();
        let interface_ver = (interface_ver as u32).to_le_bytes();

        DceRpcPacket {
            version: constant::DCERPC_VERSION.to_be_bytes(),
            packet_type: packet_type_u8,
            flags1: constant::DCERPC_FLAGS1,
            flags2: constant::DCERPC_FLAGS2,
            data_representation: constant::DCERPC_DATA_REPRESENTATION,
            serial_high: constant::DCERPC_SERIAL_HIGH.to_be_bytes(),
            obj_uuid: obj_uuid.to_bytes_le(),
            interface: interface.to_bytes_le(),
            activity: activity.to_bytes_le(),
            server_boot_time: constant::DCERPC_SERVER_BOOT_TIME,
            interface_ver,
            seq_num: seq_num.to_le_bytes(),
            opnum: op_num_u16,
            interface_hint: constant::DCERPC_INTERFACE_HINT,
            activity_hint: constant::DCERPC_ACTIVITY_HINT,
            fragment_len,
            fragment_num: constant::DCERPC_FRAGMENT_NUM.to_be_bytes(),
            auth_proto: constant::DCERPC_AUTH_PROTO.to_be_bytes(),
            serial_low: constant::DCERPC_SERIAL_LOW.to_be_bytes(),
            data,
        }
    }
}

impl Packet for DceRpcPacket {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        let mut v: Vec<u8> = vec![];

        v.extend(&self.version);
        v.extend(&self.packet_type);
        v.extend(&self.flags1);
        v.extend(&self.flags2);
        v.extend(&self.data_representation);
        v.extend(&self.serial_high);
        v.extend(&self.obj_uuid);
        v.extend(&self.interface);
        v.extend(&self.activity);
        v.extend(&self.server_boot_time);
        v.extend(&self.interface_ver);
        v.extend(&self.seq_num);
        v.extend(&self.opnum);
        v.extend(&self.interface_hint);
        v.extend(&self.activity_hint);
        v.extend(&self.fragment_len);
        v.extend(&self.fragment_num);
        v.extend(&self.auth_proto);
        v.extend(&self.serial_low);
        v.extend(&*self.data);

        Ok(v)
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

impl TryFrom<Vec<u8>> for DceRpcPacket {
    type Error = anyhow::Error;

    fn try_from(value: Vec<u8>) -> anyhow::Result<Self, Self::Error> {
        let packet_type = match value.get(1..2) {
            Some(p) => constant::PacketType::from_u8(p[0]).unwrap(),
            None => return Err(anyhow!("DCE/RPC packet type cannot be created")),
        };

        let obj_uuid = match value.get(8..24) {
            Some(o) => TryInto::<&[u8; 16]>::try_into(o).unwrap(),
            None => return Err(anyhow!("DCE/RPC object UUID cannot be created")),
        };

        let interface = match value.get(24..40) {
            Some(i) => TryInto::<&[u8; 16]>::try_into(i).unwrap(),
            None => return Err(anyhow!("DCE/RPC interface cannot be created")),
        };

        let activity = match value.get(40..56) {
            Some(a) => TryInto::<&[u8; 16]>::try_into(a).unwrap(),
            None => return Err(anyhow!("DCE/RPC activity cannot be created")),
        };

        let interface_ver = match value.get(60..64) {
            Some(iv) => {
                let b = u32::from_le_bytes(TryInto::<[u8; 4]>::try_into(iv).unwrap());
                constant::InterfaceVersion::from_u32(b).unwrap()
            }
            None => return Err(anyhow!("DCE/RPC interface version cannot be created")),
        };

        let seq_num = match value.get(64..68) {
            Some(s) => u32::from_le_bytes(*TryInto::<&[u8; 4]>::try_into(s).unwrap()),
            None => return Err(anyhow!("DCE/RPC sequence number cannot be created")),
        };

        let opnum_u16 = match value.get(68..70) {
            Some(o) => u16::from_le_bytes(*TryInto::<&[u8; 2]>::try_into(o).unwrap()),
            None => return Err(anyhow!("DCE/RPC opnum cannot be created")),
        };
        let opnum = constant::OpNum::from_u16(opnum_u16).unwrap();

        match value.get(74..76) {
            Some(f) => u16::from_le_bytes(*TryInto::<&[u8; 2]>::try_into(f).unwrap()),
            None => return Err(anyhow!("DCE/RPC fragment length cannot be created")),
        };

        // the rest of the bytes are stub data (payload)
        let payload_start: usize = 80;
        let payload = match value.get(payload_start..) {
            Some(p) => p.to_vec().into_boxed_slice(),
            None => return Err(anyhow!("DCE/RPC stub data cannot be created")),
        };

        let dcerpc_packet = DceRpcPacket::new(
            packet_type,
            Uuid::from_slice(obj_uuid).unwrap(),
            Uuid::from_slice(interface).unwrap(),
            interface_ver,
            Uuid::from_slice(activity).unwrap(),
            seq_num,
            opnum,
            payload,
        );

        Ok(dcerpc_packet)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn create_write_request_dcerpc_packet_should_ok() {
        let packet_type = constant::PacketType::Request;
        let obj_uuid = Uuid::parse_str("dea00000-6c97-11d1-8271-00010313002a").unwrap();
        let interface = Uuid::parse_str("dea00001-6c97-11d1-8271-00a02442df7d").unwrap();
        let activity = Uuid::parse_str("401ca514-11a1-1e1e-9ec0-080027e3f4b9").unwrap();
        let seq_num = 4;
        let opnum = constant::OpNum::Write;

        let stub_data = "47000000470000004700\
                         00000000000047000000\
                         0008003c01000003b63d\
                         bc71b5459246b8c50761\
                         aeb88cde000000000001\
                         00010000005000000007\
                         00000000000000000000\
                         00000000000000000000\
                         00000000001402800000\
                         82";

        let payload = hex::decode(stub_data).unwrap().into_boxed_slice();

        let dcerpc_packet = DceRpcPacket::new(
            packet_type,
            obj_uuid,
            interface,
            constant::InterfaceVersion::ReadWrite,
            activity,
            seq_num,
            opnum,
            payload,
        );

        let target_bytes = "04002000100000000000\
                      a0de976cd11182710001\
                      0313002a0100a0de976c\
                      d111827100a02442df7d\
                      14a51c40a1111e1e9ec0\
                      080027e3f4b900000000\
                      01000000040000000300\
                      ffffffff5b0000000000\
                      47000000470000004700\
                      00000000000047000000\
                      0008003c01000003b63d\
                      bc71b5459246b8c50761\
                      aeb88cde000000000001\
                      00010000005000000007\
                      00000000000000000000\
                      00000000000000000000\
                      00000000001402800000\
                      82";
        let target = hex::decode(target_bytes).unwrap();
        let bytes = dcerpc_packet.concat().unwrap();

        assert_eq!(target, bytes);
        assert_eq!(dcerpc_packet.fragment_len, (91 as u16).to_le_bytes());
    }
}
