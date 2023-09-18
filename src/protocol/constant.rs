// pnio packet
pub const BLOCK_VERSION_HIGH: u8 = 0x01;
pub const BLOCK_VERSION_LOW: u8 = 0x00;
pub const AR_TYPE: u16 = 0x0006;
pub const CM_INITIATOR_MAC: [u8; 6] = [0x00; 6];
pub const CM_INITIATOR_ACT_TIMEOUT_FACTOR: u16 = 0x006e;
pub const CM_INITIATOR_UDPRT_PORT: u16 = 0x0000;
pub const READ_MAX_COUNT: u32 = 65584;
// IOD packet
pub const IOD_PADDING: u8 = 0x00;
pub const IOD_REQ_API: [u8; 4] = [0x00; 4];
pub const AR_PROPS: [u8; 4] = [0x00, 0x00, 0x01, 0x11];
// DCE/RPC Endpoint Mapper packet
pub const DCERPC_EPM_INQUIRY_TYPE: [u8; 4] = [0x00, 0x00, 0x00, 0x00];
pub const DCERPC_EPM_REF_ID: u32 = 1;
pub const DCERPC_EPM_OBJECT_OBJECT: [u8; 16] = [0x00; 16];
pub const DCERPC_EPM_INTERFACE_REF_ID: u32 = 2;
pub const DCERPC_EPM_INTERFACE_INTERFACE: [u8; 16] = [0x00; 16];
pub const DCERPC_EPM_INTERFACE_VERSION_MAJOR: u16 = 0;
pub const DCERPC_EPM_INTERFACE_VERSION_MINOR: u16 = 0;
pub const DCERPC_EPM_VERSION_OPTION: u32 = 1;
pub const DCERPC_EPM_MAX_ENTRIES: u32 = 1;
// DCE/RPC Endpoint Mapper response tower floor protocol
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum TowerFloorProtocol {
    Uuid = 0x0d,
    RpcConnectionlessProtocol = 0x0a,
    Udp = 0x08,
    Ip = 0x09,
}
impl TowerFloorProtocol {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x0d => Some(Self::Uuid),
            0x0a => Some(Self::RpcConnectionlessProtocol),
            0x08 => Some(Self::Udp),
            0x09 => Some(Self::Ip),
            _ => None,
        }
    }
}
// DCE/RPC packet
pub const DCERPC_VERSION: u8 = 4;
pub const DCERPC_FLAGS1: [u8; 1] = [0x20];
pub const DCERPC_FLAGS2: [u8; 1] = [0x00];
pub const DCERPC_DATA_REPRESENTATION: [u8; 3] = [0x10, 0x00, 0x00];
pub const DCERPC_SERIAL_HIGH: u8 = 0;
pub const DCERPC_SERVER_BOOT_TIME: [u8; 4] = [0x00; 4];
pub const DCERPC_INTERFACE_HINT: [u8; 2] = [0xff, 0xff];
pub const DCERPC_ACTIVITY_HINT: [u8; 2] = [0xff, 0xff];
pub const DCERPC_FRAGMENT_NUM: u16 = 0;
pub const DCERPC_AUTH_PROTO: u8 = 0;
pub const DCERPC_SERIAL_LOW: u8 = 0;
// TODO: this seems like a fixed value?
// https://supportportal.juniper.net/s/article/MS-RPC-UUID-Mappings?language=en_US
pub const INTERFACE: &str = "e1af8308-5d1f-11c9-91a4-08002b14a0fa";
// DCE/RPC interface version
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum InterfaceVersion {
    Lookup = 0x00000003,
    ReadWrite = 0x00000001,
}
impl InterfaceVersion {
    pub fn from_u32(value: u32) -> Option<Self> {
        match value {
            0x00000003 => Some(Self::Lookup),
            0x00000001 => Some(Self::ReadWrite),
            _ => None,
        }
    }
}
// DCE/RPC opnum
#[derive(Debug, Copy, Clone)]
pub enum OpNum {
    Write = 0x0003,
    Read = 0x0002,
    Connect = 0x0000,
}
impl OpNum {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0003 => Some(Self::Write),
            0x0002 => Some(Self::Read),
            0x0000 => Some(Self::Connect),
            _ => None,
        }
    }
}
// DCE/RPC packet type
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum PacketType {
    Request = 0x00,
    Response = 0x02,
    Reject = 0x06,
}
impl PacketType {
    pub fn from_u8(value: u8) -> Option<Self> {
        match value {
            0x00 => Some(Self::Request),
            0x02 => Some(Self::Response),
            0x06 => Some(Self::Reject),
            _ => None,
        }
    }
}
// pnio block header type
#[derive(Debug, Copy, Clone, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum BlockHeaderType {
    ArBlockReqType = 0x0101,
    ArBlockResType = 0x8101,
    IodReadReqType = 0x0009,
    IodReadResType = 0x8009,
    IodWriteReqType = 0x0008,
    IodWriteResType = 0x8008,
}
impl BlockHeaderType {
    pub fn from_u16(value: u16) -> Option<Self> {
        match value {
            0x0101 => Some(Self::ArBlockReqType),
            0x8101 => Some(Self::ArBlockResType),
            0x0009 => Some(Self::IodReadReqType),
            0x0008 => Some(Self::IodWriteReqType),
            0x8009 => Some(Self::IodReadResType),
            0x8008 => Some(Self::IodWriteResType),
            _ => None,
        }
    }
}
