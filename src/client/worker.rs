use super::{lookup::LookupClient, sender::Sender};
use crate::{
    config::HartCommand,
    device::pnio_device::PnioDevice,
    dto::{iotedge_message::IotedgeMessageDto, temp::Temp},
};
use anyhow::anyhow;
use std::{collections::HashMap, net::Ipv4Addr};

type HartCommands = Vec<HartCommand>;
type PnioDeviceWithCommands = (PnioDevice, HartCommands);
/// Name is the unique program variable to identify each
/// hart device stored inside the program store, the name
/// consists of ip_address, slot_number and subslot_number
/// joined by a `dash`.
type Name = String;

// TODO: this information is storing in memory at the moment,
// should we store this into a local db?
pub struct Worker<'a> {
    sender: &'a dyn Sender,
    pub store: HashMap<Name, PnioDeviceWithCommands>,
}

impl<'a> Worker<'a> {
    pub fn new(sender: &'a dyn Sender) -> Self {
        let config_len = (*sender).get_config().read().unwrap().len();
        Self {
            sender,
            store: HashMap::with_capacity(config_len),
        }
    }

    /// evaluate if the pnio_device exists in the memory store, perform lookup if
    /// it's not in the memory store yet.
    pub fn evaluate(&mut self, src_ip_address: Ipv4Addr) {
        let configs = self.sender.get_config();
        let configs = (*configs).read().unwrap();
        let hart_devices_len = configs.iter().map(|config| config.hart_devices.len()).sum();
        let mut device_unique_names: Vec<Name> = Vec::with_capacity(hart_devices_len);

        log::debug!("config: {:?}", configs);

        for config in configs.iter() {
            for config_hart_device in config.hart_devices.iter() {
                let device_unique_name = format!(
                    "{}-{}-{}",
                    config.ip_address,
                    config_hart_device.slot_number,
                    config_hart_device.subslot_number
                );

                // default value, skip looking up this device
                if config.ip_address == *"127.0.0.1" {
                    continue;
                }

                if self.store.get(&device_unique_name).is_none() {
                    // those configured configs were not available in
                    // the memory then perform lookup
                    let target = (
                        config.device_name.as_str(),
                        config.ip_address.as_str(),
                        config.port,
                        config_hart_device.slot_number,
                        config_hart_device.subslot_number,
                        config_hart_device.request_data_record_number,
                        config_hart_device.response_data_record_number,
                        config_hart_device.hart_device_name.as_str(),
                    );

                    let lookup_client = LookupClient::new();
                    let pnio_device = match lookup_client.lookup(src_ip_address, target) {
                        Ok(pd) => pd,
                        Err(err) => {
                            log::error!(
                                "failed when performing lookup device `{device_unique_name}`: {err}"
                            );
                            continue;
                        }
                    };

                    log::debug!("pnio_device: {:?}", &pnio_device);

                    // connect to the device
                    match pnio_device.connect_req() {
                        Ok(_) => log::debug!("pnio_device `{device_unique_name}` connected"),
                        Err(err) => {
                            log::error!(
                                "failed to connect to pnio_device `{device_unique_name}`: {err}"
                            );
                            continue;
                        }
                    }

                    self.store.insert(
                        device_unique_name.clone(),
                        (pnio_device, config_hart_device.hart_commands.clone()),
                    );
                } else {
                    // those configured configs were available in the
                    // memory then update their value doesn't matter changed or unchanged
                    if let Some(pnio_device_with_commands) = self.store.get_mut(&device_unique_name)
                    {
                        // PnioDevice
                        // ip_address, slot_number and subslot_number are impossible
                        // to be changed as wouldn't reach here
                        pnio_device_with_commands.0.request_data_record_number =
                            config_hart_device.request_data_record_number;
                        pnio_device_with_commands.0.response_data_record_number =
                            config_hart_device.response_data_record_number;
                        // HartCommands
                        pnio_device_with_commands.1 = config_hart_device.hart_commands.clone();
                    };
                };

                device_unique_names.push(device_unique_name);
            }
        }

        // if those configured configs before have been deleted
        // then we need to remove them from memory as well
        let mut names_to_be_deleted: Vec<String> = vec![];
        for (name, _) in self.store.iter() {
            if !device_unique_names
                .iter()
                .any(|configured_name| *configured_name == *name)
            {
                names_to_be_deleted.push(name.clone());
            };
        }
        for name_to_be_deleted in names_to_be_deleted.iter() {
            self.store.remove(name_to_be_deleted);
        }

        log::debug!("the program memory store: {:?}", self.store);
    }

