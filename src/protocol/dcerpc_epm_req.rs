use std::mem;

use super::{constant, Packet};

#[derive(Debug)]
pub struct DceRpcEpmRequest {
    pub inquiry_type: [u8; 4],
    pub object_reference_id: [u8; 4],
    pub object_object: [u8; 16],
    pub interface_refrence_id: [u8; 4],
    pub interface_interface: [u8; 16],
    pub interface_version_major: [u8; 2],
    pub interface_version_minor: [u8; 2],
    pub version_option: [u8; 4],
    pub handle: [u8; 20],
    pub max_entries: [u8; 4],
}

impl DceRpcEpmRequest {
    pub fn new(handle: [u8; 20]) -> Self {
        DceRpcEpmRequest {
            inquiry_type: constant::DCERPC_EPM_INQUIRY_TYPE,
            object_reference_id: constant::DCERPC_EPM_REF_ID.to_le_bytes(),
            object_object: constant::DCERPC_EPM_OBJECT_OBJECT,
            interface_refrence_id: constant::DCERPC_EPM_INTERFACE_REF_ID.to_le_bytes(),
            interface_interface: constant::DCERPC_EPM_INTERFACE_INTERFACE,
            interface_version_major: constant::DCERPC_EPM_INTERFACE_VERSION_MAJOR.to_be_bytes(),
            interface_version_minor: constant::DCERPC_EPM_INTERFACE_VERSION_MINOR.to_be_bytes(),
            version_option: constant::DCERPC_EPM_VERSION_OPTION.to_le_bytes(),
            handle,
            max_entries: constant::DCERPC_EPM_MAX_ENTRIES.to_le_bytes(),
        }
    }
}

impl Packet for DceRpcEpmRequest {
    fn concat(&self) -> anyhow::Result<Vec<u8>> {
        let mut v: Vec<u8> = vec![];

        v.extend(self.inquiry_type);
        v.extend(self.object_reference_id);
        v.extend(self.object_object);
        v.extend(self.interface_refrence_id);
        v.extend(self.interface_interface);
        v.extend(self.interface_version_major);
        v.extend(self.interface_version_minor);
        v.extend(self.version_option);
        v.extend(self.handle);
        v.extend(self.max_entries);

        Ok(v)
    }

    fn size(&self) -> usize {
        mem::size_of::<Self>()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn construct_packet_successfully() {
        let handle = [0x00; 20];
        let dcerpc_epm_request = DceRpcEpmRequest::new(handle);

        let result = dcerpc_epm_request.concat().unwrap();
        let target = hex::decode(
            "00000000010000000000\
                 00000000000000000000\
                 00000000020000000000\
                 00000000000000000000\
                 00000000000000000100\
                 00000000000000000000\
                 00000000000000000000\
                 000001000000",
        )
        .unwrap();
        assert_eq!(result, target);
    }
}
