use super::{
    metadata::Metadata, spec_comm_status::FieldDeviceCommStatus, spec_status::FieldDeviceStatus,
};
use crate::{
    protocol::{
        ArBlockReq, BlockHeaderType, DceRpcPacket, HartCommand, InterfaceVersion, IodReq, OpNum,
        Packet, PacketType, Pnio, PnioHeaderEnum,
    },
    transport::TransportClient,
};
use anyhow::anyhow;
use core::time;
use std::{
    cell::{Cell, RefCell},
    net::IpAddr,
    thread,
};
use uuid::Uuid;

#[derive(Debug)]
pub struct PnioDevice {
    pub handle: String,
    pub object_uuid: Uuid,
    pub interface_uuid: Uuid,
    pub device_id: RefCell<[u8; 5]>,

    pub ip_address: IpAddr,
    pub port: u16,

    // TODO: maybe remove this for clarity
    pub comm_status: FieldDeviceCommStatus,
    // TODO: maybe remove this for clarity
    pub status: FieldDeviceStatus,
    // TODO: maybe remove this for clarity
    pub metadata: Metadata,

    pub transport_client: Box<dyn TransportClient>,
    pub ar_uuid: RefCell<Uuid>,

    pub activity: RefCell<Uuid>,
    pub dcerpc_seq_num: Cell<u32>,
    pub pnio_seq_num: Cell<u16>,

    pub slot_num: u16,
    pub subslot_num: u16,

    /// data_ready_flag is the "response control" byte
    /// coming back in the first byte of PNIO packet's user specified data
    /// refer to the AI module documentation
    pub data_ready_flag: u8,

    /// request_data_record_number indicates the pnio data record for request each
    /// channel for the AI module, see manual for specific AI module for more info.
    pub request_data_record_number: u16,
    /// response_data_record_number indicates the pnio data record for response each
    /// channel for the AI module, see manual for specific AI module for more info.
    pub response_data_record_number: u16,
    /// hart_device_name is the hart device model
    pub hart_device_name: String,
}

