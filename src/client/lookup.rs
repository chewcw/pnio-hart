use crate::device::pnio_device::PnioDevice;
use crate::protocol::{
    DceRpcEpmRequest, DceRpcEpmResponse, DceRpcPacket, InterfaceVersion, OpNum, Packet, PacketType,
    TowerFloorProtocol, INTERFACE,
};
use crate::transport::{TransportClient, UdpClient};
use anyhow::anyhow;
use std::cell::Cell;
use std::net::Ipv4Addr;
use std::thread;
use std::time;
use uuid::Uuid;

#[derive(Debug)]
pub struct LookupClient {
    dcerpc_seq_num: Cell<u32>,
}

pub type TargetDeviceUniqueName<'a> = &'a str;
pub type TargetIpAddr<'a> = &'a str;
pub type TargetLookupPort = u16;
pub type TargetSlotNum = u16;
pub type TargetSubslotNum = u16;
pub type TargetRequestDataRecordNumber = u16;
pub type TargetResponseDataRecordNumber = u16;
pub type TargetHartDeviceName<'a> = &'a str;

impl<'a> LookupClient {
    pub const MAX_RETRY: u8 = 10;

    pub fn new() -> Self {
        LookupClient {
            dcerpc_seq_num: Cell::new(0),
        }
    }

    /// lookup looks for the pnio device which able to meet target arguments, then
    /// parse response and then create pnio_device containing neccessary information
    /// for subsequent profinet hart requests.
    pub fn lookup(
        &self,
        src_ip: Ipv4Addr,
        target: (
            TargetDeviceUniqueName<'a>,
            TargetIpAddr<'a>,
            TargetLookupPort,
            TargetSlotNum,
            TargetSubslotNum,
            TargetRequestDataRecordNumber,
            TargetResponseDataRecordNumber,
            TargetHartDeviceName,
        ),
    ) -> anyhow::Result<PnioDevice> {
        let mut retry = 0;
        let mut dcerpc_epm_response: DceRpcEpmResponse;
        let mut dcerpc_response: DceRpcPacket;

        // initial handle is 0, real handle will be set after performed DCE/RPC
        // endpoint mapper request
        let mut default_handle: [u8; 20] = [0x00; 20];

        // destination ip
        let dest_ip = target.1.parse::<Ipv4Addr>()?;

        // TODO: abstract the udp client
        let udp_client = UdpClient::new(src_ip, dest_ip, target.2)?;
        log::debug!("{:?}", udp_client);

        let target_device = format!("{}-{}-{}", target.1, target.3, target.4);

        loop {
            log::debug!("try counter: {retry}, looking up device {target_device}");

            if retry >= Self::MAX_RETRY {
                return Err(anyhow!(
                    "failed for device {target_device}, ignoring this device"
                ));
            }

            // DCE/RPC endpoint mapper packet
            let dcerpc_epm_request = DceRpcEpmRequest::new(default_handle);
            let packet_type = PacketType::Request;
            let obj_uuid = Uuid::from_slice(&[0x00; 16]).unwrap();
            let interface = Uuid::parse_str(INTERFACE).unwrap();
            let interface_ver = InterfaceVersion::Lookup;
            let activity = Uuid::new_v4();
            let opnum = OpNum::Read;
            let data = dcerpc_epm_request.concat()?.into_boxed_slice();
            // DCE/RPC packet
            let dcerpc_packet = DceRpcPacket::new(
                packet_type,
                obj_uuid,
                interface,
                interface_ver,
                activity,
                self.dcerpc_seq_num.get(),
                opnum,
                data,
            );

            log::debug!("sending dcerpc packet to {target_device}");
            udp_client.send(dcerpc_packet.concat()?.into_boxed_slice())?;

            let raw_response = udp_client.receive()?;
            log::debug!("raw response: {:?}", raw_response);
            let raw_response_vec = raw_response.into_vec();
            dcerpc_response = TryInto::<DceRpcPacket>::try_into(raw_response_vec)?;
            log::debug!("dcerpc response: {:?}", dcerpc_response);

            let dcerpc_epm_response_arr = dcerpc_response.data;
            dcerpc_epm_response =
                TryInto::<DceRpcEpmResponse>::try_into(&*dcerpc_epm_response_arr)?;
            log::debug!("dcerpc_epm response: {:?}", dcerpc_epm_response);

            default_handle =
                TryInto::<[u8; 20]>::try_into(hex::decode(&dcerpc_epm_response.handle)?).unwrap();
            log::debug!("default_handle: {:?}", default_handle);

            // TODO: how do I know if an interface is a PNIO interface?
            // the workaround is if the response entry's object is [0x00; 16]
            // then consider this entry is not pnio, which should be skipped.
            if dcerpc_epm_response.entry.object == Uuid::from_bytes([0x00; 16]) {
                self.next(&mut retry);
                continue;
            }

            let tower_pointer = dcerpc_epm_response.entry.tower_pointer;

            // match the device_name
            let annotation = tower_pointer.annotation;
            if annotation.contains(target.0) {
                // TODO: there will be multiple floors, some of the floors has UUID,
                // how do I know which floor has the PNIO interface?
                // for now assume floor 1 is always PNIO.
                let interface_uuid = match tower_pointer.floors[0].uuid {
                    Some(i) => i,
                    None => {
                        log::error!("interface uuid cannot be found");
                        self.next(&mut retry);
                        continue;
                    }
                };

                let port = match tower_pointer
                    .floors
                    .iter()
                    .filter(|floor| matches!(floor.protocol, TowerFloorProtocol::Udp))
                    .last()
                    .and_then(|floor| floor.udp_port)
                {
                    Some(p) => p,
                    None => {
                        log::error!("port cannot be found");
                        self.next(&mut retry);
                        continue;
                    }
                };

                // update the destination port
                if let Err(err) = udp_client.update_dest(dest_ip, port) {
                    log::error!("failed to update udp client's destination: {err}");
                    continue;
                };

                // create pnio_device to be used in the subsequent operation
                let pnio_device = PnioDevice::new(
                    dcerpc_epm_response.handle,
                    dcerpc_epm_response.entry.object,
                    interface_uuid,
                    port,
                    // default device_id, to be set after performed HART command 0
                    [0x00; 5],
                    udp_client.get_dst_conn_details().unwrap().0,
                    Box::new(udp_client),
                    target.3,
                    target.4,
                    0x04,
                    target.5,
                    target.6,
                    target.7.to_string(),
                );

                log::debug!("found pnio device `{target_device}`, proceed...");

                break Ok(pnio_device);
            }

            self.next(&mut retry);
        }
    }

    fn next(&self, retry: &mut u8) {
        log::debug!(
            "try counter: {}, looking up device not found, wait for next try",
            retry
        );
        thread::sleep(time::Duration::from_secs(1));
        self.dcerpc_seq_num.set(self.dcerpc_seq_num.get() + 1);
        *retry += 1;
    }
}