    pub fn read(&mut self) {
        for (device_unique_name, (pnio_device, hart_commands)) in self.store.iter() {
            if *pnio_device.device_id.borrow() == [0x00; 5] {
                // hart command 0 (without specifying device address) request
                // and this should be the first issued before any other hart command
                // this is a special hart command, its reponse is handled
                // directly in this application, see `send_common_read_req` implementation.
                log::info!("sending write request command 0 to device `{device_unique_name}`");
                if let Some(err) = pnio_device
                    .send_common_write_req(pnio_device.request_data_record_number, 0, None)
                    .err()
                {
                    log::error!(
                        "failed to send request command 0 to device `{device_unique_name}`: {err}"
                    );
                    continue;
                };

                log::info!("sending read request command 0 to device `{device_unique_name}`");
                if let Some(err) = pnio_device
                    .send_common_read_req(pnio_device.response_data_record_number, 0)
                    .err()
                {
                    log::error!(
                        "failed to send response command 0 to device `{device_unique_name}`: {err}"
                    );
                    continue;
                }
            } else {
                // other hart command, send the response bytes to the output
                for hart_command in hart_commands.iter() {
                    // send write request
                    log::info!(
                        "sending write request command {} to device `{device_unique_name}`",
                        hart_command.number
                    );
                    let command_payload = hart_command.data.as_deref();
                    if let Some(err) = pnio_device
                        .send_common_write_req(
                            pnio_device.request_data_record_number,
                            hart_command.number,
                            command_payload,
                        )
                        .err()
                    {
                        log::error!(
                            "failed to send request command {} to device `{device_unique_name}`: {err}", 
                            hart_command.number
                        );
                        continue;
                    };

                    // send read request
                    log::info!(
                        "sending read request command {} to device `{device_unique_name}`",
                        hart_command.number
                    );
                    match pnio_device.send_common_read_req(
                        pnio_device.response_data_record_number,
                        hart_command.number,
                    ) {
                        Ok(response) => {
                            log::debug!(
                                "response for hart command {} for device {device_unique_name} - bytes length: {}",
                                hart_command.number,
                                response.0,
                            );
                            // hart command response message
                            if self
                                .egress_hart_command_response(
                                    device_unique_name,
                                    pnio_device.hart_device_name.as_str(),
                                    hart_command.number,
                                    response.0,
                                    &response.1,
                                )
                                .is_err()
                            {
                                continue;
                            };

                            // general message
                            // every hart command response returned contains 2 bytes
                            // i.e. response code and device status, which are parsed
                            // and become pnio_device's FieldDeviceCommStatus and FieldDeviceStatus
                            // field respectively, send this message out as well
                            // if self.egress_hart_device_statuses().is_err() {
                            //     continue;
                            // };
                        }
                        Err(err) => {
                            log::error!("failed to send response command {} to device `{device_unique_name}`: {err}",
                                hart_command.number);
                        }
                    }
                }
            }
        }
    }

    /// egress_hart_command_response construct the message to be sent to output,
    /// for example the Azure IoT Hub message
    fn egress_hart_command_response(
        &self,
        device_unique_name: &str,
        hart_device_name: &str,
        hart_command: u8,
        length: u8,
        bytes: &[u8],
    ) -> anyhow::Result<()> {
        // construct the iot hub message
        let now = format!("{:?}", chrono::Utc::now());
        // TODO: testing, this is hard coded for the sack of quick presentation only.
        // let message = IotedgeMessageDto {
        //     timestamp: now.as_str(),
        //     // device_unique_name is the unique name set in the program, representing a
        //     // single hart device
        //     device_unique_name,
        //     // hart_device_name is the model for the profinet device, used by the subsequent
        //     // service to extract hart information out of the bytes
        //     hart_device_name,
        //     // hart_command indicating this message is the response for which hart command
        //     hart_command,
        //     // length is the byte count from hart status to hart data
        //     // the message sent out contains more bytes than what we actually need,
        //     // that's why this length is necessary
        //     length,
        //     // bytes is the actual data bytes
        //     bytes,
        // };

        // TODO: testing, remove this in production
        let data = Temp::map_to_string(hart_command, bytes).unwrap();
        let message = Temp {
            timestamp: now.as_str(),
            device_unique_name,
            hart_device_name,
            hart_command,
            length,
            data: data.as_str(),
        };

        let message = match serde_json::to_string(&message) {
            Ok(m) => m,
            Err(err) => {
                log::error!("failed to serialize the message{}", err);
                return Err(anyhow!(err));
            }
        };

        log::info!("sending message of `{device_unique_name}` to output");
        if let Err(err) = self.sender.send(message) {
            log::error!(
                "failed to egress message to output for device `{device_unique_name}`: {err}"
            );
            return Err(anyhow!(err));
        };

        Ok(())
    }

    fn egress_hart_device_statuses(&self) -> anyhow::Result<()> {
        let _message = HashMap::<&str, &str>::new();

        todo!()
    }
}