impl PnioDevice {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        handle: String,
        object_uuid: Uuid,
        interface_uuid: Uuid,
        port: u16,
        device_id: [u8; 5],
        ip_address: IpAddr,
        transport_client: Box<dyn TransportClient>,
        slot_num: u16,
        subslot_num: u16,
        data_ready_flag: u8,
        request_data_record_number: u16,
        response_data_record_number: u16,
        hart_device_name: String,
    ) -> Self {
        PnioDevice {
            handle,
            object_uuid,
            interface_uuid,
            port,
            device_id: device_id.into(),
            ip_address,
            ar_uuid: RefCell::new(Uuid::new_v4()),
            transport_client,
            slot_num,
            subslot_num,
            activity: RefCell::new(Uuid::new_v4()),
            dcerpc_seq_num: Cell::new(0),
            pnio_seq_num: Cell::new(0),
            data_ready_flag,
            request_data_record_number,
            response_data_record_number,
            hart_device_name,
            comm_status: FieldDeviceCommStatus {
                buffer_overflow: false.into(),
                communication_failure: false.into(),
                longitudinal_parity_error: false.into(),
                framing_error: false.into(),
                overrun_error: false.into(),
                vertical_parity_error: false.into(),
                communication_error: false.into(),
            },
            status: FieldDeviceStatus {
                primary_variable_out_of_limits: false.into(),
                non_primary_variable_out_of_limits: false.into(),
                loop_current_saturated: false.into(),
                loop_current_fixed: false.into(),
                more_status_available: false.into(),
                cold_start: false.into(),
                configuration_changed: false.into(),
                device_malfunction: false.into(),
            },
            metadata: Metadata {
                hart_protocol_major_revision: Default::default(),
                device_revision_level: Default::default(),
                software_revision_level: Default::default(),
                number_of_preemble_bytes_in_response: Default::default(),
                configuration_change_counter: Default::default(),
            },
        }
    }

    fn construct_pnio_req(
        &self,
        is_read: bool,
        pnio_header: PnioHeaderEnum,
        data: Option<Box<[u8]>>,
    ) -> anyhow::Result<Pnio> {
        let status: Option<[u8; 4]> = None;
        let mut pnio_data = data;

        if is_read {
            pnio_data = None;
        }

        let pnio = Pnio::new(status, pnio_header, pnio_data);

        Ok(pnio)
    }

    fn construct_dcerpc_req(&self, opnum: OpNum, data: Box<[u8]>) -> anyhow::Result<DceRpcPacket> {
        let packet_type = PacketType::Request;
        let interface_ver = InterfaceVersion::ReadWrite;

        let request_dcerpc_packet = DceRpcPacket::new(
            packet_type,
            self.object_uuid,
            self.interface_uuid,
            interface_ver,
            *self.activity.borrow(),
            self.dcerpc_seq_num.get(),
            opnum,
            data,
        );

        Ok(request_dcerpc_packet)
    }

    fn construct_iod_header_req(
        &self,
        is_read: bool,
        index: u16,
        data: &Option<Box<[u8]>>,
    ) -> anyhow::Result<IodReq> {
        let mut block_header_type = BlockHeaderType::IodWriteReqType;
        let record_data_len: u32 = match data.as_ref() {
            Some(d) => d.len().try_into()?,
            None => 65520,
        };

        if is_read {
            block_header_type = BlockHeaderType::IodReadReqType;
        }

        let iod_write_req_header = IodReq::new(
            block_header_type,
            self.pnio_seq_num.get(),
            *self.ar_uuid.borrow(),
            self.slot_num,
            self.subslot_num,
            index,
            record_data_len,
        );

        Ok(iod_write_req_header)
    }

    fn next_request(&self) {
        self.dcerpc_seq_num.set(self.dcerpc_seq_num.get() + 1);
        self.pnio_seq_num.set(self.pnio_seq_num.get() + 1);
    }

    pub fn connect_req(&self) -> anyhow::Result<()> {
        // PNIO ARBlockReq
        let session_key: u16 = 1;
        self.ar_uuid.replace(Uuid::new_v4());
        let pnio_header_arblock =
            ArBlockReq::new(*self.ar_uuid.borrow(), session_key, self.object_uuid);
        let pnio_data = None;
        // PNIO
        let pnio = self.construct_pnio_req(
            false,
            PnioHeaderEnum::ArBlockReq(pnio_header_arblock),
            pnio_data,
        )?;
        // DCE/RPC packet
        let req_dcerpc_packet =
            self.construct_dcerpc_req(OpNum::Connect, pnio.concat()?.into_boxed_slice())?;
        // send connect request
        self.transport_client
            .send(req_dcerpc_packet.concat()?.into_boxed_slice())?;
        // receive connect request's response
        self.transport_client.receive()?;

        Ok(())
    }

    pub fn send_common_write_req(
        &self,
        data_record_num: u16,
        command: u8,
        command_payload: Option<&[u8]>,
    ) -> anyhow::Result<()> {
        let device_id = *self.device_id.borrow();
        let user_specified_data =
            HartCommand::construct_write_request(device_id, command, command_payload)?;
        let pnio_data = Some(user_specified_data);

        let iod_write_req_header =
            self.construct_iod_header_req(false, data_record_num, &pnio_data)?;

        let pnio = self.construct_pnio_req(
            false,
            PnioHeaderEnum::IodReq(iod_write_req_header),
            pnio_data,
        )?;

        let req_dcerpc_packet =
            self.construct_dcerpc_req(OpNum::Write, pnio.concat()?.into_boxed_slice())?;

        if let Err(err) = self
            .transport_client
            .send(req_dcerpc_packet.concat()?.into_boxed_slice())
        {
            return Err(anyhow!("failed to send dcerpc packet: {err}"));
        };

        // receive write request's response
        let buffer = self.transport_client.receive()?;
        let res_dcerpc_packet = TryInto::<DceRpcPacket>::try_into(buffer.to_vec())?;
        let res_pnio_packet = TryInto::<Pnio>::try_into(res_dcerpc_packet.data.to_vec())?;
        if res_pnio_packet.status == Some([0x00; 4]) {
            return Ok(());
        }
        Err(anyhow!("{:?}", res_pnio_packet.status.unwrap()))
    }

    // send read request to read the response,
    // return HART statuses 2 bytes (all commands have this),
    // and the rest is command specific response,
    // check HART specification for relevant HART command
    pub fn send_common_read_req(
        &self,
        data_record_number: u16,
        command: u8, // this is just to verify whether the response of the request
                     // is indeed the correct corresponds
    ) -> anyhow::Result<(u8, Box<[u8]>)> {
        const RETRY_MAX: u8 = 10;
        let mut read_again = true;
        let mut retry = 0;
        // first byte is the response code, second byte is device status
        // and the rest are actual data
        let mut status_and_hart_response: Box<[u8]> = Default::default();
        // response may have more bytes, this valid_data_length is indicating the
        // number the valid bytes, starting from the statuses
        // TODO: this data_length include statuses or not?
        let mut data_length: u8 = Default::default();
        let mut final_device_id: [u8; 5] = [0x00; 5];

        while read_again {
            if retry >= RETRY_MAX {
                return Err(anyhow!("failed to get valid data after 10 tries"));
            }

            // increment the request's sequence number
            self.next_request();

            // PNIO IODReadReqHeader
            let iod_read_req_header =
                self.construct_iod_header_req(true, data_record_number, &None)?;
            // PNIO packet
            let pnio_data = None;
            let pnio = self.construct_pnio_req(
                true,
                PnioHeaderEnum::IodReq(iod_read_req_header),
                pnio_data,
            )?;
            // DCE/RPC packet
            let req_dcerpc_packet =
                self.construct_dcerpc_req(OpNum::Read, pnio.concat()?.into_boxed_slice())?;

            // send read request
            self.transport_client
                .send(req_dcerpc_packet.concat()?.into_boxed_slice())?;
            // receive read request's response
            let buffer = self.transport_client.receive()?;
            // DCE/RPC response packet
            let res_dcerpc_packet = TryInto::<DceRpcPacket>::try_into(buffer.to_vec())?;
            // PNIO response packet
            let res_pnio_packet = TryInto::<Pnio>::try_into(res_dcerpc_packet.data.to_vec())?;
            // handle first command 0 to find the device_id
            // retrieve the device id from PNIO response packet's payload
            if command == 0 && *self.device_id.borrow() == [0x00; 5] {
                read_again = res_pnio_packet.pnio_data.is_some_and(|pnio_data| {
                    if pnio_data
                        .first()
                        .is_some_and(|v| *v == self.data_ready_flag)
                    {
                        // TODO:
                        // to get the device_type_code in order to form the device_id,
                        // it's uncertain that it would work for all kind of hart devices,
                        // but it certainly works on the device that i am working on.
                        let device_type_code_result =
                            TryInto::<[u8; 2]>::try_into(pnio_data.get(9..11).unwrap()).map_err(
                                |_| "failed to parse device_type_code from the command 0 response",
                            );

                        if device_type_code_result.is_err() {
                            log::error!("{:?}", device_type_code_result.err());
                            // read again
                            return true;
                        }

                        let device_id_result = TryInto::<[u8; 3]>::try_into(
                            pnio_data.get(17..20).unwrap(),
                        )
                        .map_err(|_| "failed to parse device_id from the command 0 response");

                        if device_id_result.is_err() {
                            log::error!("{:?}", device_id_result.err());
                            // read again
                            return true;
                        }

                        let device_type_code = device_type_code_result.unwrap();
                        let device_id = device_id_result.unwrap();
                        final_device_id[0] = device_type_code[0];
                        final_device_id[1] = device_type_code[1];
                        final_device_id[2] = device_id[0];
                        final_device_id[3] = device_id[1];
                        final_device_id[4] = device_id[2];
                        // got device_id for this hart device
                        self.device_id.replace(final_device_id);

                        true
                    } else {
                        // read again
                        true
                    }
                });
            } else {
                // handle common responseo other than command 0, just parse the
                // payload and return the bytes
                read_again = res_pnio_packet.pnio_data.is_some_and(|data| {
                    if data.first().is_some_and(|v| *v == self.data_ready_flag) {
                        data_length = data[9];
                        let hart_only = &data[10..];
                        status_and_hart_response = hart_only.to_vec().into_boxed_slice();
                        false
                    } else {
                        // read again
                        true
                    }
                });
            };

            if !read_again {
                break;
            }

            log::debug!(
                "retry counter: {retry}, response for command `{command}` is not ready yet, send request again"
            );
            retry += 1;

            thread::sleep(time::Duration::from_secs(1));
        }

        Ok((data_length, status_and_hart_response))
    }
}
